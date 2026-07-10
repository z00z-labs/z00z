use z00z_aggregators::{
    bind_publication_contract, AggregatorId, BatchId, BatchRoute, DistLevel, DistNote,
    DistNoteKind, PublicationRecord, PublicationState, PublishedBatch, ShardExecState,
    ShardExecTicket, ShardId, ShardPlacementView,
};
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, CheckpointDaLocatorKind, CheckpointDaProviderFamily,
        CheckpointDaReferenceV1, CheckpointDaReferenceVersion, CheckpointExecInputId, CheckpointId,
        CheckpointLifecycleStatus, CheckpointLifecycleV1, CheckpointPubIn,
        CheckpointPublicationEvidenceV1, CheckpointPublicationEvidenceVersion,
        CheckpointPublicationState,
    },
    fixture_support::checkpoint_fixtures,
    settlement::{PublicationRouteSnapshotV1, SettlementStateRoot},
};
use z00z_validators::{Verdict, VerdictKind};
use z00z_watchers::{
    AlertCounts, AlertKind, AlertSeverity, AlertSubject, EvidenceKey, EvidenceRecord,
    ProviderOutcome, ProviderSignal, ProviderStage, PublicationWatchErr, WatcherBoundary,
    WatcherInput,
};

#[test]
fn snapshot_prefers_exec_placement() {
    let batch_id = BatchId::from_bytes([0x71; 32]);
    let published = published_batch(batch_id, 5, 12, [0xA1; 32]);
    let publication = publication_record(&published, PublicationState::Posted);
    let fallback = placement_view(3, 11, 4);
    let exec = exec_ticket(batch_id, 5, 12, 8, ShardExecState::RecoveryPending);
    let binding = bind_publication_contract(
        batch_id,
        published.checkpoint_id,
        published.publication_route.route_table_digest,
        &published.pub_in,
    );
    let verdict = Verdict {
        batch_id,
        checkpoint_id: Some(published.checkpoint_id),
        publication: Some(binding.clone()),
        kind: VerdictKind::Accepted,
        reject: None,
        object_verdicts: Vec::new(),
    };
    let signal = ProviderSignal {
        provider_name: "local-da".to_string(),
        batch_id,
        stage: ProviderStage::Observe,
        outcome: ProviderOutcome::Success,
        blob_ref: Some(published.blob_ref.clone()),
    };
    let input = WatcherInput {
        published: published.clone(),
        publication: publication.clone(),
        soft_confirmation: None,
        placement: Some(fallback),
        exec_ticket: Some(exec.clone()),
        verdict: Some(verdict.clone()),
        provider_signal: Some(signal),
        runtime_notes: Vec::new(),
    };

    let snapshot = WatcherBoundary
        .checked_snapshot(
            &input,
            AlertCounts {
                info: 1,
                warn: 2,
                critical: 0,
            },
        )
        .expect("checked snapshot");

    assert_eq!(snapshot.batch_id, batch_id);
    assert_eq!(snapshot.publication_state, publication.state);
    assert_eq!(snapshot.shard_id, Some(ShardId::new(5)));
    assert_eq!(snapshot.aggregator_id, Some(AggregatorId::new(8)));
    assert_eq!(snapshot.routing_generation, Some(12));
    assert_eq!(snapshot.exec_state, Some(ShardExecState::RecoveryPending));
    assert_eq!(snapshot.binding_digest, Some(binding.binding_digest()));
    assert_eq!(
        snapshot.route_table_digest,
        Some(binding.route_table_digest())
    );
    assert_eq!(snapshot.verdict_kind, Some(verdict.kind));
    assert_eq!(snapshot.provider_stage, Some(ProviderStage::Observe));
    assert_eq!(snapshot.provider_outcome, Some(ProviderOutcome::Success));
    assert_eq!(snapshot.alert_counts.warn, 2);
}

#[test]
fn test_keeps_adapter_advisory() {
    let batch_id = BatchId::from_bytes([0x79; 32]);
    let mut published = published_batch(batch_id, 5, 12, [0xA9; 32]);
    let publication = publication_record(&published, PublicationState::Accepted);
    let binding = bind_publication_contract(
        batch_id,
        published.checkpoint_id,
        published.publication_route.route_table_digest,
        &published.pub_in,
    );
    let verdict = Verdict {
        batch_id,
        checkpoint_id: Some(published.checkpoint_id),
        publication: Some(binding.clone()),
        kind: VerdictKind::Accepted,
        reject: None,
        object_verdicts: Vec::new(),
    };
    published.da_provider = "forged-local-bridge".to_string();
    published.blob_ref = "local-da://forged-local-bridge/deadbeef".to_string();
    let input = WatcherInput {
        published: published.clone(),
        publication: publication.clone(),
        soft_confirmation: None,
        placement: None,
        exec_ticket: None,
        verdict: Some(verdict.clone()),
        provider_signal: None,
        runtime_notes: Vec::new(),
    };

    let watch = WatcherBoundary
        .publication_watch(&input)
        .expect("adapter metadata must stay advisory before binding drift");

    assert_eq!(watch.publication.binding_digest(), binding.binding_digest());

    let mut drift_input = input.clone();
    drift_input.published.pub_in = tampered_pub_in(&drift_input.published.pub_in);

    let err = WatcherBoundary
        .publication_watch(&drift_input)
        .expect_err("binding drift must reject even with forged local adapter metadata");

    assert_eq!(err, PublicationWatchErr::BindingMismatch);
}

#[test]
fn watcher_signal_does_not_open_challenge_without_publication_readiness() {
    let batch_id = BatchId::from_bytes([0x7A; 32]);
    let published = published_batch(batch_id, 5, 12, [0xAA; 32]);
    let publication = publication_record(&published, PublicationState::Accepted);
    let binding = bind_publication_contract(
        batch_id,
        published.checkpoint_id,
        published.publication_route.route_table_digest,
        &published.pub_in,
    );
    let input = WatcherInput {
        published: published.clone(),
        publication,
        soft_confirmation: None,
        placement: None,
        exec_ticket: None,
        verdict: Some(Verdict {
            batch_id,
            checkpoint_id: Some(published.checkpoint_id),
            publication: Some(binding),
            kind: VerdictKind::Accepted,
            reject: None,
            object_verdicts: Vec::new(),
        }),
        provider_signal: Some(ProviderSignal {
            provider_name: "local-da".to_string(),
            batch_id,
            stage: ProviderStage::Observe,
            outcome: ProviderOutcome::Success,
            blob_ref: Some(published.blob_ref.clone()),
        }),
        runtime_notes: Vec::new(),
    };
    let artifact = checkpoint_fixtures::artifact();
    let link = checkpoint_fixtures::link(
        derive_checkpoint_id(&artifact).expect("checkpoint id"),
        CheckpointExecInputId::new([8u8; 32]),
    );
    let lifecycle = CheckpointLifecycleV1::from_artifact(&artifact)
        .expect("sealed lifecycle")
        .link(&artifact, &link, None, [0xA5; 32])
        .expect("linked lifecycle");

    let watch = WatcherBoundary
        .publication_watch(&input)
        .expect("watcher publication proof");

    assert_eq!(watch.publication_state, PublicationState::Accepted);
    assert_eq!(lifecycle.status(), CheckpointLifecycleStatus::Linked);
    assert!(matches!(
        lifecycle.challenge_open(11),
        Err(z00z_storage::CheckpointError::LifecycleMix)
    ));
}

#[test]
fn watcher_accepts_publication_readiness_bundle() {
    let batch_id = BatchId::from_bytes([0x7B; 32]);
    let published = published_batch(batch_id, 5, 12, [0xAB; 32]);
    let publication = ready_publication_record(&published, PublicationState::Accepted);
    let binding = bind_publication_contract(
        batch_id,
        published.checkpoint_id,
        published.publication_route.route_table_digest,
        &published.pub_in,
    );

    let watch = WatcherBoundary
        .publication_watch(&WatcherInput {
            published,
            publication,
            soft_confirmation: None,
            placement: None,
            exec_ticket: None,
            verdict: Some(Verdict {
                batch_id,
                checkpoint_id: Some(binding.checkpoint_id()),
                publication: Some(binding),
                kind: VerdictKind::Accepted,
                reject: None,
                object_verdicts: Vec::new(),
            }),
            provider_signal: None,
            runtime_notes: Vec::new(),
        })
        .expect("ready publication watch");

    assert!(watch.da_reference.is_some());
    assert!(watch.publication_evidence.is_some());
    assert_eq!(
        watch.lifecycle.as_ref().expect("lifecycle").status(),
        CheckpointLifecycleStatus::PublicationReady
    );
}

#[test]
fn evidence_keeps_publication_story() {
    let batch_id = BatchId::from_bytes([0x81; 32]);
    let published = published_batch(batch_id, 6, 13, [0xB1; 32]);
    let publication = publication_record(&published, PublicationState::Accepted);
    let exec = exec_ticket(batch_id, 6, 13, 9, ShardExecState::Completed);
    let binding = bind_publication_contract(
        batch_id,
        published.checkpoint_id,
        published.publication_route.route_table_digest,
        &published.pub_in,
    );
    let verdict = Verdict {
        batch_id,
        checkpoint_id: Some(published.checkpoint_id),
        publication: Some(binding.clone()),
        kind: VerdictKind::Accepted,
        reject: None,
        object_verdicts: Vec::new(),
    };
    let evidence = EvidenceRecord {
        evidence_key: EvidenceKey {
            batch_id,
            sequence: 1,
        },
        kind: AlertKind::PublicationLag,
        severity: AlertSeverity::Warn,
        subject: AlertSubject::Batch(batch_id),
        publication: Some(publication.clone()),
        published: Some(published.clone()),
        soft_confirmation: None,
        placement: Some(placement_view(1, 3, 2)),
        exec_ticket: Some(exec.clone()),
        verdict: Some(verdict.clone()),
        provider_signal: None,
    };

    assert_eq!(evidence.runtime_placement(), Some(&exec.placement));
    assert_eq!(evidence.runtime_exec(), Some(&exec));
    assert_eq!(evidence.publication_binding(), Some(&binding));
    assert_eq!(evidence.binding_digest(), Some(binding.binding_digest()));
    assert_eq!(
        evidence
            .publication
            .as_ref()
            .expect("publication")
            .checkpoint_id,
        Some(published.checkpoint_id)
    );
    assert_eq!(
        evidence
            .published
            .as_ref()
            .expect("published")
            .checkpoint_id,
        published.checkpoint_id
    );
    assert_eq!(
        evidence.verdict.as_ref().expect("verdict").checkpoint_id,
        Some(published.checkpoint_id)
    );
    let watch = evidence.publication_watch().expect("publication watch");
    assert_eq!(watch.publication.binding_digest(), binding.binding_digest());
    assert_eq!(watch.publication_route, published.publication_route);
    assert_eq!(watch.runtime_route, Some(exec.placement.route));
}

#[test]
fn watcher_rejects_missing_publication_evidence_when_ready() {
    let batch_id = BatchId::from_bytes([0x90; 32]);
    let published = published_batch(batch_id, 6, 13, [0xC0; 32]);
    let mut publication = ready_publication_record(&published, PublicationState::Accepted);
    publication.publication_evidence = None;
    let binding = bind_publication_contract(
        batch_id,
        published.checkpoint_id,
        published.publication_route.route_table_digest,
        &published.pub_in,
    );
    let input = WatcherInput {
        published,
        publication,
        soft_confirmation: None,
        placement: None,
        exec_ticket: None,
        verdict: Some(Verdict {
            batch_id,
            checkpoint_id: Some(binding.checkpoint_id()),
            publication: Some(binding),
            kind: VerdictKind::Accepted,
            reject: None,
            object_verdicts: Vec::new(),
        }),
        provider_signal: None,
        runtime_notes: Vec::new(),
    };

    let err = WatcherBoundary
        .publication_watch(&input)
        .expect_err("ready lifecycle without publication evidence must reject");

    assert_eq!(err, PublicationWatchErr::ReadinessMismatch);
    assert_eq!(
        WatcherBoundary.validator_state_alerts(&input)[0].kind,
        AlertKind::InvalidBatch
    );
}

#[test]
fn watcher_rejects_detached_da_ref_when_ready() {
    let batch_id = BatchId::from_bytes([0x92; 32]);
    let published = published_batch(batch_id, 6, 13, [0xC2; 32]);
    let mut publication = ready_publication_record(&published, PublicationState::Accepted);
    let evidence = publication
        .publication_evidence
        .as_ref()
        .expect("publication evidence")
        .clone();
    publication.da_reference = Some(
        CheckpointDaReferenceV1::new(
            CheckpointDaReferenceVersion::CURRENT,
            evidence.provider_family(),
            CheckpointDaLocatorKind::OpaqueProviderRef,
            "local-da://watcher-detached",
            evidence.payload_commitment(),
            evidence.statement_core_digest(),
            evidence.archive_manifest_root(),
            evidence.readiness_height(),
        )
        .expect("detached da ref"),
    );
    let binding = bind_publication_contract(
        batch_id,
        published.checkpoint_id,
        published.publication_route.route_table_digest,
        &published.pub_in,
    );
    let input = WatcherInput {
        published,
        publication,
        soft_confirmation: None,
        placement: None,
        exec_ticket: None,
        verdict: Some(Verdict {
            batch_id,
            checkpoint_id: Some(binding.checkpoint_id()),
            publication: Some(binding),
            kind: VerdictKind::Accepted,
            reject: None,
            object_verdicts: Vec::new(),
        }),
        provider_signal: None,
        runtime_notes: Vec::new(),
    };

    let err = WatcherBoundary
        .publication_watch(&input)
        .expect_err("detached da ref must reject");

    assert_eq!(err, PublicationWatchErr::ReadinessMismatch);
    assert_eq!(
        WatcherBoundary.validator_state_alerts(&input)[0].kind,
        AlertKind::InvalidBatch
    );
}

#[test]
fn watcher_rejects_binding_drift() {
    let batch_id = BatchId::from_bytes([0x91; 32]);
    let mut published = published_batch(batch_id, 3, 9, [0xC1; 32]);
    let publication = publication_record(&published, PublicationState::Accepted);
    let binding = bind_publication_contract(
        batch_id,
        published.checkpoint_id,
        published.publication_route.route_table_digest,
        &published.pub_in,
    );
    let verdict = Verdict {
        batch_id,
        checkpoint_id: Some(published.checkpoint_id),
        publication: Some(binding),
        kind: VerdictKind::Accepted,
        reject: None,
        object_verdicts: Vec::new(),
    };
    published.pub_in = tampered_pub_in(&published.pub_in);

    let input = WatcherInput {
        published,
        publication,
        soft_confirmation: None,
        placement: None,
        exec_ticket: None,
        verdict: Some(verdict),
        provider_signal: None,
        runtime_notes: Vec::new(),
    };

    let err = WatcherBoundary
        .publication_watch(&input)
        .expect_err("binding drift must reject");

    assert_eq!(err, PublicationWatchErr::BindingMismatch);
    let alerts = WatcherBoundary.validator_state_alerts(&input);
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].kind, AlertKind::InvalidBatch);
}

#[test]
fn watcher_rejects_checkpoint_id_drift() {
    let batch_id = BatchId::from_bytes([0xA1; 32]);
    let published = published_batch(batch_id, 4, 10, [0xD1; 32]);
    let mut publication = publication_record(&published, PublicationState::Accepted);
    let binding = bind_publication_contract(
        batch_id,
        published.checkpoint_id,
        published.publication_route.route_table_digest,
        &published.pub_in,
    );
    let verdict = Verdict {
        batch_id,
        checkpoint_id: Some(published.checkpoint_id),
        publication: Some(binding),
        kind: VerdictKind::Accepted,
        reject: None,
        object_verdicts: Vec::new(),
    };
    publication.checkpoint_id = Some(CheckpointId::new([0xDE; 32]));

    let input = WatcherInput {
        published,
        publication,
        soft_confirmation: None,
        placement: None,
        exec_ticket: None,
        verdict: Some(verdict),
        provider_signal: None,
        runtime_notes: Vec::new(),
    };

    let err = WatcherBoundary
        .publication_watch(&input)
        .expect_err("checkpoint drift must reject");

    assert_eq!(err, PublicationWatchErr::CheckpointMismatch);
}

#[test]
fn watcher_rejects_exec_batch_drift() {
    let batch_id = BatchId::from_bytes([0xB1; 32]);
    let published = published_batch(batch_id, 6, 13, [0xE1; 32]);
    let publication = publication_record(&published, PublicationState::Accepted);
    let binding = bind_publication_contract(
        batch_id,
        published.checkpoint_id,
        published.publication_route.route_table_digest,
        &published.pub_in,
    );
    let verdict = Verdict {
        batch_id,
        checkpoint_id: Some(published.checkpoint_id),
        publication: Some(binding),
        kind: VerdictKind::Accepted,
        reject: None,
        object_verdicts: Vec::new(),
    };
    let input = WatcherInput {
        published,
        publication,
        soft_confirmation: None,
        placement: None,
        exec_ticket: Some(exec_ticket(
            BatchId::from_bytes([0xB2; 32]),
            6,
            13,
            9,
            ShardExecState::Completed,
        )),
        verdict: Some(verdict),
        provider_signal: None,
        runtime_notes: Vec::new(),
    };

    let err = WatcherBoundary
        .publication_watch(&input)
        .expect_err("exec batch drift must reject");

    assert_eq!(err, PublicationWatchErr::ExecMismatch);
}

#[test]
fn watcher_rejects_route_digest_drift() {
    let batch_id = BatchId::from_bytes([0xC1; 32]);
    let published = published_batch(batch_id, 6, 13, [0xF1; 32]);
    let publication = publication_record(&published, PublicationState::Accepted);
    let verdict = Verdict {
        batch_id,
        checkpoint_id: Some(published.checkpoint_id),
        publication: Some(bind_publication_contract(
            batch_id,
            published.checkpoint_id,
            [0xF2; 32],
            &published.pub_in,
        )),
        kind: VerdictKind::Accepted,
        reject: None,
        object_verdicts: Vec::new(),
    };

    let err = WatcherBoundary
        .publication_watch(&WatcherInput {
            published,
            publication,
            soft_confirmation: None,
            placement: None,
            exec_ticket: None,
            verdict: Some(verdict),
            provider_signal: None,
            runtime_notes: Vec::new(),
        })
        .expect_err("route digest drift must reject");

    assert_eq!(err, PublicationWatchErr::RouteMismatch);
}

#[test]
fn watcher_rejects_route_generation_drift() {
    let batch_id = BatchId::from_bytes([0xC2; 32]);
    let published = published_batch(batch_id, 6, 13, [0xF3; 32]);
    let publication = publication_record(&published, PublicationState::Accepted);
    let verdict = Verdict {
        batch_id,
        checkpoint_id: Some(published.checkpoint_id),
        publication: Some(bind_publication_contract(
            batch_id,
            published.checkpoint_id,
            published.publication_route.route_table_digest,
            &published.pub_in,
        )),
        kind: VerdictKind::Accepted,
        reject: None,
        object_verdicts: Vec::new(),
    };

    let err = WatcherBoundary
        .publication_watch(&WatcherInput {
            published,
            publication,
            soft_confirmation: None,
            placement: Some(placement_view(6, 14, 9)),
            exec_ticket: None,
            verdict: Some(verdict),
            provider_signal: None,
            runtime_notes: Vec::new(),
        })
        .expect_err("route generation drift must reject");

    assert_eq!(err, PublicationWatchErr::RouteMismatch);
}

#[test]
fn watcher_rejects_stale_route_activation() {
    let batch_id = BatchId::from_bytes([0xC3; 32]);
    let mut published = published_batch(batch_id, 6, 13, [0xF4; 32]);
    let publication = publication_record(&published, PublicationState::Accepted);
    let verdict = Verdict {
        batch_id,
        checkpoint_id: Some(published.checkpoint_id),
        publication: Some(bind_publication_contract(
            batch_id,
            published.checkpoint_id,
            published.publication_route.route_table_digest,
            &published.pub_in,
        )),
        kind: VerdictKind::Accepted,
        reject: None,
        object_verdicts: Vec::new(),
    };
    published.publication_checkpoint = published.publication_route.activation_checkpoint - 1;

    let err = WatcherBoundary
        .publication_watch(&WatcherInput {
            published,
            publication,
            soft_confirmation: None,
            placement: None,
            exec_ticket: None,
            verdict: Some(verdict),
            provider_signal: None,
            runtime_notes: Vec::new(),
        })
        .expect_err("stale route activation must reject");

    assert_eq!(err, PublicationWatchErr::RouteMismatch);
}

#[test]
fn watcher_missing_verdict_incomplete() {
    let batch_id = BatchId::from_bytes([0xC4; 32]);
    let published = published_batch(batch_id, 6, 13, [0xF5; 32]);
    let publication = publication_record(&published, PublicationState::Accepted);
    let input = WatcherInput {
        published: published.clone(),
        publication,
        soft_confirmation: None,
        placement: None,
        exec_ticket: None,
        verdict: None,
        provider_signal: None,
        runtime_notes: Vec::new(),
    };

    let alerts = WatcherBoundary.validator_state_alerts(&input);
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].kind, AlertKind::ValidatorIncomplete);

    let snapshot = WatcherBoundary.project_snapshot(&input, AlertCounts::default());
    assert_eq!(snapshot.verdict_kind, Some(VerdictKind::Incomplete));
    assert_eq!(snapshot.alert_counts.warn, 1);

    let evidence = WatcherBoundary
        .validator_state_evidence(&input, 7)
        .expect("incomplete evidence");
    assert_eq!(evidence.kind, AlertKind::ValidatorIncomplete);
    assert!(evidence.verdict.is_none());
    assert_eq!(evidence.published.as_ref(), Some(&published));
}

#[test]
fn watcher_missing_binding_incomplete() {
    let batch_id = BatchId::from_bytes([0xC5; 32]);
    let published = published_batch(batch_id, 6, 13, [0xF6; 32]);
    let publication = publication_record(&published, PublicationState::Accepted);
    let input = WatcherInput {
        published,
        publication,
        soft_confirmation: None,
        placement: None,
        exec_ticket: None,
        verdict: Some(Verdict {
            batch_id,
            checkpoint_id: Some(CheckpointId::new([0x44; 32])),
            publication: None,
            kind: VerdictKind::Accepted,
            reject: None,
            object_verdicts: Vec::new(),
        }),
        provider_signal: None,
        runtime_notes: Vec::new(),
    };

    let alerts = WatcherBoundary.validator_state_alerts(&input);
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].kind, AlertKind::ValidatorIncomplete);

    let snapshot = WatcherBoundary.project_snapshot(&input, AlertCounts::default());
    assert_eq!(snapshot.verdict_kind, Some(VerdictKind::Incomplete));
    assert_eq!(snapshot.alert_counts.warn, 1);
}

#[test]
fn watcher_retry_incomplete() {
    let batch_id = BatchId::from_bytes([0xC6; 32]);
    let published = published_batch(batch_id, 6, 13, [0xF7; 32]);
    let publication = publication_record(&published, PublicationState::RetryPending);
    let binding = bind_publication_contract(
        batch_id,
        published.checkpoint_id,
        published.publication_route.route_table_digest,
        &published.pub_in,
    );
    let verdict = Verdict {
        batch_id,
        checkpoint_id: Some(published.checkpoint_id),
        publication: Some(binding),
        kind: VerdictKind::Incomplete,
        reject: None,
        object_verdicts: Vec::new(),
    };
    let input = WatcherInput {
        published,
        publication,
        soft_confirmation: None,
        placement: None,
        exec_ticket: None,
        verdict: Some(verdict),
        provider_signal: None,
        runtime_notes: Vec::new(),
    };

    let alerts = WatcherBoundary.validator_state_alerts(&input);
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].kind, AlertKind::ValidatorIncomplete);

    let snapshot = WatcherBoundary.project_snapshot(&input, AlertCounts::default());
    assert_eq!(snapshot.verdict_kind, Some(VerdictKind::Incomplete));
    assert_eq!(snapshot.alert_counts.warn, 1);
}

#[test]
fn watcher_gap_incomplete() {
    let batch_id = BatchId::from_bytes([0xC7; 32]);
    let published = published_batch(batch_id, 6, 13, [0xF8; 32]);
    let publication = publication_record(&published, PublicationState::Posted);
    let binding = bind_publication_contract(
        batch_id,
        published.checkpoint_id,
        published.publication_route.route_table_digest,
        &published.pub_in,
    );
    let verdict = Verdict {
        batch_id,
        checkpoint_id: Some(published.checkpoint_id),
        publication: Some(binding),
        kind: VerdictKind::Accepted,
        reject: None,
        object_verdicts: Vec::new(),
    };
    let input = WatcherInput {
        published,
        publication,
        soft_confirmation: None,
        placement: None,
        exec_ticket: None,
        verdict: Some(verdict),
        provider_signal: Some(ProviderSignal {
            provider_name: "local-da".to_string(),
            batch_id,
            stage: ProviderStage::Observe,
            outcome: ProviderOutcome::Pending,
            blob_ref: Some("blob://pending".to_string()),
        }),
        runtime_notes: Vec::new(),
    };

    let alerts = WatcherBoundary.validator_state_alerts(&input);
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].kind, AlertKind::ValidatorIncomplete);

    let snapshot = WatcherBoundary.project_snapshot(&input, AlertCounts::default());
    assert_eq!(snapshot.verdict_kind, Some(VerdictKind::Incomplete));
    assert_eq!(snapshot.alert_counts.warn, 1);
}

#[test]
fn test_projects_runtime_notes() {
    let batch_id = BatchId::from_bytes([0xC8; 32]);
    let published = published_batch(batch_id, 6, 13, [0xF9; 32]);
    let publication = publication_record(&published, PublicationState::Accepted);
    let binding = bind_publication_contract(
        batch_id,
        published.checkpoint_id,
        published.publication_route.route_table_digest,
        &published.pub_in,
    );
    let verdict = Verdict {
        batch_id,
        checkpoint_id: Some(published.checkpoint_id),
        publication: Some(binding),
        kind: VerdictKind::Accepted,
        reject: None,
        object_verdicts: Vec::new(),
    };
    let runtime_notes = vec![
        DistNote::new(
            DistNoteKind::RouteRollout,
            DistLevel::Info,
            "staged route rollout generation 13",
        ),
        DistNote::new(
            DistNoteKind::ShardStall,
            DistLevel::Warn,
            "owner unavailable during local dispatch",
        ),
        DistNote::new(
            DistNoteKind::ShardFreeze,
            DistLevel::Critical,
            "same-term divergent root froze the shard",
        ),
    ];
    let input = WatcherInput {
        published,
        publication,
        soft_confirmation: None,
        placement: None,
        exec_ticket: None,
        verdict: Some(verdict),
        provider_signal: None,
        runtime_notes: runtime_notes.clone(),
    };

    let snapshot = WatcherBoundary.project_snapshot(&input, AlertCounts::default());
    assert_eq!(snapshot.runtime_notes, runtime_notes);
    assert!(!snapshot.runtime_truth);
    assert_eq!(snapshot.verdict_kind, Some(VerdictKind::Accepted));
    assert_eq!(snapshot.alert_counts.info, 1);
    assert_eq!(snapshot.alert_counts.warn, 1);
    assert_eq!(snapshot.alert_counts.critical, 1);

    let alerts = WatcherBoundary.runtime_note_alerts(&input);
    assert_eq!(alerts.len(), 3);
    assert_eq!(alerts[0].kind, AlertKind::RouteRollout);
    assert_eq!(alerts[1].kind, AlertKind::ShardStall);
    assert_eq!(alerts[2].kind, AlertKind::ShardFreeze);
}

fn published_batch(
    batch_id: BatchId,
    shard_id: u16,
    generation: u64,
    route_digest: [u8; 32],
) -> PublishedBatch {
    let artifact = checkpoint_fixtures::artifact();
    let checkpoint_id = derive_checkpoint_id(&artifact).expect("checkpoint id");
    PublishedBatch {
        batch_id,
        checkpoint_id,
        publication_checkpoint: 11,
        publication_route: PublicationRouteSnapshotV1::new(
            generation,
            route_digest,
            10,
            vec![u32::from(shard_id)],
        ),
        pub_in: artifact.pub_in(),
        subject_digest: None,
        certificate_digest: None,
        theorem_digest: None,
        da_provider: "local-da".to_string(),
        blob_ref: "blob://hjmt-publication".to_string(),
    }
}

fn publication_record(published: &PublishedBatch, state: PublicationState) -> PublicationRecord {
    PublicationRecord {
        batch_id: published.batch_id,
        checkpoint_id: Some(published.checkpoint_id),
        state,
        da_reference: None,
        publication_evidence: None,
        lifecycle: None,
    }
}

fn ready_publication_record(
    published: &PublishedBatch,
    state: PublicationState,
) -> PublicationRecord {
    let statement_core_digest = [0xD4; 32];
    let archive_manifest_root = [0xD5; 32];
    let payload_commitment = [0xD6; 32];
    let observations_root = [0xD7; 32];
    let readiness_height = published.publication_checkpoint;
    let da_reference = CheckpointDaReferenceV1::new(
        CheckpointDaReferenceVersion::CURRENT,
        CheckpointDaProviderFamily::LocalArchive,
        CheckpointDaLocatorKind::OpaqueProviderRef,
        "local-da://watcher-ready",
        payload_commitment,
        statement_core_digest,
        archive_manifest_root,
        readiness_height,
    )
    .expect("da reference");
    let publication_evidence = CheckpointPublicationEvidenceV1::new(
        CheckpointPublicationEvidenceVersion::CURRENT,
        statement_core_digest,
        da_reference.da_ref(),
        archive_manifest_root,
        payload_commitment,
        CheckpointPublicationState::DaPublicationReady,
        CheckpointDaProviderFamily::LocalArchive,
        readiness_height,
        readiness_height,
        observations_root,
    )
    .expect("publication evidence");
    let artifact = checkpoint_fixtures::artifact();
    let link = checkpoint_fixtures::link(
        published.checkpoint_id,
        CheckpointExecInputId::new([8u8; 32]),
    );
    let lifecycle = CheckpointLifecycleV1::from_artifact(&artifact)
        .expect("sealed lifecycle")
        .link(&artifact, &link, None, statement_core_digest)
        .expect("linked lifecycle")
        .publication_ready(&publication_evidence)
        .expect("publication-ready lifecycle");

    PublicationRecord {
        batch_id: published.batch_id,
        checkpoint_id: Some(published.checkpoint_id),
        state,
        da_reference: Some(da_reference),
        publication_evidence: Some(publication_evidence),
        lifecycle: Some(lifecycle),
    }
}

fn placement_view(shard_id: u16, generation: u64, aggregator_id: u16) -> ShardPlacementView {
    ShardPlacementView {
        route: BatchRoute {
            shard_id: ShardId::new(shard_id),
            routing_generation: generation,
        },
        primary_id: AggregatorId::new(aggregator_id),
        secondaries: Vec::new(),
        expected_journal_lineage: [0x91; 32],
    }
}

fn exec_ticket(
    batch_id: BatchId,
    shard_id: u16,
    generation: u64,
    aggregator_id: u16,
    state: ShardExecState,
) -> ShardExecTicket {
    ShardExecTicket {
        batch_id,
        placement: placement_view(shard_id, generation, aggregator_id),
        state,
    }
}

fn tampered_pub_in(pub_in: &CheckpointPubIn) -> CheckpointPubIn {
    let mut tampered = CheckpointPubIn::new_settlement(
        pub_in.prev_settlement_root(),
        SettlementStateRoot::settlement_v1([0xBC; 32]),
        pub_in.spent_delta().to_vec(),
        pub_in.created_delta().to_vec(),
    );
    if let Some(claim_root) = pub_in.claim_root() {
        tampered = tampered.with_claim_root(claim_root);
    }
    tampered
}
