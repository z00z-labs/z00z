<!-- markdownlint-disable MD013 -->

# Z00Z Wallet Interactive Concept

This self-contained prototype is the executable companion to [`../UI-UX-SPEC.md`](../UI-UX-SPEC.md). It uses fabricated data and never connects to a wallet or signs a transaction.

## ▶️ Run

From the repository root:

```bash
python3 -m http.server 4173 --directory .planning/phases/110-Wallet-UX-UI/demo
```

Open `http://127.0.0.1:4173`.

The files can also be opened directly, but a local HTTP server gives browser tools and automated screenshots a stable origin.

## 🧪 Suggested walkthrough

1. Resize between desktop and a 390 px mobile viewport.
2. Use Pay, Receive, Claim, and Give permission from Home.
3. Confirm that submitted payments and claims show a settling state rather than a false final success.
4. Open Wallet and compare Assets, Claims, and Permissions; conditional and zero-value objects never appear in Available.
5. In Settings → Network, compare Overview, Reticulum, OnionNet, and Carriers. The detailed status is explicitly a Phase 080 target simulation because the current RPC is stubbed.
6. Inspect Settings → Policies and the restriction-layer / “Why blocked?” model.
7. Inspect Settings → Advanced for the target UI ↔ YAML synchronization, revision, validation, and last-known-good contract.
8. Switch System/Dark/Light appearance and inspect accent safeguards.
9. Filter Activity and open technical details.
10. Lock the wallet, then unlock with any four or more characters.
11. Open the wallet switcher to walk through Create and Recover. The recovery helper inserts 24 demonstration words that are never a real seed.

## 🧱 Constraints

- No production cryptography or RPC calls.
- No external scripts, fonts, styles, images, or CDN dependencies.
- Inline SVG symbols provide icons.
- CSS tokens are intended to seed the Leptos production design system.
- Network detail, compliance-profile loading, and YAML write/watch controls are simulated target capabilities, not claims about the live backend.
