<!-- markdownlint-disable MD013 -->

# Z00Z Wallet UI/UX Verification and Design Review

## Summary

**Text verified:** `UI-UX-SPEC.md` v0.2 and the self-contained HTML demo, checked against current wallet/network/object/config code, Phase 080, and the live `z00z.io` surface.

**Claims extracted:** 12 material claims.

| Rating | Count |
| --- | ---: |
| VERIFIED | 10 |
| PLAUSIBLE | 2 |
| UNVERIFIED | 0 |
| DISPUTED | 0 |
| FABRICATION RISK | 0 |

**Items requiring attention:** No disputed claim remains in v0.2. Three drifts in v0.1 were corrected: Tor/OnionNet/Reticulum were modelled as peer route choices; `Right` was overgeneralized as Budget; configuration/profile target capabilities were not explicit enough about missing live RPC support.

## Flagged Items

No current DISPUTED or FABRICATION RISK items.

Resolved before this report:

1. Replaced “Tor or Onionet route” with OnionNet privacy overlay → carrier adapter → Reticulum/QUIC/TLS/Tor.
2. Replaced “Budgets” as the generic right surface with “Permissions”; Budget remains one amount-limited recipe.
3. Marked detailed Phase 080 network status, signed policy profiles, and UI/YAML write/watch as simulated target capabilities rather than live backend behavior.

## All Claims

### VERIFIED

#### C1 — Three object families

- **Claim:** The wallet must project `Asset`, `Voucher`, and `Right` separately.
- **Source:** `crates/z00z_core/src/assets/object_family.rs:11`.
- **Notes:** The enum has exactly these three variants; the owned-object inventory has matching families.

#### C2 — Voucher is conditional value-bearing state

- **Claim:** Vouchers are value-bearing but must not enter Available before the outcome settles.
- **Source:** `crates/z00z_core/src/assets/object_family.rs:28`, `crates/z00z_core/src/vouchers/voucher_policy.rs:9`.
- **Notes:** `Asset | Voucher` are value-bearing. Voucher policy requires receiver acceptance and defines transfer, partial redemption, refund, beneficiary, refund-authority, and required-right behavior.

#### C3 — Right is bounded zero-value authority

- **Claim:** Permission must express action, scope, uses, delegation, and attenuation; not every right is a monetary budget.
- **Source:** `crates/z00z_core/src/rights/right_policy.rs:14`, `:23`, `:80`.
- **Notes:** Scope and allowed actions are explicit; `max_uses`, `delegation_allowed`, `attenuation_only`, and `zero_value_only` are protocol fields.

#### C4 — OnionNet and Reticulum have different responsibilities

- **Claim:** OnionNet is the privacy overlay/contract and Reticulum is a resilient carrier/delivery fabric.
- **Source:** `.planning/phases/080-Network-Reticulum/080-Reticulum-OnionNet.md:711`, `:1138`, `:1248`.
- **Notes:** The target architecture routes wallet envelopes through OnionNet then a carrier adapter. Reticulum must not replace or own OnionNet semantics.

#### C5 — User-facing network modes and measurable status

- **Claim:** Auto, Private, Resilient, and Direct have distinct fallback/privacy behavior, and UI should show concrete route properties rather than “anonymous.”
- **Source:** `.planning/phases/080-Network-Reticulum/080-Reticulum-OnionNet.md:3576`.
- **Notes:** v0.2 uses the same modes and shows overlay, privacy floor/hops, carrier, chain, and scan separately.

#### C6 — Current network RPC is not authoritative for target status

- **Claim:** Existing `app.network.switch_to_onionet` / `switch_to_tor` cannot justify detailed live Phase 080 health.
- **Source:** `crates/z00z_wallets/src/rpc/network_rpc_impl.rs:1`, `crates/z00z_wallets/src/services/app_chain_network.rs:6`, `crates/z00z_wallets/src/app/app_kernel.rs:117`.
- **Notes:** The RPC implementation and service call themselves placeholders/stubs; OnionNet returns an unsuccessful Devnet fallback.

#### C7 — Current YAML runtime is read/merge oriented

- **Claim:** Current runtime supports embedded defaults plus `Z00Z_WALLET_CONFIG_PATH` overlay, but not the specified UI write/watch/revision contract.
- **Source:** `crates/z00z_wallets/src/services/wallet_runtime_config.rs:28`, `:79`, `:402`; RPC search for config/settings methods.
- **Notes:** Recursive YAML merge and validation exist. No config watcher, lossless write, revision/ETag, typed patch, or `config.changed` RPC was found.

#### C8 — Current wallet-local rules are narrower than rich profiles

- **Claim:** Live local policy fields cover amount/day, assets, recipients, confirmation, and time restrictions; signed compliance profiles are not implemented.
- **Source:** `crates/z00z_wallets/src/wallet/policy.rs:12`, `crates/z00z_wallets/src/rpc/wallet_types.rs:187`, `crates/z00z_extensions/compliance/profiles`.
- **Notes:** `PersistWalletSettings.policy_rules` exists and is consumed by transaction/asset support; the compliance profile surface is a placeholder and no complete settings/profile RPC exists.

#### C9 — Intent-level UI boundary

- **Claim:** UI must request reviewed wallet intents and must not receive arbitrary signing/derivation APIs.
- **Source:** `.planning/phases/080-Network-Reticulum/080-Reticulum-OnionNet.md:3480`.
- **Notes:** The gateway and review-flow contract in v0.2 is aligned.

#### C10 — Live Z00Z design direction

- **Claim:** The wallet can mirror the site's compact navigation, neutral surfaces, thin borders, readable typography, and gold brand accent without copying the docs layout.
- **Source:** `https://www.z00z.io/` live response on 2026-07-19.
- **Notes:** The response redirects to `/docs` and declares corporate theme, Geist/Open Sans/Inter/monospace fonts, compact sticky navigation, bordered neutral surfaces, and primary accent treatment.

### PLAUSIBLE

#### C11 — Tauri + Leptos is the best production UI choice

- **Claim:** Tauri 2 + Leptos CSR is the best fit among the compared options.
- **Notes:** Platform support and repository fit are verified, but “best” is a weighted product decision. A technical spike must still validate mobile lifecycle, accessibility bridge, binary size, sidecar packaging, and performance.

#### C12 — Restriction intersection is the safest profile composition

- **Claim:** Protocol → managed → local → per-action layers should compose as restrictions only, with ambiguous conflicts failing closed.
- **Notes:** This is consistent with rights attenuation and safe wallet behavior, but it is a target product/security decision. Backend policy semantics and profile signature authority require a dedicated specification and tests.

## Internal Consistency

No blocking contradiction remains.

- SPEC and demo use the same four workspaces and Wallet/Settings contextual tabs.
- Assets, Claims, and Permissions map to the three object families without summing conditional or zero-value state into Available.
- Network overlay, carrier, chain, and scan are consistently distinct.
- Detailed network/config/profile controls are consistently labelled target simulations where the backend is not ready.
- UI/YAML is one effective settings model with provenance and conflict handling; UI exposes a subset rather than a second store.
- Theme customization protects status, privacy, environment, warning, error, and focus semantics.

One deliberate terminology exception remains: current function/dialog names use `budget` internally because the interactive flow demonstrates the Budget recipe. Visible navigation and generic right copy use Permission.

## Frontend Design Review: Z00Z Wallet v0.2 Concept

### Context

- **Purpose:** Define an implementation-ready private wallet interface for Windows, Linux, iOS, and Android.
- **Aesthetic direction:** Calm dark-first technical workspace, navy surfaces, restrained gold identity, cyan privacy rail, mint confirmation, compact site-like tab hierarchy.
- **User task:** Understand available value, act on assets/claims/permissions, verify settlement/privacy state, and safely control policy/configuration.

### Assessment Summary

**Pass for concept validation** — the visual concept is preserved and the product model is more accurate, transparent, and implementation-specific. Target-only backend capabilities remain clearly disclosed.

### Aesthetic Quality

- [x] Clear non-cyberpunk Z00Z direction.
- [x] Cohesive tokenized palette and protected semantics.
- [x] Compact contextual tabs reflect the site without imitating documentation pages.
- [x] Desktop and mobile compositions preserve the same content hierarchy.
- [x] Inline SVG icons; no external dependencies or emoji controls.
- [x] Focus, hover, reduced-motion, forced-color, and responsive rules exist.

### Pillar Assessment

| Pillar | Status | Notes |
| --- | --- | --- |
| Frictionless | 🟢 | Four global destinations; family/settings depth uses contextual tabs; one dominant action per flow |
| Quality Craft | 🟢 | Strong visual continuity, responsive cards/tabs, 100 accessibility score on Advanced screen |
| Trustworthy | 🟢 | Honest settlement, object separation, concrete privacy state, target-capability disclaimers, policy provenance |

### Issues

**Blocking:** None for the concept baseline.

**Major before production:**

1. Implement config service and profile/network capability RPCs before enabling simulated controls.
2. Run real screen-reader and mobile safe-area/keyboard tests in Tauri targets; browser automation is not sufficient.
3. Specify profile signing authority, migration, rollback, and rule-conflict tests in a backend/security document.

**Minor refinement:**

1. Add an explicit Quarantine demo state after object RPC capability exists.
2. Add long localization/200% text fixtures and RTL evaluation when language scope is known.
3. Replace the simulated YAML preview with a real editor component only after lossless backend semantics are implemented.

## Verification Commands and Results

| Check | Result |
| --- | --- |
| `node --check demo/app.js` | Pass |
| `html-validate demo/index.html` | Pass |
| Chromium desktop screenshots: Network and Advanced YAML | Pass; visually inspected |
| Chromium mobile screenshot: Permissions at 390 × 844 | Pass; no visible page overflow |
| Lighthouse Advanced screen | Accessibility 100, Best Practices 100 |
| CodeGraph + direct source checks | Object, network RPC, policy, and config claims reconciled |

## What Was Not Checked

- Real Tauri/Leptos implementation, native accessibility trees, biometrics, secure storage, mobile suspend/resume, and `z00z-netd` packaging; they do not exist in this prototype.
- Production visual parity across Windows/Linux/iOS/Android renderers.
- Legal sufficiency of any future compliance profile.
- Performance with large object/activity inventories and real RPC latency.
- Figma component-library parity; Figma is intentionally downstream of the validated HTML behavior.

## Limitations

- The prototype is deterministic fabricated data and does not sign, persist, or call RPC.
- Phase 080 is a target architecture document; current network methods remain stubs.
- A supporting source establishes alignment, not absolute correctness; target decisions still require implementation and security review.
