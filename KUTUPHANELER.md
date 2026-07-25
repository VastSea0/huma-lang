# Hüma 0.6 Kütüphaneleri

Bu belge depodaki çalışan giriş noktalarını listeler. Tam imza için ilgili `.hb` kaynağı ve Rust yerleşik kayıtları esas alınır.

## Gömülü `lib/` kütüphaneleri

Bu dosyalar binary içine gömülür ve her dizinden yüklenebilir:

```huma
"matematik.hb"'yi yükle
```

| Dosya | Başlıca API |
|---|---|
| `matematik.hb` | `karesi`, `küpü`, `mutlak`, `kuvvet`, `yuvarla`, `faktöriyel`, `ebob`, `ekok`, `asal_mı` |
| `istatistik.hb` | `ortalama`, `en_büyük`, `en_küçük`, `varyans`, `standart_sapma` |
| `liste.hb` | `yazdır_liste`, `içeriyor_mu`, `ters_cevir`, `eşle`, `filtrele`, `indirge`, `dilimle` |
| `dizgi.hb` | `büyük_mü`, `küçük_mü`, `boşluk_mu`, başlangıç/bitiş uyumluluk adları |
| `dosya.hb` | `güvenli_oku`, `satırlara_ayır` |
| `rastgele.hb` | `r_tamsayı`, `r_seç`, `r_karıştır` |
| `renkler.hb` | terminal renk sabitleri, `renkli_yaz`, `başarı_yaz`, `uyarı_yaz`, `hata_yaz` |
| `zaman.hb` | `beklet`, `kronometre_başlat`, `kronometre_bitir` |
| `birim_test.hb` | `test_et`, `iddia_et`, `test_raporu` |
| `yapay_zeka_temel.hb` | vektör/matris yardımcıları, aktivasyonlar, kayıplar, başlatıcılar, metrikler |

Yaygın Rust yerleşikleri arasında `uzunluk`, `listeye_ekle`, `içeriyor`,
`kırp`, `böl`, `birleştir`, `küçük_harf`, `büyük_harf`, JSON, dosya, ağ,
SQLite, vektör/matris ve tensor işlevleri bulunur. Kayıtlı yerleşikler argüman
sayısını/türünü, sayısal sonluluğu ve işlemine uygun boyut sınırlarını doğrular;
geçersiz girdi yakalanabilir `Hata` üretir. Koleksiyon veya dış kaynak
kullanılan kod yine de bu hatayı ele almalıdır.

## `huma_modulleri/` paketleri

### `nlp_temel`

```huma
"nlp_temel"'i yükle
```

Türkçe temizleme/tokenizasyon, durak kelime filtresi, kural tabanlı kök bulma, duygu/POS/NER yardımcıları sağlar. Bunlar kural tabanlı prototiplerdir; dilbilimsel doğruluk veri kümesiyle garanti edilmez.

### `nlp_ileri`

```huma
"nlp_ileri"'i yükle
```

TF-IDF ve CPU tabanlı sözcük gömme araçlarını yükler. Önceki eksik Hüma
modülü BPE taslağı kararlı API'den çıkarılmıştır. Çekirdekteki `bpe_eğit`,
`bpe_kodla` ve `bpe_çöz` işlevleri UTF-8'i kayıpsız işleyen, bayt düzeyli ve
testli temel bir tokenizer sağlar; model kalıcılığı yoktur.

### `yapay_zeka`

```huma
"yapay_zeka"'i yükle

model = sinir_agi() olsun
model.ilklendir() olsun
model.katman_ekle(8, 16, "relu") olsun
model.katman_ekle(16, 1, "sigmoid") olsun
```

Yoğun katman, MSE tabanlı geri yayılım, Adam güncellemesi, gradyan kırpma ve JSON model kaydı sağlar. Çalışma zamanı CPU ve `f64` odaklıdır.

### Sistem modülleri

- `ag_istekleri`: HTTP istemci sarmalayıcıları
- `huma_sunucu`: HTTP sunucu sarmalayıcıları
- `huma_sqlite`: SQLite bağlantı/sorgu API’si
- `gui`: egui tabanlı masaüstü arayüz API’si

Bu modüller ilgili CLI yeteneği verilmeden dış kaynağa erişemez. Yetenek adları:
`dosya-okuma`, `dosya-yazma`, `ağ-istemci`, `ağ-sunucu`, `süreç`, `ffi`,
`veritabanı` ve `gui`. Bu model en az ayrıcalık denetimidir; işletim sistemi
sandbox’ı değildir.

FFI yalnız açık `f64()`, `f64(f64)` ve `f64(f64,f64)` imzalarını kabul eder.
`ffi_yükle`, `ffi_çağır` ve `ffi_boşalt` ile yaşam döngüsü yönetilir. Yanlış
haricî ABI ev sahibi süreci çökertebileceğinden FFI yalnız güvenilen
kitaplıklarda kullanılmalıdır.
