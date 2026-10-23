# Ajan Günlüğü

Yalnız ekleme. Her yineleme bir satır.

| Tarih | Commit | Bulgu | Özet | Yeni bulunanlar |
|---|---|---|---|---|
| 2026-10-23 #1 | 9c1ca1c | B-001, B-017 | Kapılar çalıştırıldı: fmt/clippy temiz, 195 test geçti; www lint 4 hata verdi, JSX metinleri süslü paranteze alınarak düzeltildi; lint+build yeşil; 3 www commit'i ile birlikte push | B-017 |
| 2026-10-23 #2 | 13b463a | B-018 | Ayrıştırıcıya 200 düzeylik kesin derinlik sınırı; ayrıştırıcı, yorumlayıcı ve bytecode derleyicisine paylaşılan `huma_syntax::yigin::yigini_koru` (stacker). 12 düşmanca yuvalama biçimi tek tanı veriyor; sınır altı program 2 MiB test iş parçacığında iki arka uçta doğru sonuç veriyor | B-019…B-029 |
| 2026-10-23 #3 | 4fbd7d3 | B-007, B-026, B-027 | `exit_codes` üzerindeki gereksiz `allow(dead_code)` kaldırıldı; eksik dosya artık 1 yerine belgelenmiş 2 koduyla çıkıyor; kodlar `--help`'te ve e2e testte. B-026/B-027 doğrulanıp reddedildi | B-030…B-033 |
| 2026-10-23 #4 | e18095c | B-006 | Ayrıştırıcının hiç üretmediği `Ifade::NesneOlustur` ve `Komut::Atama` varyantları ile yorumlayıcı/bytecode/AOT'taki dört ölü eşleşme kolu silindi | — |
| 2026-10-23 #5 | 28ef29f | B-034 | Linux'a özgü `needless_return` clippy hatası düzeltildi; ders: her push'tan sonra `gh run list` ile uzak CI denetlenecek, yerel macOS kapıları Linux `cfg` bloklarını görmüyor | B-035…B-038 |
| 2026-10-23 #6 | 5a55d91 | B-035 | rustls, quinn-proto, rustls-webpki, webpki-roots, chacha20 semver-uyumlu güncellendi; hyper 0.14 `full` yerine yalnız HTTP/1 sunucu özellikleri → h2 0.3 ağaçtan çıktı. `cargo audit`: 12 açık → 0. HTTP/1 sunucu için ilk uçtan uca test eklendi (eski/yeni özelliklerle geçiyor) | B-039…B-043 |
| 2026-10-23 #7 | af26ae9 | B-041 | `huma test` alt süreçleri gizli `__test-dosyasi` alt komutuyla çalışıyor; paket testleri en yakın `huma.json` kökünde. CI örnek paket bağımlılıklarını kuruyor. `huma test tests` 8/8, `huma test examples` 4/4 yerelde geçti | — |
| 2026-10-23 #8 | f061be4 | B-036 | security-audit işi `cargo install cargo-audit --locked --version 0.22.2` + `cargo audit` olarak yeniden yazıldı; yerelde 1.94.1 ile kurulum ve denetim (çıkış 0) doğrulandı | — |
| 2026-10-23 #9 | (bu commit) | B-047 | Bağımlılıksız projede `huma kur` boş kilit üretir; `ilkle` ile aynı `bos_kilit_json` yardımcısı. Uzak CI: `tests` işi af26ae9'da ilk kez yeşil | B-044…B-048 |
