# Hüma Bytecode Kapsayıcısı

Bu belge `.hbc` dosya kapsayıcısının 1 numaralı biçimini tanımlar. Kapsayıcı,
ham serileştirilmiş verinin yanlış sürümle veya bozulmuş halde VM'ye ulaşmasını
engelleyen bir sınırdır. Hüma kaynak dilinin tamamının bytecode tarafından
desteklendiği anlamına gelmez.

## Başlık

Bütün çok baytlı tamsayılar little-endian kodlanır.

| Ofset | Uzunluk | Alan |
|---:|---:|---|
| 0 | 8 | ASCII magic: `HUMA-HBC` |
| 8 | 2 | Biçim sürümü (`u16`), mevcut değer `1` |
| 10 | 8 | Payload uzunluğu (`u64`) |
| 18 | 32 | Payload SHA-256 özeti |
| 50 | değişken | Serileştirilmiş `Program` payload'u |

Payload, `bincode 1.3` sabit tamsayı kodlamasıyla üretilir. Kod çözücü:

- 64 MiB payload sınırı uygular.
- Eksik veya sonda ek veri bulunan dosyaları reddeder.
- Bilinmeyen biçim sürümlerini reddeder.
- SHA-256 özetini karşılaştırır.
- Sabit indekslerini, atlama hedeflerini ve koleksiyon/argüman sınırlarını
  VM başlamadan önce doğrular.

SHA-256 alanı bir dijital imza değildir. Dosyanın kaynağını veya yayıncısını
doğrulamaz; yalnızca kapsayıcı içindeki payload değişikliğini/bozulmasını tespit
eder. Güvenilmeyen bytecode için imzalı dağıtım ve çalışma zamanı yetki modeli
ayrı güvenlik katmanları olarak gereklidir.

## Uyumluluk

Biçim sürümü eşleşmiyorsa okuyucu tahminde bulunmaz ve dosyayı çalıştırmaz.
Önceki ham `bincode` `.hbc` dosyaları v1 kapsayıcısı değildir; kaynak dosyadan
yeniden derlenmelidir.

Bytecode opcode ve değer modeli 1.0'a kadar kararlı kamu ABI'si sayılmaz. Her
uyumsuz kapsayıcı değişikliği biçim sürümünü artırmalıdır.
