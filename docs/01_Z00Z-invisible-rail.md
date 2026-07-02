# Z00Z Invisible Rail

Да, это **главный продуктовый риск Z00Z**: если пользователь должен понимать `AssetLeaf`, `RightLeaf`, `VoucherPolicy`, `FeeEnvelope`, bridge, DA, checkpoint, nullifier — полезность погибает. Поэтому концептуально Z00Z надо проектировать не как “сложную систему прав”, а как **невидимый слой частного действия**.

Формула:

> **Пользователь не “настраивает Z00Z”. Пользователь делает действие: Pay, Claim, Use, Delegate, Prove. Z00Z сам выбирает правильный объект, fee path, settlement path и disclosure mode.**

То есть Z00Z должен ощущаться не как блокчейн, а как **private action button**.

------

## 1. Главный принцип: Z00Z должен быть “invisible rail”, а не отдельный мир

Пользователь не должен заходить “в Z00Z”, чтобы понять, что делать. Он должен находиться в Bittensor, Hyperliquid, DeXe, Astroport, DAO-приложении, AI-agent app, marketplace, wallet — и видеть одну понятную кнопку:

**Pay privately**
**Claim reward**
**Use credit**
**Give agent budget**
**Prove payment**
**Withdraw privately**

Z00Z в этом случае — не destination, а **частный слой выполнения**.

Это прямо совпадает с corpus-логикой: Z00Z не должен поглощать все внешние функции; внешние системы могут давать UX, custody, identity, scheduling, task coordination, а Z00Z дает private transfer and replay-safe settlement для самого right.

Практически это значит: **интегратор владеет контекстом, Z00Z владеет приватным экономическим действием**.

Например:

- в Hyperliquid пользователь видит не “создать Z00Z RightLeaf”, а “Fund trading agent privately”;
- в Bittensor subnet — не “redeem ClaimTxPackage”, а “Claim subnet reward privately”;
- в DeXe — не “issue voucher”, а “Pay contributor privately”;
- в Astroport — не “private LP right”, а “Receive LP reward privately”.

------

## 2. Самое важное: оставить native Z00Z скучным

Самая большая ошибка — сделать так, чтобы каждая Z00Z-монета могла быть “особенной”: с условиями, правилами, expiry, refund, правами, разрешениями. Это убьет понимание.

Правильный дизайн:

> **Обычный Z00Z = просто деньги. Все сложное — не внутри денег, а рядом: claim, voucher, right, budget, receipt.**

В документах это уже сформулировано как правило: native `Asset` должен оставаться clean final-value cash с фиксированной cash semantics, а не arbitrary programmable value. Получатель должен понимать: это деньги, а не условная претензия или скрытый policy object.

Это критично для one-click payment.

Пользовательская семантика должна быть такой:

| Что видит пользователь      | Что это внутри Z00Z       |
| --------------------------- | ------------------------- |
| “Я получил деньги”          | Clean Asset               |
| “Я получил claim”           | Voucher / claim           |
| “Мне дали доступ”           | Right                     |
| “Я дал агенту бюджет”       | Agent spending envelope   |
| “Комиссия уже включена”     | FeeEnvelope               |
| “Вот доказательство оплаты” | Receipt / selective proof |

То есть **не смешивать типы в голове пользователя**. Деньги — это деньги. Claim — это claim. Access — это access. Budget — это budget.

------

## 3. Свести всю систему к пяти глаголам

Я бы вообще запретил на первом уровне все технические nouns и оставил только **пять действий**.

### 1. Pay

“Заплатить приватно”.

Это должен быть самый простой режим. Без вопроса “какой тип права?”, “какой voucher?”, “какой settlement route?”. Просто:

> “Pay 100 USDC privately”
> “Pay 50 Z00Z privately”
> “Send private payment”

Внутри может быть Asset, external asset right, package, checkpoint — но пользователь этого не видит.

### 2. Claim

“Получить то, что мне положено”.

Это для Bittensor rewards, DAO grants, LP rewards, rebates, cashback, useful-work payout.

Пользователь не должен понимать, что такое `RewardAuthorization` или nullifier. Он видит:

> “Claim reward”
> “Claim rebate”
> “Claim bounty”
> “Claim LP incentive”

Внутри Z00Z проверяет право, предотвращает double-claim и выдает private payout.

### 3. Use

“Использовать право”.

Это для compute credits, API calls, inference, access pass, data access, subscription.

Пользователь видит:

> “Use 50 GPU minutes”
> “Use inference credits”
> “Access dataset”
> “Enter private session”

Не “передать токен”, а **потратить capability**.

### 4. Delegate

“Дать ограниченный бюджет агенту или приложению”.

Это особенно важно для AI и DeFi.

Пользователь не должен настраивать 15 параметров. Он выбирает готовый режим:

> “Give agent $100 for this task”
> “Allow trading up to $500, expires tonight”
> “Allow compute only, max 2 hours”
> “Allow DAO reviewer payout up to X”

Внутри это Agent Spending Envelope. Документы прямо говорят: агенту не нужен full wallet; ему нужен bounded private object — spend up to amount, buy only allowed services, act only inside task, expire later.

### 5. Prove

“Доказать, что условие выполнено, не раскрывая лишнего”.

Это для paywalls, DAO audit, enterprise compliance, access gates.

Пользователь видит:

> “Prove I paid”
> “Prove I’m eligible”
> “Prove this agent stayed within budget”
> “Prove this reward was authorized”

Не “раскрыть историю транзакций”, а **дать один scoped proof**.

------

## 4. Не “создать право”, а “выбрать карточку действия”

Чтобы Z00Z был простым, права должны поставляться как **карточки/рецепты**, а не как конструктор.

Плохая модель:

> “Выберите VoucherPolicy, RightPolicy, expiry, fee envelope, attestation root, disclosure mode…”

Хорошая модель:

> “Private DAO bounty payout”
> “Agent compute budget”
> “Trading agent risk envelope”
> “Private LP reward claim”
> “Private subscription pass”
> “One-time access ticket”
> “Private rebate claim”

Каждая карточка уже содержит sane defaults.

Например, карточка **Agent compute budget**:

- spend limit: preset;
- provider scope: compute providers only;
- expiry: 24h / 7d;
- fee: sponsored or included;
- audit: owner-only;
- unused value: returns automatically.

Пользователь может открыть “advanced”, но базовое действие остается one-click.

Это как банковская карта: ты не настраиваешь settlement network, interchange, fraud model, authorization hold. Ты просто платишь. Z00Z должен быть таким же: **сложность внутри шаблона, не в выборе пользователя**.

------

## 5. Fee должен исчезнуть из головы пользователя

Один из главных убийц простоты в crypto — gas.

Для Z00Z нельзя допустить, чтобы пользователь думал:

> “А чем я плачу fee?”
> “На какой сети gas?”
> “Мне нужен Z00Z для комиссии?”
> “Нужно ли агенту держать gas asset?”

На уровне идеи должно быть:

> **Каждое действие уже несет свой processing budget.**

То есть пользователь видит:

> “Fee included”
> “Paid by app”
> “Paid from reward”
> “Sponsored by campaign”
> “Deducted from claim”

Внутри это `FeeEnvelope`: отдельный объект, который отвечает, кто платит за publication, relay, verification, settlement. Документы подчеркивают, что right и fee path нельзя смешивать: right отвечает “что можно сделать”, FeeEnvelope отвечает “кто и как платит за обработку”.

Продуктовый принцип:

> **Ни один обычный пользователь и ни один обычный agent не должны получать unrestricted gas wallet. Fee всегда должен быть встроен, спонсирован или вычтен автоматически.**

------

## 6. Разделить “простое” и “мощное” не настройками, а слоями

Нужно не делать “простые настройки” и “сложные настройки” рядом. Нужно сделать **два разных слоя мышления**.

### User layer

Пять действий:

> Pay, Claim, Use, Delegate, Prove.

### App/integrator layer

Готовые recipes:

> DAO bounty payout, subnet reward claim, trading agent envelope, LP reward, access pass.

### Protocol layer

Технические объекты:

> Asset, Voucher, Right, FeeEnvelope, TxPackage, ClaimTxPackage, checkpoint evidence.

Обычный пользователь никогда не должен видеть protocol layer. Интегратор почти никогда не должен писать protocol layer вручную. Он должен выбирать recipe.

Это особенно важно потому, что Z00Z built around wallet-local possession and delayed settlement. Документы описывают модель так: possession begins in the wallet, verification can happen locally, final settlement happens later through checkpointed reconciliation. Пользователю это надо переводить не как “asynchronous rights settlement”, а как:

> “Можно использовать сейчас, сеть подтвердит потом.”

------

## 7. Ввести “одну кнопку, три статуса”

Чтобы Z00Z не казался неопределенным, каждое действие должно иметь максимум три понятных состояния:

1. **Ready** — можно использовать.
2. **Used / Sent / Claimed** — действие принято локально или отправлено.
3. **Settled** — подтверждено settlement.

Не надо показывать пользователю “package admitted”, “publication pending”, “checkpoint artifact”, “DA resolved”. Это статус для оператора, не для пользователя.

Для offline / delayed flows это особенно важно. В документах прямо сказано: package handoff не равен окончательной settlement finality; локальный импорт и handoff делают wallet aware of candidate ownership, но settlement требует publication, validation and reconciliation.

Поэтому пользовательская формулировка должна быть честной, но простой:

> “Received — pending settlement”
> “Usable with risk limit”
> “Settled”

Не “pending block confirmations”, не “checkpoint theorem”, не “artifact validation”.

------

## 8. “Z00Z inside”, а не “go to Z00Z”

Для интеграций с Bittensor, Hyperliquid, Astroport, DeXe и похожими проектами правильный дизайн — **Z00Z не должен быть отдельной destination chain в голове пользователя**.

Правильный паттерн:

> Пользователь остается в приложении партнера. Z00Z появляется только как private mode.

Примеры:

### В Bittensor

Кнопка:

> “Claim privately”

Не:

> “Bridge TAO into Z00Z rights settlement layer”.

### В Hyperliquid

Кнопка:

> “Fund agent with private limit”

Не:

> “Create bounded mandate right with FeeEnvelope and external settlement adapter”.

### В DeXe

Кнопка:

> “Pay contributor privately”

Не:

> “Issue a voucher authorized by DAO governance and consume claim path”.

### В Astroport

Кнопка:

> “Receive LP incentives privately”

Не:

> “Import LP eligibility attestation into Z00Z private claim domain”.

То есть **Z00Z должен быть privacy/payment/rights capability внутри чужого продукта**, не еще одним местом, куда нужно перейти.

------

## 9. Предустановленные “safety defaults” вместо настроек

Пользователь не должен выбирать безопасность. Безопасность должна быть default.

Для каждого типа действия должны быть стандартные ограничения:

| Действие | Default safety                                |
| -------- | --------------------------------------------- |
| Pay      | clean cash, no clawback, fee included         |
| Claim    | one claim only, expires, anti-double-claim    |
| Use      | limited provider, limited amount, expiry      |
| Delegate | max budget, max duration, provider whitelist  |
| Prove    | minimum disclosure, no full history reveal    |
| Withdraw | clear trust tier, visible external dependency |

Особенно важно для внешних активов. Z00Z не должен притворяться банком или гарантом чужих резервов. Документы формулируют границу так: Z00Z может гарантировать private transfer, replay-safe settlement, checkpoint continuity и internal protection against double-spending, но не external reserves, liquidity, legal redemption или issuer honesty.

Поэтому пользователь должен видеть не техническое предупреждение, а простую категорию:

> “Backed by external USDC locker”
> “Issuer-backed”
> “Internal credit”
> “Not redeemable outside this app”

Это не UX-деталь, а **концептуальная честность**.

------

## 10. Не называть все “токенами”

Еще один способ убить простоту — все назвать токенами.

“Access token”, “reward token”, “voucher token”, “agent token”, “LP token”, “fee token” — это делает систему нечитабельной.

Лучше сделать человеческую онтологию:

| Неудачное слово | Лучше                                            |
| --------------- | ------------------------------------------------ |
| token           | money / credit / claim / pass / budget / receipt |
| right           | permission / allowance / pass                    |
| voucher         | claim / coupon / grant / reward                  |
| FeeEnvelope     | fee included / sponsored                         |
| checkpoint      | settled                                          |
| nullifier       | already claimed protection                       |
| bridge          | deposit / withdraw                               |
| AssetLeaf       | private balance item                             |

Внутри можно сохранить точные protocol terms. Но наружу надо вывести **семантический язык действий**.

------

## 11. MVP должен быть не “универсальные права”, а 3 суперпростых flows

Чтобы не перегрузить запуск, я бы сделал первый продуктовый слой из трех flows.

### Flow 1: Private Pay

Самый базовый.

> “Send privately”
> “Receive privately”
> “Settled”

Это доказывает, что native Z00Z — не сложная программа, а clean cash.

### Flow 2: Private Claim

Для партнеров.

> “You have a reward. Claim privately.”

Это идеальный вход в Bittensor, DeXe, Astroport rewards, Hyperliquid rebates.

### Flow 3: Private Budget

Для agents/apps.

> “Give this app/agent a limited private budget.”

Это самый сильный Z00Z-native сценарий, потому что он решает реальную проблему: не давать агенту полный кошелек. Документы прямо говорят, что bounded rights снижают blast radius и не дают агенту унаследовать полный balance, strategy, counterparty map или future optionality владельца.

Все остальное — access passes, compute credits, LP rights, DAO selective audit — можно строить как варианты этих трех.

------

## 12. Как это должно звучать в одном предложении

Не так:

> “Z00Z is a privacy-first asynchronous rights settlement protocol with wallet-local possession, vouchers, rights, FeeEnvelopes and checkpointed reconciliation.”

А так:

> **“Z00Z lets users pay, claim rewards, use services, and give agents limited budgets privately — with one click, without giving apps full wallet power.”**

Еще короче:

> **“Private payments and private permissions, without wallet exposure.”**

И для интеграторов:

> **“Add one private button: Pay, Claim, Use, or Delegate. Z00Z handles the private settlement underneath.”**

------

## 13. Мой главный дизайн-закон для Z00Z

Я бы записал его как product constitution:

> **No raw rights for normal users. No manual fee decisions. No custom policy on native cash. No visible protocol vocabulary in partner apps. Every user action must collapse into Pay, Claim, Use, Delegate, or Prove.**

Тогда сложность Z00Z становится преимуществом, а не burden.

Пользователь не должен думать:

> “Я взаимодействую с правами, voucher policies и settlement evidence.”

Он должен думать:

> “Я заплатил приватно.”
> “Я получил reward.”
> “Я дал агенту ограниченный бюджет.”
> “Я доказал доступ, не раскрывая историю.”

Вот тогда Z00Z становится не “еще одной сложной privacy-chain”, а **one-click private action layer**.

## Что действительно retail-ценно

- Приватные everyday-платежи без публичного relationship graph. Главная ценность тут не “скрыть сумму”, а не создавать публичную карту “кто кому платит”; receive model специально уходит от постоянного публичного адреса к `ReceiverCard` и `PaymentRequest` [Z00Z-Main-Whitepaper.md, line 39](Z00Z-Main-Whitepaper.md), [Z00Z-Main-Whitepaper.md, line 456](Z00Z-Main-Whitepaper.md).
- Offline / low-connectivity cash semantics. QR/NFC/portable package + later reconciliation выглядит как самый понятный consumer wedge, особенно для in-person payment и плохой связи [Z00Z-UseCases-Whitepaper.md, line 217](Z00Z-UseCases-Whitepaper.md), [Z00Z-UseCases-Whitepaper.md, line 256](Z00Z-UseCases-Whitepaper.md), [Z00Z-Main-Whitepaper.md, line 462](Z00Z-Main-Whitepaper.md).
- Private one-shot rights вместо публичных allowances. Subscriptions, merchant-bound money, expiring vouchers, soft escrow/chargeback windows у docs оформлены как bounded private objects, а не как видимый onchain app-state [Z00Z-UseCases-Whitepaper.md, line 372](Z00Z-UseCases-Whitepaper.md), [Z00Z-UseCases-Whitepaper.md, line 429](Z00Z-UseCases-Whitepaper.md), [Z00Z-UseCases-Whitepaper.md, line 442](Z00Z-UseCases-Whitepaper.md).
- Private vouchers / coupons / community money. Это сильный mass-user surface, даже если buyer не end-user, а город, NGO, DAO или merchant program; wallet docs уже мыслят это как отдельный voucher/right inventory, включая transferable claim и partial redeem [Z00Z-UseCases-Whitepaper.md, line 575](Z00Z-UseCases-Whitepaper.md), [Z00Z-UseCases-Whitepaper.md, line 589](Z00Z-UseCases-Whitepaper.md), [WALLET-GUIDE.md, line 22](../crates/z00z_wallets/docs/WALLET-GUIDE.md), [WALLET-GUIDE.md, line 41](../crates/z00z_wallets/docs/WALLET-GUIDE.md).
- Accountless paid access. Один из самых недооценённых retail use cases в corpus: article/VPN/download/model/API session как private session right без reusable account/subscription graph [Z00Z-UseCases-Whitepaper.md, line 679](Z00Z-UseCases-Whitepaper.md), [Z00Z-UseCases-Whitepaper.md, line 685](Z00Z-UseCases-Whitepaper.md).
- Reduced first-use friction через sponsored fees / `FeeCredit`. Это важно именно для mass adoption: first use не должен начинаться с “сначала купи gas” [Z00Z-Litepaper.md, line 224](Z00Z-Litepaper.md), [Z00Z-Marketing-Srategy.md, line 255](Z00Z-Marketing-Srategy.md).
