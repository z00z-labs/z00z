---
id: wallet.import
title: 导入
summary: 导入会先检查磁盘中的公开资产包，再交给当前钱包。
scope: context
---
## 使用此视图 {#current-view}
- 选择不超过 64 KiB 的 `AssetPkgWire` JSON 包。
- 检查钱包、网络、类别、金额、序列 ID、域、状态标志和所有者绑定。
- 选择**导入资产**；原生钱包会验证密码学、所有权、重放和 claim 冲突。

## 本地和安全行为
- 禁止 `secret` 字段；不会保存绝对文件路径，也不会将其发送到 RPC。
- 结果会区分新导入、已存在资产和明确的 `IMPORT_*` 原因。
