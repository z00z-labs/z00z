---
id: telemetry.aggregators.placement
title: Toplayıcı yerleşimi
summary: Runtime’a ait shard generation, primary owner, secondary readiness ve journal lineage görünümünü açıklar.
scope: context
---
## Bu görünümü kullanma {#current-view}
- Küresel topoloji çıkarmadan `ShardPlacementView` sözleşmesini inceleyin.
- Kullanılamaz, güncel placement table gözlemi bağlı olmadığı anlamına gelir.

## Fail-closed sınırı
- Tablo tam shard ve routing generation değerine sahip olmalıdır.
- Aggregator ID operasyonel veridir; endpoint ve cüzdan kimlikleri gizli kalır.
