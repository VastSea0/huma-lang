# Hüma 0.6 Paket Güvenliği

Hüma 0.6 paket yöneticisi yalnız yerel kaynak ağacındaki paketleri çözer. Uzak
URL, Git deposu veya kayıt kurulumu; imzalı kayıt/provenans protokolü olmadığı
için açıkça reddedilir.

## Çözümleme ve kilit

- Paket ve proje metadata’sı bilinmeyen alanları reddeden katı JSON şemasıyla
  okunur.
- Paket adı NFC, yol güvenliği ve uzunluk kurallarından geçer.
- Bütün geçişli bağımlılık grafiği yazma başlamadan çözülür.
- Döngüler, sürüm çakışmaları ve karşılanmayan SemVer aralıkları hata verir.
- `huma.lock`, her paket için kesin sürüm, kanonik SHA-256 içerik özeti ve
  `yerel` kaynak bilgisini saklar.
- Özet metadata ile normal dosyaların tamamını, platformdan ve JSON harita
  ekleme sırasından bağımsız biçimde kapsar.

## İşlemsel kurulum ve kaldırma

Paketler önce ayrı hazırlama dizinlerine kopyalanır ve doğrulanır. Bütün grafik
hazır olmadan kurulu ağaç, `huma.json` veya `huma.lock` değiştirilmez. Etkinleştirme
sırasında mevcut dizinler yedeklenir; herhangi bir hata bütün paket dizinlerini,
proje metadata’sını ve kilidi eski hâline döndürür. Kaldırma da bağımlı paketleri
denetler ve aynı geri alma modelini kullanır.

## Kaynak sınırları

- Metadata: 1 MiB
- Paket başına normal dosya: 10.000
- Tek dosya: 64 MiB
- Toplam paket verisi: 256 MiB
- Bağımlılık veya betik sayısı: 1.024
- Dizin derinliği: 64

Sembolik bağlantılar ve özel aygıt/soket türleri kabul edilmez. Giriş dosyası
paket kökünün içinde kalan bir `.hb` dosyası olmalıdır.

## Betikler ve native kod

Paket betikleri bir kabukta yorumlanmaz. Komut, tırnak kurallarıyla program ve
argümanlara ayrılır; kabuk yönlendirme/zincirleme operatörleri reddedilir ve
süreç doğrudan en fazla 300 saniye çalıştırılır.

Hüma 0.6 sürümlü ve doğrulanabilir bir native paket ABI’si tanımlamaz.
Metadata’daki Rust/crate/native alanları bu nedenle `--güvenilir` verilse bile
reddedilir. Bu uyumluluk bayrağı güvenlik denetimini kapatmaz.
