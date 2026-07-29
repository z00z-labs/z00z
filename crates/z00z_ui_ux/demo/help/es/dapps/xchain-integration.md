---
id: dapps.xchain-integration
title: "dApps: X-Chain Integration"
route: dapps.xchain-integration
scope: context
---

# dApps: X-Chain Integration

[TOC]

## App View {#current-view}

![dApps: X-Chain Integration application view](help/assets/en/dapps-xchain-integration.png)

This image is captured from the live Demo view. It is a local architecture
fixture and does not claim that any external adapter is deployed.

## Overview

X-Chain Integration is result-first. The user describes what must be achieved,
what can be provided, the minimum acceptable result, destination, deadline,
maximum total cost, and fallback policy. Wallet compares compatible solver
plans and presents one normalized plan for confirmation.

**Boundary:** a Z00Z dApp does not control the wallet. It proposes a typed
action. Wallet selects eligible inputs, verifies evidence, builds the package,
shows the complete plan, requests confirmation, and only then starts execution.
No solver, venue, bridge, locker, issuer, or publication system becomes Z00Z
settlement authority.

## What stays under the hood

Wallet can automate the mechanical integration work:

- discover eligible solver, protocol, venue, and adapter combinations;
- request signed quotes and normalize them into one comparable schema;
- simulate result, cost, timing, resource, and failure constraints;
- select eligible private inputs and required external evidence;
- monitor execution, delivery, finality, cancellation, and refunds;
- use a fallback only when it remains inside every reviewed limit.

Automation is not hidden authority. Before confirmation, Wallet must expose:

1. the exact achievable result and destination;
2. the selected solver and execution method;
3. every protocol step and trust dependency;
4. required source funds, liquidity, gas, allowance, custody, or collateral;
5. the all-in price, including price impact and recovery allowance;
6. quote expiry, expected completion, delivery, and finality milestones;
7. the irreversible point, cancellation deadline, refund owner, and fallback.

## Integrations

X-Chain has one catalogue with exactly six resolver integrations. EVM Locker,
Native Issuer Rail, Celestia DA publication, swaps, venue orders, and other
adapter mechanics are not separate integration families. They appear only as
disclosed method steps inside the selected plan.

The App view renders all six resolver cards directly; there is no second
integration group and no expand step hiding their methods or dependencies.

| Integration | Role and method | Resources | Price and speed disclosure | Trust boundary |
| --- | --- | --- | --- | --- |
| NEAR Intents | Collect competing signed result quotes and normalize the selected solver plan. | Solver liquidity, adapters, source funding, and delivery evidence. | Solver, venue, adapter, network, and recovery costs; quote expiry through delivery and finality. | Selected solver and every dependency in its signed plan. |
| Ethereum | Execute reviewed EVM steps such as an EVM Locker or asset-specific Issuer Rail and verify finalized event evidence. | Contract capacity, gas, allowance or custody state, RPC evidence, and monitoring. | Gas and adapter charges; submission, inclusion, confirmation, and finality windows. | Contracts, locker or issuer operators where present, evidence providers, and network finality. |
| Liquity BOLD | Use a route-specific BOLD liquidity and settlement plan. | BOLD liquidity, redemption or market path, collateral constraints, and gas. | Spread, price impact, protocol, solver, network, and recovery costs; liquidity through external finality. | Liquity state plus the selected market, custody, solver, and adapter. |
| Hyperliquid | Execute a bounded market or limit plan with price, fill, and withdrawal conditions. | Venue balance, market depth, adapter, withdrawal path, and delivery evidence. | Spread, slippage, trading, withdrawal, network, solver, and recovery costs; fill through delivery. | Venue operation, custody exposure, adapter, liquidity, and withdrawal availability. |
| Uniswap | Execute a bounded on-chain swap with minimum output, slippage limit, and deadline. | Pool liquidity, allowance or permit, router contracts, gas, and delivery evidence. | Pool fee, price impact, gas, adapter, and recovery costs; quote expiry through external finality. | Pools, router contracts, token behavior, adapter logic, and network finality. |
| External Solvers | Accept signed competing or fallback plans through the same normalized schema. EVM Locker, Issuer Rail, or Celestia DA publication can appear only as explicit method steps. | Solver identity, liquidity or bond, adapters, funding, monitoring, and refund capacity. | Every cost component and every start, completion, finality, and refund deadline. | Solver identity and bond or reputation plus all declared route dependencies. |

This catalogue is architectural. It does not claim that any integration is
deployed or currently executable.

Wallet tracks **Z00Z checkpoint finality** separately from each external
effect, delivery, or publication finality. One completed plan step never
implies that another step is final.

## Plan selection contract

Wallet rejects a plan unless all of these statements are true:

- output meets or exceeds the minimum acceptable result;
- destination is exactly the reviewed destination;
- total price stays below the maximum total cost;
- expected completion stays inside the deadline;
- methods and trust dependencies stay inside the reviewed limits;
- required resources are available and evidence is current;
- fallback and recovery remain equivalent to what the user approved.

The execution preference ranks only valid plans. “Lowest total cost” cannot
weaken the result. “Fastest expected completion” cannot increase the cost
ceiling. “Best overall result” cannot add an undisclosed trust dependency.

## How to use this view

1. Choose the desired result: receive, deliver, fulfill, or publish.
2. Describe exactly what you provide.
3. Set the minimum acceptable result.
4. Enter the destination for external delivery or service fulfillment. Wallet
   binds the active receiver automatically for a Z00Z receive.
5. Choose the execution preference.
6. Set the execution deadline and maximum total cost.
7. Choose whether every route change requires approval or an equivalent
   fallback is allowed inside all reviewed limits.
8. Send the result intent to Wallet to compare plans and show the selected plan.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Minimum acceptable result | Hard output floor or exact service or publication outcome. A lower result invalidates the quote. |
| Destination | Exact external recipient or service target. It is mandatory for external delivery and fulfillment. |
| Execution preference | Ranking rule applied only after all hard result, price, time, and trust limits pass. |
| Maximum total cost | One ceiling covering solver, protocol, venue, network, price-impact, and recovery costs. |
| Signed quote | Solver commitment to the normalized result, method, resources, cost, timing, evidence, and recovery fields. |
| Fallback policy | Whether Wallet must ask before every route change or may use an equivalent reviewed alternative. |
| Irreversible point | Last stage after which cancellation or route replacement is no longer safe. |
| Internal finality | Z00Z checkpoint state of the private package. |
| External finality | Separately observed completion state for each release, mint, trade, delivery, service, or publication step. |

## Status and recovery

A plan can move through independently observed states such as:

| State | Meaning |
| --- | --- |
| Quoted | A signed plan satisfies the reviewed constraints but has not been accepted. |
| Fundable | Required inputs and resources are available; the irreversible point has not been crossed. |
| Executing | One or more disclosed external steps are in progress. |
| Delivered, awaiting finality | The expected result is visible but one or more finality milestones remain. |
| Recoverable | Execution failed before the irreversible point; cancellation, refund, or permitted fallback is available. |
| Manual recovery | An external dependency failed after automatic recovery became unsafe; owner, evidence, and deadline remain visible. |
| Completed | Wallet separately verified the reviewed result and all required finality milestones. |

Monitoring is derived evidence, not settlement truth. Reconnection is required
for fresh quotes, current external state, and execution.

## Safety and limits

- Wallet selects inputs and verifies evidence; the dApp and solver do neither.
- Every external event or attestation must be replay-safe and accepted once.
- No plan may silently change result, destination, cost ceiling, deadline, or
  trust dependencies.
- No route may change after the irreversible point.
- Contradictory, stale, unavailable, or insufficient evidence fails closed.
- Custody, issuer, solver, venue, liquidity, pause, recovery, and maturity
  boundaries remain explicit.
- A public DA lane must never contain a private receiver, settlement target,
  secret wallet material, or private transaction payload.
- `Target`, `Candidate`, and `Concept` are product maturity labels, not
  deployment claims.

Architecture basis: [Cross-Chain Integration whitepaper](https://www.z00z.io/whitepapers/Cross-Chain-Integration).

<!-- help-sync:source {"page_path":"dapps/xchain-integration.md","route_id":"dapps.xchain-integration","screenshot":"help/assets/en/dapps-xchain-integration.png","topic_id":"dapps.xchain-integration"} -->
