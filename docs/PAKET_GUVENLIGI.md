# Hüma 0.6 Paket Güvenliği

Hüma 0.6 paket yöneticisi yalnız yerel kaynak ağacındaki paketleri çözer. Uzak
URL, Git deposu veya kayıt kurulumu; güvenilir kayıt indeksi ve şeffaflık
protokolü tamamlanmadığı için açıkça reddedilir. Bu kapalı davranış doğrulama
atlama seçeneği sunmaktan daha güvenlidir.

## Çözümleme ve kilit

- Paket ve proje metadata’sı bilinmeyen alanları reddeden katı JSON şemasıyla
  okunur.
- Paket adı NFC, yol güvenliği ve uzunluk kurallarından geçer.
- Bütün geçişli bağımlılık grafiği yazma başlamadan çözülür.
- Döngüler, sürüm çakışmaları ve karşılanmayan SemVer aralıkları hata verir.
- `huma.lock`, her paket için kesin sürüm, kanonik SHA-256 içerik özeti ve
  kaynak/imza bilgisini saklar.
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

## İmzalar

`huma paket imzala --anahtar <dosya>` kanonik paket kimliği, SemVer'i ve
SHA-256 özetini alan ayrımlı Ed25519 mesajına bağlar. Anahtar dosyası 32 baytlık
hex tohumdur; sembolik bağ olamaz ve Unix'te `0600`'den geniş izin taşıyamaz.
Doğrulama zayıf açık anahtarları ve anahtar kimliği/imza/özet uyuşmazlığını
reddeder. `huma.sig` kendi imzaladığı özetin dışında bırakılır.

`huma paket kur` varsayılan olarak her pakette geçerli Ed25519 imzası ister.
İmzasız paket yalnız kullanıcının kaynak ağacını bizzat denetlediği yerel
geliştirme için `--güvenilir` ile kurulabilir; bu istisna
`yerel-imzasız-güvenilir` provenansı olarak kilide yazılır ve süreç içi native
kodu etkinleştirmez. Böylece sessiz bir imza atlama yolu yoktur.

Sürüm ikilileri için `huma artifact sign/verify` ayrı alan ayrımlı dağıtım
zarfını kullanır. Etiket sürüm hattı üç platform ikilisi, bağımlılık envanteri,
`SHA256SUMS`, Ed25519 zarfı ve GitHub build provenance üretir; anahtar sırrı yoksa
imzasız sürüm yayımlanmaz.

## Betikler ve native kod

Paket betikleri bir kabukta yorumlanmaz. Komut, tırnak kurallarıyla program ve
argümanlara ayrılır; kabuk yönlendirme/zincirleme operatörleri reddedilir ve
süreç doğrudan en fazla 300 saniye çalıştırılır.

Metadata’daki eski Rust/crate/native alanları `--güvenilir` verilse bile
reddedilir. Native paketler süreç dışı [HMI v1](HMI.md) sözleşmesi kullanır;
çalıştırılabilir yol paket içinde kalmalı, normal dosya olmalı ve Unix'te
çalıştırma izni taşımalıdır. Bu sınır süreç çökmesini ayırır ancak işletim
sistemi sandbox'ı değildir.
