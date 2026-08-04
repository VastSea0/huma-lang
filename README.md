# Hüma Programlama Dili

Hüma, Türkçe dilbilgisini koruyan genel amaçlı modern bir programlama dili
zemini olarak geliştirilmektedir. Projenin bugünkü odağı AI veya başka bir alan
kütüphanesi üretmek değil; ileride binlerce güvenilir kütüphaneyi taşıyabilecek
doğru, hızlı, güvenli ve sürümlenebilir dil/runtime mimarisini kurmaktır.

Depodaki mevcut AI, NLP, GUI, ağ ve veri kütüphaneleri deneysel doğrulama
malzemesidir. Kamu API'si sayılmaz ve çekirdek tasarımı kısıtlamaz. Ürün yönü,
değişmezler ve kabul kapıları [Hüma Mühendislik
Anayasası](docs/MUHENDISLIK_ANAYASASI.md) belgesinde tanımlıdır.

[English README](README.en.md)

## Doğrulanmış durum

| Bileşen | Durum | Doğrulanan kapsam |
|---|---|---|
| Yorumlayıcı | Kanonik yürütme yolu | Fonksiyon, özyineleme, sınıf, liste/sözlük, modül, döngü, `dene/yakala`, yerleşik kütüphaneler |
| Bytecode VM | Doğrulanmış alt küme | Bağımsız frame/closure’lar, fonksiyonlar, koleksiyonlar ve kontrol akışı; desteklenmeyen AST açık derleme hatası verir |
| Cranelift AOT | Deneysel sayısal alt küme | Sayısal ifadeler ve desteklenen kontrol akışı; metin, modül, sınıf ve benzeri yapılar sessiz sonuç üretmek yerine reddedilir |
| LSP | Temel araç desteği | Ayrıştırıcı tanıları, tamamlama, hover ve tanıma gitme |
| HMI | Sürümlü süreç dışı sınır | İmza/etki/hata kataloğu, API uyumluluk denetimi, çerçeve sınırları ve zaman aşımında süreç sonlandırma |
| Heap/isolate | Nesilsel çevrim toplayıcı | Kararlı `Gc` tutamaçları, genç nesil + yazma bariyeri, major çevrim toplama ve paylaşılmayan isolate heap'leri |
| Alan kütüphaneleri | Deneysel / kararsız | Çekirdek kabulünün parçası değildir; yeniden yazılabilir veya kaldırılabilir |

“Doğrulanmış”, desteklenen kapsamda hataların sessizce yutulmaması ve regresyon
testlerinin geçmesi anlamındadır. Hüma henüz statik tip sistemi, eksiksiz AOT
arka ucu, işletim sistemi sandbox’ı veya üretim ölçeği performans garantisi
sunmaz. Güncel ve ölçülü sınırlar [Durum ve Yol
Haritası](docs/DURUM_VE_YOL_HARITASI.md) belgesindedir.

## Kurulum

Araç zinciri `rust-toolchain.toml` ile Rust 1.94.1'e sabitlenmiştir:

```bash
git clone https://github.com/VastSea0/huma-lang.git
cd huma-lang
cargo build --release --locked
./target/release/huma --version
```

Geliştirici kabul denetimi:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo audit
cargo run --locked -p huma-cli -- test tests
```

Eski web sitesi yaklaşan sıfırdan tasarım öncesinde kaldırılmıştır; kaynak depo
şu anda web dağıtımı üretmez.

## İlk program

Her Hüma programı paket yöneticisiyle oluşturulur ve `huma.json` içindeki bir
paket betiği üzerinden çalıştırılır. Gevşek `.hb` dosyalarının doğrudan
yürütülmesi kapalıdır:

```bash
huma paket yeni fibonacci_uygulamasi
cd fibonacci_uygulamasi
```

Paket yöneticisinin oluşturduğu giriş dosyasını aşağıdaki içerikle düzenleyin:

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
huma paket run baslat
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
# huma.json içindeki örnek betik:
# "veri": "huma run uygulama.hb --izin dosya-okuma --izin ağ-istemci"
huma paket run veri
```

Dosya yazma, ağ sunucusu, süreç, HMI/FFI, veritabanı ve GUI ayrı izinlerdir.
`--tüm-izinler` yalnız güvenilen kod içindir ve işletim sistemi sandbox’ı
sağlamaz. Kaynak/bytecode/çıktı boyutları ile uzun süren süreç ve testler
sınırlandırılır.

Native kütüphanelerin varsayılan yolu süreç dışı [HMI](docs/HMI.md)'dır.
Süreç içi dar FFI ancak ayrıca `--güvenilir-süreç-içi-ffi` bayrağıyla açılır.

## Kütüphane politikası

AI dâhil alan kütüphaneleri ancak çekirdek/runtime sözleşmeleri tamamlandıktan
sonra kararlı yüzeye alınacaktır. Dil çekirdeği GUI, ağ, SQL, tensor veya belirli
bir uygulama alanını doğrudan bilmeyecek; bunlar ayrı yetenekli adaptör ve paket
katmanlarında yaşayacaktır.

Çalışma alanındaki fiziksel sınırlar: `huma-syntax`, `huma-bytecode`,
`huma-runtime`, `huma-vm`, `huma-compiler` ve `huma-hmi` zemin katmanlarıdır;
`huma-stdlib-file`, `huma-stdlib-net`, `huma-stdlib-process`,
`huma-stdlib-sqlite`, `huma-stdlib-native` ve `huma-stdlib-ai` yalnız
adaptördür. Eski aynı-süreç GUI adaptörü, bakımı bırakılmış bir font ayrıştırıcı
zinciri nedeniyle varsayılan workspace/CLI dağıtımından karantinaya alınmıştır.
`huma-core` geriye dönük uyumluluk şemsiyesidir; bu ayrımı ortadan kaldıran bir
çekirdek bağımlılığı değildir.

## Belgeler

- [Dil Tanımı](docs/DIL_TANIMI.md)
- [Kanonik EBNF](docs/DIL_GRAMERI.ebnf)
- [Bytecode Biçimi](docs/BYTECODE_BICIMI.md)
- [Kütüphaneler](KUTUPHANELER.md)
- [Paket Güvenliği](docs/PAKET_GUVENLIGI.md)
- [HMI v1](docs/HMI.md)
- [API ve Uyumluluk Politikası](docs/API_STABILITE_POLITIKASI.md)
- [Performans ve Bellek Ölçümü](docs/PERFORMANS.md)
- [Diller Arası Benchmark](docs/KARSILASTIRMALI_BENCHMARK.md)
- [Durum ve Yol Haritası](docs/DURUM_VE_YOL_HARITASI.md)
- [Mühendislik Anayasası](docs/MUHENDISLIK_ANAYASASI.md)
- [Değişim Günlüğü](CHANGELOG.md)

## Lisans

MIT
