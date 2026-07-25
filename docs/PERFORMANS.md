# Hüma Performans ve Bellek Ölçümü

Bu belge başka dillere karşı ölçülmemiş hız iddiası içermez. Amacı aynı Hüma
sürümündeki gerilemeleri yeniden üretilebilir bir iş yüküyle görünür kılmaktır.

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

## Dış süreç ve tepe bellek

Önce sürüm ikilisini üretin:

```bash
cargo build --release -p huma-cli
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
