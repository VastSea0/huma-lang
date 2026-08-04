# Hüma 0.6 Deneysel Kütüphane Envanteri

Bu belge yalnız depodaki mevcut deneysel giriş noktalarını listeler. Bunlar
kararlı kamu API'si değildir, gelecekteki kütüphane mimarisini belirlemez ve
çekirdek zemin kurulurken yeniden yazılabilir veya kaldırılabilir. Projenin
odağı yeni alan kütüphanesi eklemek değil [Mühendislik
Anayasası](docs/MUHENDISLIK_ANAYASASI.md) kapsamındaki genel amaçlı zemini
tamamlamaktır.

Mevcut `.hb` modülleri hâlâ deneysel envanterdir. Yeni native/haricî paketlerin
makinece okunabilir imza, etki ve hata kataloğu [HMI v1](docs/HMI.md) ile
tanımlanır; `huma paket api-kontrol` kırıcı değişiklikleri SemVer'e karşı sınar.

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
`kırp`, `böl`, `birleştir`, `küçük_harf`, `büyük_harf` ve JSON bulunur.
Dosya/CSV/JSONL (`huma-stdlib-file`) ile tensor/BPE (`huma-stdlib-ai`) fiziksel
olarak ayrı deneysel adaptörlerdir; ağ, süreç, SQLite, GUI ve native sınırları da
ayrı crate'lerde yaşar. Kayıtlı yerleşikler argüman
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
- `gui`: eski aynı-süreç masaüstü prototipi. Bakımı bırakılmış transitive font
  ayrıştırıcısı nedeniyle varsayılan workspace ve CLI'dan karantinadadır;
  kararlı GUI yolu değildir.

Bu modüller ilgili CLI yeteneği verilmeden dış kaynağa erişemez. Yetenek adları:
`dosya-okuma`, `dosya-yazma`, `ağ-istemci`, `ağ-sunucu`, `süreç`, `ffi`,
`veritabanı` ve gelecekteki GUI adaptörleri için `gui`. Bu model en az ayrıcalık denetimidir; işletim sistemi
sandbox’ı değildir.

Açılan yaşam döngülü kaynaklar açıkça kapatılmalıdır: SQLite bağlantıları
`dahili_sql_kapat`, HTTP sunucuları `dahili_sunucu_kapat`, HMI modülleri
`hmi_kapat`, güvenilir süreç içi FFI kitaplıkları `ffi_boşalt` kullanır. Sunucu,
bağlantı ve modül tabloları ayrıca sabit eşzamanlı kaynak sınırlarına sahiptir.

Genel native kütüphane yolu `hmi_başlat`, `hmi_çağır` ve `hmi_kapat` ile ayrı
süreçte çalışan HMI'dır. Eski süreç içi FFI yalnız açık `f64()`, `f64(f64)` ve
`f64(f64,f64)` imzalarını kabul eder; varsayılan kayıtlı değildir ve hem `ffi`
yeteneği hem `--güvenilir-süreç-içi-ffi` gerektirir. Yanlış ABI ev sahibi süreci
çökertebildiğinden dağıtılan kütüphaneler bu yolu kullanmamalıdır.
