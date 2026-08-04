# Hüma Mühendislik Anayasası

Bu belge Hüma'nın proje boyunca geçerli ürün yönünü, değişmezlerini ve kabul
ölçütlerini tanımlar. README, yol haritası, sürüm notları veya örnekler bu
belgeyle çelişemez. Çelişki durumunda bu belge ve normatif
[`DIL_TANIMI.md`](DIL_TANIMI.md) esas alınır.

## 1. Misyon

Hüma bir “AI dili” değildir. Hüma; web, veri, otomasyon, bilimsel hesaplama,
masaüstü, sistem entegrasyonu ve gelecekte AI dâhil farklı alanlarda güvenilir
kütüphaneler kurulabilen, Türkçe yüzeyli genel amaçlı modern bir programlama
dili olmayı hedefler.

Projenin mevcut aşamasında öncelik yeni alan kütüphaneleri üretmek değil;
doğru, hızlı, güvenli, yeniden üretilebilir ve uzun süre desteklenebilir dil
zeminini kurmaktır. Depodaki mevcut Hüma kütüphaneleri deneysel doğrulama
malzemesidir; kamu API'si veya mimari kısıt değildir ve gerektiğinde yeniden
yazılabilir ya da kaldırılabilir.

## 2. Değişmez kimlik: Türkçe dilbilgisi

Korunması zorunlu ürün kimliği şunlardır:

- UTF-8 kaynak ve Unicode NFC tanımlayıcılar,
- Türkçe anahtar sözcükler ve kanonik Türkçe programlama yüzeyi,
- kesme işaretli ekler, ünlü uyumu, kaynaştırma ve denetlenebilir ünsüz
  benzeşmesi,
- Türkçe kaynak konumlu tanı ve hata iletileri,
- [`DIL_GRAMERI.ebnf`](DIL_GRAMERI.ebnf) ile makinece tanımlanan yüzey grameri.

Türkçe yüzey serbest doğal dil yorumlama iddiası taşımaz. Gramer değişikliği;
dil tanımı, EBNF, lexer/parser regresyonları ve yorumlayıcı semantiği birlikte
güncellenmeden kabul edilemez. İngilizce takma sözcükler kanonik Türkçe yüzeyin
yerini alamaz.

## 3. Öncelik sırası

Bir tasarım kararı çatışma doğurduğunda sıra şöyledir:

1. Semantik doğruluk ve veri kaybetmeme
2. Bellek ve süreç güvenliği
3. Yeniden üretilebilirlik ve geriye dönük sözleşmeler
4. Öngörülebilir performans ve kaynak sınırları
5. Kütüphane yazarı deneyimi ve araç desteği
6. Yeni özellik veya alan kütüphanesi sayısı

Sessizce yanlış sonuç üretmek, desteklenmeyen yapıyı açıkça reddetmekten her
zaman daha kötüdür. Hız uğruna güvenlik sınırı veya tanımlı semantik gevşetilemez.

## 4. Hedef mimari

Çekirdek alan bağımsız ve küçük tutulacaktır:

- `huma-syntax`: kaynak metin, Türkçe biçimbilim, token, AST, parser ve tanılar
- `huma-bytecode`: sürümlü komut kümesi, kapsayıcı ve yapısal doğrulayıcı
- `huma-runtime`: değer/heap modeli, yapılandırılmış hata ABI'si, modüller ve
  normatif yorumlayıcı
- `huma-vm`: yalnız yorumlayıcıyla eşliği kanıtlanan bytecode yürütücüsü
- `huma-compiler`: kaynak/bytecode hattı ve açıkça deneysel AOT adaptörü
- `huma-stdlib`: yalnız saf, alan bağımsız temel işlevler
- ayrı adaptörler: dosya, ağ, süreç, SQL, GUI, FFI ve ilerideki alan
  kütüphaneleri

LSP yalnız syntax, sembol ve modül arayüzü katmanlarına bağlı olmalıdır. GUI,
GPU, HTTP, SQL veya AI bağımlılıkları dil çekirdeğine ve LSP'ye taşınamaz.

## 5. Bellek ve eşzamanlılık kararı

Hüma'nın hedef bellek modeli, kararlı nesne tutamaçlarına sahip iz süren
nesilsel bir çöp toplayıcıdır (generational tracing GC).

- Heap bir `isolate`'a aittir; bir isolate içindeki mutasyon tek iş parçacığında
  ve veri yarışı olmadan yürür.
- Global bağlar, frame'ler, closure'lar, modül ortamları ve askıdaki görevler
  açık kök kümesidir.
- Döngüsel liste, sözlük, nesne ve closure grafikleri erişilemez olduğunda
  toplanır.
- Birden çok isolate farklı iş parçacıklarında paralel çalışabilir; aralarında
  doğrudan paylaşılan değiştirilebilir heap yoktur. İletişim sınırlı, doğrulanan
  mesaj değerleriyle yapılır.
- Dosya/soket/FFI gibi kaynaklar açık `kapat` yaşam döngüsü kullanır. GC
  finalizer'ı kullanıcı kodu çalıştırmaz ve yalnız sızıntı emniyet ağıdır.
- Kamu API'si ham işaretçi veya doğrulanmamış heap adresi açığa çıkarmaz.

Mevcut uygulama, iç sahipliği `Rc` ile yöneten fakat bunu kararlı `Gc`
tutamaçlarının arkasında saklayan iki nesilli bir çevrim toplayıcıdır. Genç
nesil, yazma bariyeri/remembered-set ve tam tarama davranışı birim ve soak
testleriyle korunur. İç sahiplik tekniği kararlı runtime ABI'si değildir;
uzun süreli bellek platosu ve çoklu-isolate testleri sürüm kapısında kalır.

## 6. Backend sözleşmesi

- Yorumlayıcı, tam dilin tek normatif yürütme yoludur.
- VM ancak desteklenen bütün dil yüzeyinde yorumlayıcıyla aynı değer, çıktı,
  yan etki veya aynı yapılandırılmış hatayı ürettiği sürekli diferansiyel
  testlerle kanıtlandığında kararlı sayılır.
- AOT, tam değer/heap/hata ABI'si ve platform matrisi tamamlanana kadar
  deneysel alt kümedir.
- Desteklenmeyen bir backend yapısı açık derleme hatasıdır; başka backend'e
  sessiz düşüş veya sahte değer yasaktır.

## 7. Kütüphane ve hata sözleşmesi

Kamu kütüphaneleri için sürümlü Hüma Modül Arayüzü (HMI) tanımlanmıştır. HMI;
dışa aktarılan sembolü, parametre ve dönüş tür/sözleşmesini, etkileri, hata
kodlarını, gereken yetenekleri, Hüma sürüm aralığını ve içerik özetini makinece
okunabilir biçimde taşır.

Hatalar yalnız metin değildir. Kararlı hata sınırı en az şu alanları taşır:

- sürümlü hata kodu,
- kategori ve Türkçe ileti,
- kaynak konumu ve çağrı izi,
- güvenli yapılandırılmış ayrıntılar,
- nedensellik zinciri.

Paket SemVer'i HMI farkıyla denetlenir. Kırıcı kamu API değişikliği uygun ana
sürüm artışı olmadan yayımlanamaz.

## 8. Güvenlik sınırı

Dil kodu varsayılan olarak dış dünya yeteneğine sahip değildir. Yetenekler en
az ayrıcalıkla modül/çalıştırma kapsamına verilir. Bu model işletim sistemi
sandbox'ının yerine geçmez.

Native uzantılar varsayılan olarak süreç dışında, sürümlü mesaj protokolüyle
çalışır. Aynı süreç FFI yalnız açıkça güvenilen kod ve sürümlü dar ABI için
sunulabilir. Sürüm hattı SHA-256 manifesti, Ed25519 imzası ve build provenance
üretmeden yayın yapamaz. Uzak installer bu doğrulamayı zorunlu kılana kadar
uzak paket kurulumu kapalı kalır.

## 9. Performans sözleşmesi

Performans bir sürüm çıktısıdır. Sabit donanımlı ölçüm ortamında en az şu
sınırlar izlenir:

- lexer/parser throughput ve tepe bellek,
- soğuk başlangıç ve boş program maliyeti,
- fonksiyon çağrısı ve closure,
- sayısal döngü ve dallanma,
- liste/sözlük erişimi ve tahsis,
- metin/Unicode işlemleri,
- modül yükleme,
- uzun süreli heap platosu ve GC duraklamaları.

Yanlış çıktı veren veya desteklenmeyen backend ölçüme alınmaz. Adanmış runner'da
medyan sürede %5'ten, p95 veya tepe bellekte %10'dan büyük kötüleşme açıklama ve
onay olmadan kabul edilemez. Eşikler yeterli tarih oluştuğunda daha da
sıkılaştırılır.

## 10. Ana dal ve sürüm kabulü

Her commit temiz bir klonda, sabit araç zinciri ve kilit dosyasıyla şu kapıları
geçmelidir:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo run --locked -p huma-cli -- test tests
```

Ayrıca bütün izlenen Hüma kaynakları ayrıştırılmalı; platform matrisi,
diferansiyel test, fuzz, hata enjeksiyonu, soak/bellek ve güvenlik denetimleri
ilgili kararlılık iddiasını kapsamalıdır. Kırmızı ana daldan sürüm üretilemez.

## 11. Mevcut kullanım durumu

1.0 kabul ölçütleri tamamlanana kadar Hüma; dil mühendisliği, prototip ve
deneysel uygulama ortamıdır. Üretim güvenilirliği veya kararlı paket ABI'si
iddia etmez. Bu sınırlama eksikliği gizlemek için değil, kararlı ilan edilen
her yüzeyin ölçülebilir olmasını sağlamak içindir.

Yeni web sitesi bu anayasa ve doğrulanmış sürüm durumu üzerinden sıfırdan
tasarlanacaktır. Eski site geçiş sırasında kaynak ağacından kaldırılmıştır.
