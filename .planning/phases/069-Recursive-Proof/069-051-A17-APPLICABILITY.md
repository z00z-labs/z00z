# 069-051 A-17 applicability ledger

Status: active authority-approved conditional residual; unconditional
polynomial-depth knowledge soundness is not claimed.

## Pinned result

- Primary source: IACR ePrint 2024/232, *On the Security of Nova Recursive
  Proof System: Limitations of and Alternatives to Bounded-Depth Analysis*.
- Source record revision: 2026-02-13 (the ePrint record's latest revision at
  the 2026-07-19 review).
- Exact result: Theorem 5, using Definition 3 (poly-depth knowledge soundness),
  Definition 7 (EAGM), Theorem 4's group-based NIFS knowledge soundness, the
  discrete-log assumption, and the general zero-testing assumption over `H`.
- Implementation candidate: `nova-snark = 0.73.0`, lock checksum
  `62afd983558f08e4a27a11edd6701177961379761b866a6f34b4f2bc39d1bbfa`,
  Pallas/Vesta, Poseidon Fiat-Shamir, Spartan SNARK/IPA compression, feature
  `io`, private owner `z00z_storage::checkpoint::nova`.

## Applicability decision

| Theorem-5 premise | Pinned implementation evidence | Decision |
| --- | --- | --- |
| Nova Figure-3 construction with group-based NIFS | dependency/source audit identifies the corrected two-curve Nova group folding path | conditional match |
| Polynomially many rounds | authority bound `N=4,294,967,296` cumulative Nova steps | accepted bounded premise |
| Extended Algebraic Group Model, Definition 7 | no executable test can establish that real adversaries satisfy EAGM representations/extension requirements | accepted explicit residual |
| Discrete-log hardness | Pallas/Vesta prime-order group selection is pinned | accepted pinned Pallas/Vesta DL assumption |
| General zero-testing assumption for the instantiated `H` | the implementation uses concrete Poseidon transcript hashes; no reduction establishes the paper's GZT premise for this instantiation | accepted explicit residual |
| Compression preserves knowledge soundness | Theorem 5 covers Nova IVC; the selected Spartan/IPA compressed proof needs its separate pinned reduction and implementation assumptions | accepted separate compression premise |

The exact named theorem is therefore identified, but its concrete EAGM and GZT
premises are not demonstrated for `nova-snark 0.73.0` plus the selected
Poseidon/Spartan stack. A successful release proof, fold-count bound, or
mutation corpus cannot change that conclusion. A-17 remains explicit, and no
report may state unconditional "128-bit cumulative IVC security".

The exact external decision carrier is
[`069-051-A17-RESIDUAL-ACCEPTANCE-PACKET.md`](069-051-A17-RESIDUAL-ACCEPTANCE-PACKET.md).
It records the explicit `APPROVE_CONDITIONAL` decision for EAGM, GZT, pinned
Pallas/Vesta DL, polynomial `N=2^32`, and the separate compression premise.
The decision was supplied by the repository authority; the packet did not
approve itself.

## Mechanical rejection rule

An A-17 report is invalid if it omits `ePrint 2024/232`, revision 2026-02-13,
Theorem 5, Definitions 3 and 7, the GZT premise, the EAGM premise, or the
separate compression-backend premise; or if it infers polynomial-depth
knowledge soundness only from successful proofs or local `Q_fold / 2^127`
accounting.
