# Hüma Modül Arayüzü (HMI) v1

HMI, native veya başka bir dilde yazılmış modüllerin Hüma sürecine yüklenmeden
ayrı bir süreçte çalışmasını sağlayan kararlı sınırdır. HMI bir C ABI değildir;
uzunluk önekli, boyut sınırlı ve sürümlü mesaj protokolüdür.

## Paket sözleşmesi

`huma.json` içindeki `hmi` alanı şunları taşır:

- protokol ana/alt sürümü ve `stdio-json-v1` taşıması,
- paket köküne göre güvenli çalıştırılabilir yol,
- arayüz şema sürümü ve desteklenen Hüma sürüm aralığı,
- her dışa aktarım için parametre/dönüş türleri,
- dosya, ağ, süreç, veritabanı, GUI, saat, rastgelelik ve native etkileri,
- kararlı hata kodları ve yeniden denenebilirlik bilgisi.

Yol mutlak olamaz, `..` içeremez ve paket kurulumunda sembolik bağ olmayan
çalıştırılabilir normal dosyaya çözülmelidir. Bilinmeyen sözleşme alanları
reddedilir.

## Taşıma ve yaşam döngüsü

Her çerçeve 4 bayt big-endian uzunluk ve en fazla 16 MiB JSON payload taşır.
Ana makine sırasıyla `initialize`, sıfır veya daha fazla `call`, sonra
`shutdown` gönderir. İstek kimliği `1..2^53-1` aralığındadır. Yanıt aynı kimliği
ve uyumlu protokol ana sürümünü taşımak zorundadır.

`ProcessClient` modülü kabuk kullanmadan, temizlenmiş ortamla ve borulu
stdin/stdout ile başlatır. Okuma ayrı iş parçacığında yapılır. Yanlış kimlik,
bozuk çerçeve, uyumsuz sürüm, beklenmedik EOF veya zaman aşımı fail-closed
davranır; süreç sonlandırılır. Uzak hata `code`, `message` ve `retryable`
alanlarıyla veri olarak döner.

HMI süreç ayrımı bellek bozulmasının dil sürecine yayılmasını engeller; bir
işletim sistemi sandbox'ı değildir. Çocuk süreç varsayılan kullanıcı haklarıyla
çalışır. Hüma tarafında ayrıca `ffi` yeteneği gerekir.

## Değer modeli

Sınırdan yalnız sonlu `f64`, boolean, UTF-8 metin, bayt, liste, metin anahtarlı
harita ve boş değer geçer. Azami iç içelik 128, toplam öğe sayısı 1.000.000'dur.
Döngüsel heap, fonksiyon, sınıf, görev ve süreç içi tutamaçlar taşınamaz.

## Uyumluluk

`huma paket api-kontrol <önceki.json> <yeni.json>` denetimi aşağıdaki
değişiklikleri kırıcı sayar:

- fonksiyon kaldırma,
- mevcut parametrenin adını, sırasını, türünü veya isteğe bağlılığını değiştirme,
- zorunlu parametre ekleme veya dönüş türünü değiştirme,
- yeni etki ya da yeni hata davranışı ekleme,
- HMI sözleşmesini kaldırma veya protokol ana sürümünü değiştirme.

Sona isteğe bağlı parametre eklemek ve etki kaldırmak geriye uyumludur. Kırıcı
değişiklik yalnız paket ana SemVer sürümü artırıldığında kabul edilir.

## Süreç içi FFI

Eski dar `f64()` ABI adaptörü varsayılan olarak kaydedilmez. Yalnız güvenilen
kod için hem `ffi` yeteneği hem de CLI'da
`--güvenilir-süreç-içi-ffi` verilerek açılabilir. Yanlış ABI bu durumda ev sahibi
süreci çökertebilir; genel kütüphane dağıtım yolu HMI'dır.
