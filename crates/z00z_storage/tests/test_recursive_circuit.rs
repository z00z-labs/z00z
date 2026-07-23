//! Release-profile contract target for the sole recursive V2 circuit owner.
//!
//! Full private R1CS/proof execution remains in `checkpoint::nova::tests`; this
//! target is the stable executable selected by T4 profiling tools and guards
//! its production owner, shape, continuity relation, and terminal semantics.

const NOVA: &str = include_str!("../src/checkpoint/nova.rs");

#[test]
fn test_circuit_owner_shape_exact() {
    assert_eq!(NOVA.matches("struct CheckpointNovaCircuitV2").count(), 1);
    assert_eq!(NOVA.matches("struct CheckpointNovaRunnerV2").count(), 1);
    assert!(NOVA.contains("const RUNNING_STATE_ARITY_V2: usize ="));
    assert!(NOVA.contains("assert_eq!(metrics.constraints, 809_802)"));
    assert!(NOVA.contains("assert_eq!(metrics.auxiliaries, 675_408)"));
    assert!(NOVA.contains("assert_eq!(metrics.nonzeros, 3_332_400)"));
    assert!(NOVA.contains("fn finalized_successor_digest_words"));
    assert!(NOVA.contains("next_block_prior_finalized_state_link_"));
}

#[test]
fn test_snapshot_terminal_is_explicit() {
    let continuous = NOVA
        .find("pub(crate) struct NovaContinuousSessionV2")
        .expect("one continuous session owner");
    let continuous_owner = &NOVA[continuous..];
    assert_eq!(
        continuous_owner
            .matches("pub(crate) fn snapshot(&self)")
            .count(),
        1,
        "the continuous-session owner has one non-consuming snapshot method"
    );
    assert!(NOVA.contains(
        "expected_public_state(&self.public_input, cumulative_steps, self.is_terminalized)"
    ));
    assert!(NOVA.contains("runner.finish_block(false)?;"));
    assert!(!NOVA.contains("runner.finish_block(true)?;"));
}
