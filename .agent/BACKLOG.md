# Hüma Bulgu Listesi

> Bu liste ilk teşhisten gelen **başlangıç** listesidir ve bilerek eksik
> bırakılmıştır. Ajan her keşif turunda yeni bulgular ekler. Kimlikler asla
> yeniden kullanılmaz.

Başlangıç durumu (2026-10-23): `cargo clippy -D warnings` temiz,
`cargo test --workspace` yeşil (~200 test / ~28k satır Rust). Yerel `main`,
`origin/main`'in 3 commit önünde (www sitesi commit'leri), henüz push edilmedi.

## Bulgular

| ID | Öncelik | Alan | Açıklama | Kanıt | Durum |
|---|---|---|---|---|---|
| B-001 | P0 | git | Yerel `main` origin'in 3 commit önünde; ilk iş olarak kapıları çalıştırıp push et | `git status` | bitti (9c1ca1c) |
| B-002 | P1 | docs/cargo | Belgeler eski aynı-süreç GUI prototipinin "varsayılan workspace/CLI'dan karantinaya alındığını" söylüyor, fakat `huma-stdlib-gui` hâlâ workspace üyesi ve `huma-cli` ona doğrudan bağımlı | `docs/DURUM_VE_YOL_HARITASI.md`, `Cargo.toml:15`, `crates/huma-cli/Cargo.toml:24` | açık |
| B-003 | P2 | runtime | `interpreter.rs` 6887 satır, yalnızca 56 `fn` — devasa fonksiyonlar/`match` blokları. Karakterizasyon testleriyle modüllere böl | `crates/huma-runtime/src/interpreter.rs` | açık |
| B-004 | P1 | tests | `tests/*.hb` (kutuphane_test, test_derle, test_faktoriyel, test_list2, test_mimari, test_sinif, test_turkce, tum_ozellikler_test) hiçbir Rust testinde adıyla çağrılmıyor; `source_corpus_tests.rs` görünüşe göre yalnız ayrıştırma yapıyor, çıktı doğrulamıyor. Golden-output koşucusu gerekli (yorumlayıcı + VM) | `tests/source_corpus_tests.rs:19` | açık |
| B-005 | P1 | runtime | Üretim kodunda 223 `unwrap()/expect()` ve çok sayıda `panic!`. Kullanıcı girdisiyle tetiklenebilenleri tanıya çevir | `rg -n 'unwrap\(\)\|expect\(\|panic!' crates` | açık |
| B-006 | P1 | syntax | `ast.rs` içinde `#[allow(dead_code)]` ile susturulmuş enum varyantları (ör. `NesneOlustur`). Ya kullan ya sil | `crates/huma-syntax/src/ast.rs:28,63` | açık |
| B-007 | P1 | cli | `exit_codes` modülü `#[allow(dead_code)]` ile susturulmuş; tanımlı çıkış kodları gerçekten kullanılmıyor olabilir | `crates/huma-cli/src/main.rs:21` | açık |
| B-008 | P2 | runtime | `#[allow(unpredictable_function_pointer_comparisons)]` — değer eşitliğinde fonksiyon işaretçisi karşılaştırması; semantik olarak tanımsız sonuç riski | `crates/huma-runtime/src/value.rs:59` | açık |
| B-009 | P2 | modules | `nlp_temel` modülünün en az 3 kopyası var ve birbirinden sapmış (`huma_modulleri/`, `examples/tor-qurtu/huma_modulleri/`, `examples/musteri_geri_bildirim_analizci/huma_modulleri/`). Vendoring yerine paket yöneticisi/yol bağımlılığı kullan | `diff -rq` | açık |
| B-010 | P1 | examples | Kapsam şişmesi + scratch dosyalar: `examples/huma_gpt/` (scratch_test_100m.hb, transformer_100m.hb, eğitim verisi), `examples/kelimeyivektore/` (~900 KB JSON), `examples/tor-qurtu/` (1.1 MB), `debug_heceleme.hb`, `simple_test.hb`, `check_type.hb`. Anayasa'ya göre çekirdek odaklı olunmalı | `du -sh examples/*` | açık |
| B-011 | P1 | docs | README "Verified status" tablosu ve CHANGELOG'daki her iddia (nesilli GC + yazma bariyeri, interpreter/VM hata eşliği, O(1) argmax, 100M Transformer) bir teste bağlanmalı veya silinmeli | `README.md`, `README.en.md`, `CHANGELOG.md`, `docs/*.md` | açık |
| B-012 | P1 | www | `www/site/README.md` create-next-app şablonu; `public/` altında kullanılmayan şablon SVG'leri (next.svg, vercel.svg, file.svg, globe.svg, window.svg) | `www/site/` | açık |
| B-013 | P2 | repo | `.gitignore` artık var olmayan `src-tauri`, `huma-ide-*` girdileri içeriyor; `*.txt` küresel olarak yok sayılıyor ama `examples/huma_gpt/egitim_metni.txt` izleniyor — tutarsız | `.gitignore` | açık |
| B-014 | P2 | arch | Çekirdek–alan ayrımı: `huma-stdlib-ai` (autograd, tokenizer) çekirdek workspace'te; Anayasa alan kütüphanelerinin çekirdeği kısıtlamaması gerektiğini söylüyor. CLI'ya hangi stdlib crate'lerinin varsayılan bağlandığını denetle, feature flag arkasına al | `Cargo.toml`, `crates/huma-cli/Cargo.toml` | açık |
| B-015 | P2 | ci | CI yalnızca `cargo test` koşuyor; örneklerin çalıştığını, yorumlayıcı–VM eşliğini, fuzz smoke'unu, `www/site` build'ini ve `vscode-huma` paketini doğrulamıyor | `.github/workflows/ci.yml` | açık |
| B-016 | P3 | spec | EBNF (119 satır) ile parser'ın kabul ettiği dil arasında otomatik eşlik testi yok | `docs/DIL_GRAMERI.ebnf`, `crates/huma-syntax/src/parser.rs` | açık |
| B-017 | P0 | www | Push edilmemiş www commit'leri `npm run lint`'te 4 hatayla kırmızıydı (JSX metin düğümünde `//` ve kaçışsız `"`); CI www lint koşmadığı için yakalanmadı (B-015 ile ilişkili) | `www/site/app/[lang]/belgeleme/page.tsx:114,116,219,221` | bitti (9c1ca1c) |
| B-018 | P0 | syntax/runtime/compiler | Derin yuvalama süreci yığın taşmasıyla düşürüyordu (SIGABRT, çıkış 134): 5000 parantez/liste/tekli eksi, debug'da tek ifadede 600 terimli `+` zinciri, 5000 iç içe blok/sınıf; 2 MiB iş parçacığında 190 blok + 31 çağrı. Ayrıştırıcıya kesin ağaç derinliği hesabıyla 200 düzey sınırı + ayrıştırıcı/yorumlayıcı/bytecode derleyicisinde `stacker` ile yığın koruması | `huma run derin.hb` → `thread 'main' has overflowed its stack`; lldb: `Komut::clone`, `Derleyici::ifade_derle` | bitti (bkz. LOG 2026-10-23 #2) |
| B-019 | P1 | cli/docs | `huma run dosya.hb` paket yöneticisi dışında reddediliyor (da361dc), fakat `huma run --help` ("Bir Hüma kaynak dosyasını … çalıştır"), CLI modül belgesi (`main.rs:1-10`), `docs/DIL_TANIMI.md` §13 ve `docs/PERFORMANS.md:75-83` doğrudan dosya çalıştırmayı anlatıyor; PERFORMANS komutları bu hâliyle çalışmaz | `huma run x.hb` → "yalnız paket yöneticisiyle çalıştırılır" | açık |
| B-020 | P1 | vm | Yorumlayıcı–VM hata eşliği yok: yeteneği kapalı `dosya_oku` yorumlayıcıda "'dosya-okuma' yeteneği verilmedi", VM'de "Tanımsız değişken: dosya_oku"; `a[-1]` iletileri farklı. README'deki eşlik iddiası (B-011) bununla çelişiyor | `huma run [--vm] dosya.hb` | açık |
| B-021 | P3 | vm | VM indeksli atamayı (`b[0] = x olsun`) ve doğal liste eklemeyi (`a'ya x'i ekle`) derlemiyor (açık hata veriyor, §12'ye uygun) — çekirdek yapılar; desteklenen alt küme belgede listelenmeli veya uygulanmalı | `huma run --vm kendiliste.hb` | açık |
| B-022 | P2 | syntax | UTF-8 BOM ile başlayan kaynak "Bilinmeyen karakter: \u{feff}" verir; BOM ya atlanmalı ya özel tanı verilmeli, karar DIL_TANIMI §1'e yazılmalı | `printf '\xef\xbb\xbf' > bom.hb` | açık |
| B-023 | P2 | runtime | Yorumlayıcı AST'yi değerle alıyor; her fonksiyon çağrısında gövde derin `clone` ediliyor (lldb izinde `Komut::clone`). Hem performans hem yığın maliyeti; referans/`Rc` ile değerlendirme | `interpreter.rs` `ifade_hesapla(&mut self, ifade: Ifade)` | açık |
| B-024 | P3 | diagnostics | Tanılar `tracing` ile zaman damgası + `ERROR` + ANSI kodlarıyla basılıyor (çıktı borulansa bile); dosya adı, kaynak satırı alıntısı ve `^^^` işaretçisi yok (PROMPT §3.3) | `huma run sifir.hb 2>&1 \| cat -v` | açık |
| B-025 | P3 | diagnostics | Özyineleme sınırı hatası 32 özdeş çerçeveli tek satırlık çağrı izi basıyor; tekrarlar katlanmalı ("f (2:4) × 32") | `huma run ozyineleme.hb` | açık |
| B-026 | P1 | compiler | `crates/huma-compiler/src/lib.rs:149` `parse_program()` ile sözdizimi tanılarını yok sayıyor olabilir — doğrula | `crates/huma-compiler/src/lib.rs:149` | açık |
| B-027 | P1 | tests | `test_bekle_http_istekleri` varsayılan test takımında; ağ gerektiriyorsa CI'da kırılgan/çevrimdışı kırmızı — doğrula | `tests/integration_tests.rs` | açık |
| B-028 | P2 | runtime | `varsayilan_global_degiskenler` tek fonksiyon ~4300 satır (`interpreter.rs:844-5204`); B-003'ün alt işi: yerleşikleri alana göre modüllere böl | `grep -n "fn " interpreter.rs` | açık |
| B-029 | P2 | tests/ci | `soak.rs` testleri `#[ignore = "scheduled soak gate"]`; bunları çalıştıran zamanlanmış CI işi var mı doğrula, yoksa ölü test | `crates/huma-runtime/tests/soak.rs` | açık |

## Keşif kaydı

Ajan derinlemesine taradığı her alanı buraya işler.

| Tarih | Alan | Yeni bulgu ID'leri | Not |
|---|---|---|---|
| 2026-10-23 | Genel yüzey taraması (ilk teşhis) | B-001…B-016 | Derin okuma yapılmadı; tüm crate'ler yeniden taranmalı |
| 2026-10-23 | CLI düşmanca girdi bataryası + yorumlayıcı/VM çapraz yürütme | B-018…B-029 | Boş dosya, BOM, geçersiz UTF-8, derin yuvalama, sıfıra bölme, büyük sayı, sonsuz döngü/özyineleme, döngüsel liste, kapalı yetenek, NFC denendi. NFC, sıfıra bölme, adım/özyineleme sınırları iki arka uçta da doğru |
