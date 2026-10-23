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
| B-006 | P1 | syntax | `ast.rs` içinde `#[allow(dead_code)]` ile susturulmuş enum varyantları (ör. `NesneOlustur`). Ya kullan ya sil | `crates/huma-syntax/src/ast.rs:28,63` | bitti (e18095c) |
| B-007 | P1 | cli | `exit_codes` modülü `#[allow(dead_code)]` ile susturulmuş; tanımlı çıkış kodları gerçekten kullanılmıyor olabilir | `crates/huma-cli/src/main.rs:21` | bitti (4fbd7d3) |
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
| B-018 | P0 | syntax/runtime/compiler | Derin yuvalama süreci yığın taşmasıyla düşürüyordu (SIGABRT, çıkış 134): 5000 parantez/liste/tekli eksi, debug'da tek ifadede 600 terimli `+` zinciri, 5000 iç içe blok/sınıf; 2 MiB iş parçacığında 190 blok + 31 çağrı. Ayrıştırıcıya kesin ağaç derinliği hesabıyla 200 düzey sınırı + ayrıştırıcı/yorumlayıcı/bytecode derleyicisinde `stacker` ile yığın koruması | `huma run derin.hb` → `thread 'main' has overflowed its stack`; lldb: `Komut::clone`, `Derleyici::ifade_derle` | bitti (13b463a) |
| B-019 | P1 | cli/docs | `huma run dosya.hb` paket yöneticisi dışında reddediliyor (da361dc), fakat `huma run --help` ("Bir Hüma kaynak dosyasını … çalıştır"), CLI modül belgesi (`main.rs:1-10`), `docs/DIL_TANIMI.md` §13 ve `docs/PERFORMANS.md:75-83` doğrudan dosya çalıştırmayı anlatıyor; PERFORMANS komutları bu hâliyle çalışmaz | `huma run x.hb` → "yalnız paket yöneticisiyle çalıştırılır" | bitti (d210c9f) |
| B-020 | P1 | vm | Yorumlayıcı–VM hata eşliği yok: yeteneği kapalı `dosya_oku` yorumlayıcıda "'dosya-okuma' yeteneği verilmedi", VM'de "Tanımsız değişken: dosya_oku"; `a[-1]` iletileri farklı. README'deki eşlik iddiası (B-011) bununla çelişiyor | `huma run [--vm] dosya.hb` | açık |
| B-021 | P3 | vm | VM indeksli atamayı (`b[0] = x olsun`) ve doğal liste eklemeyi (`a'ya x'i ekle`) derlemiyor (açık hata veriyor, §12'ye uygun) — çekirdek yapılar; desteklenen alt küme belgede listelenmeli veya uygulanmalı | `huma run --vm kendiliste.hb` | açık |
| B-022 | P2 | syntax | UTF-8 BOM ile başlayan kaynak "Bilinmeyen karakter: \u{feff}" verir; BOM ya atlanmalı ya özel tanı verilmeli, karar DIL_TANIMI §1'e yazılmalı | `printf '\xef\xbb\xbf' > bom.hb` | açık |
| B-023 | P2 | runtime | Yorumlayıcı AST'yi değerle alıyor; her fonksiyon çağrısında gövde derin `clone` ediliyor (lldb izinde `Komut::clone`). Hem performans hem yığın maliyeti; referans/`Rc` ile değerlendirme | `interpreter.rs` `ifade_hesapla(&mut self, ifade: Ifade)` | açık |
| B-024 | P3 | diagnostics | Tanılar `tracing` ile zaman damgası + `ERROR` + ANSI kodlarıyla basılıyor (çıktı borulansa bile); dosya adı, kaynak satırı alıntısı ve `^^^` işaretçisi yok (PROMPT §3.3) | `huma run sifir.hb 2>&1 \| cat -v` | açık |
| B-025 | P3 | diagnostics | Özyineleme sınırı hatası 32 özdeş çerçeveli tek satırlık çağrı izi basıyor; tekrarlar katlanmalı ("f (2:4) × 32") | `huma run ozyineleme.hb` | açık |
| B-026 | P1 | compiler | `crates/huma-compiler/src/lib.rs:149` `parse_program()` ile sözdizimi tanılarını yok sayıyor olabilir — doğrula | `crates/huma-compiler/src/lib.rs:149` | reddedildi (yalnız `#[cfg(test)]` içinde, girdi geçerli; genel sorun B-031'de) |
| B-027 | P1 | tests | `test_bekle_http_istekleri` varsayılan test takımında; ağ gerektiriyorsa CI'da kırılgan/çevrimdışı kırmızı — doğrula | `tests/integration_tests.rs` | reddedildi (test 127.0.0.1:0 üzerinde yerel sunucu açıyor, dış ağ gerekmiyor) |
| B-028 | P2 | runtime | `varsayilan_global_degiskenler` tek fonksiyon ~4300 satır (`interpreter.rs:844-5204`); B-003'ün alt işi: yerleşikleri alana göre modüllere böl | `grep -n "fn " interpreter.rs` | açık |
| B-029 | P2 | tests/ci | `soak.rs` testleri `#[ignore = "scheduled soak gate"]`; bunları çalıştıran zamanlanmış CI işi var mı doğrula, yoksa ölü test | `crates/huma-runtime/tests/soak.rs` | açık |
| B-030 | P1 | tests | Derleyici testlerinden bazıları çıktı doğrulamıyor: `compile_and_run` yalnız hata olmadığına bakıyor (`compiler_vm_ic_ice_fonksiyon` "iç" yazdığını denetlemiyor), `compiler_derle_kontrollu_basarili` yalnız `is_ok` | `crates/huma-compiler/src/lib.rs:21,135-150` | açık |
| B-031 | P2 | syntax | `Parser::parse_program()` genel API'si sözdizimi tanılarını sessizce atıyor; üretimde kullanılmıyor ama testlerde kullanılıyor (`tests/integration_tests.rs` derin özyineleme testi). Kaldır veya tanıları döndürsün | `crates/huma-syntax/src/parser.rs` `parse_program` | açık |
| B-032 | P1 | cli | `huma run --json` "Output diagnostics in JSON" diyor fakat çalışma zamanı tanısı yine `tracing` metni olarak basılıyor; JSON çıktı yok | `huma run --json sifir.hb` | açık |
| B-033 | P3 | cli | `huma build x.hb` çıktı adı verilmezse `x.hbc` yerine sabit `cikti.hbc` yazıyor; aynı dizinde iki derleme birbirini ezer | `huma build sifir.hb` → `cikti.hbc` | açık |
| B-034 | P0 | ci | Main CI kırmızı: `rust-quality` Linux'a özgü `cfg` bloğunda `clippy::needless_return` ile düşüyor (`huma-perf/src/main.rs:301`); macOS'ta yerel clippy bu bloğu görmediğinden yakalanmadı. Ayrıca CI `tests` işi `needs: rust-quality` olduğu için hiç çalışmadı | `gh run view 39922940847 --log-failed` | bitti (28ef29f) |
| B-035 | P0 | security | `cargo audit` 12 açık buluyor: rustls 0.23.42 (4), quinn-proto 0.11.16 (5), rustls-webpki 0.103.13, webpki-roots 1.0.9, h2 0.3.27; ayrıca yanked chacha20 0.10.1, bakımsız ttf-parser | `cargo audit` | bitti (5a55d91; ttf-parser bakımsız uyarısı GUI zincirinde kaldı, B-002/B-014 ile ele alınacak) |
| B-036 | P0 | ci | `security-audit` işi kendi aracını kuramıyor: `rustsec/audit-check` cargo-audit'i `--locked` olmadan derliyor, kstring 1.96 istiyor, depo 1.94.1'e pinli → iş her zaman kırmızı ve açıkları raporlamıyor | CI log: `kstring@2.0.5 requires rustc 1.96.0` | bitti (f061be4) |
| B-037 | P2 | perf | `huma-perf` RSS okunamazsa `unwrap_or(0)` ile 0 KiB raporluyor; performans kapısı sahte bellek değeri karşılaştırır | `crates/huma-perf/src/main.rs:287` | açık |
| B-038 | P1 | ci | `performance.yml` `[self-hosted, huma-perf]` etiketli çalıştırıcı istiyor; böyle bir çalıştırıcı yoksa iş PR'larda sonsuza kadar kuyrukta kalır, kapı fiilen yok | `.github/workflows/performance.yml:18` | açık |
| B-039 | P1 | net | Sunucu bağlantı görevleri thread-local `LocalSet` üzerinde yalnız bir görev `bekle`nirken sürülüyor; `dahili_sunucu_yanitla` sonrası yanıt bir sonraki `bekle`ye kadar istemciye yazılmıyor, programın son yanıtı hiç gitmiyor. Sunucu çok iş parçacıklı ayrı çalışma zamanında sürülmeli | `crates/huma-stdlib-net/src/lib.rs` `TaskExecutor`; test `sunucu_http1_istegini_uctan_uca_yanitlar` geçici sürme görevi | açık |
| B-040 | P1 | net/security | `dahili_sunucu_baslat` her zaman `0.0.0.0`'a bağlanıyor; yerel geliştirme sunucusu varsayılan olarak tüm ağ arayüzlerine açık. Varsayılan 127.0.0.1 olmalı, dış arayüz açık argümanla | `crates/huma-stdlib-net/src/lib.rs` `server_start` | açık |
| B-041 | P0 | cli/tests | `huma test` her dosyayı `huma run` alt süreciyle çalıştırıyor; da361dc'den beri yönetilen-çalıştırma kısıtına takılıp tüm testler "ERROR" veriyor. CI `tests` işi rust-quality kırmızı olduğu için hiç çalışmadığından görünmedi. Paket testleri ayrıca paket kökünden ve kurulu bağımlılıklarla çalışmalı | CI run 39924505886 | bitti (af26ae9) |
| B-042 | P2 | packages | `huma.lock` `guncelleme_zamani` zaman damgası taşıyor; her `huma kur` hash'ler aynıyken kilit dosyasını kirletiyor (yeniden üretilebilirlik) ve dosya sonu satırsonunu siliyor | `git diff examples/akilli_zemin_studyosu/huma.lock` | açık |
| B-043 | P3 | cli | İmzasız paket hata iletisi kaynak koddaki girintiyi taşıyor (satır içinde çok sayıda boşluk ve satır sonu) | `huma kur` (imzasız paket) | açık |
| B-044 | P1 | benchmarks | `benchmarks/cross_language/run.py` Hüma'yı `huma run <dosya>` ile çağırıyor; da361dc'den beri reddedildiği için koşucu `RuntimeError` ile duruyor. `results/` altındaki tek sonuç 2026-07-26 tarihli (kısıttan önce); belgedeki sayılar artık yeniden üretilemez | `run.py:110,125` | açık |
| B-045 | P2 | cli | `huma paket run` alt sürecin çıkış kodunu yutuyor: sözdizimi (3) / çalışma zamanı (4) hatası yerine her zaman 1 ("Betik hata ile sonlandı") | `package_manager.rs` `run_script` | bitti (bkz. LOG 2026-10-23 #12) |
| B-046 | P1 | cli | `huma paket run` programın stdout'una "Hüma: X betiği çalıştırılıyor: …" satırı basıyor; tek yürütme yolu bu olduğu için her programın çıktısı kirleniyor (boru/golden test bozulur). Bilgi stderr'e gitmeli | `huma paket run cekirdek` | bitti (7f29415) |
| B-047 | P1 | packages | Bağımlılıksız projede `huma kur` "Kurulacak bağımlılık yok" deyip `huma.lock` yazmıyordu; yönetilen çalıştırma kilit istediği ve "huma paket kur çalıştırın" dediği için `ilkle` dışı projeler hiç çalıştırılamıyordu | `huma kur` → `huma paket run` | bitti (82a145f) |
| B-048 | P3 | packages | Eksik manifest alanı serde'nin İngilizce, dosya bağlamsız iletisiyle raporlanıyor ("missing field `aciklama` at line 8 column 1"); Türkçe ve `huma.json` konumlu olmalı | eksik alanlı `huma.json` + `huma paket run` | açık |

## Keşif kaydı

Ajan derinlemesine taradığı her alanı buraya işler.

| Tarih | Alan | Yeni bulgu ID'leri | Not |
|---|---|---|---|
| 2026-10-23 | Genel yüzey taraması (ilk teşhis) | B-001…B-016 | Derin okuma yapılmadı; tüm crate'ler yeniden taranmalı |
| 2026-10-23 | CLI düşmanca girdi bataryası + yorumlayıcı/VM çapraz yürütme | B-018…B-029 | Boş dosya, BOM, geçersiz UTF-8, derin yuvalama, sıfıra bölme, büyük sayı, sonsuz döngü/özyineleme, döngüsel liste, kapalı yetenek, NFC denendi. NFC, sıfıra bölme, adım/özyineleme sınırları iki arka uçta da doğru |
| 2026-10-23 | .github/workflows (CI gerçekten neyi koruyor?) — kısmi | B-034…B-038 | Push sonrası `gh run list` ile CI durumu denetlendi; son 4 main çalışması kırmızıydı |
