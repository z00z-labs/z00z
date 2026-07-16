Да, модель с challenge window 60–90 дней логична. Но после окна checkpoint можно будет только перепроверить, а не перепровести заново из первичных данных.

Разница принципиальна:

- **verify forever**: есть proof, public statement, state root, manifest и параметры verifier → любой проверяет корректность checkpoint.
- **reprove / replay forever**: нужны raw transactions, witnesses, delta-журналы → без них это уже невозможно.

Правильная целевая политика:

| Хранить вечно                                                | Хранить 60–90 дней                                           |
| ------------------------------------------------------------ | ------------------------------------------------------------ |
| Finalized epoch checkpoint: height/range, state roots, finality certificate, Plonky3 proof, `EpochManifestV2`, verifier/parameter manifest и исторический VK | raw transaction packages                                     |
| Последние/периодические bootstrap state snapshots и их commitments | witnesses и execution traces                                 |
| Текущее состояние: UTXO/nullifier/JMT state, нужное для новых транзакций | deltas/journals, если snapshot уже закреплён                 |
| Nova/Plonky parameter-version registry                       | Nova per-block proofs, internal Plonky base/aggregation proofs, prover spools |

После 90 дней raw-data можно удалить, только если одновременно:

1. Epoch proof и manifest финализированы.
2. Есть protocol-level finality certificate.
3. Все challenge/reorg периоды закрыты.
4. Доступность raw-data в окно доказана независимыми архивными replicas и retrieval audits.
5. Нет active forensic/legal hold.
6. Есть проверяемый state snapshot, чтобы новый узел мог стартовать без replay от genesis.

Но у текущего Phase 069 есть ограничение: recursive evidence пока **non-authoritative**, а Plonky3 не доказывает end-to-end validity всех вложенных signature/range/spend примитивов. Поэтому в текущем плане proof ещё не может быть юридическим триггером для удаления raw-data через 60–90 дней. Для этого нужна отдельная finality/promotion спецификация.

И ещё поправка к формулировке: навсегда нужно хранить не checkpoint, который можно «reprove», а checkpoint, который можно **verify**. Reprove после удаления raw невозможно по определению.

Это резко уменьшает perpetual archive: постоянно растут только epoch proof + manifest + snapshots, а большие raw/witness/delta blobs образуют ограниченное 60–90-дневное кольцо.



----

---



Сейчас честный ответ: **в секундах ещё неизвестно**. Есть измеренные байты Nova, но нет принятого benchmark результата для `setup`, `fold`, `compress`, `prove` и `verify`; строгий verifier ещё не завершён. Для Plonky3 локальных временных результатов вообще нет — branch пока выключен.

| Операция                        | Частота при блоке 5 s                   | Требование для устойчивой работы                  | CPU-профиль                                                  |
| ------------------------------- | --------------------------------------- | ------------------------------------------------- | ------------------------------------------------------------ |
| Nova setup: PP/PK/VK generation | только при key rotation                 | не влияет на block time                           | самая тяжёлая Nova операция; отдельный bounded worker, cap RSS 24 GiB |
| Nova fold                       | каждый блок                             | среднее должно быть `< 5 s`, иначе растёт backlog | последовательная IVC цепь, CPU-bound                         |
| Nova compression                | по выбранной snapshot cadence           | async, не должна тормозить canonical checkpoint   | тяжёлая, но не обязана быть каждый блок                      |
| Nova verify                     | при получении snapshot                  | отдельный budget                                  | значительно легче prover, но секунд пока нет                 |
| Plonky3 epoch prove             | раз в 1000 блоков = раз в 83 min 20 sec | среднее `< 5000 s`, иначе epoch backlog           | наиболее тяжёлая периодическая задача: 1000 canonical transitions + STARK recursion |
| Plonky3 verify                  | раз в epoch                             | отдельный budget                                  | легче epoch prover, но пока не измерен                       |

Ключевой смысл CPU:

- **Nova fold** должен укладываться в block interval: это постоянная нагрузка.
- **Nova compression** можно делать реже и в фоне.
- **Plonky3** может занимать минуты, если всё равно укладывается в свои 83 минуты; это не block-critical операция.
- `24 GiB` — не «агрегатор всегда ест 24 GiB», а потолок безопасности для худшего Nova setup/proving job. Реальное RSS и CPU seconds ещё должны быть измерены.

Текущий YAML всё ещё содержит `compressed_proof_snapshot_cadence_blocks: 1`, но по исправленному плану это не должна считаться утверждённой production-политикой до измерений Plan 06/11. В частности, она не доказывает, что compression раз в 5 секунд выдержит hardware. [Измерительный gate (line 34)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-11-PLAN.md:34)



---

---



Nova fold — это непрерывное «сворачивание» доказательств:

```
A₀ + block₁ → A₁
A₁ + block₂ → A₂
A₂ + block₃ → A₃
...
```

`Aₙ` доказывает: «все переходы состояния от начала до блока `n` корректны». Новый блок добавляется к уже существующему аккумулятору; старые доказательства не нужно заново пересчитывать. Fold выполняется на агрегаторе для каждого блока, но сам по себе не обязан создавать файл, отправляемый в сеть.

Nova compression — это создание компактного, переносимого снимка текущего аккумулятора:

```
живой Nova accumulator A₁₀₀
          │
          ├─ продолжает fold: A₁₀₁, A₁₀₂, …
          │
          └─ compression → компактный proof P₁₀₀
```

Compression не сбрасывает и не «съедает» `A₁₀₀`. Это асинхронный снимок: его можно проверить другим узлом с VK. Именно тут появляется ваш измеренный proof ≈ 37 KiB; PP/PK/VK в этот proof не входят.

Практически:

- **Nova fold** — быстрый внутренний конвейер агрегатора, на каждый блок.
- **Nova compression** — редкое сохранение/публикация проверяемого снимка по выбранной cadence, например раз в 100 блоков, а не обязательно раз в 5 секунд.
- **Nova snapshot** — временное доказательство непрерывности, ECC и не post-quantum.

Plonky3 накладывается не как «доказательство того, что Nova доказал». Он строит отдельное STARK-доказательство всего точного диапазона epoch:

```
blocks 1 … 1000
  │
  ├─ Nova: A₁ → … → A₁₀₀₀, возможны snapshots P₁₀₀, P₂₀₀, …
  │
  └─ Plonky3: заново доказывает canonical transitions 1…1000
                  ↓
             EpochProof E₁
```

У `EpochProof E₁` можно привязать `nova_chain_root` — хеш-цепочку Nova-снимков — для согласованности и аудита. Но Plonky3 обязан доказать сами переходы состояния по canonical данным, а не просто проверить один Nova proof. Иначе post-quantum слой фактически зависел бы от ECC Nova и не давал бы независимой защиты.

Итоговая логика:

| Слой             | Частота                           | Роль                                                    | Где живёт                            |
| ---------------- | --------------------------------- | ------------------------------------------------------- | ------------------------------------ |
| Nova fold        | каждый блок                       | непрерывная внутренняя аккумуляция                      | память/рабочее хранилище агрегатора  |
| Nova compression | по cadence                        | компактный проверяемый snapshot                         | временный sidecar до закрытия epoch  |
| Plonky3          | раз в epoch, например 1000 блоков | независимое PQ-ориентированное доказательство диапазона | постоянный epoch artifact + manifest |

Если Nova compression делать каждые 100 блоков, то за epoch 1000 блоков будет максимум 10 Nova-снимков, а не 1000. После успешного Plonky3 epoch proof и архивной проверки эти Nova-sidecar’ы можно удалять по политике retention; вечным артефактом остаётся epoch proof, manifest и необходимые checkpoint/state snapshots.

Важно: в текущем Phase 069 оба контура пока evidence-only, не источник консенсусной finality.

---

----





При модели с 90-дневным challenge window вечно не должны храниться ни PP/PK, ни Nova snapshots, ни raw/witness/delta. Вечный компактный слой должен быть таким:

- canonical headers / finality certificates / state roots;
- один Plonky3 epoch proof на epoch;
- тонкие epoch/archive manifests, audit/rotation ledger;
- VK только пока на него ссылается сохранённое доказательство;
- актуальные bootstrap snapshots с ротацией, а не бесконечно все snapshot bodies.

При 5 сек на блок:

- `17 280` блоков/день;
- epoch = `1 000` блоков = 83 мин 20 сек;
- `17.28` epoch/день;
- snapshot раз в `10 000` блоков = `1.728` snapshot/день.

| Вечный объект                            | Оценка роста на одну архивную реплику                        |
| ---------------------------------------- | ------------------------------------------------------------ |
| Plonky3 proof, target 0.5–4 MiB/epoch    | **8.64–69.12 MiB/день**                                      |
| Plonky3 proof, hard cap 16 MiB/epoch     | **до 276.48 MiB/день**                                       |
| Epoch/archive manifests, receipts, roots | должны быть KB, но пока не измерены                          |
| Snapshot chunks                          | `1.728 × размер snapshot` в день, если хранить каждый навсегда |

Последняя строка важна: snapshot-body может быть главным источником вечного роста. Правильная policy — постоянно хранить лишь последние `N` проверенных bootstrap snapshots, а старые snapshot chunks удалять после появления и аудита нового; их manifests и state roots остаются навсегда. Иначе при snapshot размером 1 GiB получится +1.728 GiB/день на реплику.

Nova сюда не входит:

- измеренный compressed Nova proof: **36.92 KiB**;
- если по ошибке compress/persist каждый блок — **≈623 MiB/день**, но это временный слой, не forever archive;
- при compression раз в 100 блоков — **≈6.23 MiB/день**, и он живёт только до готового Plonky3 epoch proof;
- PP 436.57 MiB и VK 260.52 MiB — не «в день»: это материал одной key generation/rotation. PP приватен, VK хранится лишь пока есть сохранённые proof, которые его требуют.

Но текущая конфигурация пока строже и дороже желаемой модели: `raw_tx_packages` и `witness_data` имеют `archive_required`, а `archive_node_pruning_allowed: false`. Поэтому raw/witness/delta фактически растут бессрочно; 90 дней (`1,555,200` блоков) сейчас лишь минимальное условие для pruning обычного full node, не архива. Это видно в [checkpoint_contract.yaml (line 204)](/home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml:204) и [retention policy (line 219)](/home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml:219).

Их дневной объём сейчас невозможно честно назвать: он зависит от фактических tx/witness/delta. Формула:

```
raw archive per day = 17,280 × средний размер raw + witness + delta + exact tx proofs
```

Для масштаба: один лишь лимит `max_batch_bytes = 8 MiB` означает теоретический потолок **135 GiB/день**, а не ожидаемый расход. Это лимит защиты, не capacity estimate.

Значит, есть явное проектное противоречие: желаемая тобой 60–90-day модель требует сделать raw/witness/delta **window-retained**, а не `archive_required` forever. Это безопасно только после появления настоящего finality/expiry правила: закрыто окно challenge, epoch proof и manifest проверены, архив в течение окна был доступен, есть проверенный snapshot для bootstrap. В Phase 069 пока proofs non-authoritative, поэтому прямо сейчас удаление raw через 90 дней ещё нельзя считать обоснованным.



---

---



Да, но важная поправка: «без серверов и провайдеров» буквально невозможно — данные всегда лежат на чьих-то дисках. Реальная цель: без единственного оператора и без собственных постоянно работающих серверов.

| Вариант                   | Модель                                                       | Хорош для                                          | Плох для                                                     |
| ------------------------- | ------------------------------------------------------------ | -------------------------------------------------- | ------------------------------------------------------------ |
| **Arweave**               | разовая предоплата за экономически «permanent» хранение      | маленьких вечных P3 proof, manifests, certificates | raw/witness: удалить потом нельзя, запись может быть дорогой |
| **Filecoin direct deals** | аренда у независимых storage providers, доказательства хранения | холодного raw-архива, чтение редкое                | нужно renew/repair; получение может быть медленнее и платным; сделки по docs 180–540 дней |
| **Sia**                   | контракты с множеством hosts, платятся storage + upload/download bandwidth | зашифрованного архивного слоя                      | нужен клиент/autopilot для renew; это не eternal storage     |
| **Walrus**                | erasure coding через storage committee, срок покупается заранее | 90–730 дней, без выбора конкретного провайдера     | не вечное; данные оплачиваются по закодированному размеру ≈4.5× + 64 MiB overhead на blob |
| **Swarm**                 | prepaid postage stamps, storage rent                         | децентрализованной публикации на ограниченный срок | stamp истекает, требуется top-up; срок и цена динамические   |
| **IPFS**                  | адресация и транспорт, не storage                            | CID, распространение и hot cache                   | сам по себе ничего не гарантирует: без pinning bytes могут исчезнуть |

IPFS нельзя считать архивом: официальная документация прямо говорит, что cache может быть очищен, а persistence требует pinning. [IPFS persistence](https://docs.ipfs.tech/concepts/persistence/)

Для Z00Z я бы выбрал не один backend, а три слоя:

```
Вечные compact proofs/manifests  → Arweave + CID/root в canonical chain
Raw/witness/delta на 90–180 дней → Filecoin или Sia, зашифрованные и пакетированные
Без внешнего рынка               → shards у validator/watcher archive quorum
```

Самый интересный «без провайдеров» вариант — сделать storage обязанностью сети:

```
epoch archive
  → encrypt
  → Reed–Solomon 10-of-16 shards
  → 16 независимых watcher/archive nodes
  → периодические availability challenges
```

Для восстановления достаточно любых 10 из 16 shards. Физический объём — около `1.6×` исходных данных вместо `3×` при трёх полных копиях; чтение сложнее, потому что нужно собрать кворум и декодировать. Это не внешний провайдер, но watchers тогда становятся протокольными storage-операторами: нужны stake, штрафы, разные failure domains и retrieval audits.

Для 90-day raw данных особенно важно:

- хранить не блоками по 5 секунд, а крупными epoch/day CAR bundles — иначе комиссии и metadata съедят экономику;
- шифровать raw/witness per-epoch ключом;
- после expiry удалять/уничтожать ключ. Нельзя доказать, что все внешние узлы физически удалили ciphertext, но без ключа plaintext становится недоступным;
- не класть raw в Arweave: «90 дней» там не реализуемо.

Коротко по рынку:

- **Filecoin** — наиболее близок к «дёшево записать, редко и потенциально дороже/медленнее читать». Storage deal подтверждается Proof-of-Spacetime; retrieval может идти непосредственно через storage provider и оплачиваться. [Filecoin storage proving](https://docs.filecoin.io/storage-providers/filecoin-economics/storage-proving), [retrieval market](https://docs.filecoin.io/basics/what-is-filecoin/retrieval-market)
- **Sia** — похожая рыночная модель, но bandwidth платный в обе стороны и надо поддерживать контракты. [Sia storage contracts](https://docs.sia.tech/store-your-data/about-renting)
- **Walrus** удобнее для короткого периода, но не выглядит «холодно-дешёвым»: срок максимум около двух лет, а малые объекты особенно невыгодны из-за 64 MiB overhead. [Walrus pricing](https://docs.wal.app/docs/system-overview/storage-costs)
- **Arweave** — для вечного малого доказательного слоя: платишь при записи, сеть использует endowment для дальнейшего хранения. [Arweave storage endowment](https://docs.arweave.org/developers/development/protocol)
- **Swarm** — не permanent: stamp постепенно расходуется и данные могут быть удалены по истечении TTL. [Swarm postage stamps](https://docs.ethswarm.org/docs/concepts/incentives/postage-stamps/)

Для нашей схемы я бы не отправлял 697 MiB PP/VK bundle ни в один из этих слоёв. В public/permanent storage идут только P3 proof, тонкие manifests, finality roots и rotation records; Nova PP/PK остаются у prover, Nova snapshots живут до закрытия epoch.

---

---



Главный вывод: **135 GiB/день — не требование системы**. Это результат умножения аварийного лимита `8 MiB` на каждый 5-секундный блок. Capacity так считать нельзя.

Но есть физическое ограничение: если пользователи действительно создают 135 GiB уникальных, уже высокоэнтропийных данных в день, lossless compression не превратит их в 1 GiB. Можно:

- устранить дублирование;
- не сохранять производные данные;
- хранить полный материал ограниченное время;
- распределить shards между узлами;
- ограничить средний ingress;
- после finality заменить историю компактным доказательством её корректности.

ZK-proof — это «semantic compression»: он сохраняет утверждение «переход корректен», но из proof нельзя восстановить исходные транзакции.

## Зачем вообще нужна история

| Возможность                                                  | Что требуется                                              |
| ------------------------------------------------------------ | ---------------------------------------------------------- |
| Проверить актуальное состояние                               | state root + finality certificate + latest recursive proof |
| Запустить новый узел                                         | актуальный state snapshot + proof                          |
| Проверить старую транзакцию                                  | tx receipt/inclusion proof у пользователя либо архив tx    |
| Повторно проиграть старый блок                               | raw tx + exact proofs + исходное state                     |
| Построить новое доказательство после криптографического бага | практически полная история                                 |
| Explorer и историческая аналитика                            | tx bodies/index                                            |
| Challenge в течение 90 дней                                  | полный материал только внутри окна                         |

Я бы сделал обязательным для протокола первые два пункта и 90-дневный challenge. Полную историческую реконструкцию после 90 дней — optional archive service, а не обязанность каждого узла.

## Рекомендуемая иерархия

```
L0 — transient, часы
  uncompressed ingestion
  proving traces
  HJMT witnesses
  Nova working state

L1 — до завершения epoch
  Nova snapshots
  internal Plonky3 proofs
  replay-ready block material

L2 — cyclic 90-day window
  compressed EpochPack
  canonical raw tx/proofs stored ровно один раз
  state deltas
  journals
  content-addressed index

L3 — permanent
  block/epoch roots
  finality certificates
  thin manifests
  rolling Plonky3 proof
  current state snapshot root
  wallet-recovery stream
```

После 90 дней L2 удаляется/перезаписывается логически. В content-addressed storage это не буквальный overwrite: старые CID unpin, ссылки удаляются, неиспользуемые chunks проходят garbage collection. Roots и manifests остаются.

## Где сейчас есть реальное дублирование

Текущий DA payload одновременно сериализует:

- `TxPackage` как JSON;
- `CheckpointExecInput` как binary.

Это видно в [da.rs (line 511)](/home/vadim/Projects/z00z/crates/z00z_rollup_node/src/da.rs:511). При этом `CheckpointExecInput` уже содержит `tx_proof` внутри каждой транзакции — [exec_input.rs (line 171)](/home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/exec_input.rs:171). Затем отдельно создаётся ещё один `exact_tx_proof_bytes`.

То есть одни proof bytes представлены:

```
TxPackage.proof
CheckpointExecTx.tx_proof
exact_tx_proof_bytes
```

DA payload гарантированно содержит первые две формы; archive contract моделирует третью. Это нужно заменить на:

```
ProofBlob {
    digest,
    bytes
}

RawTxRecord {
    fields,
    proof_blob_digest
}

ExecRecord {
    inputs,
    outputs,
    proof_blob_digest
}

WitnessRecord {
    source_exec_digest,
    derived_path_metadata
}
```

Один canonical blob — множество ссылок. Это lossless и не выбрасывает информацию.

## EpochPack вместо 17 280 маленьких объектов

Каждые 1000 блоков формируется один immutable pack:

```
EpochPackV1
├── header + schema/version
├── block offset index
├── canonical transaction section
├── unique proof blob section
├── state delta section
├── nullifier/spent section
├── journal section
└── Merkle chunk table
```

Порядок обработки:

```
canonical binary
→ content deduplication
→ column/delta encoding
→ Zstd dictionary compression
→ encryption при необходимости
→ erasure coding
```

Proof/signature bytes обычно уже похожи на случайные и почти не сжимаются. Основная экономия будет от:

- отказа от JSON и hex в архивном формате;
- единственного экземпляра proof;
- удаления повторяющихся строк `chain_name`, `kind`, `status`, версий;
- delta/varint encoding;
- упаковки 1000 блоков одним словарём;
- отсутствия файловой/manifest overhead на каждый блок.

## Witness не нужно хранить как самостоятельную копию

Witness — производный объект. Его можно восстанавливать по запросу:

```
base snapshot
+ canonical transactions
+ ordered state deltas
→ state at block H
→ HJMT membership paths
→ proving witness
```

Это ровно модель «дёшево писать, дорого читать». Challenge старого блока может потребовать загрузить snapshot и проиграть несколько тысяч deltas, зато мы не сохраняем HJMT path для каждой транзакции.

Permanent остаётся `witness_root`; witness bytes живут только внутри challenge window или генерируются заново.

## Rolling state вместо бесконечных snapshots

`StateSnapshotV1` уже является тонким manifest из roots/digests, а не многогигабайтным state dump — [state_snapshot.rs (line 36)](/home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/state_snapshot.rs:36).

Физическое состояние стоит организовать как content-addressed copy-on-write HJMT:

```
live root Sₕ
window-base root Sₕ₋W
несколько recent recovery roots
shared immutable chunks
```

Где `W = 1,555,200` блоков, то есть 90 дней.

При новом блоке:

1. Live state переходит `Sₕ → Sₕ₊₁`.
2. Window-base применяет самый старый delta.
3. Истёкший EpochPack удаляется.
4. Chunks, недостижимые ни из live root, ни из window-base/recovery roots, проходят GC.

После 90-го дня объём challenge-архива выходит на плато. Растёт только активное состояние и компактный permanent ledger.

## Иерархическая компрессия доказательств

Необязательно хранить каждый Plonky3 epoch proof навсегда:

```
1000 блоков → EpochProof
~18 EpochProof → DailyProof
30 DailyProof → MonthlyProof
MonthlyProof + следующий месяц → rolling history proof
```

Каждый следующий уровень доказывает:

- корректность предыдущего Plonky3 accumulator;
- новый canonical transition range;
- новый MMR/manifest root.

После успешного DailyProof нижние EpochProof можно удалить. После MonthlyProof можно удалить DailyProof. Навсегда остаются:

- latest rolling proof;
- manifests/leaf roots;
- периодические boundary proofs при upgrade/key rotation.

Так permanent proof storage можно приблизить к `0.5–4 MiB/день` или даже к почти постоянному latest-proof storage вместо нынешней идеи `17.28 proofs/day`. Исторические roots продолжают расти, но это килобайты/мегабайты, не гигабайты.

## Варианты и компромиссы

| Архитектура                                | Объём                               | Что теряем                                           |
| ------------------------------------------ | ----------------------------------- | ---------------------------------------------------- |
| Всё raw навсегда                           | бесконечно растёт                   | ничего; слишком дорого                               |
| **90-day ring + rolling proof**            | ограниченный                        | нельзя replay/reprove историю старше 90 дней         |
| Tx bodies навсегда, witnesses 90 дней      | средний                             | поздняя генерация старых witnesses затруднена        |
| Только state/proof/roots                   | минимальный                         | нет explorer/history, wallet должен хранить receipts |
| Вся история erasure-sharded между watchers | низко на один узел                  | общая сеть всё равно хранит растущий объём           |
| Community archive optional                 | ноль обязательного protocol storage | нет гарантии доступности старой истории              |

Мой выбор: **90-day ring + rolling state + rolling Plonky3 proof + отдельный компактный wallet-recovery stream**.

Последний нужен потому, что после удаления raw новый wallet иначе может не найти старые принадлежащие ему outputs. В permanent recovery stream достаточно хранить output commitments/encrypted note announcements и spent/nullifier status — без transaction proofs и witnesses.

## Реальные бюджеты

При 5-секундных блоках:

```
1 GiB/day ÷ 17,280 = 60.7 KiB уникальных данных/block
100 MiB/day ÷ 17,280 = 5.9 KiB/block
```

Следовательно, одного `max_batch_bytes = 8 MiB` недостаточно. Нужны отдельные:

```
max_burst_bytes_per_block
target_unique_bytes_per_day
rolling_ingress_token_bucket
storage_fee_per_unique_byte
max_90d_window_bytes
```

Если фактический ingress равен 1 GiB/day:

- один 90-day logical ring: 90 GiB;
- при Reed–Solomon `10-of-16`: 144 GiB на всю сеть;
- каждый из 16 archive watchers хранит примерно 9 GiB.

Это уже не пугающий объём. Но если мы хотим существенно меньше, протокол обязан ограничивать средний ingress или перекладывать receipts/history на пользователей.

## Что я бы проверил первым

1. Byte-attribution benchmark на реалистичных блоках: сколько занимает unique tx, повторный proof, JSON/hex, exec input, witness, delta.
2. Прототип `EpochPackV1` на 1000 блоков: binary + dedup + Zstd dictionary.
3. Восстановление witness для случайного блока из snapshot+deltas.
4. Симуляция 100 дней: после дня 90 disk должен выйти на стабильное плато.
5. Rolling Plonky3 spike: предыдущий P3 accumulator + новый epoch без роста размера proof.

Критическая граница: текущий Phase 069 ещё не даёт authoritative end-to-end доказательство всех transaction signature/range/spend semantics. Поэтому 90-day deletion нельзя включать до promotion этого theorem/finality пути. Архитектурно ring правильный; криптографический gate для него пока не закрыт.

EXPANDABLE: details available for `EpochPackV1`, rolling P3 accumulator и точной retention-матрицы.

----

-----



Да, ты прав: **сеть не обязана хранить пользовательскую transaction history**. Если Z00Z изначально строится как client-held-data protocol, каждый wallet хранит собственные receipts, а сеть хранит только актуальное состояние и доказательства его корректности.

## Правильное разделение

Wallet хранит:

- собственные входящие и исходящие transactions;
- `FinalizedReceiptV1`;
- принадлежащие ему output leaves;
- amounts, blindings и private metadata;
- checkpoint inclusion proof;
- finality certificate;
- историю расходов;
- encrypted backup.

Сеть хранит:

- актуальный unspent state;
- spent/nullifier accumulator;
- текущий state root;
- latest rolling Nova/Plonky3 proof;
- compact finality/key-rotation metadata;
- raw/witness/delta только во временном challenge buffer.

```
Wallet = моя история и мои доказательства
Network = текущее глобальное состояние и доказательство его корректности
```

В репозитории уже есть wallet-owned `ReceiptStorage` — [receipt_storage.rs (line 9)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/persistence/receipt_storage.rs:9). Full backup также отдельно восстанавливает tx-history sidecar — [wallet_store_transfer_import.rs (line 203)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_store_transfer_import.rs:203). То есть направление уже заложено.

## Что должно находиться в receipt

После finalization агрегатор возвращает обоим wallet:

```
FinalizedReceiptV1
├── chain/network/generation
├── tx_digest
├── owned output leaf
├── checkpoint height + ID
├── state root
├── tx/output position
├── inclusion proof
├── finality certificate
├── epoch/manifest root
└── receipt signature/digest
```

Такой receipt должен быть самодостаточным: пользователь может спустя десять лет показать, что конкретный output был включён в финализированный checkpoint, даже если сеть давно удалила transaction body.

Для последующей траты wallet получает свежий membership witness против текущего state root от любого state mirror. Доверять mirror не нужно: wallet сам проверяет proof. Старую транзакцию для этого хранить не требуется.

## Единственная настоящая проблема — восстановление wallet

Сегодня wallet получает `ScanChunk` исторических checkpoints и локально ищет свои outputs — [scan_engine.rs (line 38)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/chain/scan_engine.rs:38). После удаления истории seed-only rescan перестанет работать.

Это нужно заменить двумя режимами:

1. **Полный backup restore**
   Восстанавливает keys + receipts + owned outputs + history.
2. **Seed-only recovery**
   Сканирует только актуальный unspent-state snapshot по view key/owner tag и восстанавливает текущий баланс. Старая история платежей не восстанавливается.

То есть потеря backup не должна приводить к потере денег, но может привести к потере бухгалтерской истории.

## Offline recipient

Если получатель offline, он ещё не успел сохранить receipt. Здесь есть три решения:

- sender хранит encrypted recipient receipt до подтверждения доставки;
- ==encrypted receipt mailbox хранится сетью ограниченное время, например 90 дней;==
- ==seed-only recovery находит непротраченный output прямо в актуальном state snapshot.==

Я бы использовал последние два. Mailbox — временный и маленький, а current-state recovery является страховкой.

## После 90 дней

Можно удалять:

- raw transaction packages;
- exact historical tx proof bytes;
- replay witnesses;
- HJMT paths;
- deltas и journals;
- Nova snapshots;
- внутренние Plonky3 proofs;
- DA payloads.

Остаются:

```
current state
current nullifier/spent state
rolling recursive proof
finality/key manifests
latest snapshots
небольшой challenge/mailbox buffer
```

И даже старые checkpoint roots сеть не обязана хранить поштучно, если receipt содержит соответствующий finality certificate и proof chain. Можно оставить только rolling MMR/root anchors.

## Почему всё-таки нужен временный buffer

Не для wallet history, а для:

- challenge/dispute;
- обнаружения ошибки агрегатора;
- восстановления незавершённого prover job;
- создания Plonky3 epoch proof;
- доставки receipt offline-получателю;
- recovery до появления следующего проверенного snapshot.

Если Plonky3 станет authoritative validity proof и receipt delivery будет закрыта, окно можно потенциально уменьшить с 90 дней до нескольких дней или даже до `epoch finality + safety margin`.

## Главный компромисс

После удаления raw невозможно:

- независимо переиграть всю историю;
- построить новый proof старой эпохи при обнаружении бага;
- восстановить полную историю wallet только из seed;
- построить глобальный historical explorer.

Но это не должно мешать проверке актуального состояния или владению средствами.

Мой итог: **mandatory network-wide raw archive Z00Z не нужен**. Нужна модель:

```
client-held receipts
+ recoverable current state
+ authoritative rolling validity proof
+ bounded challenge buffer
```

Тогда storage сети становится bounded, а исторические данные распределяются естественно: каждый участник хранит только относящиеся к нему receipts.

---

---

> i togda kakoj rasklad na forever sotrage per day prirost (esli on voobwe est) ?

Да: в client-held модели прирост обязательного forever archive можно сделать почти нулевым. Остаётся рост только актуального состояния, а не истории.

| Данные                  | Retention                    | Прирост после стабилизации      |
| ----------------------- | ---------------------------- | ------------------------------- |
| Raw/witness/delta       | 90-day ring                  | `0/day` после заполнения        |
| Nova snapshots          | до следующего P3 proof       | `0/day`                         |
| Plonky3 epoch proofs    | до rolling successor         | `0/day`                         |
| Latest rolling P3 proof | хранить последние 2–3        | `0/day`, фиксированные 1–12 MiB |
| State snapshots         | последние 2–3, copy-on-write | только изменение live state     |
| PP/VK                   | active generation            | `0/day`, только редкая rotation |
| Wallet receipts/history | на устройствах пользователя  | `0/day` для сети                |
| Thin epoch anchors      | опционально навсегда         | `9–70 KiB/day`                  |

При 1000 блоков/epoch получается `17.28 epoch/day`. Если один permanent epoch anchor занимает:

- `512 B` → `8.64 KiB/day`, около `3.1 MiB/year`;
- `1 KiB` → `17.28 KiB/day`, около `6.2 MiB/year`;
- `4 KiB` → `69.12 KiB/day`, около `24.6 MiB/year`.

Это разумный консервативный вариант: навсегда хранить лишь маленький:

```
EpochAnchor
├── height range
├── start/end state roots
├── rolling history root
├── finality certificate digest
├── Plonky3 accumulator digest
├── parameter generation
└── previous anchor digest
```

Можно сделать ещё агрессивнее: включить anchors в rolling MMR/P3 accumulator и хранить только latest root/proof. Тогда cryptographic archive growth будет практически `0 B/day`, кроме редких upgrade/key-rotation anchors.

## 90-дневный buffer

Если фактический уникальный payload равен `U/day`, размер кольца:

```
max buffer = 90 × U
```

Примеры:

- `100 MiB/day` → примерно `8.8 GiB`;
- `500 MiB/day` → примерно `44 GiB`;
- `1 GiB/day` → `90 GiB`.

На 91-й день новый сегмент заменяет истёкший, поэтому net growth становится нулевым.

## Что всё-таки может расти

Не история, а live state:

```
Δstate/day =
  новые непротраченные outputs
− потраченные/pruned outputs
+ nullifier/spent representation
```

Если outputs активно создаются и тратятся, UTXO-часть может стабилизироваться. Dust и бесконечное создание мелких outputs нужно сдерживать minimum-output/storage-fee/consolidation правилами.

Nullifiers — более сложный случай:

- если каждый nullifier хранится каждым validator навсегда, state растёт;
- если validators stateless и хранят только accumulator root, а wallets приносят witnesses, per-validator growth можно почти убрать;
- фактические nullifier данные тогда распределены между wallets/state shards, а не исчезают физически.

То есть честная формула:

```
forever growth per validator =
    0…70 KiB/day compact anchors
  + net live-state growth
  + rare parameter/key rotations
```

Моя рекомендация для Z00Z:

```
target permanent evidence growth: ≤ 100 KiB/day
raw challenge storage: bounded 90-day ring
proof storage: rolling, constant-size
wallet history: client-held
live state: separately capacity-controlled
```

Это примерно до `36 MiB/year` compact protocol history вместо десятков GiB в год.

Важно: текущий config пока не отражает эту модель — там raw/witness остаются `archive_required`, а каждый P3 epoch proof permanent: [checkpoint_contract.yaml (line 219)](/home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml:219). В предложенной архитектуре эти retention classes должны быть изменены.



---

----

> koshelek mozhet imet opciju pinit v IPFS svoi backups posle kazhdoj trati;
> starij bachup unpin novij pin CID i togda vsegda mozhno osstanovit svoju history i state



Да, это очень подходящая модель. Но не стоит сразу unpin старый CID: нужна атомарная ротация и несколько страховочных поколений.

```
wallet state changed
→ build compact backup
→ compress
→ encrypt
→ pin new CID
→ retrieve from independent peer
→ decrypt and verify
→ update signed latest-CID pointer
→ позже unpin oldest backup
```

## Backup должен быть инкрементальным

Не нужно после каждой траты заново загружать всю историю:

```
WalletBackupRoot
├── wallet-state snapshot
├── receipts Merkle root
├── receipt chunk 001
├── receipt chunk 002
├── receipt chunk 003 ← новый
├── current state root
├── previous backup CID
└── checkpoint/finality binding
```

IPFS переиспользует неизменившиеся chunks. После новой транзакции загружаются новый receipt chunk и маленький root object, а не вся история.

Периодически, например раз в месяц или после 1000 receipts, wallet делает compact snapshot и начинает новую DAG generation.

## Нужен стабильный указатель на последний CID

Сам CID каждый раз меняется. Для восстановления wallet должен знать, какой CID последний. Возможные варианты:

- подписанный IPNS record;
- несколько собственных устройств, хранящих signed `BackupHeadV1`;
- QR/export с последним CID;
- небольшой mutable record в выбранной decentralized-storage сети.

```
BackupHeadV1 {
    backup_sequence,
    latest_cid,
    previous_cid,
    checkpoint_height,
    wallet_state_root,
    receipts_root,
    encryption_generation,
    signature
}
```

`backup_sequence` и `previous_cid` защищают от rollback на старый backup и позволяют обнаруживать forks между несколькими устройствами.

## Правильная ротация

Я бы держал:

- latest backup;
- предыдущие 2–3 поколения;
- один monthly recovery snapshot.

Старое поколение unpin только когда:

1. Новый CID успешно pinned.
2. Он скачан не из локального cache, а с независимого peer.
3. Backup расшифрован.
4. Manifest, receipts root и wallet state проверены.
5. Latest pointer обновлён.
6. Прошёл небольшой grace period.

Это даёт bounded storage, но защищает от загрузки повреждённого backup.

## Шифрование обязательно

IPFS content обычно публично доступен по CID. До публикации:

```
canonical wallet backup
→ Zstd
→ XChaCha20-Poly1305
→ padding
→ IPFS
```

Нужно использовать отдельный backup encryption secret. Нельзя создавать circular recovery, когда ключ расшифровки хранится только внутри самого backup.

Также IPFS раскрывает:

- время обновления;
- размеры blobs;
- связь обновлений через IPNS.

Padding и периодические batched uploads уменьшают metadata leakage.

## Главное ограничение IPFS

CID сам по себе ничего не хранит. Восстановление возможно только если:

- хотя бы один peer продолжает pin;
- пользователь сохранил backup encryption secret;
- доступен latest-CID pointer.

Если wallet pinит только на потерянном телефоне — backup исчезнет вместе с телефоном. Без внешнего провайдера нужны хотя бы 2–3 собственных устройства/доверенных peers:

```
phone + desktop + hardware/home node
```

Либо wallet может опционально покупать Filecoin/Walrus storage непосредственно через протокол.

## Когда создавать backup

Не только после траты. Backup нужен после любого wallet-state change:

- получен новый output;
- создана транзакция;
- транзакция финализирована;
- output потрачен;
- обновлён receipt или membership witness;
- изменился scan cursor;
- выполнена key rotation.

Можно debounce несколько событий в один upload, например ждать 30–60 секунд.

Итоговая модель получается сильной:

```
Network:
  current global state
  rolling proof
  bounded challenge buffer

Wallet:
  private history
  receipts
  owned outputs
  encrypted rotating IPFS backup
```

Тогда сеть действительно не обязана хранить пользовательскую историю. Единственная поправка к твоей формулировке: не «старый сразу unpin, новый pin», а **сначала pin+retrieve+verify новый, затем с задержкой unpin старые поколения**.

---

---

> pust delaet hot v ba google drive, chem eto nam meshaet ?

Ничем не мешает. Google Drive — нормальный пользовательский backend для backup. Он вообще не должен входить в trust model Z00Z.

Правильная схема:

1. Кошелёк локально формирует backup:
   - актуальный state;
   - личные receipts/history;
   - параметры восстановления;
   - номер поколения backup;
   - последний подтверждённый checkpoint/root.
2. Локально подписывает и шифрует его.
3. Загружает уже ciphertext в выбранное хранилище:
   - Google Drive;
   - iCloud;
   - OneDrive;
   - IPFS;
   - WebDAV;
   - USB/локальный файл.
4. После загрузки проверяет новый backup и только затем удаляет старый. Можно оставлять последние 2–3 поколения для защиты от повреждения и случайного удаления.

При восстановлении кошелёк обязан проверить:

- подпись и формат backup;
- правильную сеть/chain ID;
- монотонный sequence number против rollback;
- соответствие state root сетевому checkpoint/recursive proof.

Google при этом:

- не видит открытых данных;
- не может подделать состояние кошелька;
- не влияет на consensus/finality;
- не хранит ничего обязательного для работы сети;
- может максимум удалить, задержать или вернуть старую копию — это обнаруживается проверками.

Backup желательно обновлять после любого изменения состояния: получения средств, финализации receipt, траты или ротации ключей. Можно объединять события и загружать раз в несколько минут.

Главный нюанс: ключ расшифрования нельзя хранить только внутри этого же backup. Нужен seed/recovery password либо отдельный recovery secret.

Итого: Z00Z задаёт единый зашифрованный формат backup и адаптеры хранилищ, а пользователь выбирает Google Drive или что угодно ещё. Это как раз позволяет не перекладывать персональную историю всех пользователей на сеть.

---

----



## FINAL DESIGN

## Вердикт

**Рискованно, но исправимо.** Правильная архитектура для Z00Z:

> **Сеть хранит проверяемое текущее состояние и компактную нотариальную цепочку.
> Полная пользовательская история хранится кошельками.
> Сырые данные существуют только в ограниченном challenge/audit window.**

Текущая реализация ещё не готова включать такую ротацию: T2 не завершил proof relation, а действующий retention-конфиг требует вечного хранения raw packages и witnesses. [Текущий статус (line 33)](/home/vadim/Projects/z00z/.planning/STATE.md:33), [retention config (line 219)](/home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml:219).

## Итоговая схема

```
Каждые 5 секунд — canonical lane:

transactions
    ↓
проверка + staged HJMT state
    ↓
DA publication ready
    ↓
quorum/finality certificate
    ↓
АТОМАРНО: новый state root + canonical head + outbox
    ├── wallet receipt delivery
    ├── archive EpochPack
    └── asynchronous recursive proving


Асинхронный recursive lane:

finalized checkpoint
    ↓
Nova fold каждого блока
    ↓
Nova recovery snapshot периодически
    ↓
epoch close каждые 1000 блоков
    ├── Nova compressed proof
    └── независимый Plonky3 epoch proof
             ↓
journaled evidence commit
             ↓
pruning eligibility после 90 дней


Wallet lane:

finalized receipt
    ↓
АТОМАРНО: receipt + outputs + scan cursor + backup job
    ↓
encrypted immutable backup
    ↓
upload → read-back verify → CAS head → удаление старых копий
```

Глобальной транзакции, охватывающей consensus, DA, wallet и Google Drive, быть не может. Правильная атомарность — это **три локальные транзакции, связанные digest-ами, idempotency keys и durable outbox**.

## Что хранить

| Данные                                          | Где                                    | Retention                                |
| ----------------------------------------------- | -------------------------------------- | ---------------------------------------- |
| Текущий HJMT/state                              | validators/state mirrors, шардированно | Пока output существует                   |
| Последние COW state snapshots                   | state mirrors                          | Последние 3 проверенных snapshot heads   |
| Epoch anchors и rotation ledger                 | Все узлы                               | Навсегда                                 |
| Raw transaction packages и exact tx proof bytes | Archive watchers                       | 90 дней                                  |
| Replay deltas/journal                           | Archive watchers                       | 90 дней, только если нельзя восстановить |
| Witnesses, traces, assignments                  | Prover scratch                         | Часы или текущий epoch                   |
| Nova accumulator/recovery snapshots             | Recursive prover                       | Current + 2 последних                    |
| Nova proof                                      | Специализированные watchers            | Current/previous и challenge window      |
| Plonky3 epoch proof                             | Archive watchers                       | До rolling successor + 90 дней           |
| PP/PK                                           | Только Nova prover                     | Active generation + recovery window      |
| VK                                              | Только recursive verifier roles        | Пока существуют соответствующие proofs   |
| Receipts и история платежей                     | Wallet                                 | По политике пользователя                 |
| Wallet backups                                  | Drive/IPFS/WebDAV/local                | Последние 3 поколения минимум            |

Сеть **не должна** хранить пользовательские receipts/history навсегда.

Однако сеть обязана хранить **текущее состояние**. Один state root недостаточен: кому-то нужно иметь реальные текущие leaves/chunks, чтобы новый wallet мог найти output и получить свежий membership witness. Это не история — это текущая нотариальная книга собственности.

## Постоянный рост

При блоке каждые 5 секунд:

- 17 280 блоков/день;
- 17.28 эпохи/день при `epoch = 1000`;
- одна эпоха — 83 мин 20 сек.

Рекомендую `EpochAnchorV1`:

- range/epoch;
- start/end state roots;
- block/QC Merkle root;
- `EpochPack` root и expiry height;
- Nova/P3 proof digests;
- committee/parameter generation;
- previous anchor/MMR root.

Целевой размер — около **1 KiB**, hard cap — **4 KiB**.

Это даёт:

- 1 KiB: **17.28 KiB/день**, около **6.2 MiB/год**;
- 4 KiB cap: **69.12 KiB/день**, около **24.6 MiB/год**.

Итоговая цель:

> **Постоянный исторический рост ≤100 KiB/день плюс реальный прирост текущего unspent state.**

Текущий state может расти — буквально нулевой рост невозможен. Его нужно ограничивать экономически:

- компактный фиксированный output leaf;
- запрет произвольных пользовательских payload;
- max output fan-out;
- minimum output/dust policy;
- refundable state-storage deposit;
- стимулирование consolidation.

Баланс нельзя автоматически уничтожать из-за истечения срока.

## 90-дневный challenge ring

Retention должен считаться по finalized height:

```
1,555,200 блоков = 90 дней при 5 секундах
```

Логический размер ring:

```
90 × фактические unique ingress bytes/day
```

Например, при 1 GiB новых уникальных данных в день ring стабилизируется примерно на **90 GiB**. Он больше не растёт по времени.

Для production вместо трёх полных копий рекомендую erasure coding, например RS `10-of-16`:

- общий overhead: 1.6× вместо 3×;
- 90 GiB logical → 144 GiB по всей сети;
- примерно 9 GiB на каждого из 16 watcher nodes.

Для раннего MVP допустимы 3 независимые полные реплики; после появления достаточного количества операторов — переход на shards.

`EpochPack` должен быть lossless:

- один canonical binary format;
- один `ProofBlob` на уникальный digest;
- records только ссылаются на него;
- witnesses не копируются, а восстанавливаются;
- column/delta encoding;
- zstd dictionary;
- content-addressed COW chunks;
- никаких JSON/hex-копий в archive path.

`max_batch_bytes = 8 MiB` должен остаться burst/DoS cap, но рядом необходимы:

- `target_unique_bytes_per_day`;
- rolling ingress token bucket;
- `max_90d_window_bytes`;
- storage fee за уникальный байт.

## Recursive proofs

Рекомендуемые cadence:

- Nova fold: каждый finalized block;
- local recovery snapshot: начальный кандидат каждые 100 блоков, затем выбрать по benchmark;
- Nova compressed publication: раз в 1000 блоков и on-demand;
- Plonky3 proof: один на точный ordered epoch range;
- обычная block finality: по-прежнему каждые 5 секунд и не ждёт proof.

PP/PK/VK никогда не входят в block traffic. Это уже правильно сформулировано в Plan 051. [Artifact roles и cadence (line 287)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-051-PLAN.md:287).

Текущий verifier artifact — около **273 MiB**, но это статический VK одного generation, а не сообщение каждые пять секунд. Он слишком велик для всех validators, поэтому Nova verification должна быть специализированной opt-in ролью. [Измеренный статус (line 34)](/home/vadim/Projects/z00z/.planning/STATE.md:34).

Текущий Nova proof 37 808 B, публикуемый раз в epoch:

- около **0.623 MiB/день** на verifier;
- около **56 MiB** за весь 90-дневный ring.

### Критически важное ограничение Plonky3

Plan 08 сейчас доказывает отдельный epoch range, но не сворачивает всю цепь epoch proofs в один постоянный history proof. [Plan 08 (line 102)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-08-PLAN.md:102).

Поэтому нужно добавить:

```
HistoryAccumulatorProofV1:
    previous_history_proof
  + exact_new_epoch_proof
  + previous/new state roots
  + committee/parameter transition
  → new rolling history proof
```

До реализации и аудита этого theorem нельзя утверждать, что достаточно хранить последние `N` Plonky3 proofs.

Plonky3 должен независимо доказывать полный transition predicate. Он не должен просто проверять Nova proof: иначе ECC/Nova не становится post-quantum.

## Wallet как владелец истории

Нужен `FinalizedReceiptV2`, содержащий:

- network/genesis;
- tx digest;
- checkpoint/height/epoch;
- finalized state root;
- owned output leaf или encrypted recovery capsule;
- HJMT membership proof;
- tx/block inclusion proof;
- quorum/finality certificate;
- epoch anchor;
- committee/key generation;
- receipt digest.

Проверка wallet:

```
finality certificate
    → authenticated checkpoint root
        → transaction/output inclusion
            → ожидаемый принадлежащий wallet output
```

Для траты wallet получает свежий witness к текущему root от любого mirror и проверяет его локально. Текущий код уже умеет проверять HJMT membership, но существующий receipt слишком бедный и не содержит checkpoint/finality/inclusion bindings. [Membership witness (line 23)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/tx/state_witness.rs:23), [текущий Receipt (line 9)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/persistence/receipt_storage.rs:9).

Seed recovery должен восстанавливать **текущий spendable state**, а не полную историю. Для этого текущие unspent leaves должны содержать или ссылаться на recipient-encrypted recovery capsule/view tag. Полная история восстанавливается из wallet backup.

Google Drive, IPFS или локальный диск сети не мешают: provider получает только ciphertext и не входит в trust model.

### Backup V5

Существующее шифрование можно сохранить. Нужно добавить:

- `backup_seq`;
- parent backup digest;
- device ID;
- checkpoint height/root;
- wallet-state root;
- receipts root;
- payload digest;
- MAC/signature head;
- rollback/fork detection.

Протокол обновления:

```
создать immutable generation
→ upload
→ скачать обратно
→ decrypt/checksum/state-root verify
→ CAS заменить head
→ отметить backup complete
→ удалить старые поколения
```

Нельзя держать ровно одну копию: ошибка upload может уничтожить единственный backup. Минимум — последние 3 успешных generation. Текущий Backup V4 не имеет sequence/parent/head модели. [Backup metadata (line 9)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/backup/backup_exporter.rs:9).

## Атомарность и crash consistency

Canonical head и authoritative state должны переключаться одной DB-транзакцией. Immutable artifacts могут лежать в filesystem/content store, но становятся видимыми только через committed DB manifest/head.

Правильный порядок записи файла:

```
temp write → file fsync → rename → parent-directory fsync
```

Текущий общий `write_file()` делает `flush + rename`, но не `sync_all` файла и директории. [atomic_write.rs (line 7)](/home/vadim/Projects/z00z/crates/z00z_utils/src/io/atomic_write.rs:7). Кроме того, checkpoint seal сейчас последовательно пишет несколько файлов, поэтому это ещё не одна crash-atomic logical transaction. [store.rs (line 743)](/home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/store.rs:743).

Plan 10 правильно требует journal/commit marker и fail-closed recovery; эту модель нужно применить и к canonical checkpoint commit, а не только к recursive evidence. [Plan 10 (line 175)](/home/vadim/Projects/z00z/.planning/phases/069-Recursive-Proof/069-10-PLAN.md:175).

## Что обязательно исправить в 069-06…13

1. **069-06:** выбрать recovery/compression/publication cadence по измерениям.
2. **069-07/08:** сохранить независимый P3 predicate и добавить cross-epoch rolling history theorem.
3. **069-09:** заменить permanent raw/witness archive на height-based 90-day ring; archive nodes должны уметь удалять expired shards.
4. **069-10:** durable outbox, DB commit marker, idempotent recovery и failpoints.
5. **069-11:** измерять actual unique bytes/day, P3 proof bytes/CPU и state growth.
6. **069-12:** прогнать минимум 100 дней, crashes, partitions, rotations, delayed proofs и archive repair.
7. **069-13:** синхронизировать документацию и `checkpoint_contract.yaml`.

## Главные blockers

- **S1:** T2 proof relation неполна — recursive promotion запрещён.
- **S1:** live config всё ещё требует вечные raw packages/witnesses.
- **S1:** текущий filesystem seal не является одной durable transaction.
- **S1:** 90-дневное удаление нельзя связывать с ещё неавторитетным proof lane.
- **S2:** Wallet Backup V4 не защищает от rollback/fork и partial remote update.
- **S2:** 273 MiB VK неприемлем для массовой validator distribution.
- **Не измерено:** реальные размер/CPU Plonky3 и feasibility rolling-history proof.

==И ещё важная семантика: пока recursive lane non-authoritative, 90 дней — это **audit/reprove window**, а не optimistic rollback window. Если challenge должен отменять финальность или наказывать validators, нужен отдельный формальный dispute/slashing protocol.==

### Doublecheck

Проверено 10 ключевых утверждений:

| Рейтинг    | Количество |
| ---------- | ---------- |
| VERIFIED   | 8          |
| DISPUTED   | 1          |
| UNVERIFIED | 1          |

- **DISPUTED:** «уже можно хранить только последние N Plonky3 proofs» — текущего rolling-history theorem нет.
- **UNVERIFIED:** реальные Plonky3 proof size, CPU и постоянная rolling cadence — нужны implementation benchmarks.
- Внутреннее противоречие подтверждено: client-centric 90-day модель несовместима с текущими `archive_required` raw/witness и archive-node no-prune правилами.

**Окончательное решение:** строить Z00Z как нотариус текущего состояния с wallet-owned history, 90-дневным lossless challenge ring, компактными вечными epoch anchors и асинхронной recursive audit lane. Но pruning и proof promotion оставить заблокированными до закрытия перечисленных S1.

EXPANDABLE: точные изменения для 069-051 и планов 069-06…13.
