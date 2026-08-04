---
id: telemetry.aggregators.recovery
title: Toplayıcı kurtarma
summary: Bağlı route, generation, primary ve journal lineage karşısında restart ve secondary takeover kontrollerini açıklar.
scope: context
---
## Bu görünümü kullanma {#current-view}
- `ShardRecoveryRecord`, recovery intent, durable state ve execution ticket sözleşmesini inceleyin.
- Kullanılamaz, bağlı committed recovery snapshot olmadığı anlamına gelir.

## Fail-closed sınırı
- Yanlış generation, primary, shard, batch, route veya lineage reddedilir.
- Renderer failover başlatamaz veya Storage recovery truth değerini değiştiremez.
