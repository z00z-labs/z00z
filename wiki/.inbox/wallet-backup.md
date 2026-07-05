> chem wallet backup luchshe ili huzhe prostogo kopirovanija .wlt fila ? v chrm raznica mezhdu backup i copirovaniem

Коротко: в этом репо `wallet backup` лучше обычного копирования `.wlt`, если цель это нормальное восстановление или перенос кошелька. Копия `.wlt` это raw snapshot физического store-файла. Backup это уже поддерживаемый логический export/import-контракт.

Главная разница:

- `.wlt` здесь это on-disk wallet store, zstd-compressed RedB файл [tables.rs (line 16)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/redb_store/tables.rs:16). Но canonical tx history хранится отдельно в JSONL sidecar, и код прямо говорит: `.wlt` remains wallet state only [tx_storage_impl.rs (line 45)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/persistence/tx_storage_impl.rs:45).
- Backup собирает `WalletExportPack` с profile, owned assets, owned objects, scan state, stealth meta, TOFU pins, keys, seed phrase и wallet identity [wallet_store_export_pack.rs (line 84)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_store_export_pack.rs:84). Потом это заворачивается в `BackupPayload` с `network`, `chain` и optional forensic archive [backup_wire.rs (line 580)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/backup/backup_wire.rs:580).
- При создании backup сервис отдельно подтягивает canonical tx-history JSONL и пишет единый `backup-<wallet>-<timestamp>.json` [wallet_actions_backup.rs (line 1240)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_actions_backup.rs:1240), [wallet_actions_backup.rs (line 1268)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_actions_backup.rs:1268), [wallet_actions_backup.rs (line 1291)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_actions_backup.rs:1291).
- Restore по умолчанию идёт в режиме `WalletPlusHistory`, а не просто “подменить файл”; есть ещё `WalletOnly` и `TxHistoryOnly` [wallet_actions_backup.rs (line 1401)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_actions_backup.rs:1401), [wallet_actions_backup.rs (line 1442)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_actions_backup.rs:1442), [backup_importer.rs (line 87)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/backup/backup_importer.rs:87).
- Restore сделан атомарно: staging, `.bak`, `.restore.json`, rollback при сбое, потом publish [wallet_actions_backup.rs (line 1042)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_actions_backup.rs:1042), [wallet_actions_backup.rs (line 1069)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_actions_backup.rs:1069), [wallet_actions_backup.rs (line 1101)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_actions_backup.rs:1101), [wallet_actions_backup.rs (line 1128)](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_actions_backup.rs:1128). Тест подтверждает, что при failpoint original `.wlt` и history остаются нетронутыми [test_wallet_restore_atomic.rs (line 272)](/home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_wallet_restore_atomic.rs:272).

Что это значит practically:

- `copy .wlt` лучше только в одном узком смысле: это самый простой low-level raw copy внутреннего store-файла.
- `wallet backup` лучше почти во всём остальном: он переносит не только `.wlt`, но и history plane, имеет явный формат, проверку/валидацию, режимы restore и безопасный rollback.

То есть:

- Если нужен supported recovery/migration: используй backup.
- Если нужен developer/raw snapshot внутреннего store: можно копировать `.wlt`, но `.wlt` alone это неполный backup для этого репо.

Если хочешь, могу ещё отдельно разобрать: что именно потеряешь при копировании только `.wlt` на уровне UX/данных пользователя.
