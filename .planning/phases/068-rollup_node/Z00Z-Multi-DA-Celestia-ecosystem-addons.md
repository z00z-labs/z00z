# Z00Z Multi-DA Celestia Ecosystem Addons

<https://celestia.org/ecosystem/>

## Invariant Boundary

### ZINV-CHECKPOINT-002

Invariant reference: `ZINV: CHECKPOINT-002`

Любой explorer, watcher, namespace indexer, or DA helper в Celestia ecosystem
может только наблюдать и подтверждать уже committed checkpoint publication. Он
не должен становиться альтернативной authority path для state acceptance.

### ZINV-CHECKPOINT-001

Invariant reference: `ZINV: CHECKPOINT-001`

Watcher-side use of Celestia data must preserve one checkpoint lineage: any
blob, namespace, or proof lookup is valid only if it binds back to the same
prior-root to next-root chain without replay or double-consume drift.

Конкретно для Z00Z они полезны **не одинаково**. Я бы разделил так:

```text
P0: Celenium
P1: Evolve / Rollkit architecture
P1/P2: Prism as architectural reference
P2: Hyperlane / Axelar / Squid
```

## 1. Celenium — “видимость и доказуемость” Z00Z на Celestia

**Celenium** — это explorer/API/indexer для Celestia: blobs, namespaces, transactions, blocks, stats. У Celenium есть REST API для исторических данных, blobs и статистики, а их indexer open-source и может быть запущен самостоятельно. ([api-docs.celenium.io](https://api-docs.celenium.io/?utm_source=chatgpt.com))

Для Z00Z это полезно вот так:

### A. Investor dashboard

Ты можешь показать инвестору не просто “мы используем Celestia”, а живой dashboard:

```text
Z00Z testnet epoch: 1245
Celestia height: 5839201
Namespace: z00z-testnet-v1
Blob status: included
prev_root: 0x...
new_root: 0x...
spent_delta_root: 0x...
created_delta_root: 0x...
checkpoint_proof: available
```

Это очень сильно для грантов/инвесторов: **каждый checkpoint Z00Z реально лежит в Celestia DA**.

### B. Watcher service

Z00Z watcher может через Celenium API искать blobs по namespace и проверять:

```yaml
watcher_checks:
  - blob_exists
  - blob_height_confirmed
  - namespace_correct
  - checkpoint_schema_valid
  - prev_root_matches_last_known_root
  - new_root_matches_state_transition
  - proof_ref_available
```

То есть Celenium помогает не строить всё с нуля на первом этапе.

### C. Debugging DA problems

Если aggregator сказал: “я отправил checkpoint в Celestia”, watcher может проверить:

```text
1. tx exists?
2. blob exists?
3. correct namespace?
4. blob decodable?
5. expected checkpoint hash == actual hash?
```

Celestia node API тоже позволяет получать blobs по namespace на заданной высоте, но Celenium удобнее для dashboard/explorer/indexed history. ([Celestia Documentation](https://docs.celestia.org/build/rpc/node-api/?utm_source=chatgpt.com))

### D. Grant optics

Для Celestia grant это важно: ты показываешь **DA usage**.

```text
Z00Z generates continuous useful demand for Celestia blobs:
- checkpoint blobs
- state transition blobs
- proof reference blobs
- optional inbox snapshot CIDs
```

**Что не делать:** не класть в Celestia/Celenium приватные wallet secrets, receiver data, raw ownership data. Только commitments, roots, deltas, encrypted blobs, proof refs.

------

## 2. Evolve / Rollkit architecture — “как правильно построить sovereign rollup node”

Это не обязательно брать как dependency. Для Z00Z это скорее **архитектурный шаблон**.

Rollkit исторически давал modular node для rollups поверх DA layer и ABCI-compatible интерфейс. ([Celestia Blog](https://blog.celestia.org/introducing-rollkit-a-modular-rollup-framework/?utm_source=chatgpt.com)) Сейчас у Celestia-экосистемы есть Evolve / ev-node, где важны sync modes: hybrid sync, DA-only sync, P2P-priority sync. ([GitHub](https://github.com/celestiaorg/optimint?utm_source=chatgpt.com))

Для Z00Z это полезно конкретно в 5 местах.

### A. Разделить node на правильные компоненты

Z00Z validator node должен быть не “одна программа всё делает”, а примерно так:

```text
z00z-node
  ├── da_client              # Celestia submit/fetch blobs
  ├── p2p_sync               # обмен checkpoint headers между nodes
  ├── mempool                # pending wallet bundles
  ├── aggregator_interface   # accepts tx bundles
  ├── executor               # applies spent_delta / created_delta
  ├── jmt_state              # Jellyfish Merkle Tree
  ├── proof_verifier         # Bulletproof+/Nova/etc
  ├── consensus              # HotStuff
  └── watcher_api            # local/public verification
```

Evolve/Rollkit помогает понять, где границы между DA, execution, sync, block production, node networking.

### B. DA-only sync для Z00Z

Это очень важно.

Z00Z watcher может вообще не доверять aggregator P2P. Он может сказать:

```text
Я синхронизируюсь только из Celestia DA.
Если checkpoint blob там есть и proof валиден — принимаю.
Если в P2P сказали одно, а в Celestia лежит другое — верю Celestia.
```

Это ложится на Evolve-style **DA-only sync**. ([GitHub](https://github.com/celestiaorg/optimint?utm_source=chatgpt.com))

Для Z00Z это критично, потому что Celestia становится “публичной памятью” твоего rollup.

### C. Hybrid sync

Для скорости Z00Z node может получать данные через P2P, но финально сверять с Celestia:

```text
fast path:
  P2P receives checkpoint immediately

safe path:
  later verify same checkpoint exists in Celestia DA
```

Это полезно для low-latency wallet UX: пользователь не ждёт долго, но безопасность приходит от DA.

### D. Block/checkpoint format

Rollkit/Evolve помогает думать не “transaction by transaction”, а “block/checkpoint by checkpoint”:

```yaml
Z00Z_Checkpoint_Header:
  chain_id: z00z-testnet
  epoch: 1245
  prev_root: 0xabc
  new_root: 0xdef
  tx_bundle_root: 0x111
  spent_delta_root: 0x222
  created_delta_root: 0x333
  da_height: 5839201
  da_namespace: z00z-testnet-v1
  da_blob_commitment: 0x444
  proposer_sig: 0x555
  hotstuff_qc: 0x666
```

Это как “block header” для Z00Z, только вместо balances/accounts у тебя commitments/deltas.

### E. Не изобретать sync logic с нуля

У Z00Z будут сложные вопросы:

```text
- что делать если DA blob есть, но P2P block нет?
- что делать если P2P block есть, но DA blob отсутствует?
- как догонять node после 3 месяцев offline?
- откуда брать canonical history?
- как отличать finalized checkpoint от garbage?
```

Evolve/Rollkit architecture даёт готовые паттерны мышления для таких случаев.

**Что не делать:** я бы не тащил Evolve/Rollkit как core dependency, если ты хочешь Rust + HotStuff + custom privacy execution. Бери архитектуру, термины, sync modes, checkpoint pipeline.

------

## 3. Prism — пример “sovereign rollup + key transparency + JMT proofs”

Prism интересен не как библиотека для прямого использования, а как **очень близкий reference case**. Prism работает как sovereign rollup на Celestia; Celestia не валидирует его blocks, сами nodes rollup network делают validation. ([docs.prism.rs](https://docs.prism.rs/rollup.html?utm_source=chatgpt.com)) В документации Prism также прямо упоминаются key transparency и Jellyfish Merkle Proofs. ([docs.prism.rs](https://docs.prism.rs/quickstart.html?utm_source=chatgpt.com))

Для Z00Z это полезно в трёх местах.

### A. Key transparency против подмены receiver key

У Z00Z есть проблема:

```text
Alice хочет отправить Bob.
Mallory подменил QR / view_pk Bob.
Alice шифрует coin на Mallory key.
Coin уходит не туда.
```

Prism-style key transparency может дать идею для optional registry:

```yaml
Z00Z_PaymentRequest_Registry:
  identity_pk: 0x...
  current_view_pk_commitment: 0x...
  req_id: 0x...
  valid_until: 2026-07-01
  signature: Sign(identity_sk, fields)
  celestia_anchor: height + namespace + blob_commitment
```

Важно: это **не должен быть глобальный address book**, иначе убьёшь privacy. Но для merchant/business payments это может быть полезно:

```text
“Проверь, что этот view_pk действительно опубликован/подписан владельцем identity_pk”
```

### B. JMT proof UX

Prism полезен как пример, как объяснять и строить proofs поверх JMT:

```text
- membership proof: coin/output exists
- non-membership / deleted proof: old input no longer exists
- root transition proof: prev_root -> new_root
```

У тебя в Z00Z уже JMT/deltas модель. Prism даёт хороший reference, как это оформить в документации и light-client verification.

### C. Split-view / equivocation protection

Для Z00Z важно, чтобы aggregator не показывал разным участникам разные истории:

```text
Wallet A видит root X
Wallet B видит root Y
Celestia DA показывает root Z
```

Prism/key-transparency мышление помогает формализовать:

```text
Canonical root is the one anchored in Celestia namespace
and accepted by Z00Z HotStuff QC.
```

То есть Prism полезен как security model example.

**Что не делать:** не превращать Z00Z в публичный identity registry. Для anonymous e-cash это опасно. Prism-style registry должен быть optional для merchants, exchanges, bridges, DAO, grants — не для обычных private users.

------

## 4. Hyperlane — cross-chain messages и TIA/assets

Hyperlane в Celestia context полезен для bridging TIA и arbitrary cross-chain messages. Celestia docs прямо говорят, что Hyperlane можно использовать для TIA bridging между Celestia и EVM-compatible chains, а также для arbitrary cross-chain messages. ([Celestia Documentation](https://docs.celestia.org/learn/features/bridging/hyperlane/?utm_source=chatgpt.com))

Для Z00Z это не core privacy layer, а **вход/выход из внешнего мира**.

### A. Fees в TIA / USDC / ETH

У Z00Z может быть проблема:

```text
Пользователь хочет пользоваться Z00Z, но у него нет Z00Z token.
У него есть USDC on Base или ETH on Arbitrum.
```

Через Hyperlane later можно сделать:

```text
USDC/Base -> bridge/message -> Z00Z fee gateway
```

И дальше aggregator получает оплату.

### B. Cross-chain settlement signals

Например:

```yaml
EVM_Deposit_Event:
  chain: Base
  token: USDC
  amount: 100
  recipient_commitment: 0x...
  message_to_z00z: mint_private_receipt
```

Hyperlane message может сказать Z00Z:

```text
На Base заблокировано 100 USDC.
Создай private Z00Z receipt/asset commitment.
```

### C. Treasury / DAO

Z00Z Treasury может держать часть средств не только в Z00Z, но и в TIA/USDC/ETH. Hyperlane помогает с cross-chain movement.

**Что не делать:** не строить безопасность Z00Z coin ownership на Hyperlane. Ownership Z00Z должен оставаться внутри Z00Z cryptography.

------

## 5. Axelar — более широкий interchain gateway

Axelar полезен похожим образом, но шире. Axelar docs описывают его как cross-chain communication для Web3. ([docs.axelar.dev](https://docs.axelar.dev/?utm_source=chatgpt.com)) Axelar также писал про integration с Rollkit/Celestia, чтобы новые rollups было проще подключать к EVM, Cosmos, Bitcoin, Polkadot и другим экосистемам. ([axelar.network](https://www.axelar.network/blog/celestia-rollkit-interoperability?utm_source=chatgpt.com))

Для Z00Z конкретная польза:

### A. Подключить Z00Z к большим liquidity ecosystems

Z00Z сам по себе сначала будет isolated. Axelar может помочь сделать:

```text
Ethereum / Base / Arbitrum / Cosmos / Bitcoin-adjacent liquidity
      -> Axelar
          -> Z00Z gateway
              -> private Z00Z asset
```

### B. Wrapped Z00Z на EVM

Например:

```text
Native private Z00Z inside Z00Z
Wrapped public wZ00Z on Base/Ethereum
```

Пользователь может:

```text
burn/lock wZ00Z on Base
  -> message to Z00Z
    -> receive private Z00Z coin in wallet
```

Обратное направление:

```text
spend private Z00Z
  -> prove burn/exit
    -> unlock/mint wZ00Z on EVM
```

### C. Investor story

Для инвесторов это звучит хорошо:

```text
Z00Z is not isolated. It can plug into EVM/Cosmos liquidity via Axelar/Hyperlane later.
```

**Что не делать:** не начинать с Axelar. Сначала нужен Z00Z core + Celestia DA. Cross-chain без core только усложнит.

------

## 6. Squid — самый UX-практичный слой

Squid — это не security layer, а UX/liquidity layer. Он даёт cross-chain swaps, bridges и contract calls через один integration/API/SDK across 100+ chains. ([docs.squidrouter.com](https://docs.squidrouter.com/?utm_source=chatgpt.com)) Squid также позиционирует себя как bridge + cross-chain swap в одном интерфейсе. ([Squid](https://www.squidrouter.com/?utm_source=chatgpt.com))

Для Z00Z это полезно в wallet.

### A. “У пользователя нет Z00Z, но есть USDC”

В Z00Z wallet можно сделать кнопку:

```text
Top up private Z00Z
```

Под капотом:

```text
USDC on Base
  -> Squid route
  -> bridge/swap
  -> Z00Z gateway receives value
  -> wallet receives private Z00Z coin commitment
```

### B. “Pay fees with anything”

Пользователь платит не обязательно Z00Z:

```text
User has:
  - ETH on Ethereum
  - USDC on Arbitrum
  - TIA on Celestia
  - SOL on Solana

Squid finds route:
  asset -> fee asset accepted by aggregator
```

Это решает onboarding.

### C. Treasury rebalancing

Z00Z Treasury может автоматически менять:

```text
TIA -> USDC
ETH -> TIA
wZ00Z -> USDC
```

через Squid routes.

**Что не делать:** не полагаться на Squid для privacy. Squid — публичный cross-chain route. Privacy начинается уже после входа в Z00Z private wallet.

------

## Самая практичная схема для Z00Z

## Phase 1 — core testnet

```text
Wallet
  -> Aggregator
    -> Z00Z checkpoint
      -> Celestia DA blob
        -> Celenium dashboard
          -> Watchers verify
```

Здесь нужен **Celenium**.

## Phase 2 — robust sovereign node

```text
Celestia DA
  -> DA-only sync
  -> Z00Z node catches up
  -> HotStuff finality
  -> JMT root transition
```

Здесь полезна **Evolve/Rollkit architecture**.

## Phase 3 — stronger security docs

```text
PaymentRequest / receiver key verification
JMT proofs
anti split-view model
canonical Celestia-anchored roots
```

Здесь полезен **Prism**.

## Phase 4 — liquidity and onboarding

```text
USDC/ETH/TIA/SOL
  -> Squid / Hyperlane / Axelar
    -> Z00Z gateway
      -> private Z00Z wallet asset
```

Здесь полезны **Hyperlane / Axelar / Squid**.

## Мой короткий verdict

```text
Celenium:
  брать почти сразу.
  Это dashboard, explorer, watcher, proof-of-DA для grants/investors.

Evolve/Rollkit:
  не обязательно брать код.
  Взять архитектуру node/sync/DA-only/hybrid sync.

Prism:
  взять идеи security model:
  key transparency, JMT proofs, sovereign rollup validation.

Hyperlane:
  later для TIA/assets/messages между Celestia/EVM.

Axelar:
  later для большого interchain gateway и wrapped Z00Z/liquidity.

Squid:
  later для UX: swap/bridge/pay fees with any token.
```

Главное: **Celenium и Evolve/Rollkit помогают построить сам Z00Z testnet. Prism помогает правильно оформить security model. Hyperlane/Axelar/Squid помогают потом подключить деньги и пользователей.**
