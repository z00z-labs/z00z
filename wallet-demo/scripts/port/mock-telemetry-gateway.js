"use strict";

((root) => {
  const demo = root.Z00ZDemo;
  if (!demo?.PORT_CONTRACT || !demo.capabilityProfile || !demo.navigationNodeForRoute) {
    throw new Error("Z00Z contracts and navigation model must load before the mock telemetry gateway.");
  }

  const FIXTURE_SOURCE_ID = "deterministic-telemetry-fixture-v1";
  const FIXTURE_OBSERVED_AT = "2026-07-26T12:00:00.000Z";
  const STALE_FIXTURE_OBSERVED_AT = "2026-07-25T12:00:00.000Z";
  const SAFE_ISSUE_SEVERITIES = Object.freeze(["neutral", "warning", "danger"]);

  function deepFreeze(value) {
    if (!value || typeof value !== "object" || Object.isFrozen(value)) return value;
    Object.values(value).forEach(deepFreeze);
    return Object.freeze(value);
  }

  const scenario = ({
    availability,
    evidenceSource,
    freshness,
    observedAt = null,
    retryable = false,
    issue = null,
    records = null
  }) => deepFreeze({
    availability,
    evidenceSource,
    freshness,
    observedAt,
    retryable,
    issue,
    records
  });

  const TELEMETRY_SCENARIOS = deepFreeze({
    loading: scenario({
      availability: "unavailable",
      evidenceSource: "none",
      freshness: "unknown"
    }),
    success: scenario({
      availability: "available",
      evidenceSource: "fixture",
      freshness: "timestamp",
      observedAt: FIXTURE_OBSERVED_AT,
      records: [
        {
          publicId: "public-fixture-observation-001",
          kind: "status",
          value: "Available",
          provenance: "Deterministic fixture"
        },
        {
          publicId: "public-fixture-observation-002",
          kind: "evidence",
          value: "Verified fixture shape",
          provenance: "Deterministic fixture"
        }
      ]
    }),
    degraded: scenario({
      availability: "degraded",
      evidenceSource: "fixture",
      freshness: "stale",
      observedAt: STALE_FIXTURE_OBSERVED_AT,
      retryable: true,
      issue: {
        code: "partial_evidence",
        severity: "warning",
        message: "Only a stale partial fixture observation is available.",
        recoveryAction: "Refresh or inspect the available evidence."
      },
      records: [
        {
          publicId: "public-fixture-observation-001",
          kind: "status",
          value: "Partial",
          provenance: "Deterministic stale fixture"
        }
      ]
    }),
    unavailable: scenario({
      availability: "unavailable",
      evidenceSource: "none",
      freshness: "unknown",
      retryable: true,
      issue: {
        code: "native_bridge_unavailable",
        severity: "neutral",
        message: "No authoritative telemetry bridge is registered.",
        recoveryAction: "Retry after a native telemetry source is connected."
      }
    }),
    empty: scenario({
      availability: "available",
      evidenceSource: "fixture",
      freshness: "timestamp",
      observedAt: FIXTURE_OBSERVED_AT,
      issue: {
        code: "no_observations",
        severity: "neutral",
        message: "The deterministic source returned no matching observations.",
        recoveryAction: "Change the filter or refresh the observation."
      },
      records: []
    }),
    malformed: scenario({
      availability: "degraded",
      evidenceSource: "fixture",
      freshness: "unknown",
      observedAt: FIXTURE_OBSERVED_AT,
      retryable: true,
      issue: {
        code: "malformed_source_payload",
        severity: "danger",
        message: "The source payload failed validation and was discarded.",
        recoveryAction: "Retry with a valid source or inspect sanitized diagnostics."
      }
    }),
    error: scenario({
      availability: "unavailable",
      evidenceSource: "none",
      freshness: "unknown",
      retryable: true,
      issue: {
        code: "telemetry_gateway_error",
        severity: "danger",
        message: "The telemetry query failed before authoritative data was returned.",
        recoveryAction: "Retry the query or inspect sanitized diagnostics."
      }
    })
  });

  const WATCHER_SOURCE_OPTIONS = deepFreeze([
    {
      id: "runtime_projection",
      label: "Runtime projection",
      authority: "WatcherBoundary::project_snapshot",
      description: "Deterministic projection of publication, verdict, placement, provider, and alert-count fields."
    },
    {
      id: "evidence_archive",
      label: "Evidence archive",
      authority: "EvidenceRecord",
      description: "Deterministic sanitized evidence records keyed by public batch ID and sequence."
    }
  ]);

  const WATCHER_ROUTE_RECORDS = deepFreeze({
    "telemetry.watchers.overview": [
      {
        id: "watcher-snapshot-001",
        recordType: "observation_snapshot",
        batchId: "batch_4f91c7a0",
        publicationState: "RetryPending",
        providerStage: "Resolve",
        providerOutcome: "RetryPending",
        runtimeTruth: false,
        alertCounts: { info: 1, warn: 1, critical: 1 },
        mapping: "ObservationSnapshot"
      }
    ],
    "telemetry.watchers.alerts": [
      {
        id: "watcher-alert-001",
        recordType: "watcher_alert",
        kind: "PublicationLag",
        severity: "warn",
        subject: { kind: "batch", publicId: "batch_4f91c7a0" },
        observedAt: FIXTURE_OBSERVED_AT,
        provenance: { module: "watchers::publication", evidence: "deterministic fixture" },
        affectedPublicIds: ["batch_4f91c7a0", "checkpoint_000184"],
        summary: "Publication has not advanced within the fixture observation window.",
        nextAction: { kind: "inspect_evidence", label: "Inspect sanitized evidence" },
        explorerAction: { kind: "open_explorer", label: "Open checkpoint in Explorer", publicId: "checkpoint_000184" }
      },
      {
        id: "watcher-alert-002",
        recordType: "watcher_alert",
        kind: "MissingBlob",
        severity: "critical",
        subject: { kind: "published_batch", publicId: "batch_a13d9e22" },
        observedAt: FIXTURE_OBSERVED_AT,
        provenance: { module: "watchers::da_health", evidence: "deterministic fixture" },
        affectedPublicIds: ["batch_a13d9e22", "da_ref_72be91"],
        summary: "The fixture provider returned a typed Missing outcome for its opaque DA reference.",
        nextAction: { kind: "inspect_evidence", label: "Inspect sanitized evidence" },
        explorerAction: { kind: "open_explorer", label: "Open DA evidence in Explorer", publicId: "da_ref_72be91" }
      },
      {
        id: "watcher-alert-003",
        recordType: "watcher_alert",
        kind: "RouteRollout",
        severity: "info",
        subject: { kind: "batch", publicId: "batch_72bc108f" },
        observedAt: FIXTURE_OBSERVED_AT,
        provenance: { module: "watchers::engine", evidence: "deterministic fixture" },
        affectedPublicIds: ["batch_72bc108f"],
        summary: "A runtime note marks a route rollout; it is evidence, not runtime authority.",
        nextAction: { kind: "inspect_evidence", label: "Inspect sanitized evidence" },
        explorerAction: { kind: "open_explorer", label: "Open batch in Explorer", publicId: "batch_72bc108f" }
      }
    ],
    "telemetry.watchers.publication": [
      {
        id: "publication-check-binding",
        recordType: "publication_check",
        check: "Publication binding",
        status: "matched",
        detail: "Batch, checkpoint, and pub_in binding agree in the deterministic witness.",
        mapping: "PublicationWatch::try_from_runtime"
      },
      {
        id: "publication-check-route",
        recordType: "publication_check",
        check: "Route snapshot",
        status: "matched",
        detail: "Committed PublicationRouteSnapshotV1 covers the fixture runtime route.",
        mapping: "check_route_binding_v1"
      },
      {
        id: "publication-check-readiness",
        recordType: "publication_check",
        check: "Readiness bundle",
        status: "retry_pending",
        detail: "The fixture lifecycle is intentionally incomplete and remains non-final.",
        mapping: "PublicationRecord::validate_readiness_bundle"
      }
    ],
    "telemetry.watchers.providers": [
      {
        id: "provider-signal-primary",
        recordType: "provider_signal",
        providerName: "local-bridge",
        batchId: "batch_4f91c7a0",
        stage: "Resolve",
        outcome: "RetryPending",
        blobRef: "da_ref_72be91",
        mapping: "ProviderSignal"
      },
      {
        id: "provider-signal-secondary",
        recordType: "provider_signal",
        providerName: "archive-mirror",
        batchId: "batch_72bc108f",
        stage: "Observe",
        outcome: "Success",
        blobRef: "da_ref_18ac40",
        mapping: "ProviderSignal"
      }
    ],
    "telemetry.watchers.censorship": [
      {
        id: "censorship-boundary-001",
        recordType: "censorship_boundary",
        signalKind: "CensorshipSuspect",
        status: "concept_fixture_only",
        observationWindow: "12 deterministic publication slots",
        detail: "CensorshipWatch is currently a marker type; this demo does not invent a detector or global censorship verdict.",
        mapping: "CensorshipWatch + AlertKind::CensorshipSuspect"
      }
    ],
    "telemetry.watchers.evidence": [
      {
        id: "watcher-evidence-001",
        recordType: "evidence_record",
        alertId: "watcher-alert-001",
        sequence: 41,
        alertKind: "PublicationLag",
        severity: "warn",
        batchId: "batch_4f91c7a0",
        checkpointId: "checkpoint_000184",
        providerRef: "da_ref_72be91",
        bindings: ["publication_binding", "route_snapshot", "provider_signal"],
        mapping: "EvidenceKey + EvidenceRecord"
      },
      {
        id: "watcher-evidence-002",
        recordType: "evidence_record",
        alertId: "watcher-alert-002",
        sequence: 42,
        alertKind: "MissingBlob",
        severity: "critical",
        batchId: "batch_a13d9e22",
        checkpointId: "checkpoint_000185",
        providerRef: "da_ref_72be91",
        bindings: ["publication_record", "provider_signal"],
        mapping: "EvidenceKey + EvidenceRecord"
      },
      {
        id: "watcher-evidence-003",
        recordType: "evidence_record",
        alertId: "watcher-alert-003",
        sequence: 43,
        alertKind: "RouteRollout",
        severity: "info",
        batchId: "batch_72bc108f",
        checkpointId: "checkpoint_000186",
        providerRef: null,
        bindings: ["runtime_note", "route_snapshot"],
        mapping: "EvidenceKey + EvidenceRecord"
      }
    ]
  });

  const EXPLORER_PUBLIC_ID_PATTERNS = deepFreeze([
    { kind: "checkpoint", pattern: /^checkpoint_[0-9]{6}$/ },
    { kind: "batch", pattern: /^batch_[0-9a-f]{8}$/ },
    { kind: "publication", pattern: /^publication_[0-9a-f]{8}$/ },
    { kind: "proof", pattern: /^proof_[0-9a-f]{8}$/ },
    { kind: "da_reference", pattern: /^da_ref_[0-9a-f]{6}$/ }
  ]);
  const EXPLORER_PRIVATE_HINT = /(?:wallet|receiver|counterparty|memo|seed|private|secret|inbox|message|contact|route[_-]?path|node[_-]?path|address)/i;
  const EXPLORER_STALE_PUBLIC_IDS = deepFreeze(["checkpoint_000183", "publication_6f830183"]);

  const EXPLORER_CHECKPOINTS = deepFreeze([
    {
      id: "checkpoint_000184",
      publicId: "checkpoint_000184",
      recordType: "checkpoint",
      lifecycleStatus: "finalized",
      publicationState: "da_publication_ready",
      publicRoot: "root_84f2d18a…4c07",
      priorPublicRoot: "root_83a19c71…ba22",
      publicationEvidenceRoot: "evidence_root_71bd…9a30",
      publicationHeight: 184,
      challengeWindowStartHeight: 178,
      freshness: "timestamp",
      observedAt: FIXTURE_OBSERVED_AT,
      batchIds: ["batch_4f91c7a0"],
      publicEvidenceIds: ["publication_6f840184", "proof_92840184", "da_ref_72be91"],
      mapping: "CheckpointLifecycleV1 + CheckpointPublicationEvidenceV1"
    },
    {
      id: "checkpoint_000185",
      publicId: "checkpoint_000185",
      recordType: "checkpoint",
      lifecycleStatus: "challenge_open",
      publicationState: "da_publication_ready",
      publicRoot: "root_85c1e419…1f92",
      priorPublicRoot: "root_84f2d18a…4c07",
      publicationEvidenceRoot: "evidence_root_85ce…162f",
      publicationHeight: 185,
      challengeWindowStartHeight: 181,
      freshness: "timestamp",
      observedAt: FIXTURE_OBSERVED_AT,
      batchIds: ["batch_a13d9e22"],
      publicEvidenceIds: ["publication_1e850185", "proof_7a850185", "da_ref_72be91"],
      mapping: "CheckpointLifecycleV1 + PublicationRecord"
    },
    {
      id: "checkpoint_000186",
      publicId: "checkpoint_000186",
      recordType: "checkpoint",
      lifecycleStatus: "publication_ready",
      publicationState: "da_publication_ready",
      publicRoot: "root_86b41d77…c610",
      priorPublicRoot: "root_85c1e419…1f92",
      publicationEvidenceRoot: "evidence_root_18ac…40d2",
      publicationHeight: 186,
      challengeWindowStartHeight: 186,
      freshness: "timestamp",
      observedAt: FIXTURE_OBSERVED_AT,
      batchIds: ["batch_72bc108f"],
      publicEvidenceIds: ["publication_3c860186", "proof_c2860186", "da_ref_18ac40"],
      mapping: "CheckpointLifecycleV1 + CheckpointPublicationEvidenceV1"
    }
  ]);

  const EXPLORER_BATCHES = deepFreeze([
    {
      id: "batch_4f91c7a0",
      publicId: "batch_4f91c7a0",
      recordType: "batch",
      checkpointId: "checkpoint_000184",
      publicationId: "publication_6f840184",
      proofId: "proof_92840184",
      daReferenceId: "da_ref_72be91",
      publicationCheckpoint: 184,
      relationship: "PublishedBatch → CheckpointPublicationV1",
      routeGeneration: 12,
      shardIds: [2, 7],
      freshness: "timestamp",
      observedAt: FIXTURE_OBSERVED_AT,
      mapping: "PublishedBatch"
    },
    {
      id: "batch_a13d9e22",
      publicId: "batch_a13d9e22",
      recordType: "batch",
      checkpointId: "checkpoint_000185",
      publicationId: "publication_1e850185",
      proofId: "proof_7a850185",
      daReferenceId: "da_ref_72be91",
      publicationCheckpoint: 185,
      relationship: "PublicationRecord → readiness bundle",
      routeGeneration: 12,
      shardIds: [2, 7],
      freshness: "timestamp",
      observedAt: FIXTURE_OBSERVED_AT,
      mapping: "PublicationRecord"
    },
    {
      id: "batch_72bc108f",
      publicId: "batch_72bc108f",
      recordType: "batch",
      checkpointId: "checkpoint_000186",
      publicationId: "publication_3c860186",
      proofId: "proof_c2860186",
      daReferenceId: "da_ref_18ac40",
      publicationCheckpoint: 186,
      relationship: "PublishedBatch → PublicationRouteSnapshotV1",
      routeGeneration: 13,
      shardIds: [3, 9],
      freshness: "timestamp",
      observedAt: FIXTURE_OBSERVED_AT,
      mapping: "PublishedBatch"
    }
  ]);

  const EXPLORER_PUBLIC_EVIDENCE = deepFreeze([
    {
      id: "publication_6f840184",
      publicId: "publication_6f840184",
      recordType: "publication",
      checkpointId: "checkpoint_000184",
      batchId: "batch_4f91c7a0",
      state: "finalized",
      publicRoot: "root_84f2d18a…4c07",
      routeSnapshot: { routingGeneration: 12, activationCheckpoint: 180, shardIds: [2, 7], routeTableDigest: "route_12d4…8e31" },
      daReferenceId: "da_ref_72be91",
      mapping: "CheckpointPublicationV1 + PublicationRouteSnapshotV1"
    },
    {
      id: "proof_92840184",
      publicId: "proof_92840184",
      recordType: "proof",
      checkpointId: "checkpoint_000184",
      publicationId: "publication_6f840184",
      publicRoot: "root_84f2d18a…4c07",
      rootGeneration: "root_generation_1",
      proofFamily: "hjmt",
      shardLeafIndex: 0,
      verificationBoundary: "check_public_checkpoint_v1",
      mapping: "CheckpointPublicationProofV1"
    },
    {
      id: "da_ref_72be91",
      publicId: "da_ref_72be91",
      recordType: "da_reference",
      checkpointIds: ["checkpoint_000184", "checkpoint_000185"],
      providerFamily: "local_bridge",
      locatorKind: "opaque_provider_ref",
      opaqueProviderRef: "provider-ref:72be91",
      publishedHeight: 184,
      payloadCommitment: "payload_72be…91a4",
      archiveManifestRoot: "archive_8c10…d29f",
      mapping: "CheckpointDaReferenceV1"
    },
    {
      id: "publication_1e850185",
      publicId: "publication_1e850185",
      recordType: "publication",
      checkpointId: "checkpoint_000185",
      batchId: "batch_a13d9e22",
      state: "challenge_open",
      publicRoot: "root_85c1e419…1f92",
      routeSnapshot: { routingGeneration: 12, activationCheckpoint: 180, shardIds: [2, 7], routeTableDigest: "route_12d4…8e31" },
      daReferenceId: "da_ref_72be91",
      mapping: "PublicationRecord + PublicationRouteSnapshotV1"
    },
    {
      id: "proof_7a850185",
      publicId: "proof_7a850185",
      recordType: "proof",
      checkpointId: "checkpoint_000185",
      publicationId: "publication_1e850185",
      publicRoot: "root_85c1e419…1f92",
      rootGeneration: "root_generation_1",
      proofFamily: "hjmt",
      shardLeafIndex: 1,
      verificationBoundary: "check_public_checkpoint_route_v1",
      mapping: "CheckpointPublicationProofV1"
    },
    {
      id: "publication_3c860186",
      publicId: "publication_3c860186",
      recordType: "publication",
      checkpointId: "checkpoint_000186",
      batchId: "batch_72bc108f",
      state: "publication_ready",
      publicRoot: "root_86b41d77…c610",
      routeSnapshot: { routingGeneration: 13, activationCheckpoint: 186, shardIds: [3, 9], routeTableDigest: "route_13a8…41f0" },
      daReferenceId: "da_ref_18ac40",
      mapping: "CheckpointPublicationV1 + PublicationRouteSnapshotV1"
    },
    {
      id: "proof_c2860186",
      publicId: "proof_c2860186",
      recordType: "proof",
      checkpointId: "checkpoint_000186",
      publicationId: "publication_3c860186",
      publicRoot: "root_86b41d77…c610",
      rootGeneration: "root_generation_1",
      proofFamily: "hjmt",
      shardLeafIndex: 0,
      verificationBoundary: "check_public_checkpoint_route_v1",
      mapping: "CheckpointPublicationProofV1"
    },
    {
      id: "da_ref_18ac40",
      publicId: "da_ref_18ac40",
      recordType: "da_reference",
      checkpointIds: ["checkpoint_000186"],
      providerFamily: "archive_mirror",
      locatorKind: "opaque_provider_ref",
      opaqueProviderRef: "provider-ref:18ac40",
      publishedHeight: 186,
      payloadCommitment: "payload_18ac…40b7",
      archiveManifestRoot: "archive_c816…7ef4",
      mapping: "CheckpointDaReferenceV1"
    }
  ]);

  const EXPLORER_ROUTE_RECORDS = deepFreeze({
    "telemetry.explorer.overview": [{
      id: "explorer-public-scope",
      recordType: "scope_summary",
      checkpointCount: EXPLORER_CHECKPOINTS.length,
      batchCount: EXPLORER_BATCHES.length,
      evidenceCount: EXPLORER_PUBLIC_EVIDENCE.length,
      acceptedKinds: EXPLORER_PUBLIC_ID_PATTERNS.map(({ kind }) => kind),
      mapping: "Public evidence DTO boundary"
    }],
    "telemetry.explorer.search": [],
    "telemetry.explorer.checkpoints": EXPLORER_CHECKPOINTS,
    "telemetry.explorer.batches": EXPLORER_BATCHES,
    "telemetry.explorer.evidence": EXPLORER_PUBLIC_EVIDENCE
  });

  const EXPLORER_PUBLIC_RECORDS = new Map([
    ...EXPLORER_CHECKPOINTS,
    ...EXPLORER_BATCHES,
    ...EXPLORER_PUBLIC_EVIDENCE
  ].map((record) => [record.publicId, record]));

  function explorerIssue(code, message, recoveryAction) {
    return deepFreeze({ code, severity: code === "private_identifier" ? "danger" : "neutral", message, recoveryAction });
  }

  function classifyExplorerPublicId(value) {
    const normalized = typeof value === "string" ? value.trim() : "";
    if (!normalized) {
      return deepFreeze({
        status: "malformed",
        publicId: null,
        publicKind: null,
        record: null,
        issue: explorerIssue("empty_identifier", "Enter a supported public identifier.", "Use a checkpoint, batch, publication, proof, or opaque DA-reference ID.")
      });
    }
    if (normalized.length > 80 || EXPLORER_PRIVATE_HINT.test(normalized)) {
      return deepFreeze({
        status: "private",
        publicId: null,
        publicKind: null,
        record: null,
        issue: explorerIssue("private_identifier", "The identifier is outside the public-evidence boundary and was not queried.", "Use an intentionally public Explorer identifier.")
      });
    }
    const descriptor = EXPLORER_PUBLIC_ID_PATTERNS.find(({ pattern }) => pattern.test(normalized));
    if (!descriptor) {
      const supportedPrefix = /^(?:checkpoint|batch|publication|proof|da_ref)_/.test(normalized);
      return deepFreeze({
        status: supportedPrefix ? "malformed" : "unsupported",
        publicId: null,
        publicKind: null,
        record: null,
        issue: explorerIssue(
          supportedPrefix ? "malformed_identifier" : "unsupported_identifier",
          supportedPrefix
            ? "The public identifier has an invalid shape and was rejected."
            : "This identifier family is not supported by the public Explorer.",
          "Use the exact public ID shown by Checkpoints, Batches, or Public evidence."
        )
      });
    }
    if (EXPLORER_STALE_PUBLIC_IDS.includes(normalized)) {
      return deepFreeze({
        status: "stale",
        publicId: null,
        publicKind: descriptor.kind,
        record: null,
        issue: explorerIssue("stale_identifier", "The fixture knows this public ID only as stale evidence and will not promote it.", "Open a current checkpoint or retry after authoritative refresh.")
      });
    }
    const record = EXPLORER_PUBLIC_RECORDS.get(normalized);
    if (!record) {
      return deepFreeze({
        status: "unknown",
        publicId: null,
        publicKind: descriptor.kind,
        record: null,
        issue: explorerIssue("unknown_identifier", "No current public evidence matches this well-formed identifier.", "Check the public ID and retry.")
      });
    }
    return deepFreeze({
      status: "found",
      publicId: normalized,
      publicKind: descriptor.kind,
      record,
      issue: null
    });
  }

  function watcherSource(sourceId) {
    const source = WATCHER_SOURCE_OPTIONS.find(({ id }) => id === sourceId);
    if (!source) throw new TypeError(`Unknown Watchers source: ${String(sourceId)}`);
    return source;
  }

  function filteredWatcherRecords(routeId, { severity = "all", kind = "all" } = {}) {
    if (!["all", "info", "warn", "critical"].includes(severity)) {
      throw new TypeError(`Unknown Watchers severity filter: ${String(severity)}`);
    }
    const records = WATCHER_ROUTE_RECORDS[routeId] || [];
    return records.filter((record) => {
      if (severity !== "all" && record.severity !== severity) return false;
      const recordKind = record.kind || record.alertKind;
      return kind === "all" || recordKind === kind;
    });
  }

  function filteredExplorerRecords(routeId, { kind = "all" } = {}) {
    const allowedKinds = ["all", "publication", "proof", "da_reference"];
    if (!allowedKinds.includes(kind)) {
      throw new TypeError(`Unknown Explorer evidence filter: ${String(kind)}`);
    }
    const records = EXPLORER_ROUTE_RECORDS[routeId] || [];
    if (routeId !== "telemetry.explorer.evidence" || kind === "all") return records;
    return records.filter((record) => record.recordType === kind);
  }

  function assertTelemetryRequest({ capabilityId, routeId, scenarioId, generation }) {
    const capability = demo.capabilityProfile(capabilityId);
    if (!capability || !String(capabilityId).startsWith("telemetry.")) {
      throw new TypeError(`Unknown telemetry capability: ${String(capabilityId)}`);
    }
    const routeNode = demo.navigationNodeForRoute(routeId);
    if (!demo.PORT_CONTRACT.telemetryRoutes.includes(routeId) || routeNode?.capabilityId !== capabilityId) {
      throw new TypeError(`Route ${String(routeId)} does not belong to ${capabilityId}.`);
    }
    if (!demo.PORT_CONTRACT.telemetryResultStates.includes(scenarioId)) {
      throw new TypeError(`Unknown telemetry scenario: ${String(scenarioId)}`);
    }
    if (!Number.isSafeInteger(generation) || generation < 0) {
      throw new TypeError("Telemetry request generation must be a non-negative safe integer.");
    }
    return capability;
  }

  function assertTelemetryObservation(observation) {
    const capability = observation?.capability;
    if (!demo.PORT_CONTRACT.telemetryResultStates.includes(observation?.status)) {
      throw new TypeError("Telemetry observation has an invalid result state.");
    }
    if (!demo.PORT_CONTRACT.maturity.includes(capability?.maturity)) {
      throw new TypeError("Telemetry observation has invalid maturity.");
    }
    if (!demo.PORT_CONTRACT.availability.includes(capability?.availability)) {
      throw new TypeError("Telemetry observation has invalid availability.");
    }
    if (!demo.PORT_CONTRACT.evidenceSources.includes(capability?.evidenceSource)) {
      throw new TypeError("Telemetry observation has an invalid evidence source.");
    }
    if (!demo.PORT_CONTRACT.freshness.includes(capability?.freshness)) {
      throw new TypeError("Telemetry observation has invalid freshness.");
    }
    if (!demo.PORT_CONTRACT.presentationModes.includes(capability?.presentationMode)) {
      throw new TypeError("Telemetry observation has an invalid presentation mode.");
    }
    if (observation.issue && !SAFE_ISSUE_SEVERITIES.includes(observation.issue.severity)) {
      throw new TypeError("Telemetry observation has an invalid issue severity.");
    }
    if (capability.evidenceSource === "fixture" && observation.source?.sourceId !== FIXTURE_SOURCE_ID) {
      throw new TypeError("Fixture telemetry must identify the deterministic fixture source.");
    }
    if (capability.evidenceSource === "none" && observation.source?.sourceId !== null) {
      throw new TypeError("Source-free telemetry must not invent a source identifier.");
    }
    if (observation.data && observation.data.total !== observation.data.records.length) {
      throw new TypeError("Telemetry observation record count is inconsistent.");
    }
    return observation;
  }

  function createMockTelemetryGateway() {
    function readObservation({
      capabilityId,
      routeId,
      scenario: scenarioId = "unavailable",
      generation = 0
    } = {}) {
      const baseCapability = assertTelemetryRequest({
        capabilityId,
        routeId,
        scenarioId,
        generation
      });
      const selected = TELEMETRY_SCENARIOS[scenarioId];
      const records = selected.records === null
        ? null
        : selected.records.map((record) => ({ ...record, routeId }));
      const observation = deepFreeze({
        schemaVersion: "1.0.0",
        request: {
          requestKey: `telemetry:${routeId}`,
          generation,
          capabilityId,
          routeId
        },
        status: scenarioId,
        capability: {
          maturity: baseCapability.maturity,
          availability: selected.availability,
          evidenceSource: selected.evidenceSource,
          freshness: selected.freshness,
          presentationMode: baseCapability.presentationMode
        },
        source: {
          sourceId: selected.evidenceSource === "fixture" ? FIXTURE_SOURCE_ID : null,
          observedAt: selected.observedAt
        },
        data: records === null ? null : {
          total: records.length,
          records
        },
        retryable: selected.retryable,
        issue: selected.issue
      });
      return assertTelemetryObservation(observation);
    }

    function readWatcherView({
      routeId,
      scenario: scenarioId = "success",
      sourceId = "runtime_projection",
      generation = 0,
      filters = {}
    } = {}) {
      const selectedSource = watcherSource(sourceId);
      const base = readObservation({
        capabilityId: "telemetry.watchers",
        routeId,
        scenario: scenarioId,
        generation
      });
      const hasFixtureRecords = ["success", "degraded", "empty"].includes(base.status);
      const allRecords = hasFixtureRecords
        ? filteredWatcherRecords(routeId, filters)
        : [];
      const records = base.status === "empty"
        ? []
        : base.status === "degraded"
          ? allRecords.slice(0, 1)
          : allRecords;
      const observation = deepFreeze({
        ...base,
        request: {
          ...base.request,
          sourceId,
          filters: {
            severity: filters.severity || "all",
            kind: filters.kind || "all"
          }
        },
        source: {
          ...base.source,
          datasetId: base.capability.evidenceSource === "fixture" ? selectedSource.id : null,
          datasetLabel: base.capability.evidenceSource === "fixture" ? selectedSource.label : null,
          authority: selectedSource.authority
        },
        data: hasFixtureRecords
          ? {
              total: records.length,
              records: records.map((record) => ({
                ...record,
                fixtureSource: selectedSource.id
              }))
            }
          : null
      });
      return assertTelemetryObservation(observation);
    }

    function readExplorerView({
      routeId,
      scenario: scenarioId = "success",
      generation = 0,
      filters = {}
    } = {}) {
      const base = readObservation({
        capabilityId: "telemetry.explorer",
        routeId,
        scenario: scenarioId,
        generation
      });
      const hasFixtureRecords = ["success", "degraded", "empty"].includes(base.status);
      const allRecords = hasFixtureRecords ? filteredExplorerRecords(routeId, filters) : [];
      const records = base.status === "empty"
        ? []
        : base.status === "degraded"
          ? allRecords.slice(0, 1)
          : allRecords;
      const observation = deepFreeze({
        ...base,
        request: {
          ...base.request,
          filters: { kind: filters.kind || "all" }
        },
        source: {
          ...base.source,
          datasetId: base.capability.evidenceSource === "fixture" ? "public_evidence_fixture" : null,
          datasetLabel: base.capability.evidenceSource === "fixture" ? "Deterministic public evidence" : null,
          authority: "storage public proof surface"
        },
        data: hasFixtureRecords
          ? {
              total: records.length,
              records
            }
          : null
      });
      return assertTelemetryObservation(observation);
    }

    function searchExplorerPublicId({
      query,
      scenario: scenarioId = "success",
      generation = 0
    } = {}) {
      const base = readObservation({
        capabilityId: "telemetry.explorer",
        routeId: "telemetry.explorer.search",
        scenario: scenarioId,
        generation
      });
      if (base.status !== "success") {
        return deepFreeze({
          schemaVersion: "explorer-search-v1",
          status: base.status,
          publicId: null,
          publicKind: null,
          record: null,
          issue: base.issue || explorerIssue(
            "public_source_not_ready",
            "The public evidence source is not ready, so the identifier was not queried.",
            "Retry after a successful authoritative refresh."
          ),
          capability: base.capability,
          source: base.source,
          request: base.request
        });
      }
      const result = classifyExplorerPublicId(query);
      return deepFreeze({
        schemaVersion: "explorer-search-v1",
        ...result,
        capability: base.capability,
        source: {
          ...base.source,
          datasetId: "public_evidence_fixture",
          authority: "storage public proof surface"
        },
        request: base.request
      });
    }

    function resolveExplorerDeepLink({ publicId } = {}) {
      const result = classifyExplorerPublicId(publicId);
      if (result.status !== "found") {
        return deepFreeze({
          ok: false,
          publicId: null,
          publicKind: null,
          routeId: null,
          error: {
            code: "invalid_public_deep_link",
            message: "The Watcher link was rejected before Explorer navigation."
          }
        });
      }
      const routeId = result.record.recordType === "checkpoint"
        ? "telemetry.explorer.checkpoints"
        : result.record.recordType === "batch"
          ? "telemetry.explorer.batches"
          : "telemetry.explorer.evidence";
      return deepFreeze({
        ok: true,
        publicId: result.publicId,
        publicKind: result.publicKind,
        routeId
      });
    }

    function prepareWatcherEvidenceExport({
      alertId,
      sourceId = "runtime_projection"
    } = {}) {
      const selectedSource = watcherSource(sourceId);
      const alert = WATCHER_ROUTE_RECORDS["telemetry.watchers.alerts"]
        .find(({ id }) => id === alertId);
      const evidence = WATCHER_ROUTE_RECORDS["telemetry.watchers.evidence"]
        .find((record) => record.alertId === alertId);
      if (!alert || !evidence) {
        throw new TypeError(`Unknown Watchers alert for evidence export: ${String(alertId)}`);
      }
      return deepFreeze({
        schemaVersion: "watcher-evidence-export-v1",
        exportId: `export-${evidence.id}`,
        preparedAt: FIXTURE_OBSERVED_AT,
        evidenceSource: "fixture",
        presentationMode: "roadmap_preview",
        source: {
          datasetId: selectedSource.id,
          authority: selectedSource.authority
        },
        alert: {
          id: alert.id,
          kind: alert.kind,
          severity: alert.severity,
          subject: alert.subject,
          observedAt: alert.observedAt,
          affectedPublicIds: alert.affectedPublicIds
        },
        evidence: {
          id: evidence.id,
          sequence: evidence.sequence,
          batchId: evidence.batchId,
          checkpointId: evidence.checkpointId,
          providerRef: evidence.providerRef,
          bindings: evidence.bindings
        },
        redactions: [
          "wallet labels and balances excluded",
          "private addressing and communication fields excluded",
          "private network-path and mailbox records excluded",
          "secret and key material excluded"
        ]
      });
    }

    return Object.freeze({
      contractVersion: demo.PORT_CONTRACT.version,
      scenarioIds: demo.PORT_CONTRACT.telemetryResultStates,
      watcherSources: WATCHER_SOURCE_OPTIONS,
      readObservation,
      readWatcherView,
      prepareWatcherEvidenceExport,
      readExplorerView,
      searchExplorerPublicId,
      resolveExplorerDeepLink
    });
  }

  Object.assign(root.Z00ZDemo, {
    TELEMETRY_SCENARIOS,
    WATCHER_ROUTE_RECORDS,
    WATCHER_SOURCE_OPTIONS,
    EXPLORER_CHECKPOINTS,
    EXPLORER_BATCHES,
    EXPLORER_PUBLIC_EVIDENCE,
    EXPLORER_ROUTE_RECORDS,
    EXPLORER_PUBLIC_ID_PATTERNS,
    classifyExplorerPublicId,
    assertTelemetryObservation,
    createMockTelemetryGateway
  });
})(typeof window === "undefined" ? globalThis : window);
