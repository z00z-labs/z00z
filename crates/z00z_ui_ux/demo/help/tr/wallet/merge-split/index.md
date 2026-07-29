---
id: wallet.merge
title: "Cüzdan: Birleştirme"
route: wallet.merge-split
scope: context
---

# Cüzdan: Birleştirme

[TOC]

## Uygulama görünümü {#current-view}

![Cüzdan birleştirme görünümü](help/assets/en/wallet-merge.png)

Bu görüntü Demo’daki güncel Birleştirme görünümünden alınmıştır.

## Genel bakış {#overview}

Birleştirme, iki veya daha fazla uyumlu gizli varlık parçasını tek bir çıktıda toplar. Çıktı aynı `definition_id` ve temel `serial_id` değerlerini korur; tutarı seçilen girdilerin toplamına eşittir. İşlem çıktı düzenini değiştirir, varlık tanımını değiştirmez veya yeni arz oluşturmaz.

Adaylar hem tanıma hem de seriye göre gruplanır. Aynı görüntüleme sembolünü kullansalar bile farklı gruplardaki parçalar birleştirilemez.

## Bu görünüm nasıl kullanılır {#how-to-use-this-view}

1. Uygulama başlığında etkin cüzdanı ve ağı doğrulayın.
2. **Birleştir** seçeneğini belirleyin.
3. Tek bir uyumluluk grubundan en az iki kullanılabilir parça seçin.
4. Seçilen girdi sayısını, toplam çıktı tutarını, tanımı ve seriyi kontrol edin.
5. **Birleştirmeyi önizle** seçeneğini belirleyin; her girdiyi ve önerilen tek çıktıyı inceleyin.
6. Yalnızca yetkilendirme, ücretler, gönderim ve mutabakatı yeniden denetleyebilen yerel bir cüzdanda devam edin.

## Terimler ve denetimler {#terms-and-controls}

| Terim veya denetim | Açıklama |
| --- | --- |
| Tanım kimliği | Varlık türünün ve politikasının değişmez kimliği. Seçilen tüm girdiler bunu paylaşmalıdır. |
| Seri kimliği | Temel ihraç serisi. Tüm girdiler ve birleştirilmiş çıktı aynı seriyi korur. |
| Varlık kimliği | Belirli bir gizli çıktının kimliği. Uyumlu parçaların varlık kimlikleri farklı olabilir. |
| Uyumlu grup | Aynı tanım ve seri kimliğine sahip kullanılabilir parçalar. |
| Kilitli | Parça bağlam için görünürdür ancak seçilemez. |
| Toplam çıktı | Ayrı bir yerel ücret politikası uygulanmadan önce seçilen girdilerin tam toplamı. |
| Birleştirmeyi önizle | Girdileri ve önerilen çıktıyı gösteren yalnızca inceleme amaçlı niyet; imzalama veya gönderim yapmaz. |

## Güvenlik ve sınırlar {#safety-and-limits}

- Bu arayüz farklı tanımlar veya temel seriler arasında birleştirme yapmaz.
- Yerel cüzdan; kilitli, harcanmış, dondurulmuş, yakılmış, cezalandırılmış veya başka şekilde kullanılamaz girdileri, eski bir ekran daha önce göstermiş olsa bile reddetmelidir.
- Parçaları birleştirmek ilişkili girdilerin eşleştirilmesini kolaylaştırabilir. Tekrarlanan veya belirgin örüntülü işlemlerden önce gizlilik etkisini inceleyin.
- JavaScript Demo herkese açık test verileri kullanır ve önizlemede durur. Anahtar tutmaz, sahipliği kanıtlamaz, imza oluşturmaz, ücret almaz, paket göndermez veya belirsiz sonucu mutabakata bağlamaz.
- Mevcut `wallet.asset.merge_assets` yardımcısı bir uyumluluk yüzeyidir ve kanonik kayıt mutabakatı yetkisi iddia etmez. Yerel entegrasyon, onayı yetkili cüzdan işlem yolu üzerinden yönlendirmelidir.

<!-- help-sync:source {"page_path":"wallet/merge-split/index.md","route_id":"wallet.merge-split","screenshot":"help/assets/en/wallet-merge.png","topic_id":"wallet.merge"} -->
