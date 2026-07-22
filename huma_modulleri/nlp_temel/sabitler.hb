// ══════════════════════════════════════════════════════════════════════════════
// nlp_temel/sabitler.hb — Hüma NLP Temel Paket Sabitleri
// ══════════════════════════════════════════════════════════════════════════════

TÜRKÇE_ÜNLÜLER = "aeıioöuüAEIİOÖUÜ" olsun

DURAK_LİSTESİ = [
    "bir", "bu", "şu", "o", "ve", "veya", "ile", "da", "de", "mi", "mı", "mu", "mü",
    "ki", "ise", "çok", "az", "daha", "en", "ne", "ama", "için", "gibi", "kadar",
    "sonra", "önce", "üzere", "göre", "hem", "ya", "ancak", "fakat", "hatta", "bile",
    "ben", "sen", "biz", "siz", "onlar", "bunlar", "şunlar", "var", "yok", "olan",
    "değil", "her", "hiç", "bazı", "tüm", "bütün", "hangi", "nasıl", "neden", "niye",
    "nerede", "nereden", "nereye", "kim", "kimi", "kime", "kimden",
    "şey", "şeyi", "şeye", "şeyden", "böyle", "öyle", "böylece", "zaten", "artık",
    "hep", "hiçbir", "herhangi", "kendi", "kendine", "kendisi", "dahi",
    "oysa", "oysaki", "lakin", "belki", "mutlaka", "kesinlikle", "tabii",
    "evet", "hayır", "peki", "tamam", "olarak", "aracılığıyla", "vasıtasıyla",
    "göre", "kadar", "beri", "itibaren", "doğru", "karşı", "rağmen"
] olsun

ÇEKİM_EKLERİ = [
    "ştırmak", "ştirmek", "laştır", "leştir", "abilmek", "ebilmek", "abilir", "ebilir",
    "acaklar", "ecekler", "acaktı", "ecekti", "maktan", "mekten", "makta", "mekte",
    "dıkça", "dikçe", "dukça", "dükçe", "tıkça", "tikçe", "tukça", "tükçe",
    "yacak", "yecek", "arak", "erek", "ınca", "ince", "unca", "ünce", "madan", "meden",
    "mıştı", "mişti", "muştu", "müştü", "tıydı", "tiydi", "tuydı", "tüydü",
    "ıyor", "iyor", "uyor", "üyor", "acak", "ecek", "ardı", "erdi", "ırdı", "irdi",
    "mış", "miş", "muş", "müş", "tık", "tik", "tuk", "tük", "dık", "dik", "duk", "dük",
    "lar", "ler", "mak", "mek", "ken",
    "lardan", "lerden", "larla", "lerle", "ından", "inden", "undan", "ünden",
    "ların", "lerin", "ndaki", "ndeki", "daki", "deki", "taki", "teki",
    "dan", "den", "tan", "ten", "nda", "nde", "nın", "nin", "nun", "nün",
    "na", "ne", "nı", "ni", "nu", "nü",
    "da", "de", "ta", "te", "ya", "ye", "yı", "yi", "yu", "yü",
    "ımız", "imiz", "umuz", "ümüz", "ınız", "iniz", "unuz", "ünüz", "mız", "miz", "nız", "niz",
    "ları", "leri", "sın", "sin", "sun", "sün",
    "dı", "di", "du", "dü", "tı", "ti", "tu", "tü", "sa", "se", "ma", "me",
    "ar", "er", "ır", "ir", "ur", "ür", "ıp", "ip", "up", "üp",
    "ın", "in", "un", "ün", "ım", "im", "um", "üm", "am", "em",
    "ı", "i", "u", "ü", "a", "e"
] olsun

POS_İSİM   = "İSİM" olsun
POS_FİİL   = "FİİL" olsun
POS_SIFAT  = "SIFAT" olsun
POS_ZARF   = "ZARF" olsun
POS_ZAMİR  = "ZAMİR" olsun
POS_BAĞLAÇ = "BAĞLAÇ" olsun
POS_EDAT   = "EDAT" olsun

FİİL_KÖKLERİ = [
    "gel", "git", "ver", "al", "yap", "bil", "gör", "kal", "çık", "gir",
    "bak", "çalış", "yaz", "oku", "söyle", "anla", "başla", "bitir", "dön",
    "dur", "geç", "istey", "iste", "konuş", "otur", "sev", "tut", "yaşa",
    "yürü", "aç", "at", "düşün", "bul", "çek", "düş", "gül", "hisset",
    "koy", "oyna", "öl", "sat", "sev", "sür", "taşı", "uç", "var", "ver",
    "vur", "yat", "ye", "yık", "yit", "yol", "yor", "yön"
] olsun

BİLİNEN_YERLER = [
    "ankara", "istanbul", "izmir", "bursa", "antalya", "adana", "konya",
    "gaziantep", "kayseri", "mersin", "eskişehir", "diyarbakır", "samsun",
    "denizli", "trabzon", "erzurum", "malatya", "van", "elazığ", "manisa",
    "kahramanmaraş", "kocaeli", "muğla", "aydın", "tekirdağ", "balıkesir",
    "türkiye", "anadolu", "kapadokya", "avrupa", "asya", "akdeniz",
    "karadeniz", "ege", "marmara", "kadıköy", "üsküdar", "beşiktaş",
    "almanya", "fransa", "italya", "ispanya", "hollanda"
] olsun

AY_LİSTESİ = [
    "ocak", "şubat", "mart", "nisan", "mayıs", "haziran",
    "temmuz", "ağustos", "eylül", "ekim", "kasım", "aralık"
] olsun

KURUM_LİSTESİ = [
    "bakanlığı", "belediyesi", "üniversitesi", "müdürlüğü",
    "başkanlığı", "merkezi", "ajansı", "kurumu", "enstitüsü", "vakfı",
    "sendikası", "federasyonu", "konfederasyonu", "birliği", "derneği",
    "şirketi", "grubu", "holdingı", "bankası", "sigortası", "hastanesi",
    "afad", "tübitak", "tobb", "tesk", "tsk", "tbmm", "chp", "akp", "mhp"
] olsun

POZİTİF_KELİMELER = [
    "iyi", "güzel", "harika", "mükemmel", "süper", "başarılı", "muhteşem",
    "kaliteli", "hızlı", "kolay", "faydalı", "mutlu", "sevindirici",
    "tavsiye", "beğendim", "beğendim", "harikadır", "üstün", "kusursuz",
    "efsane", "lezzetli", "güvenli", "sağlam", "temiz", "nazik", "ilgili",
    "teşekkür", "teşekkürler", "tebrik", "dostça", "uygun", "ucuz",
    "kazançlı", "verimli", "parlak", "ferah", "şahane", "fevkalade"
] olsun

NEGATİF_KELİMELER = [
    "kötü", "berbat", "rezalet", "başarısız", "yavaş", "zor", "zararlı",
    "üzücü", "pişman", "şikayet", "beğenmedim", "bozuk", "hasarlı",
    "kırık", "eksik", "yanlış", "hatalı", "pahalı", "kalitesiz",
    "ilgisiz", "kaba", "saygısız", "berbattır", "iğrenç", "berbat",
    "tehlikeli", "kirli", "gürültülü", "karanlık", "sıkıcı", "dandik",
    "dolandırıcı", "sahte", "aldatıcı", "kusurlu", "arızalı"
] olsun
