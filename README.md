# Hüma Programlama Dili

Hüma, Rust ile geliştirilen ve Türkçe anahtar sözcükler kullanan deneysel bir
genel amaçlı programlama dilidir. 0.6.0 sürümü; yorumlayıcı/VM doğruluğu,
yapısal hata yönetimi, Türkçe yüzey grameri, en az ayrıcalıklı dış dünya
erişimi ve farklı türde kütüphaneler için ilk doğrulanmış çekirdek sınırını
tanımlar.

[English README](README.en.md)

## Doğrulanmış durum

| Bileşen | Durum | Doğrulanan kapsam |
|---|---|---|
| Yorumlayıcı | Kanonik yürütme yolu | Fonksiyon, özyineleme, sınıf, liste/sözlük, modül, döngü, `dene/yakala`, yerleşik kütüphaneler |
| Bytecode VM | Doğrulanmış alt küme | Bağımsız frame/closure’lar, fonksiyonlar, koleksiyonlar ve kontrol akışı; desteklenmeyen AST açık derleme hatası verir |
| Cranelift AOT | Deneysel sayısal alt küme | Sayısal ifadeler ve desteklenen kontrol akışı; metin, modül, sınıf ve benzeri yapılar sessiz sonuç üretmek yerine reddedilir |
| LSP | Temel araç desteği | Ayrıştırıcı tanıları, tamamlama, hover ve tanıma gitme |
| AI/NLP | Çalışır CPU prototipi | Yoğun katmanlar, geri yayılım, Adam, gradyan kırpma, TF-IDF ve sözcük gömme |

“Doğrulanmış”, desteklenen kapsamda hataların sessizce yutulmaması ve regresyon
testlerinin geçmesi anlamındadır. Hüma henüz statik tip sistemi, eksiksiz AOT
arka ucu, işletim sistemi sandbox’ı veya üretim ölçeği performans garantisi
sunmaz. Güncel ve ölçülü sınırlar [Durum ve Yol
Haritası](docs/DURUM_VE_YOL_HARITASI.md) belgesindedir.

## Kurulum

Rust 1.92 veya daha yeni bir stable araç zinciri gereklidir:

```bash
git clone https://github.com/VastSea0/huma-lang.git
cd huma-lang
cargo build --release
./target/release/huma --version
```

Geliştirici kabul denetimi:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo audit
cargo run -p huma-cli -- test tests
cargo run -p huma-cli -- test examples
cd www/site && npm ci && npm audit && npm run lint && npm run build
```

Web kabul kapısı Node.js 22 kullanır.

## İlk program

```huma
fibonacci fonksiyon olsun n alsın {
    n <= 1 ise { n'i döndür }
    (fibonacci(n - 1) + fibonacci(n - 2))'yi döndür
}

i = 0'dan 9'a kadar {
    "fib(" + i + ") = " + fibonacci(i)'yi yazdır
}
```

```bash
huma run examples/fibonacci.hb
huma run examples/fibonacci.hb --vm
```

Yorumlayıcı tam dil için varsayılan yürütme yoludur. VM veya AOT bir yapıyı desteklemiyorsa derleme başarısız olur; başka bir değer üreterek devam etmez.

## Kanonik sözdizimi

```huma
ad = "Hüma" olsun
sayılar = [1, 2, 3] olsun

topla fonksiyon olsun a, b alsın {
    (a + b)'yi döndür
}

sonuç = topla(4, 5) olsun
sonuç'u yazdır

sonuç > 5 ise {
    "büyük"'ü yazdır
} yoksa {
    "küçük veya eşit"'i yazdır
}

dene {
    olmayan_değişken'i yazdır
} yakala hata {
    "Yakalandı: " + hata'yı yazdır
}
```

Kesme işaretinden sonraki durum ekleri kod içinde tanımlayıcı ayıracı olarak kullanılır. Lexer yalnızca tanımlı ekleri kabul eder; bilinmeyen ekler sözdizimi hatasıdır. Bu sistem Türkçeden esinlenen bir programlama dili grameridir, genel amaçlı doğal dil veya eksiksiz Türkçe biçimbilim çözümleyicisi değildir. Kesin kurallar için [Dil Tanımı](docs/DIL_TANIMI.md) belgesine bakın.

Yazımdan çıkarılabilen adlarda lexer ünlü uyumu, kaynaştırma ve sert ünsüz
benzeşmesini de doğrular. Kanonik EBNF için [Dil
Grameri](docs/DIL_GRAMERI.ebnf) dosyasına bakın.

## Güvenlik sınırı

Dış dünya yetenekleri varsayılan olarak kapalıdır:

```bash
huma run uygulama.hb --izin dosya-okuma --izin ağ-istemci
```

Dosya yazma, ağ sunucusu, süreç, FFI, veritabanı ve GUI ayrı izinlerdir.
`--tüm-izinler` yalnız güvenilen kod içindir ve işletim sistemi sandbox’ı
sağlamaz. Kaynak/bytecode/çıktı boyutları ile uzun süren süreç ve testler
sınırlandırılır.

## AI örneği

`examples/nlp_siniflandirma.hb`, TF-IDF özellikleri üzerinde yoğun bir sinir ağını gerçek geri yayılım ve Adam güncellemesiyle eğitir:

```bash
huma run examples/nlp_siniflandirma.hb
```

Bu çekirdek eğitim/çıkarım deneyi için uygundur. Büyük model eğitimi için henüz GPU, otomatik karma hassasiyet, dağıtık çalışma, tensor aygıt yönetimi ve endüstriyel veri yükleyici altyapısı yoktur.

## Belgeler

- [Dil Tanımı](docs/DIL_TANIMI.md)
- [Kanonik EBNF](docs/DIL_GRAMERI.ebnf)
- [Bytecode Biçimi](docs/BYTECODE_BICIMI.md)
- [Kütüphaneler](KUTUPHANELER.md)
- [Paket Güvenliği](docs/PAKET_GUVENLIGI.md)
- [Performans ve Bellek Ölçümü](docs/PERFORMANS.md)
- [Diller Arası Benchmark](docs/KARSILASTIRMALI_BENCHMARK.md)
- [Durum ve Yol Haritası](docs/DURUM_VE_YOL_HARITASI.md)
- [Değişim Günlüğü](CHANGELOG.md)

## Lisans

MIT
