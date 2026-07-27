---
id: telemetry.aggregators.ingress
title: Toplayıcı girişi
summary: Runtime’ın bir işlem veya claim payload’unu digest bağlı bir iş öğesi olarak nasıl kabul ettiğini açıklar.
scope: context
---
## Bu görünümü kullanma {#current-view}
- `WorkPayload` → `WorkItem` veya `RejectRecord` sözleşmesini inceleyin.
- Kullanılamaz, güncel kabul snapshot’ı olmadığı anlamına gelir; kabul veya ret anlamına gelmez.

## Fail-closed sınırı
- Object package bağlama admission digest ve intake identity değerini değiştirir.
- Ham payload, alıcı, memo ve cüzdanın yerel rotaları Yardım içine girmez.
