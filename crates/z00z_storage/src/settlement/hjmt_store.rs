use std::collections::BTreeMap;

use jmt::{proof::SparseMerkleProof, JellyfishMerkleTree, KeyHash, RootHash, Sha256Jmt, Version};
use sha2::Sha256;

use super::{
    proof_batch::{validate_live_jmt_operations_v2, JmtSha256V2, JmtTreeRoleV2, JmtUpdateTraceV2},
    tree_id::HjmtTreeId,
    SettlementStoreError,
};
use crate::backend::{
    codec::map_jmt_err,
    memory::{apply_batch, KeyValueOp, MemTreeInner, MemTreeStore},
};

#[derive(Clone)]
pub(super) struct HjmtStoreSnap {
    trees: BTreeMap<HjmtTreeId, MemTreeInner>,
    latest_versions: BTreeMap<HjmtTreeId, Version>,
}

#[derive(Clone, Default)]
pub(super) struct HjmtTreeSnap {
    inner: MemTreeInner,
    latest_version: Version,
}

#[derive(Default)]
pub(super) struct HjmtStore {
    trees: BTreeMap<HjmtTreeId, MemTreeStore>,
    latest_versions: BTreeMap<HjmtTreeId, Version>,
}

impl HjmtStore {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn snap(&self) -> HjmtStoreSnap {
        let trees = self
            .trees
            .iter()
            .map(|(tree_id, store)| (*tree_id, store.snap()))
            .collect();
        HjmtStoreSnap {
            trees,
            latest_versions: self.latest_versions.clone(),
        }
    }

    pub(super) fn restore(&mut self, snap: HjmtStoreSnap) {
        self.trees.clear();
        for (tree_id, inner) in snap.trees {
            let store = MemTreeStore::new();
            store.restore(inner);
            self.trees.insert(tree_id, store);
        }
        self.latest_versions = snap.latest_versions;
    }

    pub(super) fn tree_snap(&self, tree_id: HjmtTreeId) -> HjmtTreeSnap {
        HjmtTreeSnap {
            inner: self
                .trees
                .get(&tree_id)
                .map(MemTreeStore::snap)
                .unwrap_or_default(),
            latest_version: self.latest_versions.get(&tree_id).copied().unwrap_or(0),
        }
    }

    pub(super) fn restore_tree(&mut self, tree_id: HjmtTreeId, snap: HjmtTreeSnap) {
        let store = self.trees.entry(tree_id).or_default();
        store.restore(snap.inner);
        self.latest_versions.insert(tree_id, snap.latest_version);
    }

    pub(super) fn commit_snap_with_update_trace(
        tree: JmtTreeRoleV2,
        snap: HjmtTreeSnap,
        mut ops: Vec<KeyValueOp>,
        version: Version,
    ) -> Result<(RootHash, JmtUpdateTraceV2, HjmtTreeSnap), SettlementStoreError> {
        ops.sort_unstable_by_key(|operation| operation.0 .0);
        if ops.is_empty() || ops.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(SettlementStoreError::Proof(
                super::ProofChkErr::JmtUpdateTraceCanonical,
            ));
        }
        validate_live_jmt_operations_v2(&ops)?;
        let store = MemTreeStore::new();
        store.restore(snap.inner);
        if version > 0 {
            let _ = ensure_store_version(&store, snap.latest_version, version - 1)?;
        }
        let jmt = Sha256Jmt::new(&store);
        let old_root = if version == 0 {
            RootHash(*b"SPARSE_MERKLE_PLACEHOLDER_HASH__")
        } else {
            jmt.get_root_hash_option(version - 1)
                .map_err(map_jmt_err)?
                .unwrap_or(RootHash(*b"SPARSE_MERKLE_PLACEHOLDER_HASH__"))
        };
        let traced_jmt = JellyfishMerkleTree::<_, JmtSha256V2>::new(&store);
        let (root, proof, batch) = traced_jmt
            .put_value_set_with_proof(ops.clone(), version)
            .map_err(map_jmt_err)?;
        let trace = JmtUpdateTraceV2::from_update(
            tree,
            version.saturating_sub(1),
            version,
            old_root,
            root,
            ops,
            proof,
        )?;
        apply_batch(&store, batch)?;
        Ok((
            root,
            trace,
            HjmtTreeSnap {
                inner: store.snap(),
                latest_version: version,
            },
        ))
    }

    pub(super) fn ensure_snap(
        snap: HjmtTreeSnap,
        version: Version,
    ) -> Result<HjmtTreeSnap, SettlementStoreError> {
        let store = MemTreeStore::new();
        store.restore(snap.inner);
        let latest_version = ensure_store_version(&store, snap.latest_version, version)?;
        Ok(HjmtTreeSnap {
            inner: store.snap(),
            latest_version,
        })
    }

    pub(super) fn get_proof(
        &self,
        tree_id: HjmtTreeId,
        key: KeyHash,
        version: Version,
    ) -> Result<SparseMerkleProof<Sha256>, SettlementStoreError> {
        let store = self
            .trees
            .get(&tree_id)
            .ok_or(SettlementStoreError::EmptyTree)?;
        let jmt = Sha256Jmt::new(store);
        let (_value, proof) = jmt.get_with_proof(key, version).map_err(map_jmt_err)?;
        Ok(proof)
    }
}

fn ensure_store_version(
    store: &MemTreeStore,
    latest: Version,
    version: Version,
) -> Result<Version, SettlementStoreError> {
    if version == 0 {
        return Ok(latest.max(version));
    }
    let jmt = Sha256Jmt::new(store);
    let mut next_latest = latest;
    for next_version in latest.saturating_add(1)..=version {
        let (_root, batch) = jmt
            .put_value_set(Vec::new(), next_version)
            .map_err(map_jmt_err)?;
        apply_batch(store, batch)?;
        next_latest = next_version;
    }
    Ok(next_latest.max(version))
}

#[cfg(test)]
mod tests {
    use jmt::KeyHash;

    use super::super::proof_batch::{JmtMutationCaseV2, SettlementUpdateTraceEnvelopeV2};
    use super::{HjmtStore, HjmtTreeSnap, JmtTreeRoleV2};
    use crate::settlement::{
        keys::{definition_key, serial_key},
        BucketId, BucketRootLeaf, DefinitionId, DefinitionRootLeaf, RootGeneration, SerialId,
        SerialRootLeaf,
    };

    fn hierarchy_update_traces(
        terminal_root_override: Option<[u8; 32]>,
    ) -> (
        super::JmtUpdateTraceV2,
        super::JmtUpdateTraceV2,
        super::JmtUpdateTraceV2,
        super::JmtUpdateTraceV2,
    ) {
        let definition_id = DefinitionId::new([0x41; 32]);
        let serial_id = SerialId::new(7);
        let bucket_id = BucketId::new([0x52; 32]);
        let (terminal_root, terminal, _) = HjmtStore::commit_snap_with_update_trace(
            JmtTreeRoleV2::Terminal(
                definition_id.into_bytes(),
                serial_id.get(),
                bucket_id.into_bytes(),
            ),
            HjmtTreeSnap::default(),
            vec![(KeyHash([0x63; 32]), Some(vec![0x74]))],
            0,
        )
        .expect("terminal update");
        let bucket_leaf = BucketRootLeaf {
            definition_id,
            serial_id,
            bucket_id,
            terminal_jmt_root: terminal_root_override.unwrap_or(terminal_root.0),
            bucket_policy_id: [0x85; 32],
        };
        let (bucket_root, bucket, _) = HjmtStore::commit_snap_with_update_trace(
            JmtTreeRoleV2::Bucket(definition_id.into_bytes(), serial_id.get()),
            HjmtTreeSnap::default(),
            vec![(KeyHash(bucket_id.into_bytes()), Some(bucket_leaf.encode()))],
            0,
        )
        .expect("bucket update");
        let serial_leaf = SerialRootLeaf {
            definition_id,
            serial_id,
            serial_root: bucket_root.0,
        };
        let (serial_root, serial, _) = HjmtStore::commit_snap_with_update_trace(
            JmtTreeRoleV2::Serial(definition_id.into_bytes()),
            HjmtTreeSnap::default(),
            vec![(
                serial_key(definition_id, serial_id),
                Some(serial_leaf.encode()),
            )],
            0,
        )
        .expect("serial update");
        let definition_leaf = DefinitionRootLeaf {
            definition_id,
            definition_root: serial_root.0,
        };
        let (_definition_root, definition, _) = HjmtStore::commit_snap_with_update_trace(
            JmtTreeRoleV2::Definition,
            HjmtTreeSnap::default(),
            vec![(
                definition_key(definition_id),
                Some(definition_leaf.encode()),
            )],
            0,
        )
        .expect("definition update");
        (terminal, bucket, serial, definition)
    }

    fn hierarchy_delete_traces() -> (
        super::JmtUpdateTraceV2,
        super::JmtUpdateTraceV2,
        super::JmtUpdateTraceV2,
        super::JmtUpdateTraceV2,
    ) {
        let definition_id = DefinitionId::new([0x41; 32]);
        let serial_id = SerialId::new(7);
        let bucket_id = BucketId::new([0x52; 32]);
        let terminal_role = JmtTreeRoleV2::Terminal(
            definition_id.into_bytes(),
            serial_id.get(),
            bucket_id.into_bytes(),
        );
        let (terminal_root, _terminal_put, terminal_snap) =
            HjmtStore::commit_snap_with_update_trace(
                terminal_role.clone(),
                HjmtTreeSnap::default(),
                vec![(KeyHash([0x63; 32]), Some(vec![0x74]))],
                0,
            )
            .expect("terminal insert");
        let (_terminal_empty, terminal_delete, _) = HjmtStore::commit_snap_with_update_trace(
            terminal_role,
            terminal_snap,
            vec![(KeyHash([0x63; 32]), None)],
            1,
        )
        .expect("terminal delete");

        let bucket_role = JmtTreeRoleV2::Bucket(definition_id.into_bytes(), serial_id.get());
        let bucket_leaf = BucketRootLeaf {
            definition_id,
            serial_id,
            bucket_id,
            terminal_jmt_root: terminal_root.0,
            bucket_policy_id: [0x85; 32],
        };
        let (bucket_root, _bucket_put, bucket_snap) = HjmtStore::commit_snap_with_update_trace(
            bucket_role.clone(),
            HjmtTreeSnap::default(),
            vec![(KeyHash(bucket_id.into_bytes()), Some(bucket_leaf.encode()))],
            0,
        )
        .expect("bucket insert");
        let (_bucket_empty, bucket_delete, _) = HjmtStore::commit_snap_with_update_trace(
            bucket_role,
            bucket_snap,
            vec![(KeyHash(bucket_id.into_bytes()), None)],
            1,
        )
        .expect("bucket delete");

        let serial_role = JmtTreeRoleV2::Serial(definition_id.into_bytes());
        let serial_leaf = SerialRootLeaf {
            definition_id,
            serial_id,
            serial_root: bucket_root.0,
        };
        let (serial_root, _serial_put, serial_snap) = HjmtStore::commit_snap_with_update_trace(
            serial_role.clone(),
            HjmtTreeSnap::default(),
            vec![(
                serial_key(definition_id, serial_id),
                Some(serial_leaf.encode()),
            )],
            0,
        )
        .expect("serial insert");
        let (_serial_empty, serial_delete, _) = HjmtStore::commit_snap_with_update_trace(
            serial_role,
            serial_snap,
            vec![(serial_key(definition_id, serial_id), None)],
            1,
        )
        .expect("serial delete");

        let definition_leaf = DefinitionRootLeaf {
            definition_id,
            definition_root: serial_root.0,
        };
        let (_definition_root, _definition_put, definition_snap) =
            HjmtStore::commit_snap_with_update_trace(
                JmtTreeRoleV2::Definition,
                HjmtTreeSnap::default(),
                vec![(
                    definition_key(definition_id),
                    Some(definition_leaf.encode()),
                )],
                0,
            )
            .expect("definition insert");
        let (_definition_empty, definition_delete, _) = HjmtStore::commit_snap_with_update_trace(
            JmtTreeRoleV2::Definition,
            definition_snap,
            vec![(definition_key(definition_id), None)],
            1,
        )
        .expect("definition delete");
        (
            terminal_delete,
            bucket_delete,
            serial_delete,
            definition_delete,
        )
    }

    #[test]
    fn traced_jmt_update_is_canonical_and_reverifies() {
        let (root, trace, snap) = HjmtStore::commit_snap_with_update_trace(
            JmtTreeRoleV2::Definition,
            HjmtTreeSnap::default(),
            vec![(KeyHash([7u8; 32]), Some(vec![9u8; 3]))],
            0,
        )
        .expect("first traced JMT update");
        assert_eq!(trace.old_root(), *b"SPARSE_MERKLE_PLACEHOLDER_HASH__");
        assert_eq!(trace.new_root(), root.0);
        assert_eq!(trace.operations().len(), 1);
        assert_eq!(
            trace.semantic_cases_for_test().expect("typed proof case"),
            vec![JmtMutationCaseV2::EmptyInsert]
        );
        trace.verify_native().expect("native update proof");

        let bytes = trace.canonical_bytes().expect("canonical trace bytes");
        let decoded = super::JmtUpdateTraceV2::from_canon(&bytes).expect("strict decode");
        assert_eq!(decoded.new_root(), root.0);
        assert_eq!(decoded.operations().len(), 1);

        let envelope =
            SettlementUpdateTraceEnvelopeV2::new(RootGeneration::SettlementV2, vec![decoded])
                .expect("V2 trace envelope");
        let envelope_bytes = envelope.canonical_bytes().expect("envelope bytes");
        assert_eq!(
            SettlementUpdateTraceEnvelopeV2::from_canon(&envelope_bytes)
                .expect("strict envelope decode")
                .trace_digest(),
            envelope.trace_digest()
        );
        let mut trailing = envelope_bytes.clone();
        trailing.push(0);
        assert!(SettlementUpdateTraceEnvelopeV2::from_canon(&trailing).is_err());

        let mut proof_mutation = envelope_bytes;
        let last = proof_mutation
            .last_mut()
            .expect("nonempty canonical envelope wire");
        *last ^= 1;
        assert!(SettlementUpdateTraceEnvelopeV2::from_canon(&proof_mutation).is_err());

        let noop = SettlementUpdateTraceEnvelopeV2::new_noop(RootGeneration::SettlementV2)
            .expect("typed noop envelope");
        let noop_bytes = noop.canonical_bytes().expect("noop envelope bytes");
        assert!(SettlementUpdateTraceEnvelopeV2::from_canon(&noop_bytes)
            .expect("strict noop envelope decode")
            .is_noop());
        let mut kind_mutation = noop_bytes.clone();
        kind_mutation[2] = 1;
        assert!(SettlementUpdateTraceEnvelopeV2::from_canon(&kind_mutation).is_err());
        let mut digest_mutation = noop_bytes;
        digest_mutation[3] ^= 1;
        assert!(SettlementUpdateTraceEnvelopeV2::from_canon(&digest_mutation).is_err());

        let (next_root, next_trace, _next_snap) = HjmtStore::commit_snap_with_update_trace(
            JmtTreeRoleV2::Definition,
            snap,
            vec![(KeyHash([7u8; 32]), None)],
            1,
        )
        .expect("second traced JMT update");
        assert_eq!(next_trace.old_root(), root.0);
        assert_eq!(next_trace.new_root(), next_root.0);
        next_trace.verify_native().expect("native delete proof");
        assert_eq!(
            next_trace
                .semantic_cases_for_test()
                .expect("typed delete proof case"),
            vec![JmtMutationCaseV2::DeleteToEmpty]
        );
    }

    #[test]
    fn first_update_at_later_global_version_uses_canonical_null_root() {
        let (root, trace, _snap) = HjmtStore::commit_snap_with_update_trace(
            JmtTreeRoleV2::Terminal([3u8; 32], 4, [5u8; 32]),
            HjmtTreeSnap::default(),
            vec![(KeyHash([6u8; 32]), Some(vec![7u8; 8]))],
            9,
        )
        .expect("late-born JMT update");

        assert_eq!(trace.old_root(), *b"SPARSE_MERKLE_PLACEHOLDER_HASH__");
        assert_eq!(trace.new_root(), root.0);
        trace
            .verify_native()
            .expect("late-born native update proof");
    }

    #[test]
    fn traced_jmt_update_executes_every_mutation_case() {
        let key_a = KeyHash([0x00; 32]);
        let key_b = KeyHash([0x80; 32]);
        let key_c = KeyHash([0x40; 32]);
        let tree = JmtTreeRoleV2::PathIndex;

        let (_root_a, insert, snap_a) = HjmtStore::commit_snap_with_update_trace(
            tree.clone(),
            HjmtTreeSnap::default(),
            vec![(key_a, Some(vec![1]))],
            0,
        )
        .expect("empty insert");
        assert_eq!(
            insert.semantic_cases_for_test().expect("empty insert case"),
            vec![JmtMutationCaseV2::EmptyInsert]
        );

        let (_root_updated, update, snap_updated) = HjmtStore::commit_snap_with_update_trace(
            tree.clone(),
            snap_a,
            vec![(key_a, Some(vec![2]))],
            1,
        )
        .expect("existing update");
        assert_eq!(
            update
                .semantic_cases_for_test()
                .expect("existing update case"),
            vec![JmtMutationCaseV2::ExistingUpdate]
        );

        let (_root_split, split, snap_split) = HjmtStore::commit_snap_with_update_trace(
            tree.clone(),
            snap_updated,
            vec![(key_b, Some(vec![3]))],
            2,
        )
        .expect("split insert");
        assert!(matches!(
            split
                .semantic_cases_for_test()
                .expect("split insert case")
                .as_slice(),
            [JmtMutationCaseV2::SplitInsert { .. }]
        ));

        let (_root_coalesced, coalesce, _snap_coalesced) =
            HjmtStore::commit_snap_with_update_trace(
                tree.clone(),
                snap_split.clone(),
                vec![(key_b, None)],
                3,
            )
            .expect("leaf coalesce delete");
        assert_eq!(
            coalesce
                .semantic_cases_for_test()
                .expect("leaf coalesce case"),
            vec![JmtMutationCaseV2::DeleteCoalesceLeaf]
        );

        let (_root_three, third_insert, snap_three) = HjmtStore::commit_snap_with_update_trace(
            tree.clone(),
            snap_split,
            vec![(key_c, Some(vec![4]))],
            3,
        )
        .expect("third insert");
        assert!(matches!(
            third_insert
                .semantic_cases_for_test()
                .expect("third split insert case")
                .as_slice(),
            [JmtMutationCaseV2::SplitInsert { .. }]
        ));

        let (_root_preserved, preserve, _snap_preserved) =
            HjmtStore::commit_snap_with_update_trace(tree, snap_three, vec![(key_b, None)], 4)
                .expect("internal-preserving delete");
        assert_eq!(
            preserve
                .semantic_cases_for_test()
                .expect("internal-preserving case"),
            vec![JmtMutationCaseV2::DeletePreserveInternal]
        );
    }

    #[test]
    fn hierarchy_semantic_machine_binds_each_changed_child_root_and_order() {
        let (terminal, bucket, serial, definition) = hierarchy_update_traces(None);
        let expected_definition_root = definition.new_root();
        SettlementUpdateTraceEnvelopeV2::new(
            RootGeneration::SettlementV2,
            vec![
                terminal.clone(),
                bucket.clone(),
                serial.clone(),
                definition.clone(),
            ],
        )
        .expect("canonical hierarchy envelope")
        .verify_hierarchy_semantics(expected_definition_root)
        .expect("exact child-root promotion chain");

        let wrong_order = SettlementUpdateTraceEnvelopeV2::new(
            RootGeneration::SettlementV2,
            vec![bucket, terminal, serial, definition],
        )
        .expect("native-valid reordered envelope");
        assert!(wrong_order
            .verify_hierarchy_semantics(expected_definition_root)
            .is_err());

        let (terminal, substituted_bucket, serial, definition) =
            hierarchy_update_traces(Some([0x99; 32]));
        let substituted = SettlementUpdateTraceEnvelopeV2::new(
            RootGeneration::SettlementV2,
            vec![terminal, substituted_bucket, serial, definition.clone()],
        )
        .expect("native-valid substituted hierarchy envelope");
        assert!(substituted
            .verify_hierarchy_semantics(definition.new_root())
            .is_err());
    }

    #[test]
    fn hierarchy_rejects_unused_child() {
        let (terminal, bucket, serial, definition) = hierarchy_update_traces(None);
        let (_root, unused_terminal, _) = HjmtStore::commit_snap_with_update_trace(
            JmtTreeRoleV2::Terminal([0x44; 32], 9, [0x55; 32]),
            HjmtTreeSnap::default(),
            vec![(KeyHash([0x66; 32]), Some(vec![0x77]))],
            0,
        )
        .expect("unlinked but native-valid terminal update");
        let envelope = SettlementUpdateTraceEnvelopeV2::new(
            RootGeneration::SettlementV2,
            vec![
                terminal,
                unused_terminal,
                bucket,
                serial,
                definition.clone(),
            ],
        )
        .expect("native-valid envelope with unused child update");
        assert!(envelope
            .verify_hierarchy_semantics(definition.new_root())
            .is_err());
    }

    #[test]
    fn hierarchy_accepts_empty_chain() {
        let (terminal, bucket, serial, definition) = hierarchy_delete_traces();
        SettlementUpdateTraceEnvelopeV2::new(
            RootGeneration::SettlementV2,
            vec![terminal, bucket, serial, definition.clone()],
        )
        .expect("canonical deletion envelope")
        .verify_hierarchy_semantics(definition.new_root())
        .expect("exact child-root deletion chain");
    }

    #[test]
    fn traced_jmt_update_canonicalizes_keys_and_rejects_duplicates() {
        let (_root, trace, _snap) = HjmtStore::commit_snap_with_update_trace(
            JmtTreeRoleV2::PathIndex,
            HjmtTreeSnap::default(),
            vec![
                (KeyHash([9u8; 32]), Some(vec![1u8])),
                (KeyHash([1u8; 32]), Some(vec![2u8])),
            ],
            0,
        )
        .expect("unordered caller input is canonicalized before the one update");
        assert_eq!(trace.operations()[0].key(), [1u8; 32]);
        assert_eq!(trace.operations()[1].key(), [9u8; 32]);

        assert!(HjmtStore::commit_snap_with_update_trace(
            JmtTreeRoleV2::PathIndex,
            HjmtTreeSnap::default(),
            vec![
                (KeyHash([3u8; 32]), Some(vec![1u8])),
                (KeyHash([3u8; 32]), Some(vec![2u8])),
            ],
            0,
        )
        .is_err());
    }
}
