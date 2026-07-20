<!-- markdownlint-disable MD013 -->

# Z00Z Wallet Interactive Concept

This self-contained prototype is the executable companion to [`../UI-UX-SPEC.md`](../UI-UX-SPEC.md). It uses fabricated data and never connects to a wallet or signs a transaction.

## ▶️ Run

From the repository root:

```bash
python3 -m http.server 4173 --directory .planning/phases/110-Wallet-UX-UI/demo
```

Open `http://127.0.0.1:4173`.

The files can also be opened directly. The local HTTP server exists only for development screenshots and smoke tests; it is not part of the product runtime.

Optional visual smoke test (with the development server above still running):

```bash
.planning/phases/110-Wallet-UX-UI/demo/run-smoke.sh
```

## 🧪 Suggested walkthrough

1. Resize between desktop and a 390 px mobile viewport.
2. Use Pay, Receive, asset Claim, and Give permission from Home.
3. Confirm that submitted payments, claim outputs, voucher redemption, and permission delegation show honest non-final states.
4. Use **Lock wallet**, confirm the application shell is hidden, then unlock; the password field and visible sensitive presentation state must be cleared on lock.
5. Open Wallet and compare Assets, Vouchers, and Permissions in the context rail; conditional and zero-value objects never appear in Available.
6. Verify that Claim and Voucher are separate flows: Claim reviews source proof/recipient/output/nullifier, while Voucher uses accept/redeem lifecycle actions.
7. In Settings → Network, compare Overview, Reticulum, OnionNet, and Carriers. Click Network again to collapse the branch, then reopen it; leaf entries have no disclosure mark. The detailed status is explicitly a Phase 080 target simulation because the current RPC is stubbed.
8. Inspect Settings → Policies and the restriction-layer / “Why blocked?” model.
9. Inspect Settings → Advanced for the target UI ↔ YAML synchronization, revision, validation, and last-known-good contract.
10. Inspect Settings on desktop and narrow widths: the page heading, context tree, and detail card now use one aligned layout; the Network branch opens only when selected.
11. Switch System/Dark/Light appearance and inspect accent safeguards.
12. Filter Activity and open technical details.
13. Lock the wallet, then unlock with any four or more characters.
14. Open the wallet switcher to walk through Create and Recover. The recovery helper inserts 24 demonstration words that are never a real seed.

## 🧱 Constraints

- No production cryptography or RPC calls.
- No external scripts, fonts, styles, images, or CDN dependencies.
- Inline SVG symbols provide icons.
- CSS tokens are intended to seed the Leptos production design system.
- Claim intake RPC, network detail, compliance-profile loading, and YAML write/watch controls are simulated target capabilities, not claims about the live backend.
- The demo is a development-only visual reference. Production is the packaged standalone Tauri application with local-only IPC; it has no browser, container, or wallet HTTP/WebSocket profile and does not connect the demo to a wallet backend.
