# Hüma Performans ve Bellek Ölçümü

Bu belgenin amacı aynı Hüma sürümündeki gerilemeleri yeniden üretilebilir iş
yükleriyle görünür kılmaktır. Diller arası sonuçlar yalnız tanımlı mikro
iş yükleri için raporlanır; genel hız üstünlüğü iddiası olarak kullanılamaz.

## Diller arası benchmark

Python, Node.js, Ruby, C, Rust, Swift ve Java için eşdeğer kaynaklar; çıktı
doğrulayan runner, ham örnekler ve ölçüm yorumu [Diller Arası Benchmark
Raporu](KARSILASTIRMALI_BENCHMARK.md) belgesindedir.

```bash
python3 benchmarks/cross_language/run.py --samples 30 --warmups 5
```

Bu ölçüm süreç başlangıcını içerir. Hüma'nın parser, yorumlayıcı ve VM
çekirdeklerini süreç başlangıcı olmadan izlemek için aşağıdaki Criterion
benchmark'ı ayrı tutulur.

## Mikro benchmark

```bash
cargo bench -p huma-core --bench runtime
```

Criterion paketi üç ayrı sınırı ölçer:

- `parser/10k-loop-source`: kanonik döngü kaynağını lexer + parser’dan geçirme
- `interpreter/10k-loop`: önceden ayrıştırılmış 10.000 yinelemeli AST’yi yürütme
- `vm/10k-loop`: önceden derlenmiş programı yeni bir VM’de yürütme

Sonuçlar `target/criterion/` altında saklanır. Karşılaştırma ancak aynı makine,
aynı Rust araç zinciri, aynı güç/ısı koşulu ve aynı Criterion ayarlarıyla
yapılmalıdır.

## Otomatik gerileme kapısı

`huma-perf` aşağıdaki zemin iş yüklerini ısınma sonrası çoklu örnekle ölçer;
medyan, p95 ve süreç RSS değerlerini sürümlü JSON rapora yazar:

- lexer+parser ve boş yorumlayıcı başlangıcı,
- sayısal döngü ve VM karşılığı,
- 1.000 fonksiyon çağrısı ve 1.000 closure çağrısı,
- liste büyütme, sözlük yazma ve Unicode metin birleştirme,
- küçük bir dosya modülünün çözülmesi/ayrıştırılması/yüklenmesi,
- 1.000 canlı genç nesnede minor GC,
- 1.000 erişilemez çevrimde major GC.

```bash
cargo run --release --locked -p huma-perf -- measure rapor.json
cargo run --release --locked -p huma-perf -- check taban.json aday.json
```

Adanmış `huma-perf` runner'ındaki PR kapısı, aynı makinede taban ve aday
revizyonları yeniden derler. Her suite için medyan sürede %5, p95 veya RSS'te
%10'dan büyük gerileme işi başarısız yapar. Paylaşımlı GitHub runner sonuçları
donanım gürültüsü nedeniyle normatif karşılaştırma değildir.

Yerel ilk doğrulamada 10.000 yinelemeli iş yükünde VM'nin yorumlayıcıdan yavaş
olduğu da görünürdür. Bu nedenle VM hız iddiası taşımaz; performans ve semantik
eşlik kanıtlanana kadar deneysel kalır.

## Dış süreç ve tepe bellek

Önce sürüm ikilisini üretin:

```bash
cargo build --release --locked -p huma-cli
```

macOS:

```bash
/usr/bin/time -l target/release/huma run benchmarks/core_loop.hb
/usr/bin/time -l target/release/huma run benchmarks/core_loop.hb --vm
```

Linux:

```bash
/usr/bin/time -v target/release/huma run benchmarks/core_loop.hb
/usr/bin/time -v target/release/huma run benchmarks/core_loop.hb --vm
```

Kanonik dış süreç iş yükü 100.000 aralık yinelemesi yapar ve
`5000050000` üretmelidir. Süre veya tepe RSS raporlanırken işletim sistemi,
mimari, işlemci, RAM, Rust sürümü, git revizyonu ve komut birlikte
kaydedilmelidir. Windows için CI derleme desteği vardır; karşılaştırılabilir
tepe bellek komutu henüz normatif değildir.

## Performans güvenlik sınırları

Çekirdek; sınırsız işi “hız” adına kabul etmez. Varsayılan 10 milyon yürütme
adımı, 32 çağrı derinliği, 16 MiB çıktı ve 1 milyon koleksiyon öğesi sınırına
sahiptir. Matris/tensor, tokenizer, regex, dosya, HTTP, SQL ve dış süreç
işlemleri kendi boyut veya süre sınırlarını uygular. Bu sınırlar benchmark’tan
ayrıdır ve güvenlik sözleşmesinin parçasıdır.
