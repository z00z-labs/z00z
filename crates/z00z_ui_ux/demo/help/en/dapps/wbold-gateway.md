---
id: dapps.wbold-gateway
title: dApps: wCoins Gateway
route: dapps.wbold-gateway
scope: context
---

# dApps: wCoins Gateway

[TOC]

## App View {#current-view}

![dApps: wCoins Gateway application view](help/assets/en/dapps-wbold-gateway.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

wCoins Gateway proposes a bounded deposit or redemption between one Z00Z
wrapped coin and its route-specific external reserve. It supports independent
routes for **wBOLD**, **wDAI**, **wCRVUSD**, **wZCHF**, **wdEURO**, and
**wCJPY**. The local fixture shows:

**Stable routes: 6 independent lockers**

The liabilities are intentionally not combined into a synthetic USD total:
USD-, CHF-, EUR-, and JPY-referenced assets are separate accounting domains.
Every route keeps its own reference currency, protocol model, LockerID, reserve
pool, liabilities, redemption route, risk badge, status, and exposure limit.

**Boundary:** a Z00Z dApp does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## Network model

The wrapped assets exist inside Z00Z. Ethereum Mainnet in this view names the
canonical external reserve network for BOLD, DAI, CRVUSD, ZCHF, dEURO, and
CJPY; it does not mean that the Z00Z wrapped assets or Z00Z settlement run on
Ethereum.

These six concept routes deliberately use the canonical Ethereum Mainnet
reserve assets. Some underlying protocols also expose bridged assets on other
chains. A reserve on another external network is a different route, not an
alias: it requires its own LockerID, reserve pool, liabilities, redemption
route, status, risk badge, and exposure limit.

## Product positioning

| Route | Product role | Relative market role | Positioning |
| --- | --- | --- | --- |
| wBOLD | Best candidate for minimizing administrative control and the strategic primary route for Z00Z. | Smaller today than DAI and CRVUSD. | Governance-minimized private stable value. |
| wDAI | Best candidate for liquidity. | Far ahead of the other candidates in this set. | Highly liquid decentralized stable asset with governance-managed protocol risk. |
| wCRVUSD | Best compromise between decentralized architecture and real market turnover. | Meaningful trading activity and useful DEX depth. | DAO-managed crypto-collateralized stable asset without issuer address blacklist. |
| wZCHF | Strongest non-USD candidate in this set by protocol history and present scale. | More established than the dEURO and CJPY candidates, but still materially smaller than the leading USD routes. | Oracle-free CHF value with route-specific collateral and bridge dependencies. |
| wdEURO | Promising new euro candidate. | Current market activity is limited; use conservative pilot caps. | Oracle-free position core with additional governed stablecoin bridges. |
| wCJPY | Clean ETH-backed JPY candidate at the protocol level. | Current market depth is too thin for an unrestricted route. | ETH-backed JPY exposure with Chainlink oracle and liquidity risk. |

Relative liquidity, depth, and volume change over time. Revalidate current
market data before enabling live deposits; this page describes product
selection logic, not a live market feed.

### wBOLD

The primary stable asset for private payments, merchants, vouchers, scoped
budgets, subscriptions, and agent allowances.

### wDAI

The high-liquidity alternative for large ingress and egress, OTC, users already
holding DAI, movement between external DeFi and Z00Z, and backup liquidity when
BOLD depth is insufficient.

### wCRVUSD

The additional decentralized route for the Curve community, DEX liquidity,
stablecoin protocol-risk diversification, and private DeFi-oriented settlement.

### wZCHF

The most established non-USD candidate in this set for CHF-denominated private
settlement, payroll, merchant invoices, and treasury diversification.
Frankencoin uses oracle-free positions, challenges, and auctions, but accepted
collateral and stablecoin bridges still create route-specific dependencies.

### wdEURO

An early-stage EUR candidate for private settlement, subscriptions, payroll,
and merchant invoices. The position system is oracle-free, while separate
governed bridges can also mint dEURO against external euro stablecoins. Wallet
must therefore review the dEURO protocol reserve composition together with this
gateway's dEURO LockerID; it cannot infer one collateral origin for each
fungible dEURO unit.

### wCJPY

An ETH-backed JPY candidate for regional settlement, merchant invoices,
payroll, and treasury routing. Yamato V1 uses ETH collateral, direct redemption,
and Chainlink ETH/USD plus USD/JPY feeds. The route remains a candidate until
market depth, peg quality, contracts, and redemption activity pass live review.

## Route records

### USD-referenced routes

| Route record | wBOLD | wDAI | wCRVUSD |
| --- | --- | --- | --- |
| Z00Z asset | wBOLD | wDAI | wCRVUSD |
| External reserve | BOLD | DAI | CRVUSD |
| Reference currency | USD | USD | USD |
| Protocol model | Liquity V2 overcollateralized CDP | Governance-managed collateral system | LLAMMA crypto-collateralized debt |
| Reserve network | Ethereum Mainnet | Ethereum Mainnet | Ethereum Mainnet |
| LockerID | `locker.ethereum-mainnet.bold.v1` | `locker.ethereum-mainnet.dai.v1` | `locker.ethereum-mainnet.crvusd.v1` |
| Reserve pool | 3,400.00 BOLD | 6,950.00 DAI | 2,500.00 CRVUSD |
| Liabilities | 3,250.00 wBOLD | 6,800.00 wDAI | 2,400.00 wCRVUSD |
| Redemption route | wBOLD (Z00Z) → BOLD (Ethereum Mainnet) | wDAI (Z00Z) → DAI (Ethereum Mainnet) | wCRVUSD (Z00Z) → CRVUSD (Ethereum Mainnet) |
| Risk badge | Governance-minimized | Governance-managed | DAO-managed |
| Status | Active | Active | Active |
| Exposure limit | 5,000.00 BOLD | 10,000.00 DAI | 4,000.00 CRVUSD |

### Non-USD candidate routes

| Route record | wZCHF | wdEURO | wCJPY |
| --- | --- | --- | --- |
| Z00Z asset | wZCHF | wdEURO | wCJPY |
| External reserve | ZCHF | dEURO | CJPY |
| Reference currency | CHF | EUR | JPY |
| Protocol model | Oracle-free positions, challenges, auctions, and reserve equity | Oracle-free positions plus governed stablecoin bridges | ETH-backed CDP with Chainlink ETH/USD and USD/JPY feeds |
| Reserve network | Ethereum Mainnet | Ethereum Mainnet | Ethereum Mainnet |
| LockerID | `locker.ethereum-mainnet.zchf.v1` | `locker.ethereum-mainnet.deuro.v1` | `locker.ethereum-mainnet.cjpy.v1` |
| Reserve pool | 1,200.00 ZCHF | 850.00 dEURO | 180,000 CJPY |
| Liabilities | 1,000.00 wZCHF | 750.00 wdEURO | 150,000 wCJPY |
| Redemption route | wZCHF (Z00Z) → ZCHF (Ethereum Mainnet) | wdEURO (Z00Z) → dEURO (Ethereum Mainnet) | wCJPY (Z00Z) → CJPY (Ethereum Mainnet) |
| Risk badge | Mixed collateral | Early-stage · bridge-aware | Thin liquidity · oracle |
| Status | Candidate | Candidate | Candidate |
| Exposure limit | 2,000.00 ZCHF | 1,250.00 dEURO | 250,000 CJPY |

These values are bundled local fixtures. Reconnection is required to establish
current reserves, liabilities, status, finality, and redemption availability.
Candidate status does not claim that a production locker exists or that current
liquidity is sufficient.

## Gateway control model

- The gateway cannot confiscate user reserves.
- Wrapped coins cannot be minted without a confirmed deposit.
- Every external network is a separate route; an existing asset, network, or LockerID mapping cannot be changed retroactively.
- New deposits may be stopped by a circuit breaker.
- Any exit pause must have a hard deadline and a trust-minimized escape route after expiry.
- Upgrades require a long timelock or a new locker version.
- An old locker must retain redemption for its existing liabilities.
- Every route enforces its own exposure cap.

## How to use this view

Choose the exact Z00Z asset and external reserve network pair, then choose
deposit or redemption. The view shows its immutable LockerID separately. Enter
a positive amount and a maximum fee. Redemption also requires an external
recipient on the reserve network. Wallet re-checks the asset mapping, LockerID,
route status, reserve evidence, exposure cap, external finality, replay
protection, fee, and recipient before it can build a package.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Z00Z asset | Wrapped asset issued and settled inside Z00Z. |
| External reserve | Stablecoin held by the selected route outside Z00Z. |
| Reference currency | Fiat unit the external stablecoin seeks to track; it is not a guaranteed redemption promise from Z00Z. |
| Protocol model | External minting, collateral, oracle, governance, and redemption design that Wallet must not collapse into a generic stablecoin label. |
| Reserve network | External network on which the selected locker accounts for the reserve; it is not the network of the Z00Z asset. |
| Stable asset route | Exact Z00Z asset, external reserve, reserve network, and immutable LockerID. |
| Action | Deposit the external stablecoin for its wrapped coin, or redeem the wrapped coin for the external stablecoin. |
| External address | Required destination on the external network when redeeming. |
| LockerID | Versioned identifier for one reserve pool, liability ledger, redemption route, and exposure cap. |
| Reserve pool | External assets attributed to the selected locker. |
| Liabilities | Wrapped assets issued against that locker. |
| Risk badge | Route-specific governance and external protocol risk; never a guarantee. |
| Status | Whether the route accepts the requested action in the current verified state. |
| Exposure limit | Per-route ceiling that blocks additional deposits beyond the declared cap. |
| Maximum route fee | Hard ceiling checked by Wallet before confirmation. |
| Evidence | LockerID, external event reference, reserve snapshot, internal receipt, and route status. |

## Safety and limits

Z00Z can verify the internal package boundary but cannot itself establish
external custody, solvency, finality, pause state, or redemption. The gateway
must never mint from an unconfirmed deposit or silently move existing
liabilities to a new route. A deposit circuit breaker may stop new exposure,
but any exit pause needs a hard deadline and a trust-minimized escape path.
New versions must not remove redemption from an old locker.

Offline inspection is possible. Current reserve, liability, cap, status, and
redemption evidence require reconnection.

<!-- help-sync:source {"page_path":"dapps/wbold-gateway.md","route_id":"dapps.wbold-gateway","screenshot":"help/assets/en/dapps-wbold-gateway.png","topic_id":"dapps.wbold-gateway"} -->
