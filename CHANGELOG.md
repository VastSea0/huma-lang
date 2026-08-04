# Değişim Günlüğü

## Yayımlanmamış — zemin mühendisliği

- Hüma'nın AI-öncelikli değil, Türkçe dilbilgisini koruyan genel amaçlı
  kütüphane zemini olduğu kanonik mühendislik anayasasıyla tanımlandı.
- Rust araç zinciri 1.94.1'e sabitlendi; CI bağımlılık değiştiren bütün Cargo
  işlemlerini `--locked` çalıştıracak biçimde sertleştirildi.
- Yapısal ayrılma/yönelme ekli aralık ayrıştırması düzeltildi ve bütün izlenen
  Hüma kaynaklarının ayrıştırılabilirliği yeniden yeşile getirildi.
- Eski web sitesi yaklaşan sıfırdan tasarım öncesinde kaynak ağacından çıkarıldı.
- Monolit; syntax, bytecode, runtime, VM, compiler, HMI, saf stdlib ve yetkili
  alan adaptörlerine ayrıldı. AI/tensor/BPE ile dosya/CSV/JSONL kodu çekirdekten
  fiziksel olarak çıkarıldı; adaptör değerleri genel `HostObject` sınırını kullanıyor.
- Kararlı `Gc` tutamaçları, genç/eskimiş nesiller, yazma bariyeri, minor/major
  çevrim toplama ve paylaşılmayan heap'li isolate yürütmesi eklendi.
- `DiagnosticEnvelope` kararlı hata kodu, konum, çağrı izi, güvenli ayrıntılar
  ve sınırlı neden zinciri taşıyacak biçimde sürümlendi.
- Yorumlayıcı normatif; VM ve AOT deneysel olarak sürümlü destek tablosuna
  bağlandı. Üretilmiş ve örnek havuzlu diferansiyel testler eklendi.
- HMI v1; imza/etki/hata sözleşmesi, sürüm pazarlığı, boyut sınırlı stdio
  çerçevesi, zaman aşımı ve fail-closed çocuk süreç yaşam döngüsüyle eklendi.
  Süreç içi FFI ayrıca açık güven bayrağı olmadan kaydedilmiyor.
- Paket API farkı SemVer'e bağlandı; paket ve dağıtım artefaktı Ed25519
  imzaları, checksum ve build-provenance üreten çoklu platform yayın hattı eklendi.
- Parser, HMI çerçevesi ve bytecode doğrulayıcı fuzz hedefleri; GC/isolate soak,
  paket rollback hata enjeksiyonu ve zamanlanmış dayanıklılık iş akışı eklendi.
- Adanmış runner performans kapısı; parser, başlangıç, fonksiyon/closure,
  liste/sözlük, Unicode, modül yükleme, GC, yorumlayıcı ve VM için medyan/p95/RSS izliyor.
- HTTP sunucularına açık kapatma, eşzamanlı sunucu sınırı ve adaptör görevleri
  düşerken iptal edilen yaşam döngüsü eklendi.
- CI ve yayın iş akışlarındaki üçüncü taraf GitHub Actions bağımlılıkları
  değişmez commit özetlerine sabitlendi.
- Bakımı bırakılmış `ttf-parser` zincirini taşıyan deneysel aynı-süreç GUI
  adaptörü varsayılan workspace ve CLI dağıtımından karantinaya alındı; kök
  kilit dosyası/güvenlik kapısı artık bu zinciri içermez.
- Paket kurulumu geçerli Ed25519 imzasını varsayılan zorunluluk yaptı. İmzasız
  yerel geliştirme istisnası yalnız açık `--güvenilir` onayıyla çalışıyor ve
  kilit dosyasına ayrı provenans olarak kaydediliyor; native kodu açmıyor.

## 0.6.0

### Dil çekirdeği

- Kanonik EBNF eklendi; tanımlayıcılar NFC’ye normalize ediliyor ve kesme
  işaretli eklerde ünlü uyumu, kaynaştırma, sert ünsüz benzeşmesi ile zincirli
  ekler doğrulanıyor.
- Kaynak konumları AST’den bytecode fonksiyonlarına kadar korunuyor; çalışma
  zamanı hataları kaynak konumlu çağrı izi taşıyor.
- Yorumlayıcı ve VM için ortak adım, çağrı, çıktı ve koleksiyon sınırları
  eklendi; panikler yapılandırılmış hataya çevriliyor.
- Ayrıştırıcı tanıları standart hata nesnelerine taşındı; beklenmeyen ifadeler ve geçersiz atama hedefleri artık sessizce `Boş` üretmiyor.
- Bilimsel sayı gösterimi eklendi; sonlu olmayan literal değerler reddediliyor.
- `devam` ve `kır` döngü komutları eklendi.
- `ve` ve `veya` kısa devreli çalışacak şekilde düzeltildi.
- Liste/sözlük öğesi ataması, sınır ve tip hatalarıyla güvenli hale getirildi.
- Tanımsız değişken, çağrılamayan değer, sıfıra bölme, geçersiz indeks ve modül hataları yapısal çalışma zamanı hatalarına dönüştürüldü.
- `dene/yakala`, yorumlayıcı hatalarını yakalayıp programa devam edebiliyor.

### VM ve AOT

- Bytecode VM gerçek çağrı frame’leri ve sözcüksel closure ortamları kullanıyor;
  yerleşiklerden yapılan bytecode geri çağrıları da aynı VM sınırları içinde
  eşzamanlı yürütülüyor.
- Bytecode kapsayıcısı fonksiyon tablosu ve kaynak konumları taşıyan v4’e
  yükseltildi; erişilebilir kontrol akışının tamamı başlamadan doğrulanıyor.
- AOT ara dosyaları benzersiz aynı-dizin hazırlama alanında üretiliyor; C
  derleme/bağlama süre sınırına, final çıktı atomik etkinleştirme ve geri alma
  davranışına sahip.
- VM’ye kalan, büyük-eşit, mantıksal işlemler, değil ve uzunluk işlemleri eklendi.
- VM fonksiyon kapsamı ve özyinelemeli Fibonacci sonucu düzeltildi.
- Bytecode derleyici desteklemediği komut ve atama hedeflerini açıkça reddediyor.
- `.hbc` dosyaları magic imzası, biçim sürümü, uzunluk, SHA-256 bütünlük özeti ve yapısal opcode doğrulaması içeren sınırlandırılmış bir kapsayıcıya taşındı.
- Yorumlayıcı/VM hata semantiğini korumayan yinelenen `huma gen` Rust çalışma zamanı kaldırıldı.
- AOT derleyici desteklenmeyen AST için sahte `0` çıktısı üretmek yerine derleme hatası veriyor.
- Yorumlayıcı ve VM doğruluk, aritmetik, metin karşılaştırması ve sözlük anahtarı kurallarını tek normatif semantik katmanından kullanıyor.
- Örtük metin-sayı dönüşümü kaldırıldı; sonlu olmayan aritmetik sonuçlar açık hata oldu.
- Fonksiyonlar tam argüman sayısı denetliyor ve anonim/iç içe fonksiyonlar sözcüksel kapsamlarını yakalıyor.
- JSON dönüşümü sonlu olmayan, döngüsel veya temsil edilemeyen değerleri sessiz varsayılanlarla değiştirmek yerine reddediyor.
- Bytecode doğrulaması kontrol akışı boyunca yığın etkilerini denetliyor; eksilme ve uyumsuz birleşme noktaları VM başlamadan reddediliyor.
- `dene/yakala` bytecode yürütmesinde hata yığını blok girişine geri sarılıyor.

### AI/NLP

- Yoğun katman eğitiminde kullanılan Adam matris/vektör durumları ve güncellemeleri çalışır hale getirildi.
- Bilimsel epsilon literal desteği ve ayrılmış sözcük çakışmaları düzeltildi.
- Sahte güncelleme yapan eski `optimizor.hb` ile yinelenen `katman.hb` kaldırıldı.
- `yapay_zeka.hb`, çalışan `sinir_agi` API’sinin tek giriş noktası oldu.
- TF-IDF → yoğun ağ → geri yayılım → çıkarım örneği uçtan uca doğrulandı.
- Bayt düzeyli BPE, Türkçe UTF-8 metin ve boşlukları kayıpsız kodlayıp geri çözecek şekilde yeniden yazıldı ve sınır kontrolleri eklendi.

### Kalite ve temizlik

- Dosya, ağ, süreç, FFI, veritabanı ve GUI erişimi varsayılan kapalı, ayrı CLI
  yeteneklerine taşındı.
- Kaynak/bytecode/LSP/REPL/test keşfi ve çıktıları boyut/derinlik sınırlarına;
  test ve dış süreçler gerçek sonlandırmalı zaman aşımına bağlandı.
- Yerel paket çözümleyici tam geçişli grafiği yazmadan önce çözüyor; SemVer,
  döngü, çakışma, kanonik içerik özeti ve kilit provenansı doğrulanıyor.
  Kurulum/kaldırma işlemsel ve geri alınabilir; uzak/native paketler güvenli
  protokol tanımlanana kadar kapalı.
- Paket betikleri kabuk yerine doğrudan program/argv olarak, operatör denetimi
  ve zaman aşımıyla çalıştırılıyor.
- FFI ad/yol/imza ve kitaplık sayısı sınırlandırıldı; örtük yeniden yükleme
  kaldırıldı ve `ffi_boşalt` eklendi.
- Otomatik türev grafiği çevrim/iş/öğe sınırları ile yinelenen geri yayılım ve
  hatada atomik gradyan güncellemesi bakımından sertleştirildi.
- Deterministik rastgele ayrıştırıcı dayanıklılık testi ve `cargo-fuzz`
  lexer/parser hedefi eklendi.
- İzlenen bütün `.hb` kaynaklarını ayrıştıran kaynak havuzu testi eklendi.
- Rust testleri çalışma zamanı hatalarını gerçekten başarısızlık sayacak şekilde sertleştirildi.
- Hüma test çerçevesinin sayaç ve açık dönüş davranışı düzeltildi.
- `cargo clippy --workspace --all-targets -- -D warnings` sıfır uyarıya indirildi.
- LSP konum dönüşümü UTF-16/UTF-8 ve emoji öncesi Türkçe tanımlayıcılar için düzeltildi.
- Dosya ve SQLite hataları sessiz boş/başarı değerleri yerine yakalanabilir çalışma zamanı hatalarına dönüştürüldü.
- Güvenli olmayan imzasız CLI güncellemesi ile eksik/kimliği doğrulanmayan uzak paket kurulumu kaldırıldı; yerel paket özeti bütün dosyaları kapsıyor.
- Web sitesi lint ve üretim derlemesi CI kabul kapısına eklendi.
- Eski debug betikleri, yedek kaynak, üretilmiş günlükler, bozuk testler ve geçersiz yayın iş akışları kaldırıldı.
- Belgeler ölçülmemiş performans ve eksiksiz Türkçe/native destek iddialarından arındırıldı.
- Bakımı sonlanan `bincode` kaldırıldı; `.hbc` v4 kapsayıcısı boyut sınırlı
  `serde_json` payload'una geçirildi.
- Rust bağımlılıkları bilinen RustSec uyarısı kalmayacak biçimde güncellendi;
  GUI tabanı `eframe` 0.35'e taşındı ve asgari Rust sürümü 1.92 olarak
  sabitlendi.
- Web araç zinciri güncel Next.js sürümüne taşındı; temiz `npm ci`, güvenlik
  denetimi, lint ve üretim derlemesi kabul kapısına alındı.
- Hüma yorumlayıcı, VM ve AOT'yi Python, Node.js, Ruby, C, Rust, Swift ve Java
  ile aynı çıktıyı doğrulayan dört iş yükünde ölçen tekrarlanabilir benchmark
  paketi ve ham Apple M4 sonuçları eklendi.
