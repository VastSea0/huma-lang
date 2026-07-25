# Hüma diller arası benchmark paketi

Bu paket aynı kaynak iş yüklerini Hüma'nın yürütme yolları ile makinede kurulu
gerçek dil araç zincirlerinde çalıştırır. Ölçüm süreç başlangıcını, kaynak
yüklemeyi ve çalışma zamanını birlikte kapsar.

```bash
python3 benchmarks/cross_language/run.py --samples 30 --warmups 5
```

Runner:

- Her programın çıktısını ölçümden önce ve her örnekte beklenen değerle
  karşılaştırır.
- Çalıştırma sırasını sabit tohumla karıştırarak ısı ve sıra yanlılığını
  azaltır.
- Medyan, ortalama, minimum, p95, standart sapma ve ayrı süreçte tepe RSS
  ölçer.
- Hüma yorumlayıcı, VM ve destekleniyorsa AOT ile Python, Node.js, Ruby, C,
  Rust, Swift ve Java araçlarını otomatik keşfeder.
- Bulunmayan araç zincirini açıkça `skipped` olarak kaydeder.

Sonucu JSON ve Markdown olarak kaydetmek için:

```bash
python3 benchmarks/cross_language/run.py \
  --samples 30 \
  --warmups 5 \
  --json-output benchmarks/cross_language/results/sonuc.json \
  --markdown-output benchmarks/cross_language/results/sonuc.md
```

`.build/` altındaki derlenmiş programlar ölçüm girdisi değildir ve Git
tarafından izlenmez. Karşılaştırmanın kapsamı ve yorumlama kuralları
[`docs/KARSILASTIRMALI_BENCHMARK.md`](../../docs/KARSILASTIRMALI_BENCHMARK.md)
belgesindedir.
