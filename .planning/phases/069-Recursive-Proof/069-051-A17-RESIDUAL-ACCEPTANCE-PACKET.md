# 069-051 A-17 residual acceptance packet

Status: **ACTIVE — APPROVED CONDITIONAL BY REPOSITORY AUTHORITY**

Packet schema: `z00z.recursive.v2.a17-residual.v1`; revision `2`; originally
activated on 2026-07-20 and source-revalidated under the user's explicit
instruction to finish 069-051 on 2026-07-21. The repository did not infer or
self-approve the accepted residual.

## Active authority decision

```yaml
decision_schema: z00z.recursive.v2.a17-residual.decision.v1
decision_revision: 2
record_status: ACTIVE
authority_generation: 2
candidate_identity:
  source_revision_digest: 302bf11cf4516acacaf31e1f8c591c2b21e5f6dbacfeb7f5ec4b3257a0aa79ef
  nova_source_sha256: 8cc403147ca345c8e3cec83336b362f51e9aea08bfc52ece1dfd83f27d90f6cc
  cargo_lock_sha256: 23a86f3341579b25ad5be96080a642405633df5f8c6e99dd4c3329d7d51f2a11
  profile_digest: 4568feb10fdb1ea33df48bc44c66a5a57c54035fe9b5021f51e45bc66428e93e
  spec_digest: 9eccb9666171da8be090debc03845147e7ecb624f7c0be3f6e56d1f341b300b7
  grammar_digest: 3ed491b3e252bd95044dfd6e921d74f09926157d9bb4a782ed16a316eedb50f0
decision: APPROVE_CONDITIONAL
premises:
  eag_model: ACCEPT_EXPLICIT_RESIDUAL
  general_zero_testing_for_instantiated_h: ACCEPT_EXPLICIT_RESIDUAL
  discrete_log_pallas_vesta: ACCEPT_PINNED_PALLAS_VESTA_DL_ASSUMPTION
  compression_backend_separate_premise: ACCEPT_EXPLICIT_RESIDUAL
  polynomial_round_bound_n: 4294967296
claims:
  allow_conditional_polynomial_depth_knowledge_soundness: true
  forbid_unconditional_128_bit_cumulative_ivc_security: true
  forbid_empirical_proof_as_assumption_discharge: true
approval:
  authority_identity: repository-authority/user-session
  decision_reference: phase-069-051-finalize-interactive-authority-2026-07-21
  signature_or_attestation: explicit-interactive-authority-instruction
  decided_at: 2026-07-21
```

## Exact accepted residual

- Primary result: IACR ePrint 2024/232, revision 2026-02-13, Theorem 5.
- Definitions: Definition 3 (polynomial-depth knowledge soundness) and
  Definition 7 (Extended Algebraic Group Model, EAGM).
- Conditional premises: corrected group-based Nova NIFS, polynomially bounded
  rounds, EAGM, discrete-log hardness and the general zero-testing assumption
  for `H`.
- Concrete candidate: `nova-snark 0.73.0`, Pallas/Vesta, Poseidon
  Fiat-Shamir and Spartan SNARK/IPA compression.
- Compression retains a separate pinned reduction and implementation premise;
  it is not established by Theorem 5 alone.

The format-4 compact verifier-wire change does not alter the pinned crate,
features, curve cycle, Nova transcript, entropy source, compression backend, or
maximum proof-depth policy. It changes authority transport only: deterministic
Pedersen vectors are reconstructed from the same pinned setup labels and exact
authority-bound counts, then the expanded canonical VK digest and structure are
checked before verification.

Repository evidence pins the dependency, source, transcript, entropy sites,
parameters and proof-depth policy, but it cannot demonstrate that concrete
adversaries satisfy EAGM or establish the paper's GZT premise for the
instantiated Poseidon transcript. Successful proofs and finite
`Q_fold / 2^127` accounting do not discharge those assumptions.

The allowed statement is limited to: knowledge soundness is conditional on the
pinned Theorem-5 premises and the separate compression-backend premise. The
record does not permit an unconditional “128-bit cumulative IVC security”
claim and does not enable `CheckpointProofSystem::VERIFIED`.

The implementation/source audit supporting this decision is retained in
`069-051-A17-APPLICABILITY.md` and `069-051-A17-LEDGER.md`. Any change to the
pinned dependency source, features, curve cycle, transcript, entropy sites,
compression backend, source identity or maximum proof-depth policy reopens
this decision.
