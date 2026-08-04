---
id: telemetry.aggregators.planning
title: Toplayıcı planlama
summary: Settlement yetkisi iddia etmeden deterministik batch ve shard route bağlamasını açıklar.
scope: context
---
## Bu görünümü kullanma {#current-view}
- Planner mode, route generation, intake ve operation sayıları ile digest sahipliğini inceleyin.
- Kullanılamaz, doğrulanmış `BatchPlanned` snapshot’ı bağlı olmadığı anlamına gelir.

## Fail-closed sınırı
- Yapılandırma, generation, route-table digest ve yeniden hesaplanan plan eşleşmelidir.
- Planlama settlement, publication veya storage truth değerini kesinleştirmez.
