# Hüma Durum ve Yol Haritası

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
- Kaynak, çıktı, koleksiyon, dosya, ağ, süreç, SQL, tensor/matris ve benzeri
  işlemlerde deterministik boyut/süre/iş sınırları
- Yerleşik API’lerde katı argüman, boyut, sonluluk ve UTF-8 sözleşmeleri
- Çevrim güvenli otomatik türev grafiği; hatada kısmi gradyan yazmayan ve
  yinelenen geri yayılımda eski ara gradyan biriktirmeyen işlemsel güncelleme
- Tam bağımlılık grafiğini önceden çözen, SemVer/kilit/özet doğrulayan ve
  kurulum-kaldırmada geri alınabilir paket işlemleri
- Kaynak/bytecode/AOT çıktılarında aynı dizinde hazırlama, `sync` ve atomik
  etkinleştirme; başarısızlıkta eski çıktıyı geri yükleme
- İzlenen Hüma kaynak havuzu, yorumlayıcı/VM diferansiyel regresyonları,
  deterministik rastgele ayrıştırıcı dayanıklılık testi ve libFuzzer hedefi
- RustSec ve npm bağımlılık denetimleri; Node 22 web lint/üretim derlemesi ve
  Linux, macOS, Windows derleme matrisi

“Doğrulanmış”, kapsanan davranışın testle tekrar üretilebildiği ve hatanın
makul görünen sahte sonuca çevrilmediği anlamındadır. “Bütün programlar
hatasızdır” veya “işletim sistemi sandbox’ı vardır” anlamına gelmez.

## Bilinen ve açıkça sınırlandırılmış alanlar

- Cranelift AOT yalnız sürümlenmemiş deneysel sayısal alt kümeyi kapsar; tam
  Hüma değer modeli değildir.
- Dil dinamiktir. Statik tip denetleyici, etkiler sistemi ve derleme zamanlı
  modül arayüzleri henüz yoktur.
- Yetenek modeli en az ayrıcalığı sağlar ancak işletim sistemi yalıtımı değildir.
  İzin verilen süreç/FFI işlemi ev sahibi süreç haklarıyla çalışır; yanlış FFI
  ABI’si süreci çökertebilir.
- Uzak paket kaydı, yayıncı imzası ve şeffaf provenans günlüğü yoktur. Güvenli
  protokol tanımlanana kadar uzak ve native paket kurulumu kapalıdır.
- Yazımdan telaffuzu çıkarılamayan kısaltma/sayılı adlarda tam Türkçe ek uyumu
  sözlük olmadan kanıtlanamaz; bu durum dil tanımında normatif istisnadır.
- LSP ayrıştırma, tanı, tamamlama, hover ve tanıma gitme düzeyindedir; güvenli
  yeniden adlandırma ve bütün referansları bulma henüz yoktur.
- Bytecode opcode’ları, paket şeması ve yerleşik kamu API’si 1.0 öncesinde
  geriye uyumluluk garantisi taşımaz; uyumsuz dosya biçimleri sürümle reddedilir.
- Ölçülmüş performans sonucu olmayan başka dillerle hız eşitliği veya üstünlüğü
  iddia edilmez.

## Sonraki mühendislik aşamaları

### 0.7 — Derleme zamanı sözleşmeleri

- İsteğe bağlı statik tip/etki denetimi ve sürümlü modül arayüz dosyası
- LSP referans bulma, kapsam güvenli yeniden adlandırma ve kod eylemleri
- Kamu yerleşik API kataloğunun makinece okunabilir imza/etki şeması
- Linux, macOS ve Windows için açık destek tablosu ve AOT C araç zinciri seçimi

### 0.8 — Ölçülebilir yürütme

- Tam dil için sürekli yorumlayıcı/VM diferansiyel üretim testi
- Uzun süreli fuzz, hata enjeksiyonu ve bellek profili iş akışları
- Tekrarlanabilir benchmark veri kümesi; süre, tepe bellek ve çıktı boyutu raporu
- Tam değer modeli, hata ABI’si ve çöp toplama stratejisi tanımlanmış AOT arka ucu

### 0.9 — Dağıtım ve ekosistem

- İmzalı kayıt indeksi, içerik adresli paket arşivi ve yayıncı anahtar politikası
- Yeniden üretilebilir paket derlemesi ve SBOM/provenans doğrulaması
- Native eklentiler için süreç dışı yalıtım veya sürümlü, dar ve doğrulanabilir ABI

### 1.0 kabul ölçütü

1. Normatif dil tanımındaki desteklenen yapılar yorumlayıcı ve VM’de eş sonuç
   veya eş yapılandırılmış hata üretmeli.
2. Her arka uç ve platformun destek kapsamı sürümlü, otomatik testli sözleşme
   olmalı.
3. Fuzz, diferansiyel, güvenlik, hata enjeksiyonu ve uzun süreli testler CI’da
   düzenli çalışmalı.
4. Kamu API’si, bytecode ve paket biçimi geriye uyumluluk politikasıyla sabitlenmeli.
5. Sürümle birlikte yeniden üretilebilir performans ve tepe bellek raporu yayımlanmalı.
