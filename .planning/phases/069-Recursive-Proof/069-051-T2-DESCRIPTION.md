> Статус документа: сохранён как русскоязычное объяснение решений T2. Текущая
> исполняемая истина находится в `069-051-PLAN.md`,
> `069-051-PROFILING-REPORT.md`, итоговом `069-051-SUMMARY.md` после его
> закрытия, `069-051-T2-GAPS.md`,
> `069-051-T2-CRYPTO-AUDIT-2.md`, A-17 acceptance packet, активном operating
> budget и release-evidence. Числа, явно названные ниже «текущими», являются
> историческим T2-снимком, если рядом не дана более новая T3/T4 ссылка.

Да. Они прописаны не в одном месте, а в трёх слоях.

| Вопрос                               | Где зафиксирован                                             |
| ------------------------------------ | ------------------------------------------------------------ |
| F12 / F23 / F24 как T2 blockers      | [T2-GAPS: A–F closure table (line 58)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-T2-GAPS.md:58) |
| Конкретные незакрытые задачи F12/F23 | [T2-GAP-01 (line 80)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-T2-GAPS.md:80), [T2-GAP-02 (line 91)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-T2-GAPS.md:91) |
| F24 corpus                           | [T2-GAP-03 (line 103)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-T2-GAPS.md:103) — кодовая часть закрыта; остаётся честный non-claim о чужих dependency allocations |
| A-17                                 | [T2-GAPS: A-17 (line 405)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-T2-GAPS.md:405), [T2 crypto audit](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-T2-CRYPTO-AUDIT-2.md), отдельный [A-17 acceptance packet (line 1)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-A17-RESIDUAL-ACCEPTANCE-PACKET.md:1) |
| Operating budget и `k=1`             | [T2-GAPS: authority budget (line 430)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-T2-GAPS.md:430), [T2 crypto audit](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-T2-CRYPTO-AUDIT-2.md), [active generation-2 decision](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-AUTHORITY-OPERATING-BUDGET-DRAFT.md) |

Первичный источник F12/F23/F24 — `069-051-T1-DC2-RESOLUTION-LEDGER.md`; T2 их потребляет как обязательные входы.

### F12, F23, F24 — простыми словами

- **F12 — память.** Мы измерили, сколько native evaluator законно держит в памяти: `69,337,178 B` ≈ `66.13 MiB` профилированных буферов. Это не «лимит процесса», а контролируемый набор: 64 MiB JMT envelope, две current-record copies и 2 MiB sorter. Нужно решение: такой envelope допустим или надо перепроектировать streaming.
- **F23 — сохранность при падении.** Мы умеем убивать процесс до/после собственных операций записи и проверять, что хранилище не приходит в неоднозначное состояние. Но внутренние `fsync`/directory-sync redb скрыты внутри dependency. Решение: принять, что проверка project-owned boundary эквивалентна нужной гарантии redb, либо потребовать fork/redb-level fault injection.
- **F24 — утечки witness/secret.** Наш код проверен для success/error/panic/timeout/cancel/hard-kill: не пишет секреты в логи, core dump или retained artifacts. Но нельзя честно утверждать, что Rust allocator или чужая dependency физически зануляют каждую свою allocation. Поэтому corpus закрыт, а предел утверждения остаётся явным.

### Что такое A-17

Это не ошибка реализации. Это честная граница криптографического утверждения.

Nova theorem, на который опирается система, действует при предпосылках вроде EAGM, GZT, дискретного логарифма Pallas/Vesta и отдельной предпосылки compression backend. Тесты способны проверить implementation, но не способны доказать математические аксиомы для конкретного мира.

Выбранный путь такой: разрешено заявлять:

> knowledge soundness условна на перечисленные предпосылки.

Нельзя заявлять:

> безусловная 128-bit cumulative IVC security.

То есть A-17 не требует переписать circuit. Он требует принять корректную формулировку риска. [A-17 acceptance packet](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-A17-RESIDUAL-ACCEPTANCE-PACKET.md) прямо фиксирует: новая reduction — отдельное исследование, не обязательная T2 coding task.

### Что такое operating budget

На пальцах: успешный proof — это «самолёт один раз взлетел». Operating budget — это утверждённый лётный регламент:

- сколько памяти разрешено каждому этапу;
- сколько времени разрешено setup/fold/compress/verify;
- сколько verifier jobs допустимо;
- сколько шагов IVC допускается за жизнь generation;
- сколько одновременно prover процессов;
- максимальные размеры PP/PK/VK/proof/envelope;
- когда отменять или принудительно убивать процесс;
- что делать, если ни один кандидат не проходит.

Код измерил факты. Budget превращает эти факты в обязательные эксплуатационные пределы.

Активный generation-2 budget (исторический файл с суффиксом `DRAFT` сохранён
ради стабильности ссылок, но его содержимое имеет статус `ACTIVE`):

| Параметр                | Предложение            |
| ----------------------- | ---------------------- |
| setup                   | ≤30 s, ≤12 GiB         |
| один fold               | ≤1.5 s, ≤12 GiB        |
| compression prove       | ≤60 s, ≤12 GiB         |
| verifier cold-load      | ≤60 s, ≤4 GiB          |
| verifier check          | ≤15 s, ≤4 GiB          |
| полный prover job       | ≤1 h, ≤12 GiB          |
| native evaluator        | ≤1 GiB                 |
| concurrency             | 1                      |
| verifier fanout         | 64                     |
| proof / envelope        | ≤128 KiB / ≤512 KiB    |
| VK bundle               | ≤64 MiB                |
| verifier attempts `q_V` | `2^20 = 1,048,576`     |
| lifetime Nova steps `N` | `2^32 = 4,294,967,296` |

Это не произвольные числа: они выше измеренного `k=1` пути, но с запасом. Например, verifier VmHWM измерен как `3,063,189,504 B`, поэтому предлагаемый 4 GiB предел оставляет `1,231,777,792 B` запаса.

### Что такое `k=1`

`k` — compile-time SHA batch width: сколько SHA-256 block work включается в один Nova micro-step batch.

`k=1` означает самый узкий, уже измеренный вариант: один SHA block batch за micro-step. Для него зафиксированы конкретные circuit shape, PP/PK/VK, proof и verifier identity. Это не настройка, которую безопасно менять во время работы.

Почему его нужно выбрать явно:

- другое `k` → другая R1CS shape;
- другая shape → другие PP/PK/VK;
- другие параметры → другой proof identity;
- runtime выбор или смешивание `k` ломает единственный canonical verification path.

Фактически для `k=1` уже есть такие данные:

- ShapeCS: `C=533,794`, `V=401,550`, `NZ=2,036,733`;
- complete proof: 1,727 steps;
- remaining folds: `580.308 ms/fold`;
- clean verifier: `29.496 s`;
- full proof + Model C: `2,127.806 s`;
- proof: `122,288 B`;
- envelope: `342,353 B`;
- peak full-proof RSS: `6,605,221,888 B`.

Иными словами: `k=1` технически прошёл. Но фраза «это production width» — это уже решение владельца системы, потому что оно утверждает расходы, throughput и policy отказа.

`q_V` и `q_U` — разные числа:

- `q_U` — математический параметр uniqueness proof.
- `q_V` — максимум verifier attempts, который система готова принять за authority generation; это защита от operational abuse/DoS и часть общей оценки риска.

В активном решении выбран `k=1`, `{1}` как единственное допустимое множество,
`q_V=2^20`, `N=2^32`, один Nova prover и 90-дневное cyclic challenge window.



---

---



Коротко: `k=2/3` пока не существуют как рабочие режимы. Сейчас circuit и все артефакты жёстко привязаны к `k=1`. Nova должна обновляться после каждого finalized checkpoint, но переносимое compressed proof не создаётся на каждый блок: плановая частота — примерно раз в 1000 блоков. При текущей производительности mixed checkpoint требует около 16,7 минуты только на folding, поэтому 5-секундный поток блоков пока не обслуживается в реальном времени.

## Что означает `k`

`k` — количество 64-байтовых блоков SHA-256, которые circuit обрабатывает за один SHA micro-step. Это не параметр security bits.

Сейчас:

```
k = 1
один Nova SHA step → один SHA-256 compression block
```

Это compile-time identity circuit, а не настройка запуска: [nova.rs (line 20503)](/home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/nova.rs:20503).

При `k=2` circuit должен за один fold последовательно доказать:

```
SHA state_i
    → compression(block_i)
    → SHA state_i+1
    → compression(block_i+1)
    → SHA state_i+2
```

При `k=3` — три compression transitions.

Если один checkpoint содержит:

- `O` остальных micro-steps;
- SHA jobs длинами `h₁, h₂, ...` блоков,

то приблизительное число steps:

```
N₁ = O + Σ hᵢ
Nₖ = O + Σ ceil(hᵢ / k)
```

Нельзя объединять блоки через границу разных SHA jobs.

| `k`  | Выигрыш                               | Цена                                          |
| ---- | ------------------------------------- | --------------------------------------------- |
| 1    | минимальный circuit и memory per fold | больше folds                                  |
| 2    | до 2× меньше именно SHA-block steps   | SHA lane примерно вдвое шире, новые PP/PK/VK  |
| 3    | до 3× меньше именно SHA-block steps   | ещё более тяжёлый fixed-shape circuit         |
| 4+   | дальнейшее уменьшение SHA steps       | быстро растут setup, keys, RSS и fold latency |

Важный нюанс: Nova circuit fixed-shape. Расширив SHA lane до `k=2/3`, мы можем увеличить стоимость не только SHA steps, но фактически каждого fold. Поэтому общее время:

```
Tₖ = Nₖ × latency_fold(k)
```

может уменьшиться, остаться прежним или даже вырасти.

Теоретический пример, если игнорировать рост стоимости fold:

| Доля SHA steps | `k=2`                                              | `k=3`                    |
| -------------- | -------------------------------------------------- | ------------------------ |
| 50%            | steps уменьшатся до 75% — максимум 1,33× ускорение | до 66,7% — максимум 1,5× |
| 90%            | до 55% — максимум 1,82×                            | до 40% — максимум 2,5×   |

Но это верхняя оценка. Реальных цифр для `k=2/3` нет. Для каждого нового `k` понадобятся новый circuit shape, PP/PK/VK, verifier bundle, proof corpus и release/RSS measurement. Существующие `k=1` proofs станут несовместимы.

## Реальное время Nova при `k=1`

Текущие release-измерения полного mixed checkpoint: [069-051-BENCHMARKS.md (line 22)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-BENCHMARKS.md:22).

| Операция               | Измеренное время     | Как часто должна происходить                      |
| ---------------------- | -------------------- | ------------------------------------------------- |
| PP setup               | 11,074 с             | один раз на generation/start, не на блок          |
| Создание accumulator   | 0,423 с              | один раз                                          |
| Первый `prove_step`    | <1 мс                | Nova initialization                               |
| Один последующий fold  | в среднем 580,308 мс | один раз на micro-step                            |
| 1726 рабочих folds     | 1001,612 с           | полный mixed fixture                              |
| Compression setup      | 3,827 с              | на compression request                            |
| Compression prove      | 19,331 с             | на compression request                            |
| Warm/in-process verify | 5,045 с              | при проверке envelope                             |
| Cold verifier process  | 29,496 с             | загрузка VK bundle + verify                       |
| Полный audit worker    | 2127,806 с           | тестовый режим с повторным Model C, не production |

Отсюда:

```
один micro-fold       ≈ 0,580 с
mixed block folding   ≈ 1001,6 с = 16 мин 41,6 с
до первого verified envelope в fresh worker ≈ 17 мин 47,5 с
полный audit + Model C                    ≈ 35 мин 27,8 с
```

Это не значит, что каждый production block обязательно будет содержать 1727 steps. Это fixture, включающий все 17 opcode classes. Распределение реальных блоков пока не измерено.

Но если такой mixed block репрезентативен, текущая производительность не соответствует 5-секундному block time:

```
требуется: 1727 / 5 ≈ 345 micro-steps/с
измерено: 1 / 0,580 ≈ 1,72 micro-steps/с
дефицит: примерно 200×
```

Даже ускорение в 10× было бы недостаточно для этого workload. Асинхронность не решает sustained throughput: один IVC accumulator последователен и будет накапливать backlog.

Также `19,331 с` compression измерены для accumulator с 1727 steps. Compression 1000-block chain ещё не измерялась, поэтому переносить эту цифру на epoch без проверки нельзя.

## Какой интервал используется

План разделяет четыре независимые частоты: [069-06-PLAN.md (line 189)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-06-PLAN.md:189).

| Действие              | Плановый интервал          | При блоке 5 с |
| --------------------- | -------------------------- | ------------- |
| Nova folding          | каждый finalized block     | каждые 5 с    |
| Recovery snapshot     | кандидат: 100 блоков       | 8 мин 20 с    |
| Compression           | кандидат: 1000 блоков      | 83 мин 20 с   |
| Publication           | кандидат: 1000 блоков      | 83 мин 20 с   |
| On-demand compression | по авторизованному запросу | независимо    |

То есть не «одно compressed proof на каждый блок».

После блока:

1. Непрерывный accumulator получает все micro-steps блока.
2. Проверяется полученный final state.
3. Accumulator остаётся в памяти и продолжается на следующем блоке.
4. На высоте, кратной примерно 100, сохраняется recovery snapshot.
5. На границе 1000 блоков берётся non-consuming snapshot accumulator и запускается compression.
6. Полученный envelope проверяется и сохраняется content-addressed.

Эта архитектура описана в T3: [069-051-PLAN.md](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-PLAN.md). Continuous same-accumulator runner реализован и принят release-тестом для высот `1/3/5` при кумулятивных шагах `316/948/1580`; текущее source-bound свидетельство находится в `crates/z00z_storage/outputs/checkpoint/069-051/final/t3-chain-source-302/run.log`.

## Что именно кладётся в Celestia

Здесь есть два разных объекта, которые нельзя смешивать.

### 1. Per-block checkpoint DA payload

Целевой `CheckpointDaPayloadV2` публикуется для каждого checkpoint. Он должен содержать только:

- network/context;
- batch ID и height;
- statement/core digest;
- previous/new state roots;
- previous/new settlement roots;
- route/config generation;
- exec input ID;
- prep snapshot ID;
- challenge-content root;
- challenge-window length;
- optional уже известный epoch-close-anchor digest.

В нём запрещены:

- raw transaction package;
- полный `CheckpointExecInput`;
- raw nullifiers;
- witness/JMT delta;
- QC или future finalized record;
- Nova proof;
- PP/PK/VK;
- provider-specific Celestia types.

Нормативный состав описан здесь: [069-TODO.md (line 1785)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-TODO.md:1785).

Точный размер `CheckpointDaPayloadV2` пока назвать нельзя: canonical V2 codec ещё не подключён и не измерен.

Текущий live adapter всё ещё формирует legacy `celestia-local.payload.v1`, включающий JSON transaction package, execution input и nullifiers: [da.rs (line 513)](/home/vadim/Projects/z00z/crates/z00z_rollup_node/src/da.rs:513). Это не целевой production format.

### 2. Nova proof envelope

Текущий измеренный `NovaProofEnvelopeV2`:

| Компонент                 | Размер        |
| ------------------------- | ------------- |
| Header                    | 353 B         |
| Initial public state `z₀` | 109 856 B     |
| Final public state `zₙ`   | 109 856 B     |
| Compressed proof          | 122 288 B     |
| **Итого**                 | **342 353 B** |

Envelope содержит bundle digest, endpoints, heights, cumulative steps и proof digest, но не PP/PK/VK: [nova.rs (line 21736)](/home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/nova.rs:21736).

Отдельно:

- PP+PK recovery material: `858 785 714 B`, prover-local;
- verifier bundle: `47 008 185 B`, загружается verifier-ролью один раз на generation;
- ни один из них не повторяется в каждом proof envelope.

Nova envelope должен храниться один раз по content digest в `artifacts/checkpoints/nova_block`. В consensus/обычном block traffic передаётся только digest/reference. Отправка полного proof каждому validator на каждый блок запрещена.

Если поздний Plan выберет Celestia как provider для Nova evidence, envelope будет отдельным cadence-публикуемым объектом, а не частью каждого `CheckpointDaPayloadV2`.

## Сколько это занимает за 90–180 дней

Текущий нормативный default — 90 дней, не 180:

```
5 секунд на блок
17 280 блоков/день
90 дней  = 1 555 200 блоков
180 дней = 3 110 400 блоков
```

90-дневный window начинается от `da_publication_ready`, а не от локальных часов: [069-TODO.md (line 1899)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-TODO.md:1899).

### Если сохранять Nova envelope раз в 1000 блоков

Это лишь расчёт без GC:

| Период   | Envelopes | Объём                 |
| -------- | --------- | --------------------- |
| День     | 17,28     | 5,916 MB              |
| 90 дней  | ≈1555     | 532,4 MB = 507,8 MiB  |
| 180 дней | ≈3110     | 1,065 GB = 1015,5 MiB |

Если ошибочно класть envelope в каждый блок:

| Период   | Объём                |
| -------- | -------------------- |
| 90 дней  | 532,4 GB = 495,9 GiB |
| 180 дней | 1,065 TB = 991,7 GiB |

Именно поэтому per-block proof publication не используется.

Фактическая Nova retention policy ещё строже:

- максимум 8 epochs без PQ coverage;
- максимум 2 proof bodies на epoch;
- максимум 16 bodies;
- максимум 2 MiB compressed-proof body bytes.

Для текущего proof:

```
122 288 B × 16 = 1 956 608 B
```

Это укладывается в 2 MiB body cap. Но 16 полных envelopes дали бы около `5,224 MiB`; body cap не включает два public states и header. Полный physical-storage budget ещё должен быть утверждён в последующих планах.

### Размер основного 90-дневного challenge ring

Это не Nova proofs. Там хранятся:

- raw transaction packages и исходные proof bytes;
- canonical replay;
- необходимые недеривируемые witness/delta bytes;
- checkpoint artifacts, links, QC/finality bodies;
- archive manifests и retrieval evidence.

Данные losslessly deduplicated и кодируются RS(10,16), то есть coding overhead ровно `1,6×`.

Если `D` — средний объём уникальных challenge bytes на блок после dedup, но до RS:

```
90 дней:  D × 1 555 200 × 1,6
180 дней: D × 3 110 400 × 1,6
```

Примеры:

| Средние challenge bytes на блок | 90 дней   | 180 дней  |
| ------------------------------- | --------- | --------- |
| 1 KiB                           | 2,37 GiB  | 4,75 GiB  |
| 10 KiB                          | 23,73 GiB | 47,46 GiB |
| 100 KiB                         | 237,3 GiB | 474,6 GiB |
| 1 MiB                           | 2,37 TiB  | 4,75 TiB  |

К этим числам добавятся manifests, receipts, audit и filesystem overhead. Поэтому точный capacity budget требует измерения реального transaction traffic.

## Полный путь от transaction batch до finalized checkpoint

```
Transactions
    ↓
Aggregator admission + deterministic ordering
    ↓
OrderedBatch + route/generation/plan digest
    ↓
CheckpointExecInput + SettlementExecHandoff
    ↓
Preflight execution on isolated state clone
    ↓
Canonical trace + JMT update trace + expected post-roots
    ↓
Live/staged state transition
    ↓
CheckpointArtifact + Link + DA payload
    ↓
Celestia inclusion / DA-ready evidence
    ↓
Validator checks + quorum certificate
    ↓
FinalizedCheckpointRecord + canonical state head
    ↓
Asynchronous Nova fold
    ↓ every ~100 blocks
Recovery snapshot
    ↓ every ~1000 blocks
Nova compression/publication + independent Plonky3 epoch proof
```

### 1. Aggregator принимает transactions

`AggregatorIngress::admit` превращает входной `WorkPayload` в проверенный `WorkItem`. Затем `AggregatorOrdering::order` детерминированно строит `OrderedBatch`: [service.rs (line 27)](/home/vadim/Projects/z00z/crates/z00z_runtime/aggregators/src/service.rs:27).

`OrderedBatch` связывает:

- `BatchId`;
- ordered items;
- создаваемые settlement leaves;
- shard ID;
- routing generation;
- route-table digest;
- intake IDs;
- operation count;
- plan digest.

Структура находится здесь: [types.rs (line 243)](/home/vadim/Projects/z00z/crates/z00z_runtime/aggregators/src/types.rs:243).

### 2. Формируется execution handoff

Из ordered batch, storage operations и checkpoint transactions создаётся один `SettlementExecHandoff`.

Он фиксирует:

```
batch_id
shard_id
routing_generation
route_table_digest
ordered StoreOp[]
CheckpointExecTx[]
```

Это граница между aggregator и storage execution. Нельзя отдельно подменить route или список операций.

### 3. Создаётся checkpoint transition

`CanonicalCheckpointTransitionV2::from_exec` загружает:

- checkpoint binding;
- predecessor/link;
- prep snapshot;
- authority/config generation;
- pre-state root;
- pre-definition root.

Затем выполняются две фазы: [canonical_transition.rs (line 228)](/home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/canonical_transition.rs:228).

Первая — preflight:

1. Клонируется settlement state.
2. На клоне применяется handoff.
3. Вычисляются semantic flow, JMT update trace и post roots.
4. Строятся canonical source records.
5. Trace precommit полностью запечатывается до live mutation.

Вторая — live transition:

1. Тот же immutable handoff применяется к live/staged store.
2. Сравниваются preflight и live flow.
3. Сравниваются JMT traces, root, definition root и storage generation.
4. Любое расхождение fail-closed.

После этого `evaluate()` независимо replay-ит sealed trace против post-state, а `finish()` фиксирует source completion.

### 4. Формируется checkpoint artifact

Из draft, transaction package, execution input, roots и link выводятся:

- statement core;
- execution-input ID;
- archive manifest;
- DA payload commitment;
- DA reference;
- `CheckpointArtifact`;
- `CheckpointId`.

Сам checkpoint — не Nova proof. Это канонический объект перехода состояния.

### 5. Публикация в Celestia

В целевой архитектуре storage кодирует один `CheckpointDaPayloadV2` и передаёт exact bytes в Celestia namespace.

Celestia возвращает/подтверждает:

- namespace;
- blob commitment;
- inclusion reference;
- Celestia height;
- payload commitment.

Это создаёт `DA-ready` evidence и запускает 90-дневный retention clock.

Важно: DA inclusion доказывает доступность bytes, но не корректность Z00Z state transition.

### 6. QC и finality

Validators проверяют:

- checkpoint statement;
- predecessor;
- old/new roots;
- route/config generation;
- transaction/settlement theorem;
- DA-ready evidence.

После голосов формируется quorum certificate. Lifecycle идёт:

```
Sealed
→ Linked
→ PublicationReady
→ ChallengeOpen
→ Finalized
```

Эти переходы зафиксированы в [lifecycle.rs (line 111)](/home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/lifecycle.rs:111).

90-дневный retention window не означает, что блок ждёт 90 дней до finality. Блок финализируется сейчас, а доказательные bytes продолжают храниться для challenge/replay.

### 7. Nova включается после finalization

После каждого finalized checkpoint shadow prover:

1. Берёт exact canonical trace.
2. Проверяет ожидаемый predecessor и accumulator state.
3. Последовательно fold-ит все micro-steps блока.
4. Сравнивает полученный `zₙ` с независимым native result.
5. Продолжает тот же accumulator на следующем блоке.

Nova не должна задерживать canonical finality. Если Nova отстаёт или падает, блоки продолжают финализироваться, но recursive evidence lane накапливает gap/backpressure.

На высоте 1000 canonical epoch закрывается без ожидания Nova compression или Plonky3. После закрытия запускаются два независимых asynchronous jobs:

- Nova compression/publication;
- Plonky3 exact-epoch proof.

## Что реально работает после T3

Реализованы T2 circuit, proof envelope, измеренный proof corpus, continuous
same-accumulator runner, non-consuming snapshots и публичный receipt только
после post-write reload/reverify. При этом:

- `k=2/3` намеренно не реализованы: authority выбрала единственный `k=1`;
- Plan 06 всё ещё владеет persistent accumulator recovery и runtime cadence;
- `CheckpointDaPayloadV2` не интегрирован;
- реальная Celestia публикация Nova envelope не интегрирована;
- текущий `CelestiaLocalAdapter` — локальный provider simulator;
- production call sites пока не вызывают `CanonicalCheckpointTransitionV2::from_exec`; его текущие вызовы находятся в тестах;
- T3 вычислительно закрыт; T4 владеет release/profiling/review closeout.

Это прямо зафиксировано в [069-051-T2-GAPS.md (line 565)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-T2-GAPS.md:565).

------

**Проверка ключевых утверждений**

- **VERIFIED:** `k=1` — compile-time SHA batch width, а не runtime/security параметр.
- **VERIFIED:** 1727-step fixture, 580,308 мс/fold, 122 288 B proof и 342 353 B envelope взяты из текущего release ledger.
- **VERIFIED:** fold=1 block, recovery≈100, compression/publication≈1000 являются отдельными cadence.
- **VERIFIED:** целевой per-block Celestia payload не содержит Nova proof.
- **VERIFIED:** continuous runner реализован T3; Celestia V2 integration остаётся владельцам Plans 06/09/12.

----

---





## Почему clean verifier занимает 29,496 с

Это не 29,5 секунды чистой криптографической проверки proof. Это intentionally cold verifier process без кэша:

1. Читает verifier bundle размером `47 008 185 B`.
2. Проверяет header, authority identity и digests.
3. Распаковывает VK из 47 MB в `859 756 576 B`.
4. Проверяет структуру огромного VK:
   - canonical curve/scalar encodings;
   - порядок primary/secondary keys;
   - отсутствие identity/default generators;
   - commitment/IPA key ranges;
   - derandomization keys.
5. Декодирует весь VK.
6. Повторно zstd-сжимает 860 MB и сравнивает с исходными bytes для canonical re-encode.
7. Загружает envelope.
8. Декодирует proof.
9. Выполняет Nova cryptographic verification.
10. Сравнивает все 3433 limbs конечного состояния с ожидаемым checkpoint successor.

Это видно в loader: [nova.rs (line 21592)](/home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/nova.rs:21592).

Приблизительное разложение по соседнему измеренному pipeline:

| Стадия                                            | Время        |
| ------------------------------------------------- | ------------ |
| Cold load/decompress/validate VK bundle           | ≈22,5 с      |
| Envelope decode + Nova proof verify               | ≈5,0 с       |
| Process startup, file I/O, fixture reconstruction | ≈2 с         |
| **Итого**                                         | **29,496 с** |

Основная проблема — не proof размером 122 KB, а огромный decoded VK размером примерно 820 MiB и несколько полных проходов по нему. Peak verifier RSS — около 3,06 GB.

В production VK должен загружаться и валидироваться один раз на authority generation. Тогда последующие проверки должны быть ближе к измеренным `5,045 с`, а не к `29,496 с`. Но такой persistent verifier cache/runner относится к T3/later и пока не интегрирован.

## Что такое Models A/B/C

Это не режимы Nova. Это три уровня проверки возможной уязвимости, определённые Phase 069: [069-051-PLAN.md (line 815)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-PLAN.md:815).

### Model A — ошибка самой теоремы

Вопрос:

> Можно ли получить два разных semantic executions или endpoints, удовлетворяющих одному и тому же public statement?

Примеры:

- разные transaction effects дают одинаковый statement;
- JMT root не однозначно связан с updates;
- witness можно заменить без изменения public input;
- predecessor или final root недостаточно связан.

Model A проверяется независимым evaluator и semantic mutation corpus.

Текущий статус: соответствующие гипотезы отклонены — «killed by semantic corpus».

### Model B — расхождение native code и circuit

Одни и те же typed trace bytes прогоняются через:

- canonical native evaluator;
- Nova `StepCircuit`;
- native transcript;
- in-circuit transcript;
- независимую test-only reference implementation.

Сравниваются:

- каждый intermediate state;
- absorbed bytes и domain tags;
- challenges;
- counters;
- roots;
- `z₀` и `zₙ`.

Model B ищет ошибку вида:

```
native evaluator принимает transition
но circuit доказывает немного другую relation
```

Текущий статус: mutation families отклоняются TestCS/R1CS checks — «killed by per-family R1CS corpus».

### Model C — настоящий proof для неправильного checkpoint

Это самый дорогой уровень.

Тест создаёт:

1. Нормальный 1727-step proof для target checkpoint.
2. Другой checkpoint с изменённым typed predecessor/statement.
3. Полностью пересчитывает для него второй настоящий 1727-step Nova proof под теми же PP/VK.
4. Неизменённый Nova verifier правильно принимает второй proof для его собственного statement.
5. Затем application-level comparator пытается применить его к первоначальному target checkpoint.
6. Сравнение всех endpoint limbs обязано его отвергнуть.

Реализация теста: [nova.rs (line 29460)](/home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/nova.rs:29460).

Это проверяет критическую границу:

```
валидный proof для checkpoint B
не должен приниматься как proof для checkpoint A
```

Model C — не checksum mutation. Второй proof математически валиден, поэтому требуется повторить почти весь proving pipeline.

### Откуда взялись 2127,806 с

| Работа                                | Приблизительно |
| ------------------------------------- | -------------- |
| Первый полный proof/envelope pipeline | ≈1067,5 с      |
| Отдельный clean verifier              | 29,496 с       |
| Второй полный proof Model C           | 1030,604 с     |
| **Итого audit worker**                | **2127,806 с** |

То есть это не production latency одного proof. Это два полных proving run плюс отдельная cold-process verification.

Сводный статус Models A/B/C находится в [mutation ledger (line 32)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-MUTATION-LEDGER.md:32).

## Идёт ли proof `122 288 B` в DA?

Самостоятельно — нет.

`122 288 B` — только serialized `CompressedSNARK`. В нём недостаточно project-level информации, чтобы безопасно определить:

- для какого checkpoint он создан;
- какой predecessor использован;
- какой verifier bundle выбран;
- какие `z₀` и `zₙ`;
- сколько выполнено steps;
- какой public-input digest;
- какой height range.

Raw proof должен находиться внутри `NovaProofEnvelopeV2`.

```
122 288 B proof
        ↓
включается в
        ↓
342 353 B NovaProofEnvelopeV2
```

Публиковать или принимать только 122 288 proof bytes было бы небезопасно.

## Идёт ли envelope `342 353 B` в DA?

Целевая архитектура: да, это единственный portable Nova proof object, но:

- не в каждом блоке;
- не внутри обычного per-block `CheckpointDaPayloadV2`;
- только с отдельной cadence, кандидат — раз в 1000 блоков или on-demand;
- content-addressed;
- consensus передаёт преимущественно digest/reference;
- полные bytes получают только recursive-verifier roles.

Состав:

```
353 B       header
109 856 B   z₀
109 856 B   zₙ
122 288 B   compressed proof
────────────────────────────
342 353 B   полный envelope
```

Сейчас live Celestia publication этого envelope ещё отсутствует. Текущий adapter публикует legacy `celestia-local.payload.v1`, а Nova/Celestia integration относится к Plans 06/09/12: [069-051-T2-GAPS.md (line 565)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-T2-GAPS.md:565).

Итоговая схема:

| Объект                 | В обычный per-block DA payload              | Отдельная Nova publication                          |
| ---------------------- | ------------------------------------------- | --------------------------------------------------- |
| Raw proof 122 288 B    | Нет                                         | Только внутри envelope                              |
| Envelope 342 353 B     | Нет                                         | Да, планово раз в ~1000 блоков                      |
| PP/PK 858,8 MB         | Никогда                                     | Никогда, prover-local                               |
| VK bundle 47 MB        | Никогда                                     | Один раз на authority generation для verifier roles |
| Proof digest/reference | Возможно/планируется через evidence binding | Да, для announce/content addressing                 |

Главное уточнение: `342 353 B` — точный текущий portable envelope, но его реальная публикация в Celestia пока ещё не реализована.
