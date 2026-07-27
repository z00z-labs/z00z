---
id: telemetry.aggregators.publication
title: Toplayıcı yayını
summary: Sıralı batch’in checkpoint, quorum, DA ve lifecycle evidence ile nasıl bağlandığını açıklar.
scope: context
---
## Bu görünümü kullanma {#current-view}
- `PublicationRequest` → `PublishedBatch` → `PublicationRecord` akışını izleyin.
- Kullanılamaz, doğrulanmış publication veya readiness bundle bağlı olmadığı anlamına gelir.

## Fail-closed sınırı
- Eksik ya da uyuşmayan provider, height, manifest, payload, statement veya evidence reddedilir.
- Checkpoint root, proof ve lifecycle truth Storage’a aittir.
