---
id: telemetry.aggregators.ingress
title: アグリゲーター入力
summary: Runtime がトランザクションまたは claim payload を digest に結び付いた作業項目として受け入れる方法を説明します。
scope: context
---
## この画面の使い方 {#current-view}
- `WorkPayload` から `WorkItem` または `RejectRecord` への契約を確認します。
- 利用不可は新しい admission snapshot がないことを示し、受理や拒否を意味しません。

## Fail-closed 境界
- Object package の結合は admission digest と intake identity を変更します。
- Raw payload、受取人、メモ、ウォレットのローカル経路はヘルプに入りません。
