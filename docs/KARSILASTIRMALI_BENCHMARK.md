# Hüma Diller Arası Benchmark Raporu

Bu rapor Hüma 0.6 yürütme yollarını aynı makinede çalıştırılan Python,
Node.js, Ruby, C, Rust, Swift ve Java programlarıyla karşılaştırır. Sonuçlar
yalnız tanımlanan mikro iş yükleri için geçerlidir; bir dilin bütün uygulama
alanlarındaki hızını veya üretim yeterliliğini kanıtlamaz.

Ham örnekler, çalıştırılan komutlar ve tüm istatistikler:

- [Makinece okunabilir JSON](../benchmarks/cross_language/results/2026-07-26-apple-m4.json)
- [Üretilmiş tam sonuç tablosu](../benchmarks/cross_language/results/2026-07-26-apple-m4.md)
- [Runner ve eşdeğer kaynaklar](../benchmarks/cross_language/README.md)

## Deney sözleşmesi

Ölçülen dört iş yükü:

| İş yükü | Kapsam | Beklenen çıktı |
|---|---|---:|
| `branch_loop` | 200.000 koşullu sayısal durum geçişi | `99740874156250` |
| `numeric_loop` | 200.000 LCG ve modüler toplam adımı | `429301206370208` |
| `function_loop` | 100.000 kullanıcı fonksiyonu çağrısı | `2090659698` |
| `collection_loop` | 50.000 öğe ekleme, indeksleme ve toplama | `24836475` |

Bütün tamsayı ara değerleri `2^53` sınırının altında tutulur. Böylece Hüma ve
JavaScript'in IEEE-754 `f64` temsili ile C/Rust/Swift/Java tamsayıları ve
Python/Ruby geniş tamsayıları bu girdilerde aynı kesin sonucu üretebilir.
Runner, yanlış çıktı üreten bir programı ölçüme almaz ve her ölçüm örneğinde
çıktıyı yeniden doğrular.

Derlenmiş uygulamalar standart optimize ayarlarıyla üretildi:

- C: `clang -O3`
- Rust: `rustc -C opt-level=3`
- Swift: `swiftc -O`
- Hüma: release CLI ve AOT için varsayılan `-O 2`
- Java, Node.js, Python ve Ruby: kurulu çalışma zamanlarının varsayılan
  yürütme davranışı

Ölçüm uçtan ucadır: yeni sürecin başlatılması, kaynak/bytecode yükleme ve iş
yükünün yürütülmesi birlikte ölçülür. Her aday 5 kez ısıtılmış, ardından sabit
tohumla karıştırılmış sırada 30 ayrı süreçte çalıştırılmıştır. Tepe RSS,
`/usr/bin/time -l` ile üç ayrı süreçte ölçülüp medyanı alınmıştır.

## Ölçüm ortamı

- Yerel tarih: 26 Temmuz 2026
- Kaynak revizyonu: `884497db279339aa2c2e19a03721a27b3403e362`
- İşletim sistemi: macOS 26.5.1, arm64
- İşlemci: Apple M4
- Bellek: 16 GiB
- Hüma: 0.6.0
- Python: 3.9.6
- Node.js: 24.14.1
- Ruby: 2.6.10p210
- Apple Clang: 21.0.0
- Rust: 1.94.1
- Swift: 6.3.1
- OpenJDK: 26.0.1

Bu sürümler eşit yaşta değildir. Özellikle Python ve Ruby sistem sürümleridir.
Rapor bu farkı gizlemez ve farklı makine veya sürümler arasında doğrudan oran
taşımaz.

## Sonuç özeti

Aşağıdaki değerler süreç başlangıcı dâhil 30 örneğin medyanıdır:

| İş yükü | Hüma yorumlayıcı | Hüma VM | Hüma AOT | Python | Node.js | C `-O3` | Rust `-O` |
|---|---:|---:|---:|---:|---:|---:|---:|
| `branch_loop` | 213,142 ms | 131,944 ms | 3,730 ms | 46,219 ms | 22,599 ms | 2,627 ms | 2,728 ms |
| `numeric_loop` | 124,416 ms | 108,869 ms | desteklenmiyor | 46,334 ms | 27,849 ms | 2,941 ms | 3,093 ms |
| `function_loop` | 96,749 ms | 78,856 ms | desteklenmiyor | 34,242 ms | 24,261 ms | 2,656 ms | 2,828 ms |
| `collection_loop` | 39,122 ms | desteklenmiyor | desteklenmiyor | 25,746 ms | 22,545 ms | 2,384 ms | 2,572 ms |

### Nesnel çıkarımlar

1. VM, kapsadığı üç sayısal/fonksiyon iş yükünde yorumlayıcıdan hızlıdır.
   Bununla birlikte Python'a göre medyan oranların geometrik ortalaması VM için
   yaklaşık `2,49×`, yorumlayıcı için `3,27×` daha uzun süredir.
2. Koleksiyon iş yükünde Hüma yorumlayıcı Ruby'den yaklaşık `1,13×`,
   Python'dan `1,52×`, Node.js'ten `1,74×` daha uzun süre kullanmıştır.
3. AOT'nin desteklediği `branch_loop`, C'den `1,42×`, Rust'tan `1,37×`,
   Swift'ten `1,06×` daha uzun uçtan uca medyan vermiştir. Bu sonuçta yaklaşık
   2–4 ms düzeyindeki süreç başlatma maliyeti baskındır; yalnız makine kodu
   çekirdeğinin aynı oranda olduğu sonucu çıkarılamaz.
4. Sayısal işlerde Hüma yorumlayıcı/VM tepe RSS değeri yaklaşık 11,2–11,4 MiB
   aralığındadır. Aynı ölçümde Python yaklaşık 9,7 MiB, Ruby 27,9 MiB,
   Java 42 MiB ve Node.js 48–51 MiB kullanmıştır.
5. Hüma koleksiyon iş yükü yaklaşık 19 MiB tepe RSS kullanmıştır. Bu değer
   Python'ın 11,4 MiB değerinden yüksek, Ruby/Java/Node.js değerlerinden
   düşüktür.

## Ortaya çıkan backend boşlukları

Runner desteklenmeyen işleri sessizce başka yürütme yoluna düşürmez:

- AOT `%` operatörünü desteklemediği için `numeric_loop` ve `function_loop`
  AOT tablosuna alınmamıştır.
- Bytecode derleyici doğal `ListeEkle` komutunu desteklemediği için
  `collection_loop` VM'de ölçülmemiştir.
- AOT liste değer modelini desteklemediği için koleksiyon testi AOT'de
  ölçülmemiştir.

Bu sonuçlar performanstan önce kapsam sorunlarıdır. VM'de koleksiyon mutasyonu,
AOT'de kalan sayısal operatörler ve tam fonksiyon/değer modeli tamamlanmadan
“tam dil VM/AOT performansı” iddia edilemez.

## Tekrarlama

Kurulu araç zincirleri otomatik keşfedilir; bulunmayan dil açıkça atlanır:

```bash
python3 benchmarks/cross_language/run.py \
  --samples 30 \
  --warmups 5 \
  --memory-samples 3 \
  --json-output benchmarks/cross_language/results/yerel.json \
  --markdown-output benchmarks/cross_language/results/yerel.md
```

Karşılaştırma yaparken aynı makine, güç/ısı durumu, işletim sistemi, araç
zinciri sürümleri, örnek sayısı ve Git revizyonu kullanılmalıdır. Paylaşımlı CI
runner'larındaki süreler sabit performans eşiği olarak kullanılmamalıdır.
