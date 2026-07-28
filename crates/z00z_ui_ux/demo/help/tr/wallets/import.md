---
id: wallet.import
title: İçe aktar
summary: İçe aktarma, diskteki herkese açık varlık paketini etkin cüzdana vermeden önce inceler.
scope: context
---
## Bu görünümü kullanma {#current-view}
- En fazla 64 KiB olan bir `AssetPkgWire` JSON paketi seçin.
- Cüzdanı, ağı, sınıfı, tutarı, seri kimliğini, alanı, durum bayraklarını ve sahip bağını inceleyin.
- **Varlığı içe aktar** seçeneğini kullanın; kriptografi, sahiplik, replay ve claim çakışmasını yerel cüzdan doğrular.

## Yerel ve güvenli çalışma
- `secret` alanı yasaktır; mutlak dosya yolu saklanmaz veya RPC’ye gönderilmez.
- Sonuç yeni içe aktarımı, mevcut varlığı ve açık bir `IMPORT_*` nedenini ayırır.
