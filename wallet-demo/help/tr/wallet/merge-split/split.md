---
id: wallet.split
title: "Cüzdan: Bölme"
route: wallet.merge-split
scope: context
---

# Cüzdan: Bölme

[TOC]

## Uygulama görünümü {#current-view}

![Cüzdan bölme görünümü](help/assets/en/wallet-split.png)

Bu görüntü Demo’daki güncel Bölme görünümünden alınmıştır.

## Genel bakış {#overview}

Bölme, tek bir gizli varlık parçasını tüketir ve iki veya daha fazla çıktı hazırlar. Her çıktı kaynağın `definition_id` ve temel `serial_id` değerlerini korur; tüm pozitif çıktı tutarları girdi tutarına tam olarak eşit olmalıdır. İşlem çıktı düzenini değiştirir, varlık tanımını değiştirmez veya yeni arz oluşturmaz.

Ortaya çıkan her parça, aynı ihraç serisinin parçası olarak kalırken kendine ait somut bir çıktı kimliği alır.

## Bu görünüm nasıl kullanılır {#how-to-use-this-view}

1. Uygulama başlığında etkin cüzdanı ve ağı doğrulayın.
2. **Böl** seçeneğini belirleyin.
3. Kullanılabilir bir kaynak parça seçin.
4. İki ile sekiz arasında pozitif çıktı tutarı girin.
5. **Koruma** değerinin **Tam** olduğunu doğrulayın.
6. **Bölmeyi önizle** seçeneğini belirleyin; kaynağı ve önerilen her çıktıyı inceleyin.
7. Yalnızca yetkilendirme, ücretler, gönderim ve mutabakatı yeniden denetleyebilen yerel bir cüzdanda devam edin.

## Terimler ve denetimler {#terms-and-controls}

| Terim veya denetim | Açıklama |
| --- | --- |
| Kaynak varlık | Önerilen bölme tarafından tüketilen tek kullanılabilir parça. |
| Tanım kimliği | Varlık türünün ve politikasının değişmez kimliği. Her çıktı kaynak tanımını korur. |
| Seri kimliği | Temel ihraç serisi. Her çıktı kaynağın serisini korur. |
| Çıktı dağılımı | Önerilen çıktılara atanan iki ile sekiz arasındaki pozitif tutar. |
| Koruma | Girdi tutarıyla tüm çıktı tutarlarının toplamı arasındaki tam eşitlik. |
| Çıktı ekle | Arayüz sınırına kadar başka bir pozitif tutar alanı ekler. |
| Bölmeyi önizle | Kaynağı ve önerilen çıktıları gösteren yalnızca inceleme amaçlı niyet; imzalama veya gönderim yapmaz. |

## Güvenlik ve sınırlar {#safety-and-limits}

- Bölme kaynak tanımını veya temel seriyi değiştirmez.
- Sıfır, negatif, aşırı veya tutarı korumayan dağılımlar reddedilmelidir.
- Yerel cüzdan sonradan kilitlenmiş, harcanmış, dondurulmuş, yakılmış, cezalandırılmış veya başka şekilde kullanılamaz olan bir kaynağı reddetmelidir.
- Tekrarlanan veya olağandışı örüntülü dağılımlar ilişkili çıktıların eşleştirilmesini kolaylaştırabilir.
- JavaScript Demo herkese açık test verileri kullanır ve önizlemede durur. Anahtar tutmaz, sahipliği kanıtlamaz, imza oluşturmaz, ücret almaz, paket göndermez veya belirsiz sonucu mutabakata bağlamaz.
- Mevcut `wallet.asset.split_asset` yardımcısı bir uyumluluk yüzeyidir ve kanonik kayıt mutabakatı yetkisi iddia etmez. Yerel entegrasyon, onayı yetkili cüzdan işlem yolu üzerinden yönlendirmelidir.

<!-- help-sync:source {"page_path":"wallet/merge-split/split.md","route_id":"wallet.merge-split","screenshot":"help/assets/en/wallet-split.png","topic_id":"wallet.split"} -->
