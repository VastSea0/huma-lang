# Hüma Bytecode Kapsayıcısı

Bu belge `.hbc` dosya kapsayıcısının 4 numaralı biçimini tanımlar. Kapsayıcı,
ham serileştirilmiş verinin yanlış sürümle veya bozulmuş halde VM'ye ulaşmasını
engelleyen bir sınırdır. Hüma kaynak dilinin tamamının bytecode tarafından
desteklendiği anlamına gelmez.

## Başlık

Bütün çok baytlı tamsayılar little-endian kodlanır.

| Ofset | Uzunluk | Alan |
|---:|---:|---|
| 0 | 8 | ASCII magic: `HUMA-HBC` |
| 8 | 2 | Biçim sürümü (`u16`), mevcut değer `4` |
| 10 | 8 | Payload uzunluğu (`u64`) |
| 18 | 32 | Payload SHA-256 özeti |
| 50 | değişken | Serileştirilmiş `Program` payload'u |

Payload, bakımı süren `serde_json` ile UTF-8 JSON olarak kodlanan `Program`
değeridir. JSON, saldırganın küçük bir uzunluk alanıyla girdi boyutundan
orantısız koleksiyon kapasitesi ayırmasına izin vermediği için güvenilmeyen
bytecode sınırında tercih edilir. Sürüm 4 ana komut akışına ek olarak bytecode
fonksiyon tablosunu, fonksiyon
parametrelerini ve komut/fonksiyon kaynak konumlarını taşır. Kod çözücü:

- 64 MiB payload sınırı uygular.
- Eksik veya sonda ek veri bulunan dosyaları reddeder.
- Bilinmeyen biçim sürümlerini reddeder.
- SHA-256 özetini karşılaştırır.
- Sabit indekslerini, atlama hedeflerini ve koleksiyon/argüman sınırlarını
  VM başlamadan önce doğrular.
- Bütün erişilebilir kontrol akışı yollarında yığın eksilmesini, azami yığın
  yüksekliğini ve birleşme noktalarındaki yığın yüksekliği eşitliğini doğrular.

VM, kapsayıcı dışında doğrudan verilen bellek içi `Program` değerini de
çalıştırmadan önce aynı doğrulayıcıdan geçirir. `dene/yakala` sırasında hata
oluşursa yığın, `dene` girişindeki yüksekliğe geri sarılır ve yalnızca
yapılandırılmış hata değeri yakalama koluna aktarılır.

SHA-256 alanı bir dijital imza değildir. Dosyanın kaynağını veya yayıncısını
doğrulamaz; yalnızca kapsayıcı içindeki payload değişikliğini/bozulmasını tespit
eder. Güvenilmeyen bytecode için imzalı dağıtım ve çalışma zamanı yetki modeli
ayrı güvenlik katmanları olarak gereklidir.

## Uyumluluk

Biçim sürümü eşleşmiyorsa okuyucu tahminde bulunmaz ve dosyayı çalıştırmaz.
Önceki ham serileştirmeler ile v1/v2/v3 kapsayıcıları v4 değildir; kaynak
dosyadan yeniden derlenmelidir.

Bytecode opcode ve değer modeli 1.0'a kadar kararlı kamu ABI'si sayılmaz. Her
uyumsuz kapsayıcı değişikliği biçim sürümünü artırmalıdır.
