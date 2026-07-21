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

## 🌐 Languages

The concept includes English, Russian, French, German, Spanish, Japanese, and
Simplified Chinese UI catalogues. Language, regional format, and display time zone
are independent preferences. See [I18N-ARCHITECTURE.md](I18N-ARCHITECTURE.md) for
the catalogue contract, local machine-translation bridge, and required checks.

## 🧪 Suggested walkthrough

1. Resize between desktop and a 390 px mobile viewport.
2. Use Send, Receive, asset Claim, and Give permission from Home.
3. Confirm that submitted sends, claim outputs, voucher redemption, and permission delegation show honest non-final states.
4. Select Everyday, Savings, and Travel in the desktop wallet navigation. Confirm that Assets, History, Swap, Exchange, Staking, Backup, Settings, and the bottom status bar reflect only the selected wallet. Hover the copy control beside the address to reveal the full selected-wallet ID.
5. Use **Log out**, confirm the application shell is hidden, then unlock; the password field and visible sensitive presentation state must be cleared on lock.
6. Open Assets and compare Assets, Vouchers, and Permissions in the context rail; conditional and zero-value objects never appear in Available.
7. Verify that Claim and Voucher are separate flows: Claim reviews source proof/recipient/output/nullifier, while Voucher uses accept/redeem lifecycle actions.
8. Open OnionNet, Reticulum, and Aggregators from the left Network group. Each opens a separate read-only telemetry workspace, never a setup page. OnionNet uses the same second-row tab grammar as a wallet: Overview, Epoch, Privacy floor, Transport, Queues & replay, Probation, and Ingress boundary. These panels distinguish public deterministic state, local evidence, and aggregate synthetic health; they never reveal a user route, endpoint, session, or universal privacy score. Reticulum uses Node, Interfaces, Radio, Entry points, Paths, Probes, and Links for managed-node/local evidence rather than claims about the whole network. Unavailable evidence stays unavailable; Aggregators describes only a future wallet-to-node status bridge.
9. Inspect a selected wallet’s Settings → Policies and the restriction-layer / “Why blocked?” model.
10. Inspect a selected wallet’s Settings → Advanced for the local concept YAML draft. It validates and updates only that wallet’s demo state; production configuration write/watch/revision remains unavailable and is explicitly labelled.
11. Inspect Settings on desktop and narrow widths: the page heading, context tree, and detail card now use one aligned layout; the Network branch opens only when selected.
12. In application Settings → Appearance, switch System/Dark/Light and choose Z00Z Default, Black & Gold, Deep Blue Sea, Golden Twilight, or Midnight Sky. Palette changes update semantic tokens while safety colours remain protected. Appearance also selects the application-wide YAML syntax theme: One Light, Xcode, One Dark, or Night Owl.
13. Filter the selected wallet's History and open technical details.
14. Use **Add wallet** to create, open, or restore a profile, or choose **Cancel** to return to the selected wallet. **Remove wallet** confirms before removing one or more selected concept profiles; removing all profiles returns to **Add wallet**. The Wallets placeholder shows exactly three rows and one scrollbar: wallet cards, Add, and Remove are one ordered scroll list. Remove becomes disabled when the list is empty. The recovery helper inserts 24 demonstration words that are never a real seed.
15. Open Assets and select an asset name to inspect its asset-details fields; the table shows Name, Balance, Value, and Price for each asset.
16. Open a selected wallet's Settings. Confirm that General, Security, Backup, Policies, and Advanced are scoped to that wallet. Sensitive actions require a fresh password and their typed confirmation; secret/private material is never rendered or placed in YAML.

## 🧱 Constraints

- No production cryptography or RPC calls.
- The prototype loads Geist and Geist Mono from Google Fonts solely for visual review; the Z00Z wordmark uses Geist variable weight 780, matching the public docs header. Production packages the same font files locally and makes no remote font request.
- Inline SVG symbols provide icons.
- CSS tokens are intended to seed the Leptos production design system.
- Claim intake RPC, network detail, compliance-profile loading, and runtime YAML write/watch controls are simulated target capabilities, not claims about the live backend. Selected-wallet settings stay concept-local until a revisioned settings bridge exists; advanced settings can apply a safe YAML draft only to the in-browser concept state.
- The demo is a development-only visual reference. Production is the packaged standalone Tauri application with local-only IPC; it has no browser, container, or wallet HTTP/WebSocket profile and does not connect the demo to a wallet backend.
