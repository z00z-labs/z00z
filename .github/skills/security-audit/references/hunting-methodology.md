# Exploit-Driven Hunting Methodology

Use this reference during the deep scan and again during self-verification. Its
purpose is to separate exploitable vulnerabilities from suspicious patterns,
deployment assumptions, and defense-in-depth gaps.

## Build the Candidate

For each candidate, identify:

- attacker identity, access level, and controllable inputs
- protected asset and intended security boundary
- entrypoint, propagation steps, and final sink or state transition
- required configuration, timing, victim action, or third-party behavior
- concrete impact that the attacker gains beyond their intended authority

## Hunt Beyond the Happy Path

Use these lenses where they fit the target:

1. **Failure paths:** retries, fallbacks, cleanup, cancellation, timeout, and
   partial rollback.
2. **Boundary values:** empty, missing, zero, negative, maximum, maximum plus
   one, Unicode, expiration boundaries, and representation changes.
3. **Ordering and replay:** skip a step, repeat a completed step, invoke a
   callback early, or act during migration and recovery.
4. **Concurrency:** find check-then-act logic, lost updates, duplicate claims,
   and read/write or delete/use races.
5. **Parser differentials:** compare every component that interprets the same
   URL, message, token, path, archive, or serialized value.
6. **Round trips and second-order use:** trace stored data into later rendering,
   query, file, template, log, or command contexts.
7. **Configuration and defaults:** inspect missing config, environment
   overrides, debug paths, feature flags, and dependency failure behavior.
8. **Parallel doors:** compare all paths that mint identity, change the same
   state, import/export data, restore backups, or reach an administrative action.
9. **Privilege flow:** verify the right permission against the right resource
   at the final action, not only at an earlier transport layer.
10. **Incomplete fixes:** inspect sibling call paths and alternate callers of a
    sink that was previously hardened.
11. **Legitimate feature abuse:** test exports and backups for cross-scope
    disclosure, imports and restores for validation bypass, search and sorting as
    existence oracles, previews and drafts for cache leakage, and webhooks as SSRF.
12. **Chained attacks:** combine low-impact primitives across components and
    trust boundaries; re-evaluate impact when one primitive supplies identifiers,
    reachability, persistence, or authority required by another.

## Exploit Evidence Contract

Do not promote a candidate until the audit can state:

1. the exact attacker starting point
2. the exact payload, request, file, API call, or action sequence
3. a line-referenced trace from entrypoint to sink or security decision
4. why the input is attacker-controlled and the path is reachable
5. which security boundary is crossed and what observable damage occurs
6. which middleware, framework defaults, library behavior, downstream checks,
   database constraints, and deployment controls were examined
7. whether a safe local reproduction confirmed runtime-sensitive behavior

When a claim depends on parser or runtime semantics, test the actual
implementation and version when safely possible. When it depends on a component
or configuration outside the audited evidence, keep it as `requires deployment
testing` instead of lowering its severity and presenting it as a vulnerability.

## Candidate Disposition

- **confirmed issue:** the complete path, boundary crossing, impact, and relevant
  controls are verified; dynamic evidence is included when feasible.
- **likely issue:** the source trace and impact are strong, but one
  non-decisive runtime fact remains; state that assumption explicitly.
- **requires deployment testing:** a proxy, cache, identity provider, runtime
  setting, network route, or production control is decisive and unavailable.
- **hardening note:** another layer prevents exploitation, or the gap has no
  standalone boundary crossing and impact.
- **rejected:** the claimed path is unreachable, sanitized, authorized, or
  otherwise contradicted by evidence.

Only confirmed and evidence-backed likely issues belong in severity counts.

## Severity and Confidence

Rate severity from both likelihood and impact. Do not use severity as a proxy
for confidence:

- severity answers how damaging a successful exploit is
- confidence answers how completely the evidence proves the exploit

An unverified high-impact idea is a deployment-test lead, not a confirmed High
finding.
