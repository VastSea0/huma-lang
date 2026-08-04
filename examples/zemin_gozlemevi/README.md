# Hüma Zemin Gözlemevi

Yapay zekâya dayanmayan, genel amaçlı Hüma zeminini gösteren küçük bir veri
işleme projesidir. Bir CSV envanterini okur, sağlık ve performans bütçelerini
denetler, JSONL denetim izi ile Markdown raporu üretir ve JSONL dosyasını geri
okuyarak çıktıyı doğrular.

Proje kökünden çalıştırın:

```bash
cargo build --release --locked -p huma-cli
./target/release/huma run examples/zemin_gozlemevi/ana.hb \
  --izin dosya-okuma \
  --izin dosya-yazma
```

Üretilen dosyalar:

- `target/huma-zemin-raporu.md`
- `target/huma-zemin-olaylari.jsonl`

Örnek yalnızca dosya okuma/yazma yetkisi ister. Ağ, süreç, veritabanı, FFI,
GUI veya yapay zekâ izni verilmez.
