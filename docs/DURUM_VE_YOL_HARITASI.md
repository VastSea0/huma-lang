# Hüma Durum ve Yol Haritası

## Proje yönü

Hüma'nın önceliği AI/NLP veya başka bir alan kütüphanesi değil, farklı türde
gerçek kütüphanelerin güvenle kurulabileceği genel amaçlı dil zeminidir. Mevcut
alan kütüphaneleri yeniden yazılabilir deneysel girdilerdir. Türkçe dilbilgisi
ürün kimliği olarak korunur; diğer mimari kararlar doğruluk, güvenlik ve
performans için değiştirilebilir.

Kanonik ilkeler ve ayrıntılı kabul kapıları [Hüma Mühendislik
Anayasası](MUHENDISLIK_ANAYASASI.md) belgesindedir.

## 0.6.0 doğrulanmış çekirdek sınırı

Otomatik test ve kod kabul kapılarıyla korunan zemin:

- Kaynak konumlu lexer, ayrıştırıcı tanıları, çalışma zamanı stack trace’leri
  ve yakalanabilir yapısal hatalar
- NFC tanımlayıcılar, katı metin kaçışları, tanımlı Türkçe ek kümesi; yazımdan
  çıkarılabilen köklerde ünlü uyumu, kaynaştırma ve ünsüz benzeşmesi
- Paylaşılan normatif doğruluk, sonlu `f64`, aritmetik, karşılaştırma, eşitlik
  ve döngü güvenli koleksiyon semantiği
- Tam fonksiyon argüman sayısı, sözcüksel closure, özyineleme sınırı ve
  yorumlayıcı/VM hata eşliği
- AST’ye geri dönmeyen bytecode fonksiyonları, bağımsız VM frame/closure
  yığınları ve kontrol akışı/yığın doğrulaması
- Sürümlü, boyut sınırlı, SHA-256 bütünlük denetimli `.hbc` v4 kapsayıcısı
- Kanonik modül kimliği, döngü algılama, başarısız yüklemede geri alma, açık
  dışa aktarımlar ve takma adlı canlı modül ad alanları
- Varsayılan kapalı dosya, ağ, süreç, FFI, veritabanı ve GUI yetenekleri
- AI ve dosya/CSV/JSONL işlevlerinin çekirdekten fiziksel olarak ayrılmış,
  isteğe bağlı alan adaptörlerinde tutulması
- Bakımı bırakılmış transitive font ayrıştırıcısı taşıyan eski aynı-süreç GUI
  prototipinin varsayılan workspace/CLI ve güvenlik kapısından karantinaya alınması
- Kararlı `Gc` tutamaçlı iki nesilli çevrim toplama, yazma bariyeri ve ayrı
  iş parçacıklarında paylaşılmayan heap ile çalışan isolate'lar
- Sürümlü `DiagnosticEnvelope`: kararlı kod, konum, çağrı izi, güvenli
  ayrıntılar ve neden zinciri
- HMI v1 süreç sınırı: makinece okunabilir imza/etki/hata sözleşmesi, boyutlu
  çerçeveler, protokol pazarlığı, zaman aşımı ve fail-closed süreç sonlandırma
- Kaynak, çıktı, koleksiyon, dosya, ağ, süreç, SQL, tensor/matris ve benzeri
  işlemlerde deterministik boyut/süre/iş sınırları
- Yerleşik API’lerde katı argüman, boyut, sonluluk ve UTF-8 sözleşmeleri
- Çevrim güvenli otomatik türev grafiği; hatada kısmi gradyan yazmayan ve
  yinelenen geri yayılımda eski ara gradyan biriktirmeyen işlemsel güncelleme
- Tam bağımlılık grafiğini önceden çözen, SemVer/kilit/özet doğrulayan ve
  kurulum-kaldırmada geri alınabilir paket işlemleri
- Ed25519 imzalı paket ve dağıtım manifestleri, imzasız sürümü reddeden çoklu
  platform yayın hattı ve build provenance üretimi
- Kaynak/bytecode/AOT çıktılarında aynı dizinde hazırlama, `sync` ve atomik
  etkinleştirme; başarısızlıkta eski çıktıyı geri yükleme
- İzlenen Hüma kaynak havuzu, üretilmiş yorumlayıcı/VM diferansiyel
  regresyonları ve parser, HMI çerçevesi, bytecode doğrulayıcı libFuzzer hedefleri
- Adanmış runner'da medyan/p95/RSS gerileme kapısı; başlangıç, parser,
  fonksiyon/closure, liste/sözlük, Unicode, modül, GC, yorumlayıcı ve VM ölçümleri
- Zamanlanmış çevrim/çoklu-isolate soak ve protokol/paket hata enjeksiyonu
- RustSec bağımlılık denetimi ve Linux, macOS, Windows derleme matrisi

“Doğrulanmış”, kapsanan davranışın testle tekrar üretilebildiği ve hatanın
makul görünen sahte sonuca çevrilmediği anlamındadır. “Bütün programlar
hatasızdır” veya “işletim sistemi sandbox’ı vardır” anlamına gelmez.

## Bilinen ve açıkça sınırlandırılmış alanlar

- Cranelift AOT yalnız sürümlenmemiş deneysel sayısal alt kümeyi kapsar; tam
  Hüma değer modeli değildir.
- Dil dinamiktir. Statik tip denetleyici ve dil içi etkiler sistemi henüz
  yoktur; dış modül imza/etki sözleşmesi HMI'da bulunur.
- Yetenek modeli en az ayrıcalığı sağlar ancak işletim sistemi yalıtımı değildir.
  HMI çocuk süreci kullanıcı haklarıyla çalışır. Yalnız açık güven bayrağıyla
  açılan süreç içi FFI'da yanlış ABI ev sahibi süreci çökertebilir.
- Yerel paket ve sürüm artefaktı imzaları vardır; uzak kayıt, anahtar
  rotasyonu/iptali ve şeffaflık günlüğü henüz yoktur. Bu politika tamamlanana
  kadar uzak paket kurulumu kapalıdır.
- Yazımdan telaffuzu çıkarılamayan kısaltma/sayılı adlarda tam Türkçe ek uyumu
  sözlük olmadan kanıtlanamaz; bu durum dil tanımında normatif istisnadır.
- LSP ayrıştırma, tanı, tamamlama, hover ve tanıma gitme düzeyindedir; güvenli
  yeniden adlandırma ve bütün referansları bulma henüz yoktur.
- Bytecode opcode’ları, paket şeması ve yerleşik kamu API’si 1.0 öncesinde
  geriye uyumluluk garantisi taşımaz; uyumsuz dosya biçimleri sürümle reddedilir.
- Diller arası mikro benchmark yalnız tanımlı iş yükleri ve kayıtlı ortam için
  geçerlidir; başka uygulama alanlarına hız eşitliği veya üstünlüğü olarak
  genellenmez.

## Mühendislik aşamalarının durumu

### 0.7 zemini — uygulandı

- Sabit Rust araç zinciri, kilitli bağımlılıklar ve kabul iş akışları
- `syntax`, `bytecode`, `runtime`, `vm`, `compiler`, HMI ve alan adaptörlerinin
  fiziksel ayrımı
- Yorumlayıcının tek normatif backend olduğu sürümlü destek tablosu
- Makinece okunabilir HMI imza/etki/hata kataloğu ve SemVer API denetimi

### 0.8 zemini — uygulandı, tarih biriktiriyor

- İki nesilli çevrim toplayıcı, isolate tabanlı eşzamanlılık ve soak testleri
- VM'nin ilan edilen alt kümesinde sürekli diferansiyel üretim testi
- Fuzz, hata enjeksiyonu, soak ve adanmış performans iş akışları
- Sürümlü yapılandırılmış hata ABI'si

Eksik olan tek seferlik mekanizma değil, farklı donanım/sürümler üzerinde uzun
süreli performans ve bellek geçmişidir; sonuçlar biriktikçe eşikler sıkılaşır.

### 0.9 zemini — kısmen uygulandı

- Uygulandı: paket/artefakt Ed25519 imzası, checksum, build provenance, HMI ile
  varsayılan süreç dışı native sınırı, imzayı varsayılan zorunlu doğrulayan
  installer ve imzasız yayını reddeden yayın hattı.
- Kalan: imzalı kayıt indeksi, içerik adresli uzak arşiv, yayıncı anahtar
  rotasyonu/iptali ve şeffaflık günlüğü. Bunlar tamamlanana kadar uzak kurulum kapalıdır.

### 1.0 kabul ölçütü

1. Normatif dil tanımındaki desteklenen yapılar yorumlayıcı ve VM’de eş sonuç
   veya eş yapılandırılmış hata üretmeli.
2. Her arka uç ve platformun destek kapsamı sürümlü, otomatik testli sözleşme
   olmalı.
3. Fuzz, diferansiyel, güvenlik, hata enjeksiyonu ve uzun süreli testler CI’da
   düzenli çalışmalı.
4. Kamu API’si, bytecode ve paket biçimi geriye uyumluluk politikasıyla sabitlenmeli.
5. Sürümle birlikte yeniden üretilebilir performans ve tepe bellek raporu yayımlanmalı.
