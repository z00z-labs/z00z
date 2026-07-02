Моя позиция: **да, твоя схема “сырые данные держим ограниченное время, потом полагаемся на финализированные checkpoints” — правильная для Z00Z**, но надо очень чётко назвать вещи своими именами:

```text
Celestia-only DA:
  сырые batch-данные реально публикуются в Celestia.

Celestia + IPFS/checkpoints:
  Celestia фиксирует commitment/checkpoint,
  а реальная availability/retrievability обеспечивается Z00Z-инфраструктурой:
  IPFS/pinners/archive nodes/watchers/Filecoin/S3/etc.
```

Это нормальная архитектура. Просто это уже не “Celestia гарантирует мне всё”, а **Z00Z сам строит свой DA/retrievability layer, а Celestia используется как публичный якорь финальности**.

------

## Главное различие

**IPFS + checkpoint дешевле, но слабее по trust model.**
**Celestia дороже, но даёт более сильный public DA event.**

Почему? Потому что CID в IPFS доказывает только:

```text
если у тебя есть эти данные, ты можешь проверить,
что они соответствуют этому CID / hash / Merkle root.
```

Но CID **не доказывает**, что данные действительно были доступны всем участникам в момент challenge window.

Celestia, наоборот, предназначена именно для публикации данных как DA-layer: `PayForBlobs` включает сами данные, namespace, размер данных и commitment; комиссия состоит из flat fee плюс variable fee based on blob size, то есть чем больше данных ты публикуешь, тем дороже. ([GitHub](https://github.com/celestiaorg/docs/blob/main/app/learn/TIA/paying-for-blobspace/page.mdx))

IPFS официально говорит похожую вещь про persistence: контент может быть найден по CID, но IPFS сам по себе **не гарантирует persistent availability**; чтобы данные не исчезли, их надо pin’ить на одном или нескольких узлах или через pinning services. ([IPFS Docs](https://docs.ipfs.tech/concepts/persistence/))

------

# Как бы я проектировал это для Z00Z

Я бы не делал выбор “или Celestia, или IPFS”. Я бы сделал **трёхслойную модель**:

```text
1. Celestia / settlement layer:
   permanent commitments, checkpoint roots, data roots, signatures.

2. Challenge-window data layer:
   full compressed/encrypted batch bodies in IPFS + Z00Z archive nodes + object storage.

3. Long-term finality layer:
   periodic checkpoints, snapshots, watcher attestations, optional Filecoin/Arweave archival.
```

То есть Celestia используется не как Dropbox, а как **notary / timestamp / commitment / optional DA layer**.

------

## Важный момент: challenge window должен начинаться не от “агрегатор сказал”, а от “данные доступны”

Правильный flow я бы сделал так:

```text
1. Aggregator собирает batch N.

2. Aggregator формирует BatchBody:
   - tx / encrypted tx payloads
   - signatures / proofs
   - nullifier updates
   - commitment updates
   - state transition inputs
   - Merkle tree deltas
   - metadata needed for verification

3. BatchBody compress/encrypt.

4. BatchBody режется на chunks:
   chunk_0, chunk_1, ..., chunk_k

5. Строится manifest:
   - batch_id
   - previous_checkpoint
   - state_root_before
   - state_root_after
   - tx_root
   - nullifier_root
   - commitment_root
   - chunk_root
   - IPFS CIDs
   - size
   - retention_until
   - challenge_deadline
   - aggregator signature

6. Manifest / data_root / checkpoint публикуется в Celestia.

7. Watchers скачивают BatchBody из IPFS / archive source.

8. Watchers проверяют:
   - CID matches
   - chunk_root matches
   - batch decodes
   - signatures/proofs valid
   - transition replay valid, если модель optimistic
   - или validity proof valid, если модель zk

9. Watchers подписывают DA/validation certificate.

10. Challenge window начинается только после threshold watcher certificate.
```

Ключевой принцип:

```text
Нет threshold DA certificate -> batch не финализируется.
```

Не надо пытаться доказывать “данных нет”. Это сложно. Лучше сделать safe-fail:

```text
данные не были достаточно подтверждены как доступные
=> checkpoint не становится финальным.
```

------

# Где хранить данные during challenge window

Для Z00Z я бы хранил данные **не в одном месте**, а в нескольких независимых классах хранения.

## Минимальный production-вариант

```text
A. Z00Z canonical archive nodes
B. IPFS with 2–3 independent pinning providers
C. обычное object storage: S3 / R2 / Backblaze / self-hosted
D. watcher local storage
E. periodic snapshot bundles
```

Это не “красиво децентрализовано”, но практично. IPFS сам по себе не storage guarantee; pinning или свои узлы обязательны. IPFS docs прямо говорят, что данные должны быть pinned, а если sponsor перестаёт платить за pinning, контент может быть потерян. ([IPFS Docs](https://docs.ipfs.tech/concepts/persistence/))

## Более сильный вариант

```text
A. Всё из минимального варианта
B. Filecoin deals для long-term storage
C. Arweave только для особо важных snapshots/checkpoint bundles
D. erasure coding across storage providers
E. random chunk audits by watchers
```

IPFS docs описывают Filecoin как слой, где клиент и storage provider договариваются, сколько данных хранить, на какой срок и за какую стоимость; provider затем периодически подтверждает, что хранит данные. ([IPFS Docs](https://docs.ipfs.tech/concepts/persistence/))

------

# Нужно ли класть сырые транзакции в Celestia?

Зависит от того, какую security model ты хочешь.

## Если Z00Z должен быть permissionless optimistic rollup

Тогда **да, full challenge data должен попадать в Celestia или другой настоящий DA-layer**.

Не обязательно plaintext raw tx, но должен попадать **минимально достаточный batch body**, чтобы любой независимый verifier мог скачать данные и проверить transition.

```text
Celestia blob:
  compressed/encrypted batch body
  tx/proof public inputs
  state transition data
  nullifier deltas
  commitment deltas
  Merkle deltas
```

В этом режиме Celestia нужна именно как DA.

Минус: дороже, потому что Celestia fees зависят от размера blob. ([GitHub](https://github.com/celestiaorg/docs/blob/main/app/learn/TIA/paying-for-blobspace/page.mdx))

Плюс: data withholding attack намного сложнее. Aggregator не может просто сказать “вот CID, данные где-то есть”, но реально никому их не дать.

------

## Если Z00Z допускает watcher/committee trust assumption

Тогда можно **не класть full raw data в Celestia**.

Тогда модель такая:

```text
Celestia:
  checkpoint root
  data root
  CID root
  watcher certificate root
  state root
  nullifier root

IPFS/archive layer:
  actual full batch data
```

Это дешевле и, возможно, вполне подходит для Z00Z, если ты готов честно сказать:

```text
Security assumes that at least one honest watcher/archive node
downloaded and retained the batch data during the challenge window.
```

Это уже ближе к:

```text
validium / DAC / committee-attested rollup / checkpointed ledger
```

И это не обязательно плохо. Для многих practical systems это разумный tradeoff.

------

# Что нельзя делать

Я бы не делал так:

```text
Aggregator publishes CID to Celestia.
Nobody independently confirms data retrieval.
Challenge window starts immediately.
After 30 days raw data disappears.
Checkpoint considered final.
```

Проблема: агрегатор может сделать data withholding attack.

Например:

```text
1. Aggregator публикует commitment / CID.
2. Данные реально никому не отдаёт или отдаёт только своим узлам.
3. Watchers не могут пересчитать batch.
4. Challenge window проходит.
5. Потом агрегатор говорит: checkpoint финален.
```

CID сам по себе не доказывает, что данные были доступны. Он доказывает только integrity, если данные у тебя уже есть.

------

# Как сделать IPFS/checkpoint модель нормальной

Тебе нужен **Z00Z Data Availability Certificate**.

Например:

```text
DA_CERT(batch_id):
  batch_id
  data_root
  chunk_root
  total_size
  cid_list_root
  retain_until
  challenge_deadline
  watcher_set_id
  signatures: threshold M-of-N
```

Watcher подписывает это только если он:

```text
скачал данные;
проверил хэши;
декодировал chunks;
локально сохранил batch;
готов хранить до retain_until;
в идеале — replayed transition или checked proof.
```

Тогда Celestia получает не просто:

```text
"агрегатор сказал, что данные есть"
```

а:

```text
"агрегатор опубликовал commitment,
и M-of-N независимых watchers подтвердили,
что они реально получили и сохранили batch data".
```

Это уже гораздо лучше.

------

# Какой срок хранения выбрать

Я бы разделил сроки на три окна.

## 1. Hot challenge window

```text
7–30 дней
```

Здесь данные должны быть максимально доступны, быстро скачиваемы и продублированы.

Для optimistic-модели я бы не делал меньше 7 дней. Для Z00Z, если users могут быть offline, я бы скорее думал о:

```text
14–30 дней minimum
```

Но важно: Celestia light-node sampling window сейчас 7 дней; Celestia docs говорят, что blobs старше recency window pruned by default on light nodes, а rollups после публикации сами ответственны за хранение historical data. ([Celestia Documentation](https://docs.celestia.org/learn/celestia-101/retrievability/))

То есть если challenge window больше 7 дней, то **тебе всё равно нужны Z00Z archive nodes**, даже если ты публикуешь данные в Celestia.

## 2. Warm recovery window

```text
90–180 дней
```

Это окно для пользователей, которые были offline, потеряли локальный index, переустановили wallet, догоняют state, спорят с receipt, восстанавливают историю.

Для Z00Z я бы выбрал **минимум 90 дней**, лучше **180 дней**, если storage cost позволяет.

## 3. Cold historical archive

```text
1 год+ / forever-ish
```

Здесь можно хранить не всё в raw виде, а:

```text
daily/weekly snapshots;
checkpoint bundles;
compressed batch archives;
validity proofs;
state snapshot;
Merkle witnesses;
watcher certificates;
audit logs.
```

После challenge window тебе не обязательно держать все raw tx в hot storage. Но хотя бы один canonical historical archive я бы всё равно держал долго, пусть даже дешёвым cold storage.

------

# Checkpoints: да, но правильно

Твоя идея:

> “раз в день / раз в неделю checkpoint; от snapshot до checkpoint все транзакции можно пересчитать; потом несколько месяцев держать raw data; дальше полагаться на checkpoints”

Это звучит правильно.

Я бы сделал так:

```text
Micro-batch:
  каждые N минут / часов

Daily checkpoint:
  агрегирует все batches за день

Weekly checkpoint:
  state snapshot + proof/certificate bundle

Monthly archive:
  compressed historical package
```

Пример:

```text
Every 10 min:
  batch root + CID + aggregator sig

Every 1 hour:
  batch group checkpoint

Every 1 day:
  state_root_day_D
  nullifier_root_day_D
  commitment_root_day_D
  tx_volume_root
  watcher DA cert root
  validity/fraud status

Every 1 week:
  snapshot package:
    latest state
    all daily checkpoint headers
    proofs/certs
    archive manifest

After 90/180 days:
  raw per-batch data can move to cold archive
  protocol relies on finalized checkpoints
```

Но checkpoint должен быть финальным только после:

```text
challenge window closed
+
no successful fraud challenge
+
data availability certificate exists
+
watcher/validator quorum signs finality
```

------

# Когда можно удалять raw transactions

Можно удалять или переводить в cold archive только если выполняются условия:

```text
1. Challenge window закрыт.

2. Все relevant fraud/validity challenges разрешены.

3. Checkpoint финализирован watcher/validator quorum.

4. Есть state snapshot на checkpoint height.

5. Есть Merkle commitments к старым tx/batches.

6. Пользователи получили receipts / wallet proofs.

7. Есть хотя бы один cold archival path,
   если нужно восстановление истории или аудит.

8. Protocol explicitly accepts:
   new nodes sync from checkpoint, not from genesis.
```

Последний пункт важный.

Если ты удаляешь raw data, то новый node уже не сможет независимо пересчитать историю с genesis. Он будет делать:

```text
sync from trusted/finalized checkpoint
```

Это нормальная модель, но это **weak subjectivity / checkpoint trust**, а не pure full historical verification.

------

# Что именно хранить forever

Не надо хранить forever весь raw transaction stream в hot storage. Но я бы хранил навсегда или почти навсегда:

```text
checkpoint headers;
state roots;
nullifier roots;
commitment roots;
tx roots;
data roots;
watcher certificate roots;
aggregator/validator signatures;
challenge outcomes;
finality certificates;
snapshot hashes;
migration records;
protocol version records.
```

А для пользователя wallet должен хранить:

```text
own transaction receipt;
note data / encrypted note;
Merkle inclusion proof;
batch id;
checkpoint reference;
decryption material;
nullifier witness, если нужно;
```

Если пользователь потеряет локальные wallet notes, а encrypted note payloads не сохранены в сети, он может увидеть, что global state валиден, но не сможет восстановить свой баланс. Это особенно критично для privacy/ZK-money систем.

------

# Где Celestia становится узким местом

## 1. Если ты кладёшь туда full data

Тогда Celestia — bottleneck по:

```text
стоимости blobspace;
лимитам throughput;
fee volatility;
задержке публикации;
зависимости от TIA/security.
```

Зато у тебя stronger public DA.

## 2. Если ты кладёшь туда только checkpoint

Тогда Celestia **не узкое место для DA**.

Узкое место становится здесь:

```text
IPFS pinning;
Z00Z archive nodes;
watcher honesty;
watcher uptime;
data retrieval bandwidth;
anti-withholding logic;
challenge procedure.
```

Celestia в этой модели фиксирует только:

```text
вот такой root был опубликован в такое время.
```

## 3. Если challenge window длиннее Celestia retrieval window

Celestia docs говорят, что light nodes сейчас sample only within 7-day window, blobs older than that are pruned by default on light nodes, а rollups should not rely only on free public archival nodes. ([Celestia Documentation](https://docs.celestia.org/learn/celestia-101/retrievability/))

Поэтому, даже если ты используешь Celestia, для challenge window в 30/90 дней тебе нужны собственные архиваторы.

------

# Мой рекомендуемый вариант для Z00Z

Я бы сделал **не Celestia-full-data by default**, а гибрид:

```text
Default mode:
  full batch data -> IPFS + Z00Z archive nodes + object storage
  checkpoint/data_root/DA_CERT -> Celestia

High-value mode:
  compressed minimal challenge data -> Celestia
  full encrypted batch -> IPFS/archive

Emergency / high-risk mode:
  dual publish:
    Celestia + Ethereum blobs / Avail / another DA
```

То есть:

```text
Celestia = canonical public checkpoint + optional DA insurance.
Z00Z archive layer = actual challenge-window data storage.
Watchers = bridge between "CID exists" and "data was actually retrievable".
```

------

# Как это выглядит формально

```text
BatchBody_i =
  canonical_tx_data_i
  proof_inputs_i
  nullifier_deltas_i
  commitment_deltas_i
  encrypted_recipient_payloads_i
  metadata_i

Manifest_i =
  batch_id
  protocol_version
  state_root_before
  state_root_after
  tx_root
  nullifier_root
  commitment_root
  batch_body_hash
  chunk_merkle_root
  ipfs_cid_root
  size_bytes
  created_at
  challenge_deadline
  retain_until

DA_CERT_i =
  Manifest_i.hash
  watcher_set_id
  signatures M-of-N
  watcher_storage_commitments
  optional replay_result_hash

CelestiaPayload_i =
  Manifest_i.hash
  DA_CERT_i.hash
  state_root_after
  previous_checkpoint_hash
```

Если хочешь stronger mode:

```text
CelestiaPayload_i =
  Manifest_i
  compressed minimal BatchBody_i
  DA_CERT_i
```

------

# Fraud-proof vs ZK-proof: большая разница

Если Z00Z будет optimistic/fraud-proof:

```text
raw or reconstructable batch data must be available
during the whole challenge window.
```

Тут IPFS-only опасен без watchers.

Если Z00Z будет ZK-validity-proof:

```text
correctness можно доказать proof’ом,
но данные всё равно нужны для:
  user recovery,
  wallet resync,
  note discovery,
  indexing,
  auditability.
```

В ZK-модели можно хранить меньше данных в Celestia, потому что validity не зависит от fraud challenge. Но retrievability пользовательских данных всё равно надо решать.

------

# Практический ответ на твой вопрос

Да, я бы для Z00Z сделал примерно так, как ты описываешь:

```text
1. Сырые / reconstructable данные:
   хранить 90–180 дней.

2. During challenge window:
   хранить в IPFS + pinning + Z00Z archive nodes + watcher local storage.

3. В Celestia:
   публиковать checkpoint/data_root/DA certificate,
   а full data — только если нужен stronger DA mode.

4. После challenge window:
   финализировать checkpoint.

5. После warm retention window:
   raw data можно перемещать в cold archive
   или удалять, если protocol accepts checkpoint-based sync.

6. Для long-term:
   хранить snapshots/checkpoint bundles/proof certificates.
```

Но я бы обязательно добавил правило:

```text
Checkpoint cannot become final
unless enough independent watchers attest
that they downloaded, verified, and stored the data.
```

Иначе IPFS CID + Celestia checkpoint — это просто красивый hash pointer, а не data availability.

------

## Финальный вердикт

Твоя архитектурная интуиция правильная:

```text
Хранить всё сырое в Celestia всегда — скорее всего слишком дорого.
Хранить всё только в IPFS без watcher certificates — слишком слабо.
Хранить raw data несколько месяцев + checkpoints + watchers — разумный middle ground.
```

Для Z00Z я бы выбрал именно этот middle ground:

```text
Celestia:
  public commitment / checkpoint / optional DA insurance

IPFS + archive nodes:
  real challenge-window data storage

Watchers:
  independent download + replay/verify + storage attestation

Checkpoints:
  final state after challenge window

Snapshots:
  sync and recovery after old raw data expires
```

Главное честно зафиксировать security assumption:

```text
Z00Z remains safe if at least one honest watcher
downloads and retains each batch before finality.
```

Если ты хочешь убрать даже это допущение, тогда придётся публиковать full challenge data в Celestia или другой настоящий DA-layer.



---



> То, что ты назвал honest watcher, это, я так понимаю, trusted setup. Мы говорим о том, что у нас не trusted setup, но должно быть какое-то решение без того, чтобы публиковать все транзакции. Это как бы ключевая идея. Транзакции, сырые транзакции, они не публикуются. В них нет смысла, потому что вся data, собственно, все объекты, assets, rights, voucher, все хранится в дереве, этом самом jellyfish, Merkle 3. А всё сидит там. И любой микросервис может пойти проверить, так сказать, что это всё существует и валидно. А сырые транзакции, которые принесли всё это, нет смысла хранить, как бы этот мусор, только для того, чтобы проверить, что всё функционирует нормально. То есть, так сказать, нотариальное подтверждение. И правильно ли ты понимаешь эту концепцию?
> 

Да, я понимаю твою концепцию. Ты описываешь **state-centric protocol**, а не transaction-log protocol.

То есть Z00Z в твоей модели — это не:

```text
"все сырые транзакции навсегда лежат в публичном журнале,
и любой может пересчитать историю с genesis"
```

а скорее:

```text
"каноническое состояние живёт в authenticated state tree,
объекты/права/ваучеры/ассеты доказываются Merkle/Jellyfish proofs,
а сырые транзакции — это временный witness, нужный только для производства следующего state root"
```

И это абсолютно осмысленная архитектура. Но есть одна жёсткая граница.

## Корень дерева доказывает существование объекта, но не доказывает корректность эволюции состояния

Jellyfish Merkle Tree / sparse Merkle tree даёт тебе проверку вида:

```text
object X exists in state_root R
object X has value V
object X is not deleted / not nullified
object X belongs to policy / owner / rights set
```

То есть микросервис может получить:

```text
object payload
Merkle inclusion proof
current finalized root
```

и проверить:

```text
да, этот voucher / right / asset действительно существует
в каноническом состоянии Z00Z.
```

Но из одного факта:

```text
state_root_n = A
state_root_n+1 = B
```

не следует, что переход `A -> B` был корректным.

Между двумя roots может быть бесконечно много возможных историй. Merkle root — это commitment к состоянию, а не доказательство того, что это состояние было получено честно.

Поэтому главный вопрос Z00Z не “хранить ли raw transactions”, а:

```text
кто или что доказывает, что новый state root получен по правилам Z00Z?
```

------

# Watcher — это не trusted setup, но это trust assumption

Тут важно разделить две вещи.

**Trusted setup** в криптографическом смысле — это церемония генерации параметров, например для некоторых SNARK-систем.

**Honest watcher assumption** — это другое. Это операционная/security assumption:

```text
мы верим, что хотя бы один watcher скачал данные,
проверил их и не соврал.
```

Ты прав: если цель — **не полагаться на trusted watcher / DAC / committee**, тогда моя предыдущая watcher-модель тебе не подходит как основной security primitive.

Она подходит для validium/DAC-style архитектуры, но не для того, что ты сейчас описываешь.

Если ты хочешь:

```text
не публиковать raw tx
+
не доверять watcher'ам
+
всё равно иметь проверяемую корректность
```

то основной путь один:

```text
validity proof / ZK proof / STARK/SNARK-style proof
```

Сырые транзакции тогда становятся приватным witness для prover’а. Их не надо публиковать, но proof должен доказывать, что переход состояния корректный.

------

# Три возможные модели

## 1. Только checkpoint/root/signature

```text
Celestia:
  state_root
  batch_id
  aggregator signature
```

Это нотариальная модель.

Она доказывает:

```text
агрегатор зафиксировал root R в момент T.
```

Она не доказывает:

```text
root R получен корректно;
не были созданы лишние assets;
не были уничтожены права;
не был нарушен conservation rule;
не был сделан illegal transfer;
не был скрыт конфликт.
```

Это может быть полезно, но это всё ещё доверие агрегатору.

------

## 2. Root + validator/watcher quorum

```text
Celestia:
  state_root
  threshold signature from validators/watchers
```

Это лучше, чем один агрегатор.

Но security assumption всё равно такая:

```text
если quorum честный, state root корректный.
```

Это может быть децентрализованным, экономически защищённым, permissionless, со slashing, но это не “чистая” cryptographic verification для внешнего микросервиса.

Микросервис проверяет не сам переход, а подпись validator set.

------

## 3. Root + validity proof

```text
Celestia:
  previous_state_root
  new_state_root
  proof π
  circuit_id / verifier_key_id
  protocol_version
```

Вот это соответствует твоей идее лучше всего.

Сырые транзакции не публикуются.

Prover использует их как приватный witness:

```text
private witness:
  raw tx
  authorizations
  object reads
  object writes
  Merkle paths
  signatures
  nullifier updates
  voucher/rights transition data
```

А публично публикуется только:

```text
old_root
new_root
public inputs
validity proof
checkpoint metadata
```

Микросервис потом проверяет:

```text
proof π says:
  "there exists a valid batch of operations
   that transforms old_root into new_root
   according to Z00Z rules"
```

И ему не нужны raw transactions.

Это уже ближе к тому, что тебе нужно.

------

# Тогда Celestia в Z00Z — не DA для транзакций

Если ты не публикуешь сырые транзакции в Celestia, то Celestia не является DA-layer для транзакций Z00Z.

В твоей модели Celestia становится:

```text
canonical checkpoint log
timestamp layer
anti-equivocation layer
root registry
proof publication layer
migration anchor
```

То есть Celestia фиксирует:

```text
на epoch N канонический Z00Z root такой-то;
он следует из предыдущего root;
к нему привязан такой-то validity proof;
этот checkpoint был опубликован в публичной финализированной цепи.
```

Но Celestia не обязана знать и хранить raw tx.

И это нормально, если ты не строишь optimistic/fraud-proof rollup.

------

# Challenge window в такой модели меняет смысл

Если raw tx не публикуются, то классический optimistic challenge window почти не работает.

Почему?

Чтобы challenger доказал fraud, ему нужны данные перехода:

```text
какие объекты прочитаны;
какие объекты изменены;
какие подписи были использованы;
какие nullifiers появились;
какие vouchers consumed;
какие новые rights minted.
```

Если этих данных публично нет, внешний challenger не может пересчитать batch и построить fraud proof.

Поэтому:

```text
fraud-proof model требует data availability;
validity-proof model не требует fraud challenge для correctness.
```

Если Z00Z идёт по validity-proof пути, то challenge window нужен не для обычной проверки корректности state transition, а для других вещей:

```text
user complaints;
delivery failures;
receipt disputes;
censorship/omission claims;
операционные rollback windows;
bug response;
governance pause.
```

Но не для базовой криптографической проверки перехода состояния.

------

# Что значит “сырые транзакции — мусор”

В твоей модели это почти верно, но с уточнением.

Сырые транзакции — это не canonical history. Они нужны как временный witness для:

```text
1. построения нового state root;
2. генерации validity proof;
3. выдачи receipts участникам;
4. доставки recipient-specific data;
5. debug/audit/legal/forensics, если это нужно продуктово.
```

После того как:

```text
validity proof создан;
checkpoint опубликован;
proof верифицирован;
участники получили receipts / object proofs;
нужные encrypted payloads доставлены;
state snapshot сохранён;
```

сырые транзакции действительно можно считать disposable data.

Но нельзя выбрасывать всё подряд. Нужно сохранить не raw tx, а **достаточные криптографические артефакты**.

------

# Что нужно хранить долго

Вместо raw transaction log я бы хранил навсегда или почти навсегда:

```text
checkpoint headers;
old_state_root;
new_state_root;
object_root;
nullifier_root;
rights_root;
voucher_root;
state_diff_commitment;
proof π or proof commitment;
verifier_key_id;
circuit_id;
protocol_version;
aggregator/prover signature;
Celestia height / tx hash / namespace;
finality certificate;
migration records;
snapshot hashes.
```

А у пользователя / wallet должны храниться:

```text
object payload;
ownership secret / key material;
Merkle inclusion proof;
update receipt;
voucher/right certificate;
nullifier witness, если применимо;
encrypted note / recovery package;
checkpoint reference.
```

Если пользователь потерял свои object data, а raw tx больше нигде нет, один global state root ему не поможет восстановить свой voucher. Поэтому для Z00Z очень важна **user-side encrypted backup / recovery layer**.

------

# Как должна выглядеть нормальная Z00Z checkpoint-структура

Например:

```text
Z00ZCheckpoint {
  epoch;
  previous_checkpoint_hash;

  old_state_root;
  new_state_root;

  object_root;
  nullifier_root;
  rights_root;
  voucher_root;

  state_diff_commitment;
  public_input_hash;

  proof_system_id;
  circuit_id;
  verifier_key_id;
  validity_proof_hash;

  aggregator_id;
  aggregator_signature;

  created_at;
  celestia_height;
}
```

Если proof небольшой, его можно публиковать прямо вместе с checkpoint.

Если proof большой, можно публиковать:

```text
proof_hash в Celestia
сам proof в IPFS / Arweave / S3 / Z00Z archive
```

Но proof должен быть доступен долгосрочно. Если raw tx удалены и proof тоже потерян, будущий verifier уже не сможет проверить старый переход.

------

# Минимальная формула безопасности

Для твоей концепции правильное утверждение такое:

```text
Z00Z state is valid if:

1. There is a canonical chain of checkpoints:
   C0 -> C1 -> C2 -> ... -> Cn

2. Each checkpoint is anchored on Celestia or another public chain.

3. For every transition:
   state_root_i -> state_root_i+1

   there exists a validity proof π_i
   verifying that the transition obeys Z00Z rules.

4. Objects presented to microservices have valid Merkle proofs
   against the latest finalized state root.

5. Users retain or can recover their own object/voucher/right data.
```

Тогда raw transactions не нужны для обычной проверки.

------

# Где всё ещё остаются проблемы

Даже с validity proofs есть несколько важных дыр, которые надо закрыть дизайном.

## 1. Omission / censorship

Validity proof доказывает:

```text
включённые операции корректны.
```

Но он не доказывает:

```text
агрегатор включил все операции, которые должен был включить.
```

Агрегатор может просто не включить чью-то транзакцию.

Для этого нужен отдельный механизм:

```text
forced inclusion queue;
public tx commitment queue;
user submission receipt;
timeout-based escape;
alternative aggregator;
permissionless proposer;
liveness SLA with slashing.
```

Причём forced inclusion может публиковать не raw tx, а commitment:

```text
hash(tx)
encrypted_tx_pointer
user intent commitment
```

Но должен быть способ доказать:

```text
я подал операцию до deadline,
а агрегатор её проигнорировал.
```

## 2. User data availability

Даже если глобальный state root валиден, пользователь должен иметь данные своего объекта.

Для privacy/voucher/rights системы это критично.

Модель должна быть ближе к:

```text
user carries object data;
wallet stores encrypted notes;
recipient receives encrypted payload;
backup service stores encrypted recovery package;
microservice verifies object against state root.
```

Иначе можно получить ситуацию:

```text
global state valid,
но пользователь не может восстановить свой asset.
```

## 3. Circuit completeness

Если proof circuit не проверяет какое-то правило Z00Z, то неверный переход может стать “валидным” для proof system.

Поэтому правила должны быть формализованы:

```text
authorization;
ownership;
no double spend;
nullifier correctness;
voucher issuance;
rights transfer;
asset conservation;
expiry;
revocation;
policy constraints;
microservice permissions;
schema upgrades.
```

И circuit/versioning должны быть очень аккуратными.

## 4. State snapshot availability

Если новый микросервис хочет начать работать сегодня, ему не нужны raw tx, но ему нужен способ получить:

```text
latest state snapshot
+
checkpoint/proof chain
+
Merkle proof service
```

Иначе он знает root, но не может эффективно отвечать на вопросы по объектам.

------

# Где Celestia становится узким местом в твоей модели

Не в хранении raw tx.

В твоей концепции Celestia является узким местом только для:

```text
1. canonical ordering of checkpoints;
2. timestamp/finality of roots;
3. availability of checkpoint/proof commitments;
4. anti-equivocation;
5. censorship/liveness of publishing checkpoints;
6. migration anchor, если Z00Z переезжает.
```

То есть если Celestia тормозит или деградирует, Z00Z не теряет raw transaction history, потому что она там и не должна жить.

Но Z00Z может получить проблему:

```text
невозможно быстро опубликовать новый canonical root;
микросервисы не знают, какой checkpoint финальный;
агрегатор может временно зависнуть между roots;
нужна emergency migration на другой anchor.
```

Поэтому Celestia в твоей модели должна быть replaceable:

```text
CheckpointAnchor {
  publish(checkpoint_hash)
  verify_inclusion(checkpoint_hash)
  finality_depth()
}
```

А не deeply hardcoded.

------

# Что я бы сделал для Z00Z

Я бы формализовал Z00Z как **validity-proven object state ledger**.

Примерный flow:

```text
1. User / microservice submits private operation.

2. Aggregator builds private batch.

3. Aggregator/prover computes:
   old_state_root -> new_state_root.

4. Prover generates validity proof:
   "this transition is valid under Z00Z rules."

5. Z00Z publishes checkpoint to Celestia:
   old_root
   new_root
   proof_hash / proof
   circuit_id
   protocol_version
   previous_checkpoint_hash

6. Wallet receives receipt:
   operation_id
   affected object ids
   new object commitment
   Merkle proof / witness update
   checkpoint reference

7. Microservice verifies object later using:
   object payload
   Merkle proof
   latest finalized root
   checkpoint/proof validity.
```

В этой схеме raw tx не являются долгосрочной data availability. Они являются private witness.

------

# Самый важный вывод

Ты не хочешь “data availability of transactions”.

Ты хочешь:

```text
availability of state commitments;
availability of validity proofs;
availability of user-owned object witnesses;
availability of finalized snapshots.
```

Это другой дизайн.

И тогда да: **сырые транзакции можно не публиковать и не хранить долго**, если корректность перехода доказывается validity proof’ом, а не доверием к агрегатору.

Но если validity proof нет, тогда одна нотариальная печать агрегатора недостаточна. Она доказывает только:

```text
агрегатор заявил root R.
```

А тебе нужно:

```text
root R является корректным следствием предыдущего root.
```

Финальная формулировка:

```text
Raw tx are disposable.
State roots are canonical.
Objects are user-verifiable via Merkle proofs.
Transitions must be validity-proven.
Celestia anchors roots, not transaction data.
```

Вот такая концепция у тебя, как я её понимаю.





---


> Но здесь ты не прав, потому что я не хочу быть оптимистик фрод, отсутствие фрода. Я наоборот, буду отслеживать фрод, и если есть collisions на один и тот же asset, потраченный дважды или какие-то другие операции незаконные, то это как раз все буду отслеживать. Но это не на уровне блокчейна, я так думаю. Не на уровне вот этих data availability, а на уровне данных, которые сохраняются за этот challenge window, который мы можем сказать, что он там 90 дней, 180 дней. И в течение этого периода можно тогда спорить и проверить все collisions. Но я не обещаю проверять collisions десятилетней давности.

Да, теперь я понимаю точнее — и ты прав: я слишком сильно увёл это в сторону **rollup DA / optimistic fraud-proof модели**. То, что ты описываешь, ближе к другой конструкции:

```text
Z00Z = state-centric notarized ledger
      with bounded audit / challenge window

не:
  "все raw transactions доступны навсегда"

а:
  "каноническое состояние хранится в authenticated tree,
   переходы можно оспорить в течение N дней,
   после окна checkpoint становится settled"
```

И это нормальная, вполне осмысленная архитектура.

Главная поправка: тебе не обязательно публиковать **сырые транзакции**. Но тебе нужно хранить в challenge window **достаточные audit artifacts**, чтобы можно было объективно доказать collision, double-spend, unauthorized update или другой illegal transition.

------

# Как я теперь понимаю твою концепцию

У тебя есть некая state machine:

```text
assets
rights
vouchers
ownership claims
spent/nullified objects
policy state
service permissions
```

Всё это сидит в authenticated state, например:

```text
Jellyfish Merkle Tree / sparse Merkle tree / Merkle-3 structure
```

Микросервису не нужно знать историю транзакций. Ему достаточно получить:

```text
object
Merkle proof
latest finalized root
```

и проверить:

```text
этот asset / voucher / right действительно существует
и находится в валидном каноническом состоянии Z00Z.
```

То есть микросервис проверяет **state validity**, а не исторический replay.

Это сильная и правильная идея.

------

# Где здесь место raw transactions

Сырые транзакции в твоей модели — это не каноническая история. Они являются временным input material:

```text
raw tx
user signatures
service requests
off-chain messages
intents
routing metadata
transport metadata
```

После того как они привели к изменению state tree, сами raw tx действительно могут быть “мусором”, если выполнены условия:

```text
1. State transition committed.

2. Все затронутые объекты записаны в дерево.

3. Все spent/nullified assets отражены в spent/nullifier structure.

4. Пользователь получил receipt / proof.

5. Challenge window прошёл.

6. Checkpoint финализирован.

7. Есть snapshot / checkpoint root для будущей проверки.
```

Тогда тебе не нужно обещать:

```text
"любой сможет через 10 лет скачать raw tx и пересчитать genesis -> today"
```

Ты обещаешь другое:

```text
"после финализации checkpoint является базовой правдой;
объекты проверяются относительно этого checkpoint/root;
споры возможны только в пределах challenge window"
```

Это похоже на **settlement finality with statute of limitations**.

------

# Но важная граница: хранить надо не raw tx, а effect/audit data

Нельзя просто сказать:

```text
raw tx не храним,
в Celestia кладём только root,
всё остальное в дереве,
значит fraud detectable.
```

Это будет недостаточно.

Почему? Потому что текущее дерево показывает:

```text
что сейчас существует.
```

Но чтобы поймать незаконный переход, тебе нужно видеть хотя бы следы перехода:

```text
что было consumed;
что было created;
какой asset_id/version был потрачен;
какой nullifier появился;
какая policy была применена;
какая подпись или authorization существовала;
какие previous objects были использованы.
```

Поэтому правильная формулировка:

```text
Raw transactions can be disposable.

But state transition effects must be auditable
during the challenge window.
```

То есть тебе нужен не transaction log, а **effect log / audit bundle**.

------

# Что должно храниться 90–180 дней

Я бы хранил не “сырые транзакции”, а canonical audit records.

Например, для каждого batch:

```text
BatchAuditBundle {
  batch_id;

  old_state_root;
  new_state_root;

  consumed_objects_root;
  created_objects_root;
  updated_objects_root;
  nullifier_delta_root;

  asset_collision_keys_root;
  rights_delta_root;
  voucher_delta_root;

  authorization_commitments_root;
  policy_evaluation_root;

  merkle_update_witness_root;
  state_diff_root;

  ordering_root;
  receipt_root;

  aggregator_signature;
  checkpoint_reference;
}
```

Это может быть сильно меньше и чище, чем raw tx.

Raw tx может содержать:

```text
UI payload;
RPC envelope;
routing details;
temporary metadata;
network-level junk;
transport signatures;
duplicated fields;
irrelevant service data.
```

А audit bundle должен содержать только то, что нужно для проверки:

```text
был ли asset потрачен дважды;
был ли owner authorized;
не был ли voucher создан из воздуха;
не нарушена ли conservation rule;
корректно ли обновилось дерево;
соответствует ли new_root заявленным изменениям.
```

------

# Самое важное для double-spend / collision detection

Для каждого asset / voucher / right должна быть детерминированная collision key.

Например:

```text
consume_key = H(asset_id || version || domain)
```

или, если нужна приватность:

```text
nullifier = H(asset_secret || asset_id || version || domain)
```

Тогда любое spending-событие оставляет след:

```text
consume_key / nullifier
```

И fraud/collision доказывается так:

```text
Вот два разных transition records,
оба consuming один и тот же consume_key/nullifier.
Они оба входят в committed audit bundles.
Следовательно, был double spend / collision.
```

Это можно доказать без raw transactions.

Но collision surface должен быть видимым хотя бы в виде commitment/nullifier. Если collision key полностью скрыт и доступен только агрегатору, то внешний challenger не сможет доказать double-spend.

------

# Текущее дерево должно включать не только live objects

Если в дереве хранятся только live assets, этого может быть мало.

Например:

```text
asset A был потрачен;
его удалили из live tree;
потом кто-то снова попытался его потратить.
```

Чтобы это ловить, нужна одна из двух моделей.

## Вариант 1: append-only spent/nullifier set

```text
spent_nullifier_root
```

Он монотонно растёт. Каждый consumed asset оставляет nullifier/tombstone.

Тогда double spend проверяется через:

```text
nullifier already exists
```

Это наиболее чистая модель.

## Вариант 2: versioned object tree

Объект не просто исчезает, а получает состояние:

```text
asset_id -> {
  version;
  status: live / spent / revoked / expired;
  last_transition_id;
}
```

Тогда попытка потратить старую версию видна как конфликт.

Мой выбор для Z00Z: **отдельный nullifier/spent tree плюс object tree**.

```text
object_root      — что существует сейчас
nullifier_root   — что уже consumed/spent
rights_root      — права/политики
voucher_root     — ваучеры/claims
```

------

# Где здесь Celestia

В твоей модели Celestia не является местом хранения raw transactions.

Celestia нужна как:

```text
public timestamp
anti-equivocation anchor
checkpoint registry
data-root registry
finality anchor
migration anchor
```

То есть в Celestia можно публиковать:

```text
checkpoint_id
previous_checkpoint_hash
old_state_root
new_state_root
audit_bundle_root
nullifier_delta_root
state_diff_root
snapshot_root
challenge_deadline
aggregator / validator signature
```

Celestia даёт тебе важное свойство:

```text
агрегатор не может потом переписать историю
и сказать: "на самом деле checkpoint был другой".
```

Но Celestia не обязана хранить всё содержимое audit bundle.

Audit bundle может лежать в:

```text
IPFS + pinning
Z00Z archive nodes
object storage
Filecoin / Arweave for selected archives
watcher/auditor local storage
```

А в Celestia лежит commitment:

```text
audit_bundle_root
```

Если в течение challenge window кто-то приносит fraud proof, он доказывает:

```text
1. Эти audit records входят в audit_bundle_root.
2. audit_bundle_root был зафиксирован в Celestia.
3. Эти records демонстрируют collision / illegal transition.
```

------

# Это не trusted setup

Твой objection про “honest watcher” справедлив.

Если watcher — это назначенный доверенный субъект, которому все верят, то это trusted committee / DAC-like model.

Но можно сделать иначе:

```text
watchers are not trusted;
watchers are just challengers / auditors;
fraud evidence is objectively verifiable.
```

То есть watcher не говорит:

```text
"поверьте мне, я проверил"
```

Он говорит:

```text
"вот два committed records,
вот Merkle proofs,
вот collision key,
вот checkpoint commitment,
проверьте сами".
```

Тогда watcher не является trusted party. Он просто приносит доказательство.

Но остаётся другое допущение:

```text
кто-то должен успеть посмотреть и подать challenge
в течение challenge window.
```

Это не trusted setup. Это **bounded monitoring assumption**.

То есть безопасность формулируется так:

```text
Если illegal transition произошёл,
и хотя бы один заинтересованный участник обнаружил его
и подал challenge до дедлайна,
то checkpoint не финализируется.
```

После дедлайна система говорит:

```text
споры по этому периоду закрыты;
checkpoint settled.
```

------

# Это всё равно “challenge-based” модель, но не обязательно optimistic rollup

Ты не обязан называть это optimistic rollup.

Лучшее название для твоей модели:

```text
bounded-audit checkpointed state ledger
```

или:

```text
challenge-window notarized state machine
```

или:

```text
state-centric fraud-detectable ledger
```

Логика такая:

```text
1. Состояние является главным источником истины.

2. Транзакции не являются вечной историей.

3. Переходы между состояниями challengeable в течение N дней.

4. После N дней checkpoint становится settlement base.

5. Старые raw/effect data можно удалить или перевести в cold archive.

6. Десятилетней давности fraud не переоткрывает canonical state.
```

Это нормально, но важно честно принять последствие:

```text
если fraud нашли после финализации,
историю уже не откатываем.
```

Можно делать компенсации, репутационные меры, governance action, blacklisting конкретного fraudster, но не ломать settled checkpoints.

------

# Как должен выглядеть challenge

Допустим, есть checkpoint:

```text
C_k:
  old_root = R_k
  new_root = R_k+1
  audit_bundle_root = A_k
  challenge_deadline = T + 180 days
```

В течение 180 дней challenger может подать claim.

## Collision challenge

```text
Claim:
  asset X / nullifier N был consumed дважды.

Evidence:
  record_1
  Merkle proof: record_1 ∈ A_k

  record_2
  Merkle proof: record_2 ∈ A_j

  same consume_key / nullifier

Conclusion:
  double spend / collision.
```

## Unauthorized transition challenge

```text
Claim:
  object X был изменён без корректной authorization.

Evidence:
  state_diff_record
  authorization_record
  policy_id
  object policy proof
  Merkle inclusion into audit bundle

Conclusion:
  transition invalid.
```

## Bad state transition challenge

```text
Claim:
  applying committed state diff to old_root
  does not produce new_root.

Evidence:
  old_root
  state_diff
  Merkle update witnesses
  recomputation trace

Conclusion:
  checkpoint invalid.
```

## Conservation violation

```text
Claim:
  output rights/assets exceed inputs
  or voucher minted without valid issuer authority.

Evidence:
  input commitments
  output commitments
  policy rule
  issuer authorization proof
  Merkle inclusion proofs

Conclusion:
  illegal mint / conservation break.
```

Во всех этих случаях raw transaction envelope не нужен. Нужны canonical effects и witnesses.

------

# Что делать при успешном challenge

Это должно быть заранее прописано.

Например:

```text
If fraud proven before challenge_deadline:

1. checkpoint C_k marked invalid;

2. all descendants of C_k become provisional/invalid
   unless they can be replayed from last valid root;

3. aggregator/proposer bond slashed;

4. affected assets frozen or rolled back;

5. replacement checkpoint generated from last valid root;

6. fraud certificate anchored in Celestia.
```

Если ты не хочешь тяжёлых rollback’ов, можно сделать более product-friendly модель:

```text
fraud affects only conflicting object subtree;
global root patched via correction checkpoint;
fraudulent assets revoked;
victims compensated from bond/insurance;
```

Но это сложнее. Важно решить заранее, потому что через 90–180 дней зависимые транзакции могут создать длинную цепочку.

------

# Большой практический риск: длинный challenge window создаёт “pending finality”

Если challenge window 180 дней, то формально всё состояние за последние 180 дней является challengeable.

Это может быть нормально, но тогда у объектов должны быть статусы:

```text
pending
mature
finalized
disputed
revoked
```

Например:

```text
asset создан сегодня
=> его можно использовать сразу,
   но он несёт finality risk до истечения окна.

asset старше 180 дней
=> mature/finalized.
```

Для микросервисов можно дать выбор:

```text
low-risk service:
  accepts pending assets

high-value service:
  accepts only finalized/mature assets

medium-risk service:
  accepts pending assets with insurance/collateral
```

Иначе обнаруженный fraud через 120 дней может внезапно затронуть огромное дерево последующих операций.

------

# Можно сделать два уровня финальности

Я бы сделал не один challenge window, а несколько уровней:

```text
Soft confirmation:
  seconds/minutes
  aggregator accepted operation

Operational confirmation:
  minutes/hours
  batch included in checkpoint

Audited confirmation:
  24–72 hours
  enough independent replays / automated scans passed

Economic finality:
  7–30 days
  no challenge, bonds still active

Archival finality:
  90–180 days
  raw/effect data may be pruned or cold-archived
```

Тогда пользователю не нужно ждать 180 дней для всего, но протокол честно знает:

```text
до archival finality старые audit artifacts ещё доступны
и checkpoint ещё можно глубоко спорить.
```

------

# Какие данные можно удалить после 180 дней

После финализации можно удалить:

```text
raw tx envelopes;
transport metadata;
temporary signatures;
full execution traces;
debug logs;
large witness data;
intermediate service messages.
```

Но я бы не удалял полностью:

```text
checkpoint headers;
state roots;
nullifier roots;
object roots;
audit bundle roots;
fraud certificates;
finality certificates;
snapshot hashes;
protocol version records;
aggregator/proposer signatures.
```

И я бы хранил хотя бы cold archive дольше, чем 180 дней, если это дешево. Не как security guarantee, а для forensics/debug/compliance.

------

# Что обязательно должно остаться навсегда

Минимальный permanent record:

```text
CheckpointHeader {
  checkpoint_id;
  previous_checkpoint_hash;

  old_state_root;
  new_state_root;

  object_root;
  nullifier_root;
  rights_root;
  voucher_root;

  audit_bundle_root;
  state_diff_root;
  receipt_root;

  challenge_deadline;
  finality_status;

  protocol_version;
  aggregator_set_id;

  celestia_anchor;
}
```

Это то, что позволяет любому микросервису сказать:

```text
я доверяю finalized root R_n как settlement base,
а объект X доказывается Merkle proof'ом относительно R_n.
```

------

# Важная разница между “проверить объект” и “проверить всю историю”

Ты прав, что микросервису чаще всего не нужно проверять всю историю.

Ему нужно:

```text
object payload
Merkle proof
root finality proof
```

То есть:

```text
Is this voucher valid now?
Is this right assigned now?
Is this asset unspent now?
Is this object included in finalized state?
```

Для этого raw tx не нужны.

Raw/effect history нужна только для спора:

```text
а был ли этот state root получен незаконно?
```

И ты ограничиваешь этот вопрос по времени:

```text
этот вопрос можно задавать только в течение challenge window.
```

Это coherent model.

------

# Где именно у тебя узкое место

В твоей модели узкое место не Celestia и не DA.

Узкие места такие:

## 1. Достаточность audit bundle

Если audit bundle слишком бедный, fraud нельзя доказать.

Например, если ты хранишь только:

```text
old_root
new_root
```

то collision не докажешь.

Нужно хранить:

```text
consume keys/nullifiers;
state diffs;
authorization commitments;
Merkle witnesses;
policy evaluation records.
```

## 2. Видимость collision keys

Если double-spend fingerprint не виден никому кроме агрегатора, то независимый challenge невозможен.

Нужен хотя бы публичный или проверяемый:

```text
nullifier / consume_key / spent marker
```

## 3. Доступность audit data в течение окна

Не обязательно в Celestia.

Но должно быть реально доступно:

```text
IPFS pinning;
Z00Z archive nodes;
mirrors;
auditor storage;
object storage;
checksums;
periodic retrieval tests.
```

## 4. Чёткое правило финальности

После 180 дней нельзя внезапно сказать:

```text
мы нашли fraud двухлетней давности, всё откатываем.
```

Нужно заранее определить:

```text
after finality, no canonical rollback.
```

Иначе система не имеет настоящей финальности.

## 5. Пользовательские receipts

Если пользователь хочет спорить, он должен иметь:

```text
receipt;
object witness;
signed operation copy or hash;
Merkle proof;
checkpoint reference.
```

Особенно если raw tx не хранятся публично.

------

# Финальная формулировка твоей модели

Я бы описал Z00Z так:

```text
Z00Z does not preserve raw transaction history as canonical data.

Z00Z preserves canonical state roots and object commitments.

For a bounded challenge window, Z00Z preserves enough transition evidence
to detect and prove collisions, double-spends, unauthorized updates,
and invalid state transitions.

After the challenge window, checkpoints become final settlement bases.
Old raw/effect data may be pruned or moved to cold archive.

Microservices verify current validity through Merkle proofs
against finalized state roots, not by replaying transaction history.
```

Это не “trusted setup”.

Это:

```text
bounded fraud-detectable state system
```

с допущением:

```text
fraud must be detected and challenged before finality deadline.
```

------

# Мой практический вердикт

Да, твоя концепция правильная, но я бы слегка переформулировал:

Не надо хранить **raw transactions**.

Надо хранить на 90–180 дней:

```text
canonical state diffs;
consume/nullifier records;
authorization evidence;
Merkle update witnesses;
policy evaluation records;
receipt roots;
checkpoint manifests.
```

Celestia в этом дизайне нужна не как storage raw tx, а как:

```text
immutable anchor for checkpoints and audit commitments.
```

После 90–180 дней можно сказать:

```text
этот checkpoint settled;
десятилетней давности collisions не проверяются;
микросервисы работают от финализированного state root.
```

Это честная, инженерно здоровая модель — при условии, что **в течение challenge window доказательство fraud действительно объективно строится из сохранённых audit artifacts**, а не требует доверия к агрегатору.