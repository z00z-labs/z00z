Да, **Ethereum можно сделать главным расчётным хабом для покупки и продажи Z00Z**. Но важное уточнение:

> **BOLD — не L2.** Это ERC-20 стейблкоин Liquity V2, выпускаемый протоколом на Ethereum mainnet и обеспеченный WETH, wstETH и rETH. Отдельные cross-chain-версии BOLD могут обращаться в Arbitrum, Base и Optimism через Chainlink CCIP/CCT, но основной Liquity V2 и эмиссия BOLD находятся на Ethereum L1. ([docs.liquity.org](https://docs.liquity.org/v2-faq/bold-and-earn?utm_source=chatgpt.com))

## Как построить маршрут через Ethereum

Наиболее практичная схема:

```text
Visa / Mastercard
        ↓
лицензированный fiat on-ramp
        ↓
ETH или USDC на Ethereum
        ↓
Ethereum DEX
        ↓
BOLD
        ↓
BOLD ↔ Z00Z cross-chain exchange
        ↓
native Z00Z в кошельке пользователя
```

Обратное направление:

```text
native Z00Z
    ↓
cross-chain exchange / intent solver
    ↓
BOLD на Ethereum
    ↓
DEX: BOLD → USDC/ETH
    ↓
лицензированный off-ramp
    ↓
банк / карта
```

Но последняя часть `BOLD ↔ native Z00Z` **не может быть выполнена обычным Ethereum DEX напрямую**, потому что native Z00Z не является ERC-20-токеном Ethereum.

Нужен один из двух механизмов.

## Вариант 1 — wZ00Z на Ethereum

Создаётся канонический ERC-20:

```text
wZ00Z on Ethereum
```

Он обеспечен native Z00Z в соотношении 1:1:

```text
native Z00Z locked on Z00Z
          ↕
wZ00Z minted/burned on Ethereum
```

Тогда внешний DEX может иметь обычный пул:

```text
BOLD / wZ00Z
```

Покупка:

```text
BOLD → wZ00Z → burn wZ00Z → native Z00Z
```

Продажа:

```text
native Z00Z → lock → mint wZ00Z → BOLD
```

### Преимущества

- обычный Ethereum DEX;
- стандартный ERC-20 и стандартные AMM;
- простая интеграция с агрегаторами;
- понятное формирование цены;
- можно использовать Uniswap-подобные площадки;
- BOLD остаётся основным stable settlement asset.

### Недостатки

- появляется bridge/gateway;
- безопасность Z00Z зависит от механизма mint/burn;
- необходимо защищать Ethereum-контракт и Z00Z-side vault;
- wZ00Z добавляет отдельную форму актива;
- нужно контролировать, чтобы `totalSupply(wZ00Z)` всегда соответствовал заблокированным Z00Z.

Для privacy-first Z00Z мост также может ухудшать приватность: Ethereum будет видеть суммы, время ввода и вывода. Поэтому потребуется batching, задержки или privacy-preserving withdrawal proofs, но эти механизмы нужно проектировать осторожно, чтобы они не выглядели как custodial mixer.

------

## Вариант 2 — cross-chain intents без публичного wZ00Z

Более подходящая для Z00Z модель:

```text
Пользователь:
"I give 100 BOLD on Ethereum
and want at least 97 Z00Z on Z00Z"
```

Solver:

1. получает или блокирует BOLD на Ethereum;
2. отправляет пользователю native Z00Z из собственного запаса;
3. позднее балансирует свои резервы;
4. конкурирует с другими solvers за лучший курс.

```text
User BOLD on Ethereum
        ↓
   signed intent
        ↓
solver / market maker
        ↓
native Z00Z to user
```

Здесь пользователю не обязательно получать wZ00Z. DEX-функция становится **cross-chain RFQ / intent market**, а не обычным AMM-пулом.

Именно для этого концептуально подходит NEAR Intents: пользователь указывает желаемый результат, а solver определяет мосты, swaps и settlement. В документации пример сформулирован как обмен актива на одной сети на актив в другой сети без ручного управления маршрутом. ([NEAR Intents](https://docs.near-intents.org/getting-started/what-are-intents?utm_source=chatgpt.com))

Но для этого потребуется:

- добавить Z00Z как поддерживаемую сеть;
- реализовать Z00Z deposit/withdrawal adapter;
- интегрировать подписи Z00Z;
- добавить BOLD в список поддерживаемых assets;
- привлечь хотя бы двух независимых solvers;
- обеспечить им запасы BOLD и Z00Z;
- определить finality и правила возврата при неисполнении.

NEAR Intents уже поддерживает Ethereum и ERC-20, но конкретная поддержка BOLD должна проверяться через актуальный список активов 1Click API. Список токенов у них динамический. ([NEAR Intents](https://docs.near-intents.org/resources/asset-support?utm_source=chatgpt.com))

## Нужен ли тогда вообще NEAR?

Если первоначально поддерживается только один маршрут:

```text
Ethereum ↔ Z00Z
```

то **NEAR Intents не является обязательным**.

Можно сделать отдельный внешний intent/RFQ-рынок:

```text
BOLD on Ethereum ↔ native Z00Z
```

Но тогда кто-то должен разработать и эксплуатировать:

- order relay;
- escrow;
- solver registry;
- quote API;
- dispute/refund mechanism;
- Ethereum watcher;
- Z00Z watcher;
- liquidity rebalancing.

NEAR Intents имеет смысл, если Z00Z планирует позже принимать множество источников:

```text
BOLD on Ethereum
USDC on Base
USDT on Tron
BTC
NEAR
SOL
        ↓
      Z00Z
```

Тогда NEAR Intents избавляет Z00Z от собственного набора маршрутов и отдельных мостов. При этом сам NEAR Intents использует разные bridges для различных сетей, и каждый такой bridge имеет собственную trust model. ([NEAR Intents](https://docs.near-intents.org/integration/bridging/overview?utm_source=chatgpt.com))

## Что лучше для Z00Z

Я бы сделал **Ethereum каноническим liquidity hub**, но не обязательно переносил бы всю торговлю Z00Z в ERC-20.

Рекомендуемая архитектура:

```text
Canonical stable settlement asset:
BOLD on Ethereum

Primary exchange:
BOLD on Ethereum ↔ native Z00Z

Execution:
external cross-chain DEX / solver network

Optional compatibility asset:
wZ00Z on Ethereum

Fiat:
licensed external on/off-ramp
```

То есть:

### Основной путь

```text
Card → ETH/USDC → BOLD → intent → native Z00Z
```

### Запасной путь

```text
BOLD → wZ00Z on Ethereum → bridge withdrawal → native Z00Z
```

### Продажа

```text
native Z00Z → intent → BOLD → USDC/ETH → fiat off-ramp
```

## Стоит ли обязательно покупать сначала BOLD

Не всегда.

Большинство card on-ramp-провайдеров с большей вероятностью поддержит:

- ETH;
- USDC;
- USDT,

чем непосредственно BOLD.

Поэтому UX должен позволять:

```text
Card → USDC → BOLD → Z00Z
```

но solver может оптимизировать маршрут до:

```text
Card → USDC → Z00Z
```

даже если расчётная и резервная единица liquidity provider внутри системы — BOLD.

Иначе пользователь заплатит:

1. комиссию on-ramp;
2. Ethereum gas;
3. `USDC → BOLD` swap fee;
4. slippage;
5. `BOLD → Z00Z` fee;
6. возможно, bridge/intent fee.

Поэтому **BOLD лучше использовать как канонический резерв и основной liquidity pair**, но не заставлять пользователя всегда вручную покупать BOLD.

## Регуляторная сторона от Ethereum не исчезает

Проведение маршрута через Ethereum или BOLD не отменяет регулирование fiat on/off-ramp.

С точки зрения интерфейса Z00Z наиболее безопасное разделение:

```text
1. Buy crypto
   Provided by licensed third party

2. Swap to Z00Z
   Executed by independent DEX / solvers

3. Withdraw to Z00Z wallet
   User-signed, non-custodial transaction
```

Z00Z dApp желательно:

- не принимать fiat;
- не становиться merchant of record;
- не хранить BOLD или Z00Z пользователей;
- не устанавливать собственный гарантированный курс;
- не обещать исполнение;
- не быть единственным solver;
- сначала не брать собственную комиссию с обмена;
- явно показывать, какой внешний провайдер выполняет каждую операцию.

## Моя рекомендация

**Да — выбрать Ethereum и BOLD как канонический buy/sell hub для Z00Z разумно.**

Но технически правильная формулировка такая:

> BOLD является Ethereum-native stablecoin, а не Ethereum L2. Z00Z может использовать BOLD на Ethereum как основной расчётный актив для внешнего cross-chain DEX или intent-рынка.

Для первой версии:

```text
Card → USDC/ETH on Ethereum
             ↓
       external DEX
             ↓
           BOLD
             ↓
external solver / NEAR Intents
             ↓
        native Z00Z
```

После появления достаточной ликвидности можно скрыть промежуточные шаги за одной кнопкой, но оставить юридическое и техническое разделение между on-ramp, DEX и Z00Z network.

---

---



Главное: **маршрут через Ethereum и BOLD заметно снижает один вид риска — риск того, что Z00Z сам выпускает и обслуживает стейблкоин. Но он не устраняет риск того, что Z00Z dApp будет признан организатором обмена или перевода криптоактивов.**

Регулятор смотрит не столько на Ethereum, BOLD или название «DEX», сколько на то, **какие функции фактически выполняет команда Z00Z**.

## Кто за что отвечает

В схеме:

```text
Visa/MC
  ↓
лицензированный on-ramp
  ↓
BOLD/USDC на Ethereum
  ↓
внешний DEX / solver / NEAR Intents
  ↓
native Z00Z
```

ответственность потенциально разделяется так:

| Участник               | Основной риск                                        |
| ---------------------- | ---------------------------------------------------- |
| Card on/off-ramp       | Fiat, KYC, AML, sanctions, fraud, chargebacks        |
| Ethereum DEX / solver  | Обмен и перевод crypto-assets                        |
| Liquity/BOLD ecosystem | Статус и функционирование stable-value crypto-asset  |
| Z00Z protocol          | Выпуск и функционирование Z00Z                       |
| Z00Z dApp/frontend     | Возможная организация или активное содействие обмену |

То, что DEX внешний, — существенный плюс. Но **внешний DEX не является автоматическим юридическим щитом**.

FATF прямо различает сам программный код и людей или компании, которые управляют DeFi-сервисом. Код сам по себе не считается VASP, однако создатели, владельцы или операторы могут попасть под определение VASP, когда они контролируют или существенно влияют на сервис и активно облегчают обмен или перевод. Среди индикаторов FATF перечисляет административные ключи, контроль интерфейса и параметров, получение комиссий и постоянные отношения с пользователями. ([FATF](https://www.fatf-gafi.org/content/dam/fatf-gafi/guidance/Updated-Guidance-VA-VASP.pdf))

## Риск по вариантам реализации

### 1. Наиболее безопасная модель

```text
Шаг 1: Buy BOLD
Provided by independent licensed provider

Шаг 2: Swap BOLD ↔ Z00Z
Executed through an independent DEX/intent protocol
```

Пользователь:

- сам открывает сайт или hosted widget on-ramp;
- заключает договор непосредственно с on-ramp;
- получает BOLD/USDC в собственный Ethereum-кошелёк;
- отдельно запрашивает quote у внешнего DEX;
- сам подписывает Ethereum-транзакцию;
- сам подписывает получение или отправку native Z00Z.

Z00Z при этом:

- не принимает fiat;
- не принимает BOLD на свои адреса;
- не хранит пользовательские ключи;
- не исполняет order от имени пользователя;
- не устанавливает курс;
- не гарантирует исполнение;
- не получает процент от swap;
- не является solver или market maker;
- не управляет ликвидностью.

**Оценочный риск для Z00Z: низкий–средний.**

Он всё равно не нулевой, потому что Z00Z поддерживает интерфейс и выпускает актив, но аргумент будет сильным:

> Z00Z предоставляет non-custodial wallet и технический доступ к независимым протоколам. Каждую финансовую операцию пользователь совершает напрямую с соответствующим сторонним провайдером.

------

### 2. Embedded widgets внутри Z00Z dApp

Например, on-ramp и DEX визуально находятся внутри кошелька, но:

- показывается название стороннего провайдера;
- пользователь принимает его Terms;
- KYC проходит у него;
- активы идут напрямую пользователю;
- transaction signing остаётся у пользователя;
- Z00Z backend не принимает и не перенаправляет средства.

**Риск: средний.**

Причина: интерфейс уже явно способствует приобретению Z00Z, хотя исполнителями остаются третьи стороны.

Здесь особенно важно не писать:

> Buy Z00Z from Z00Z

Лучше:

> Purchase crypto through a third-party provider

и затем:

> Swap through an independent protocol

------

### 3. Одна кнопка `Buy Z00Z with Visa`

```text
Visa → USDC/BOLD → DEX → solver → Z00Z
```

Пользователь видит одну цену и один общий процесс, а Z00Z backend:

- выбирает on-ramp;
- запрашивает DEX quotes;
- выбирает маршрут;
- формирует intent;
- отслеживает выполнение;
- показывает конечную гарантированную сумму Z00Z.

Даже без custody это может выглядеть как **execution или active facilitation of exchange**.

**Риск: средний–высокий.**

MiCA исключает из своей сферы только услуги, предоставляемые полностью децентрализованно без посредника. При этом обмен crypto-assets, исполнение поручений и transfer services входят в перечень регулируемых crypto-asset services. Управляемый компанией frontend не обязательно будет считаться полностью децентрализованным. ([EUR-Lex](https://eur-lex.europa.eu/legal-content/EN/TXT/PDF/?uri=CELEX%3A32023R1114&utm_source=chatgpt.com))

------

### 4. Z00Z получает комиссию

Например:

```text
DEX fee: 0.30%
Z00Z interface fee: 0.25%
```

или Z00Z получает:

- affiliate rebate;
- revenue share;
- referral fee за объём;
- spread;
- solver rebate;
- часть bridge fee.

Это существенно усиливает позицию регулятора:

> Z00Z ведёт коммерческую деятельность по организации обмена для пользователей.

FATF прямо рассматривает получение комиссии, spread или иной выгоды как важный показатель коммерческого exchange/transfer service. ([FATF](https://www.fatf-gafi.org/content/dam/fatf-gafi/guidance/Updated-Guidance-VA-VASP.pdf))

**Риск: высокий.**

Не всякая affiliate-комиссия автоматически требует лицензии, но она ухудшает юридическую позицию.

------

### 5. Z00Z сам становится solver или liquidity provider

Например:

```text
User sends 100 BOLD
Z00Z treasury sends 95 Z00Z
```

Даже когда всё выполняется smart contracts, Z00Z фактически:

- покупает и продаёт актив;
- устанавливает или принимает цену;
- поддерживает inventory;
- исполняет обмен;
- получает spread;
- обеспечивает settlement.

**Риск: очень высокий.**

В Израиле виртуальная валюта рассматривается как финансовый актив, а услуги покупки, продажи и конвертации виртуальных валют относятся к регулируемой области. Для поставщиков таких услуг действуют лицензирование и AML/CFT-обязательства. Израильский регулятор также публикует официальный список лицензированных поставщиков виртуально-валютных услуг. ([Government of Israel](https://www.gov.il/he/pages/guide_0016?utm_source=chatgpt.com))

## Что именно даёт использование BOLD

### Риск, который уменьшается

Z00Z не нужно:

- выпускать собственный USD-stablecoin;
- обещать погашение по $1;
- хранить банковские резервы;
- управлять collateralized stablecoin protocol;
- выступать stablecoin issuer;
- обеспечивать redemption в fiat.

То есть BOLD отделяет от Z00Z **stablecoin issuance layer**.

### Риски, которые остаются

Остаются:

- обмен BOLD ↔ Z00Z;
- перевод между Ethereum и Z00Z;
- управление пользовательским frontend;
- маркетинг и распространение Z00Z;
- AML/sanctions exposure;
- consumer-protection claims;
- возможная квалификация Z00Z как crypto-asset;
- возможная квалификация оператора dApp как VASP/CASP.

И нельзя утверждать пользователю:

> BOLD полностью регулируемый доллар.

или:

> BOLD гарантированно равен $1.

Корректнее:

> BOLD is a decentralized, crypto-backed stablecoin issued through the Liquity protocol. Its market value may deviate from its target value, and it is not a bank deposit or government-issued currency.

## Buy и sell имеют разный уровень риска

### Покупка Z00Z

```text
Card → KYC on-ramp → BOLD → Z00Z
```

Здесь источник fiat известен лицензированному on-ramp. Основная проблема — разрешит ли on-ramp перевод в систему с privacy-функциями.

### Продажа Z00Z

```text
Z00Z → BOLD → off-ramp → bank
```

Этот путь сложнее.

Off-ramp может спросить:

- откуда получены Z00Z;
- каким образом сформировалась сумма;
- кому принадлежал исходный актив;
- есть ли подтверждение цены приобретения;
- не связаны ли средства с санкционными адресами;
- может ли пользователь доказать source of funds.

Поскольку Z00Z задуман как privacy-first asset, обычный Ethereum chain-analysis может не показать предыдущую историю монеты. Это не делает операцию незаконной, но повышает вероятность:

- enhanced due diligence;
- задержки операции;
- отказа solver;
- отказа off-ramp;
- запроса дополнительных документов;
- блокировки банковского перевода до объяснения происхождения.

В Израиле текущий подход банковского надзора рассматривает работу через лицензированного virtual-currency provider как фактор, снижающий риск для банка. Поэтому маршрут через известного лицензированного off-ramp будет практически значительно лучше прямого вывода через неизвестный DEX-адрес. ([Government of Israel](https://www.gov.il/he/pages/message_20_07_26?utm_source=chatgpt.com))

## Что желательно встроить в Z00Z для легального off-ramp

Не глобальную деанонимизацию, а добровольный **proof of provenance**:

```text
Пользователь по своему решению доказывает:

- что он контролирует конкретные Z00Z;
- когда и законным способом они были получены;
- что они не были созданы двойной тратой;
- какую сумму он выводит;
- при необходимости — связь с первоначальной покупкой.
```

Возможные варианты:

- selective disclosure через view key;
- пользовательский transaction receipt;
- proof of acquisition;
- proof of ownership;
- подписанное подтверждение от regulated on-ramp;
- zero-knowledge compliance proof;
- отдельный disclosed exit mode для off-ramp.

Главное: раскрытие должно происходить **по инициативе пользователя конкретному off-ramp**, а не превращать всю Z00Z-сеть в публичную.

## Рекомендуемая юридическая граница

Архитектурно я бы закрепил:

```text
Z00Z Protocol
    не продаёт и не покупает активы

Z00Z Wallet
    non-custodial transaction construction

Fiat Provider
    самостоятельный лицензированный on/off-ramp

DEX / Intent Protocol
    самостоятельный внешний exchange layer

Solver
    независимый market participant

User
    самостоятельно выбирает провайдеров
    и подписывает каждую операцию
```

Дополнительно:

- отдельные Terms каждого провайдера;
- никакого общего «договора покупки Z00Z» с Z00Z;
- fiat никогда не проходит через Z00Z;
- Z00Z не обслуживает chargebacks/refunds;
- отсутствие обещаний ликвидности или гарантированного выхода;
- отсутствие собственного spread;
- отсутствие эксклюзивного solver;
- возможность выбрать другого провайдера;
- никаких формулировок «мы продаём Z00Z».

## Итоговая оценка

**Ethereum + BOLD — юридически лучше, чем выпуск собственного zUSD или прямая продажа Z00Z за карту.**

Но безопасен не сам маршрут, а правильное разделение ролей:

```text
Отдельный card on-ramp
+
отдельный Ethereum DEX
+
отдельный cross-chain solver
+
non-custodial Z00Z wallet
```

Для первого релиза я бы разрешил такую функцию, но только как две независимые операции:

```text
1. Get BOLD from a licensed provider
2. Swap BOLD for Z00Z through an external protocol
```

Единую кнопку `Buy/Sell Z00Z with Visa` и комиссию Z00Z за обмен лучше не запускать без отдельного заключения по VASP/CASP-лицензированию в Израиле и странах, где будет доступен интерфейс.