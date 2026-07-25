# Hüma Durum ve Yol Haritası

## 0.6.0 kabul sınırı

Tamamlanan ve otomatik testle korunan işler:

- Yorumlayıcıda yapısal sözdizimi/çalışma zamanı hataları
- Özyineleme ve yerel kapsam doğruluğu
- Liste/sözlük erişimi ve atama sınır kontrolleri
- Kısa devreli mantık, `devam`, `kır`, `dene/yakala`
- Bytecode alt kümesinde kontrollü derleme ve yorumlayıcıyla Fibonacci eşliği
- Sürümlü, boyut sınırlı ve bütünlük özeti doğrulanan `.hbc` kapsayıcısı
- AOT’de desteklenmeyen sözdiziminin açıkça reddedilmesi
- İzlenen Hüma kaynaklarının tamamı için ayrıştırma testi
- Hüma ve Rust test süitleri, biçim ve sıfır-uyarı Clippy kapısı
- CPU üzerinde gerçek yoğun katman eğitimi, geri yayılım ve Adam güncellemesi

## Bilinen sınırlar

- VM fonksiyon gövdeleri, kapsam doğruluğu için halen yorumlayıcı semantiğinden yararlanır; bağımsız bir frame/closure VM’si değildir.
- AOT yalnızca sayısal alt kümeyi kapsar ve bütün Hüma değer modelini taşımaz.
- Statik tip denetleyici ve modül arayüz sistemi yoktur.
- Doğal Türkçe biçimbilim/ünlü uyumu otomatik doğrulanmaz.
- AI çalışma zamanı CPU/f64 odaklıdır; GPU, karma hassasiyet, aygıt grafiği ve dağıtık eğitim yoktur.
- Güvenilmeyen Hüma kodunu işletim sistemi düzeyinde yalıtan sandbox yoktur. Dosya, ağ, FFI ve sistem işlevleri yetkili süreç haklarıyla çalışır.
- İmzalı ikili yayın kanalı olmadığı için güvenli olmayan CLI kendini güncelleme yolu kaldırılmıştır; kurulum kaynaktan derlenir.
- Uzak paket kaydı imzalı ve çok dosyalı bir aktarım biçimine sahip değildir; 0.6.0 paket yöneticisi yalnızca kaynak ağacındaki yerel paketleri kurar.
- Performans karşılaştırması yayımlanmadığından Python, JavaScript, Rust veya başka bir dille hız eşitliği iddia edilmez.

## Sonraki aşamalar

### 0.7 — Dil sözleşmesi

- Ayrı tanım/atama semantiği ve açık modül dışa aktarımları
- Kaynak konumunu koruyan AST ve bütün çalışma zamanı hatalarında stack trace
- Özellik tabanlı/fuzz lexer-parser testleri
- İmzalı, commit/sürüm sabitlemeli ve çok dosyalı paket kayıt protokolü
- Türkçe yüzey biçimleri için isteğe bağlı ünlü uyumu linter’ı
- LSP’de sembol tablosu tabanlı yeniden adlandırma ve referans bulma

### 0.8 — Yürütme arka uçları

- Gerçek VM çağrı frame’leri, closure ve modül kapsamları
- Yorumlayıcı/VM diferansiyel test havuzu
- AOT değer gösterimi, metin/liste/sözlük ve hata ABI’si
- Tekrarlanabilir benchmark paketi ve bellek profilleri

### 0.9 — AI çalışma zamanı

- Tensor şekil/tip doğrulaması
- Toplu geri yayılım ve veri yükleyici API’si
- Kalıcı model dosyası, özel token politikası ve üretim ön-işleme kuralları olan tokenizer
- Aygıt soyutlaması; yalnızca doğrulanmış bir backend bulunduğunda GPU desteği
- Model biçimi sürümleme ve yükleme sırasında şema doğrulaması

### 1.0 ölçütü

1. Dil tanımındaki bütün yapılar yorumlayıcı ve VM’de eş sonuç üretmeli.
2. Desteklenen AOT kapsamı ayrı ve sürümlenmiş bir sözleşmeye sahip olmalı.
3. Fuzz, diferansiyel, güvenlik ve uzun süreli testler CI’da çalışmalı.
4. Kamu API’si ve paket biçimi geriye uyumluluk politikasıyla sabitlenmeli.
5. Ölçülebilir performans ve bellek raporu yayımlanmalı.
