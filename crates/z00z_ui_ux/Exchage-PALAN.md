---
title: "Wallet Send and Exchange implementation plan"
status: "completed"
owner: "z00z_ui_ux"
created: "2026-07-24"
updated: "2026-07-24"
scope:
  - "crates/z00z_ui_ux/demo"
---

# Goal

Make `Send` preserve the protocol distinction between assets, vouchers, and
permissions, keep `Swap` asset-only, and replace the disabled `Exchange` stub
with an honest, responsive concept for two materially different execution
models:

- Hyperliquid Spot: venue/order-book execution for supported spot pairs.
- NEAR Intents: solver-driven, potentially cross-chain exact-input exchange.

The demo must remain local-first. It may model quote and review states, but it
must never imply that a live provider, liquidity source, network route, or
settlement bridge is connected.

# Evidence summary

## Z00Z object semantics

- Assets are final spendable value.
- Vouchers are conditional value with backing, acceptance, lifecycle, expiry,
  partial redemption, refund, transfer, and policy constraints.
- Permissions/rights are zero-value authority with action, scope, use limits,
  expiry, delegation, attenuation, revocation, and disclosure constraints.
- Wallet UI must not flatten these families into one balance or one anonymous
  transfer object.
- The receiver/request flow must classify the family, validate the receiver,
  policy, expiry, replay context, and disclosure expectations before review.

## Exchange semantics

- Hyperliquid exposes fully on-chain spot order books. The relevant wallet
  concept is spot only; perpetual positions, leverage, margin, liquidation, and
  TP/SL are explicitly outside this task.
- Hyperliquid spot quotes need a supported pair, precision/lot constraints,
  available spot balance, order type, price limit, and fee/market state.
- NEAR Intents 1Click routes need origin asset/chain, destination asset/chain,
  exact-input amount, recipient, refund address, slippage tolerance, quote
  deadline, deposit address/memo, and an execution status.
- Established wallet flows expose output, route/provider, price impact or
  slippage protection, fees, network requirements, and a review step before
  authorization.

# Requirements

## REQ-SEND-001 — Family switcher

`Send` must expose `Assets`, `Vouchers`, and `Permissions` as mutually exclusive
families using the same context-navigation visual language as the Assets view.

## REQ-SEND-002 — Family-specific selection

The item selector must show only transferable objects from the active family.
Changing family must select a valid first item, clear family-incompatible
amount state, and preserve the recipient/private note where safe.

## REQ-SEND-003 — Family-specific facts

- Asset: name/type, spendable balance, unit, chain, and amount.
- Voucher: conditional value, lifecycle status, expiry, transferability, and
  explicit one-object transfer.
- Permission: zero-value authority, action, scope, remaining uses, expiry, and
  delegation posture.

The review screen must call each family by its correct name and must not label
voucher value or permission authority as an asset balance.

## REQ-SEND-004 — Safe empty states

If the active family has no transferable object, keep the family switcher
available and show the relevant create action instead of hiding the entire Send
view.

## REQ-SWAP-001 — Asset-only invariant

Swap selectors must be sourced only from the wallet asset catalogue. Vouchers
and permissions must never appear in Swap state, options, copy, or review.

## REQ-EXCHANGE-001 — Enabled Exchange entry

The Exchange wallet tab must be enabled and retain the existing tab styling,
icon sizing, active underline, keyboard behavior, and responsive overflow.

## REQ-EXCHANGE-002 — Execution model selector

Exchange must offer two mutually exclusive routes:

1. `Hyperliquid Spot`
2. `NEAR Intents`

The selector is an execution-model choice, not a best-price ranking and not a
claim that either provider is connected.

## REQ-EXCHANGE-003 — Shared exchange inputs

Both routes require source asset, amount, destination asset, and explicit
review. Exchange must remain asset-only.

## REQ-EXCHANGE-004 — Hyperliquid Spot inputs

Show spot pair, market/limit order type, and a price-limit control appropriate
to the selected order type. Review must expose venue, pair, order type,
available balance, fee/rate availability, and the absence of a live connector.

## REQ-EXCHANGE-005 — NEAR Intents inputs

Show origin chain, destination asset/chain, recipient, refund address,
slippage tolerance, and deadline. Review must expose exact-input mode, solver
route, minimum-received/fee/ETA availability, and the future deposit-address
and execution-status boundary.

## REQ-EXCHANGE-006 — Honest concept preview

The demo may create a deterministic local review object from entered fields.
It must label quote, fee, rate, output, and timing as unavailable until an
authoritative provider connector supplies them. It must not generate fake
prices, fees, deposit addresses, fills, or settlement success.

## REQ-UI-001 — Visual consistency

Send and Exchange cards must use the same 640 px desktop action-card contract
as Send/Swap/Staking/Backup and the same mobile width, typography, controls,
spacing, hover/focus tokens, icons, and centered footer actions.

## REQ-UI-002 — Responsive safety

At 390 px and 320 px:

- no horizontal page overflow;
- family/provider selectors remain usable;
- labels, values, and buttons do not overlap or split incorrectly;
- only content that exceeds the viewport scrolls;
- the first card starts at the same content top as sibling wallet views.

## REQ-HELP-001 — Context help

Update local Markdown help for Send and Exchange. Help must explain family
semantics, provider differences, unavailable/live boundaries, and review
fields without adding permanent explanatory paragraphs to the main view.

## REQ-I18N-001 — Translation keys

New persistent interface labels must be defined in the English anchor
catalogue and synchronized across all supported catalogues. No raw key may
appear in the interface.

# Implementation tasks

## TASK-001 — Extend presentation state

- Add per-wallet `sendFamily` to each Send draft.
- Add per-wallet Exchange draft state with provider, source asset, amount,
  target, provider-specific fields, and review step.
- Keep state ephemeral and presentation-only; no network requests.

Depends on: none  
Satisfies: REQ-SEND-001, REQ-SEND-002, REQ-EXCHANGE-002, REQ-EXCHANGE-006

## TASK-002 — Refactor Send family data access

- Split mixed `sendOptionEntries()` into family-filtered access.
- Centralize family metadata (label, icon, empty state, create action).
- Keep the gateway transfer path unchanged.
- Add compact family-specific fact rows derived from existing fixtures.

Depends on: TASK-001  
Satisfies: REQ-SEND-001 through REQ-SEND-004

## TASK-003 — Build Send family navigation

- Reuse the context-nav component contract.
- Desktop/tablet: context rail plus centered 640 px Send panel.
- Mobile: horizontally scrollable context tabs above the panel.
- Preserve form state and focus when switching family.

Depends on: TASK-002  
Satisfies: REQ-SEND-001, REQ-UI-001, REQ-UI-002

## TASK-004 — Prove Swap family isolation

- Keep `assetOptions()` as the only Swap selector source.
- Add regression assertions that voucher/permission identifiers cannot appear.

Depends on: none  
Satisfies: REQ-SWAP-001

## TASK-005 — Model Exchange provider contracts

- Add a dedicated local port module with a provider LUT for label, icon,
  execution type, supported controls, and safety copy.
- Define a destination LUT with asset, unit, chain, and provider-compatibility
  metadata.
- Keep Hyperliquid and NEAR Intents form branching explicit and data-driven.
- Test the LUT independently from the view renderer so adding a future provider
  does not require rewriting Send, Swap, or wallet navigation.

Depends on: TASK-001  
Satisfies: REQ-EXCHANGE-002 through REQ-EXCHANGE-006

## TASK-006 — Implement Exchange form and review

- Enable the top-level Exchange tab.
- Render one centered 640 px card.
- Add execution-model selector and shared fields.
- Add Hyperliquid Spot market/limit controls.
- Add NEAR Intents recipient/refund/slippage/deadline controls.
- Validate locally, then render an honest review state with unavailable
  authoritative quote fields.
- Provide Back and Done/Request-new-quote actions without claiming submission.

Depends on: TASK-005  
Satisfies: REQ-EXCHANGE-001 through REQ-EXCHANGE-006, REQ-UI-001

## TASK-007 — Add design-system styles

- Add only component classes needed for Send workspace, family fact rows,
  provider selector, and Exchange review.
- Reuse existing LUT variables; no new ad-hoc colors.
- Match existing control heights, card radius, icon containers, hover/focus,
  and selected-state rules.

Depends on: TASK-003, TASK-006  
Satisfies: REQ-UI-001, REQ-UI-002

## TASK-008 — Update help and locale catalogues

- Update `help/en/wallet-send.md` and `help/en/wallet-exchange.md`.
- Synchronize the English help hash and localized help catalogue via the
  existing help build/check scripts.
- Add and synchronize new UI labels through the locale workflow.

Depends on: TASK-003, TASK-006  
Satisfies: REQ-HELP-001, REQ-I18N-001

## TASK-009 — Automated and visual verification

- Update smoke expectations for enabled Exchange and Send family counts.
- Test Send family switching, empty states, fact fields, and all three review
  labels.
- Test Swap asset-only options.
- Test both Exchange providers, branching fields, validation, review, and
  unavailable quote semantics.
- Run syntax, port-contract, locale/help, smoke, and visual-review suites.
- Inspect desktop and 390/320 px screenshots for Send/Swap/Exchange.

Depends on: TASK-004, TASK-007, TASK-008  
Satisfies: all requirements

# Files expected to change

- `crates/z00z_ui_ux/demo/app.js`
- `crates/z00z_ui_ux/demo/styles/components.css`
- `crates/z00z_ui_ux/demo/index.html`
- `crates/z00z_ui_ux/demo/scripts/port/exchange-catalog.js`
- `crates/z00z_ui_ux/demo/scripts/port/presentation-state.js`
- `crates/z00z_ui_ux/demo/scripts/test-port-contracts.mjs`
- `crates/z00z_ui_ux/demo/locales/*.js`
- `crates/z00z_ui_ux/demo/help/*/wallet-send.md`
- `crates/z00z_ui_ux/demo/help/*/wallet-exchange.md`
- `crates/z00z_ui_ux/demo/scripts/generated/help-catalog.js`
- `crates/z00z_ui_ux/demo/smoke.spec.js`
- visual-review configuration only if Exchange is not already captured

# Test matrix

| Surface | Desktop | 390 px | 320 px | Behavior |
| --- | --- | --- | --- | --- |
| Send / Assets | yes | yes | yes | amount and asset balance |
| Send / Vouchers | yes | yes | yes | conditional value and expiry |
| Send / Permissions | yes | yes | yes | action, scope, uses, delegation |
| Swap | yes | yes | yes | asset-only selector |
| Exchange / Hyperliquid Spot | yes | yes | yes | market/limit branching |
| Exchange / NEAR Intents | yes | yes | yes | cross-chain/refund/slippage |
| Exchange review | yes | yes | yes | no invented authoritative values |

# Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| “Exchange” accidentally implies connected execution | Persistent concept-state badge and unavailable authoritative quote fields |
| Hyperliquid UI drifts into leveraged perps | Spot-only provider label, spot-pair controls, no leverage/margin/position controls |
| NEAR Intents looks like a normal same-chain order book | Explicit solver/cross-chain fields, recipient/refund/deadline/deposit boundary |
| Vouchers counted as cash | Conditional-value labels and one-object transfer semantics |
| Permissions displayed as value | Zero-value authority labels plus action/scope/use metadata |
| Mobile context rail crowds the card | Horizontal overflow only inside the family selector; page remains bounded |
| Translation drift | Existing locale audit and help hash checks remain release gates |

# Source inventory

## Repository and official Z00Z sources

- `crates/z00z_core/docs/OBJECT_FAMILY_SEMANTICS.md`
- `crates/z00z_core/src/vouchers/voucher_policy.rs`
- `crates/z00z_core/src/vouchers/voucher_lifecycle.rs`
- `crates/z00z_wallets/src/rpc/object_types.rs`
- `https://www.z00z.io/docs/protocol/assets-vouchers-rights`
- `https://www.z00z.io/docs/developers/payment-requests`
- `z00z-website/content/whitepapers/Assets-Rights-Vauchers.md`

## Provider and wallet UX sources

- `https://hyperliquid.gitbook.io/hyperliquid-docs`
- `https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/spot`
- `https://hyperliquid.gitbook.io/hyperliquid-docs/trading/order-types`
- `https://docs.near-intents.org/integration/distribution-channels/1click-api/quickstart`
- `https://docs.near-intents.org/integration/distribution-channels/1click-api/sdk`
- `https://help.phantom.com/hc/en-us/articles/5985106844435-Troubleshoot-swap-issues`
- `https://help.coinbase.com/en-au/wallet/getting-started/dex-swap`

# Review checklist

- [x] Every requirement maps to one or more tasks.
- [x] No task introduces a live network dependency.
- [x] No Exchange branch includes perpetuals, leverage, or margin.
- [x] No deterministic preview invents rate, fee, output, ETA, or success.
- [x] Send preserves asset/voucher/permission semantics and existing gateway.
- [x] Swap remains asset-only.
- [x] Mobile behavior is testable at 390 px and 320 px.
- [x] Help and locale updates are included.
- [x] Implementation verification commands are explicit.

# Plan review findings and corrections

The reviewed plan is approved with these corrections already incorporated:

1. **Provider isolation:** provider metadata moved out of `app.js` into a
   dedicated port LUT. This keeps a future Rust enum/presentation model mapping
   straightforward and prevents provider branches from leaking through the
   rest of the wallet renderer.
2. **Hyperliquid scope:** the route is explicitly `Hyperliquid Spot`. Perps,
   leverage, collateral modes, liquidation, funding, and position controls are
   excluded because they are a different risk product, not a wallet exchange
   default.
3. **NEAR Intents boundary:** the route is not presented as an order book or as
   self-settling Z00Z logic. Recipient, refund, deadline, deposit, and execution
   status remain provider-bound fields.
4. **No invented market truth:** the local preview contains only user inputs
   and provider contract labels. Rate, output, fee, price impact, ETA, deposit
   address, and execution result remain unavailable.
5. **Family-safe Send:** switching to Vouchers or Permissions changes the
   selectable inventory and the facts shown. It does not merely filter a mixed
   “Asset” select while retaining asset terminology.
6. **Permission semantics:** permissions remain zero-value. The UI shows
   action, scope, uses, expiry, and delegation posture; it never asks for or
   displays an amount.
7. **Voucher semantics:** vouchers move as one conditional object and expose
   lifecycle/expiry/receiver-acceptance facts. Their face value is not added to
   spendable asset balance.
8. **Offline-first operation:** no browser fetch, CDN, provider SDK, or remote
   asset is introduced. A production connector can later satisfy the typed
   provider boundary through Tauri/native Rust.

# Implementation completion

- [x] TASK-001 — per-wallet Send family and Exchange draft state.
- [x] TASK-002 — family-filtered Send inventory and semantic facts.
- [x] TASK-003 — shared Assets/Vouchers/Permissions context navigation.
- [x] TASK-004 — Swap selectors proven asset-only.
- [x] TASK-005 — immutable provider and destination LUTs.
- [x] TASK-006 — NEAR Intents and Hyperliquid Spot form/review branches.
- [x] TASK-007 — token-based responsive component styling.
- [x] TASK-008 — 10 locale catalogues and 10-language Markdown Help updated.
- [x] TASK-009 — automated and visual verification completed.

Verification result:

- 46 of 46 Playwright smoke scenarios passed.
- 72 desktop/mobile visual routes audited with zero layout issues.
- Locale, Help, production-port, syntax, and whitespace gates passed.
- Send and Exchange screenshots were inspected at desktop, 390 px, and 320 px.
