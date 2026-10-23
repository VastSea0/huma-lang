# Ajan Günlüğü

Yalnız ekleme. Her yineleme bir satır.

| Tarih | Commit | Bulgu | Özet | Yeni bulunanlar |
|---|---|---|---|---|
| 2026-10-23 #1 | 9c1ca1c | B-001, B-017 | Kapılar çalıştırıldı: fmt/clippy temiz, 195 test geçti; www lint 4 hata verdi, JSX metinleri süslü paranteze alınarak düzeltildi; lint+build yeşil; 3 www commit'i ile birlikte push | B-017 |
| 2026-10-23 #2 | 13b463a | B-018 | Ayrıştırıcıya 200 düzeylik kesin derinlik sınırı; ayrıştırıcı, yorumlayıcı ve bytecode derleyicisine paylaşılan `huma_syntax::yigin::yigini_koru` (stacker). 12 düşmanca yuvalama biçimi tek tanı veriyor; sınır altı program 2 MiB test iş parçacığında iki arka uçta doğru sonuç veriyor | B-019…B-029 |
| 2026-10-23 #3 | 4fbd7d3 | B-007, B-026, B-027 | `exit_codes` üzerindeki gereksiz `allow(dead_code)` kaldırıldı; eksik dosya artık 1 yerine belgelenmiş 2 koduyla çıkıyor; kodlar `--help`'te ve e2e testte. B-026/B-027 doğrulanıp reddedildi | B-030…B-033 |
| 2026-10-23 #4 | (bu commit) | B-006 | Ayrıştırıcının hiç üretmediği `Ifade::NesneOlustur` ve `Komut::Atama` varyantları ile yorumlayıcı/bytecode/AOT'taki dört ölü eşleşme kolu silindi | — |
