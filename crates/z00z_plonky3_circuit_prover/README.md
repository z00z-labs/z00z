# Z00Z Plonky3 proof backend

`z00z_plonky3_circuit_prover` is the sole workspace package for the audited
Plonky3 batch-STARK prover and the FRI-recursive verifier used by Phase 069.
Keeping both surfaces in one package prevents proof, prover-data, transcript,
and verifier types from diverging across duplicate Cargo identities.

Canonical entry points:

- `BatchStarkProver::prove_direct_tables` proves bounded domain-specific AIR
  tables without lowering them through a general circuit witness.
- `BatchStarkProver::prove_all_tables` proves `p3-circuit` runner traces.
- `FriRecursionBackend` and the functions in `recursion` build and verify the
  recursive FRI layers over those exact proof types.
- `config::{baby_bear, koala_bear, goldilocks}` constructs field-specific
  configurations.

The implementation is derived from
[Plonky3-recursion](https://github.com/Plonky3/Plonky3-recursion) revision
`b36339709a7a67ee9760fb578b3d4339fd983709` and remains dual-licensed under
MIT or Apache 2.0. The unused WHIR/sumcheck branch is excluded. There is no
separate recursion package, wrapper crate, dependency alias, compatibility
shim, or second prover type universe.

The local direct-table and one-shot paths reduce overlapping preparation and
witness lifetimes without changing the AIR, proof format, verifier,
transcript, field, hash, FRI, grinding, query, or recursion security
parameters.
