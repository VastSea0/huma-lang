// ══════════════════════════════════════════════════════════════════════════════
// nlp_temel/sabitler.hb — Hüma NLP Temel Paket Sabitleri (Genişletilmiş Sürüm)
// ══════════════════════════════════════════════════════════════════════════════

// ──────────────────────────────────────────────────────────────────────────────
// 1. FONETİK — Harfler ve Ünlü Uyumu
// ──────────────────────────────────────────────────────────────────────────────

TÜRKÇE_ÜNLÜLER = "aeıioöuüAEIİOÖUÜ" olsun

TÜRKÇE_ÜNSÜZLER = "bcçdfgğhjklmnprsştvyzBCÇDFGĞHJKLMNPRSŞTVYZ" olsun

KALIN_ÜNLÜLER = ["a", "ı", "o", "u"] olsun
İNCE_ÜNLÜLER  = ["e", "i", "ö", "ü"] olsun
DÜZ_ÜNLÜLER     = ["a", "e", "ı", "i"] olsun
YUVARLAK_ÜNLÜLER = ["o", "ö", "u", "ü"] olsun

// ──────────────────────────────────────────────────────────────────────────────
// 2. ZAMİRLER
// ──────────────────────────────────────────────────────────────────────────────

ŞAHIS_ZAMİRLERİ  = ["ben", "sen", "o", "biz", "siz", "onlar"] olsun
İŞARET_ZAMİRLERİ = ["bu", "şu", "o", "bunlar", "şunlar", "onlar", "böyle", "öyle", "şöyle"] olsun
SORU_KELİMELERİ  = [
    "kim", "ne", "nerede", "nereye", "nereden", "nasıl", "niçin", "niye",
    "neden", "kaç", "kaçıncı", "hangi", "mi", "mı", "mu", "mü"
] olsun

// ──────────────────────────────────────────────────────────────────────────────
// 3. DURAK KELİMELER (Stopwords)
// ──────────────────────────────────────────────────────────────────────────────

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
    "beri", "itibaren", "doğru", "karşı", "rağmen",
    "gerek", "yani", "üstelik", "ayrıca", "dolayısıyla", "nedeniyle", "yüzünden",
    "sayesinde", "gerçi", "meğer", "sanki", "güya", "adeta", "resmen", "aslında",
    "gerçekten", "doğrusu", "kısacası", "özetle", "sonuçta", "nihayet", "ilk", "son",
    "diğer", "öbür", "birçok", "birkaç", "çoğu", "kimse", "herkes", "hepsi", "tümü",
    "kaç", "niçin", "acaba", "işte", "aynen", "keza", "dahası", "bilhassa",
    "özellikle", "genellikle", "çoğunlukla", "nitekim", "madem", "mademki",
    "illa", "illaki", "şayet", "eğer", "şöyle", "öylesine", "böylesine", "iyice",
    "epeyce", "oldukça", "fazla", "pek", "gayet", "biraz", "azıcık", "birazcık",
    "epey", "hayli", "gayrı"
] olsun

// ──────────────────────────────────────────────────────────────────────────────
// 4. EKLER — Çekim ve Yapım
// ──────────────────────────────────────────────────────────────────────────────

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

// Yapım ekleri: yeni kelime türeten ekler (çekim eklerinden ayrı tutulur)
YAPIM_EKLERİ = [
    "lık", "lik", "luk", "lük", "cı", "ci", "cu", "cü", "çı", "çi", "çu", "çü",
    "sız", "siz", "suz", "süz", "lı", "li", "lu", "lü", "sel", "sal",
    "gen", "leş", "laş", "landır", "lendir", "msı", "msi", "cık", "cik", "cuk", "cük",
    "gil", "daş", "deş", "taş", "teş", "ıcı", "ici", "ucu", "ücü"
] olsun

// ──────────────────────────────────────────────────────────────────────────────
// 5. KELİME TÜRÜ ETİKETLERİ (POS)
// ──────────────────────────────────────────────────────────────────────────────

POS_İSİM   = "İSİM" olsun
POS_FİİL   = "FİİL" olsun
POS_SIFAT  = "SIFAT" olsun
POS_ZARF   = "ZARF" olsun
POS_ZAMİR  = "ZAMİR" olsun
POS_BAĞLAÇ = "BAĞLAÇ" olsun
POS_EDAT   = "EDAT" olsun
POS_SAYI   = "SAYI" olsun
POS_ÜNLEM  = "ÜNLEM" olsun

// ──────────────────────────────────────────────────────────────────────────────
// 6. KÖK SÖZLÜKLERİ — Konu Bazlı
// ──────────────────────────────────────────────────────────────────────────────

TÜRKÇE_KÖK_SÖZLÜĞÜ = [
    "ürün", "kötü", "harika", "izmir", "kalite", "sayı", "kitap", "okul", "bilgi",
    "yazılım", "şirket", "destek", "yardım", "kargo", "müşteri", "fiyat", "dünya",
    "insan", "konferans", "ankara", "istanbul", "türkiye", "zeynep", "ayşe", "mehmet",
    "öğrenci", "öğretmen", "ev", "araba", "yol", "deniz", "gün", "gece", "hava",
    "yağmur", "güneş", "film", "takım", "çalışma", "sonuç", "deneyim", "hizmet",
    "iade", "talep", "parça", "para", "kutu", "ambalaj", "fatura", "zaman", "bölüm",
    "alan", "hız", "önemli", "yeni", "başarılı", "büyük", "küçük", "uzun", "kısa",
    "güzel", "çirkin", "hızlı", "yavaş", "temiz", "kirli", "doğru", "yanlış", "zor",
    "kolay", "prof", "dr", "algoritma", "üniversite", "katılımcı", "bilim", "ülke",
    "masa", "sandalye", "kapı", "pencere", "duvar", "çatı", "bahçe", "hafta", "yıl",
    "mevsim", "şehir", "mahalle", "cadde", "sokak", "market", "hastane", "eczane",
    "internet", "ekran", "klavye", "program", "sistem", "veri", "dosya", "ağ",
    "sunucu", "kod", "hata", "çözüm", "proje", "plan", "hedef", "karar", "sorun",
    "fikir", "düşünce", "duygu", "gerçek", "yalan", "çevre", "enerji", "sağlık",
    "hastalık", "ilaç", "tedavi", "spor", "müzik", "resim", "tiyatro", "edebiyat",
    "tarih", "coğrafya", "matematik", "fizik", "kimya", "biyoloji", "felsefe",
    "kültür", "gelenek", "adet", "kanun", "hak", "adalet", "özgürlük", "barış",
    "ekonomi", "ticaret", "sanayi", "tarım", "teknoloji", "buluş", "keşif",
    "gelişme", "değişim", "ilerleme", "başarı", "sınav", "not", "ödev", "ders",
    "konu", "cümle", "kelime", "harf", "dilbilgisi", "anlam", "çeviri", "yazar",
    "şair", "sanatçı", "yönetmen", "oyuncu", "izleyici", "yazı", "mektup", "gazete",
    "dergi", "haber", "reklam", "marka", "tasarım", "moda", "kıyafet", "ayakkabı",
    "çanta", "saat", "gözlük", "dükkan", "mağaza", "alışveriş", "indirim",
    "kampanya", "sipariş", "teslimat", "ödeme", "banka", "kredi", "borç", "gelir",
    "gider", "bütçe", "yatırım", "kâr", "zarar"
] olsun

// Doğa ve çevre
DOĞA_KELİMELERİ = [
    "ağaç", "çiçek", "yaprak", "dal", "kök", "orman", "dağ", "tepe", "vadi", "ova",
    "çöl", "bataklık", "göl", "nehir", "dere", "şelale", "okyanus", "ada", "kıyı",
    "plaj", "kum", "taş", "kaya", "toprak", "rüzgar", "fırtına", "bulut", "kar",
    "buz", "sis", "gökkuşağı", "yıldız", "gezegen", "evren", "hayvan", "kuş",
    "böcek", "arı", "kelebek", "karınca", "balık", "at", "inek", "koyun", "tavuk",
    "kedi", "köpek"
] olsun

// Aile ve toplum
AİLE_TOPLUM_KELİMELERİ = [
    "anne", "baba", "kardeş", "abla", "ağabey", "teyze", "hala", "dayı", "amca",
    "dede", "nine", "torun", "yeğen", "kuzen", "eş", "koca", "karı", "nişanlı",
    "sevgili", "arkadaş", "dost", "akraba", "komşu", "misafir", "aile", "toplum",
    "millet", "halk", "vatandaş", "birey", "nesil", "görenek", "örf", "düğün",
    "bayram", "tatil"
] olsun

// Teknoloji
TEKNOLOJİ_KELİMELERİ = [
    "bilgisayar", "telefon", "donanım", "yazıcı", "tarayıcı", "veritabanı",
    "uygulama", "yapay zeka", "robot", "sensör", "çip", "işlemci", "bellek",
    "disk", "bulut", "güvenlik", "şifre", "virüs", "güncelleme", "sunucu",
    "tarayıcı", "sekme", "bağlantı", "indirme", "yükleme", "kullanıcı", "arayüz"
] olsun

// Duygular (duygu tanıma / affect analizi için — pozitif/negatif sözlüklerden ayrı)
DUYGU_KELİMELERİ = [
    "sevinç", "mutluluk", "neşe", "heyecan", "gurur", "umut", "huzur", "rahatlama",
    "üzüntü", "keder", "hüzün", "pişmanlık", "endişe", "kaygı", "korku", "panik",
    "öfke", "kızgınlık", "nefret", "kıskançlık", "utanç", "suçluluk", "şaşkınlık",
    "merak", "sıkılma", "yalnızlık", "özlem", "sevgi", "aşk", "minnettarlık",
    "güven", "şüphe", "çaresizlik", "bıkkınlık"
] olsun

// Meslekler
MESLEK_LİSTESİ = [
    "doktor", "hemşire", "eczacı", "avukat", "hakim", "savcı", "mühendis", "mimar",
    "öğretmen", "akademisyen", "gazeteci", "yazar", "şair", "ressam", "heykeltıraş",
    "müzisyen", "oyuncu", "yönetmen", "yapımcı", "aşçı", "garson", "berber", "terzi",
    "marangoz", "elektrikçi", "tesisatçı", "boyacı", "çiftçi", "balıkçı", "çoban",
    "şoför", "pilot", "kaptan", "denizci", "asker", "polis", "itfaiyeci",
    "muhasebeci", "bankacı", "satıcı", "kasiyer", "temizlikçi", "hizmetli",
    "sekreter", "yönetici", "müdür", "patron", "işveren", "işçi", "memur", "esnaf",
    "tüccar", "zanaatkar"
] olsun

// Yiyecek ve içecek
YİYECEK_İÇECEK_LİSTESİ = [
    "ekmek", "peynir", "zeytin", "bal", "reçel", "tereyağı", "yumurta", "süt",
    "yoğurt", "ayran", "çay", "kahve", "su", "şerbet", "çorba", "pilav", "makarna",
    "köfte", "kebap", "döner", "lahmacun", "pide", "börek", "baklava", "künefe",
    "lokum", "dondurma", "salata", "sebze", "meyve", "elma", "armut", "muz",
    "çilek", "karpuz", "kavun", "kiraz", "şeftali", "kayısı", "incir", "nar",
    "ceviz", "fındık", "fıstık", "badem", "portakal", "üzüm"
] olsun

FİİL_KÖKLERİ = [
    "gel", "git", "ver", "al", "yap", "bil", "gör", "kal", "çık", "gir",
    "bak", "çalış", "yaz", "oku", "söyle", "anla", "başla", "bitir", "dön",
    "dur", "geç", "istey", "iste", "konuş", "otur", "sev", "tut", "yaşa",
    "yürü", "aç", "at", "düşün", "bul", "çek", "düş", "gül", "hisset",
    "koy", "oyna", "öl", "sat", "sür", "taşı", "uç", "var", "vur", "yat", "ye",
    "yık", "yit", "yol", "yor", "yön",
    "anlat", "sor", "cevapla", "dinle", "izle", "göster", "öğret", "öğren",
    "unut", "hatırla", "düzelt", "kur", "üret", "tüket", "harca", "kazan",
    "kaybet", "ara", "sakla", "koru", "savun", "bırak", "kaldır", "ekle",
    "çıkart", "sil", "değiştir", "güncelle", "kaydet", "yükle", "paylaş",
    "beğen", "yorumla", "seyret", "ağla", "kız", "sevin", "üzül", "şaşır",
    "inan", "güven", "şüphelen", "reddet", "yasakla", "emret", "affet", "kes",
    "biç", "dik", "boya", "yıka", "kurut", "temizle", "süpür", "pişir",
    "karıştır", "doldur", "boşalt", "kapa", "kilitle", "çal"
] olsun

// ──────────────────────────────────────────────────────────────────────────────
// 7. ÖZEL İSİMLER — Yer, Kurum, Ülke
// ──────────────────────────────────────────────────────────────────────────────

// Türkiye'nin 81 ili (yer adı tanıma / NER için)
TÜRKİYE_İLLERİ = [
    "adana", "adıyaman", "afyonkarahisar", "ağrı", "amasya", "ankara", "antalya",
    "artvin", "aydın", "balıkesir", "bilecik", "bingöl", "bitlis", "bolu",
    "burdur", "bursa", "çanakkale", "çankırı", "çorum", "denizli", "diyarbakır",
    "edirne", "elazığ", "erzincan", "erzurum", "eskişehir", "gaziantep", "giresun",
    "gümüşhane", "hakkari", "hatay", "isparta", "mersin", "istanbul", "izmir",
    "kars", "kastamonu", "kayseri", "kırklareli", "kırşehir", "kocaeli", "konya",
    "kütahya", "malatya", "manisa", "kahramanmaraş", "mardin", "muğla", "muş",
    "nevşehir", "niğde", "ordu", "rize", "sakarya", "samsun", "siirt", "sinop",
    "sivas", "tekirdağ", "tokat", "trabzon", "tunceli", "şanlıurfa", "uşak",
    "van", "yozgat", "zonguldak", "aksaray", "bayburt", "karaman", "kırıkkale",
    "batman", "şırnak", "bartın", "ardahan", "iğdır", "yalova", "karabük",
    "kilis", "osmaniye", "düzce"
] olsun

BİLİNEN_YERLER = [
    "türkiye", "anadolu", "kapadokya", "avrupa", "asya", "akdeniz",
    "karadeniz", "ege", "marmara", "kadıköy", "üsküdar", "beşiktaş",
    "beyoğlu", "şişli", "bakırköy", "ataşehir", "çankaya", "konak", "karşıyaka",
    "taksim", "sultanahmet", "galata", "bebek", "etiler", "nişantaşı", "moda",
    "pamukkale", "efes", "göbeklitepe", "boğaz"
] olsun

ÜLKE_LİSTESİ = [
    "türkiye", "almanya", "fransa", "italya", "ispanya", "hollanda", "amerika",
    "ingiltere", "rusya", "çin", "japonya", "hindistan", "brezilya", "kanada",
    "avustralya", "mısır", "iran", "irak", "suriye", "yunanistan", "bulgaristan",
    "romanya", "azerbaycan", "gürcistan", "ukrayna", "polonya", "portekiz",
    "belçika", "isviçre", "avusturya", "i̇sveç", "norveç", "danimarka", "finlandiya"
] olsun

KURUM_LİSTESİ = [
    "bakanlığı", "belediyesi", "üniversitesi", "müdürlüğü",
    "başkanlığı", "merkezi", "ajansı", "kurumu", "enstitüsü", "vakfı",
    "sendikası", "federasyonu", "konfederasyonu", "birliği", "derneği",
    "şirketi", "grubu", "holdingı", "bankası", "sigortası", "hastanesi",
    "valiliği", "kaymakamlığı", "savcılığı", "mahkemesi", "emniyeti",
    "gümrüğü", "noterliği", "kooperatifi", "odası", "barosu",
    "afad", "tübitak", "tobb", "tesk", "tsk", "tbmm", "chp", "akp", "mhp"
] olsun

// ──────────────────────────────────────────────────────────────────────────────
// 8. ZAMAN İFADELERİ
// ──────────────────────────────────────────────────────────────────────────────

AY_LİSTESİ = [
    "ocak", "şubat", "mart", "nisan", "mayıs", "haziran",
    "temmuz", "ağustos", "eylül", "ekim", "kasım", "aralık"
] olsun

GÜN_LİSTESİ = [
    "pazartesi", "salı", "çarşamba", "perşembe", "cuma", "cumartesi", "pazar"
] olsun

MEVSİM_LİSTESİ = ["ilkbahar", "yaz", "sonbahar", "kış"] olsun

ZAMAN_İFADELERİ = [
    "bugün", "yarın", "dün", "şimdi", "az önce", "biraz sonra", "geçen hafta",
    "gelecek hafta", "geçen ay", "gelecek ay", "geçen yıl", "gelecek yıl",
    "sabah", "öğle", "akşam", "gece", "öğleden sonra", "her zaman", "bazen",
    "nadiren", "sık sık", "hemen", "derhal", "yakında", "az sonra"
] olsun

// ──────────────────────────────────────────────────────────────────────────────
// 9. SAYILAR VE RENKLER
// ──────────────────────────────────────────────────────────────────────────────

SAYI_KELİMELERİ = [
    "sıfır", "bir", "iki", "üç", "dört", "beş", "altı", "yedi", "sekiz", "dokuz",
    "on", "yirmi", "otuz", "kırk", "elli", "altmış", "yetmiş", "seksen", "doksan",
    "yüz", "bin", "milyon", "milyar"
] olsun

RENK_LİSTESİ = [
    "kırmızı", "mavi", "yeşil", "sarı", "turuncu", "mor", "pembe", "siyah",
    "beyaz", "gri", "kahverengi", "lacivert", "bej", "turkuaz", "bordo",
    "altın", "gümüş", "eflatun", "haki", "füme"
] olsun

// ──────────────────────────────────────────────────────────────────────────────
// 10. DUYGU ANALİZİ SÖZLÜKLERİ
// ──────────────────────────────────────────────────────────────────────────────

POZİTİF_KELİMELER = [
    "iyi", "güzel", "harika", "mükemmel", "süper", "başarılı", "muhteşem",
    "kaliteli", "hızlı", "kolay", "faydalı", "mutlu", "sevindirici",
    "tavsiye", "beğendim", "harikadır", "üstün", "kusursuz",
    "efsane", "lezzetli", "güvenli", "sağlam", "temiz", "nazik", "ilgili",
    "teşekkür", "teşekkürler", "tebrik", "dostça", "uygun", "ucuz",
    "kazançlı", "verimli", "parlak", "ferah", "şahane", "fevkalade",
    "memnun", "memnuniyet", "öneririm", "sorunsuz", "zamanında", "hızlıca",
    "profesyonel", "samimi", "keyifli", "eğlenceli", "rahat", "konforlu",
    "şık", "zarif", "dayanıklı", "orijinal", "avantajlı", "cömert", "bol",
    "doyurucu", "taze", "nefis", "enfes", "olağanüstü"
] olsun

NEGATİF_KELİMELER = [
    "kötü", "berbat", "rezalet", "başarısız", "yavaş", "zor", "zararlı",
    "üzücü", "pişman", "şikayet", "beğenmedim", "bozuk", "hasarlı",
    "kırık", "eksik", "yanlış", "hatalı", "pahalı", "kalitesiz",
    "ilgisiz", "kaba", "saygısız", "berbattır", "iğrenç",
    "tehlikeli", "kirli", "gürültülü", "karanlık", "sıkıcı", "dandik",
    "dolandırıcı", "sahte", "aldatıcı", "kusurlu", "arızalı",
    "memnuniyetsiz", "gecikme", "gecikmeli", "yetersiz", "vasıfsız",
    "taklit", "çürük", "kokuşmuş", "bayat", "tatsız", "rezil", "ayıp",
    "soğuk", "umursamaz", "tembel", "yavan", "defolu", "garantisiz",
    "zahmetli", "sıkıntılı", "stresli", "yorucu"
] olsun

NÖTR_KELİMELER = [
    "normal", "standart", "ortalama", "sıradan", "olağan", "vasat", "nötr",
    "tarafsız"
] olsun

// Duygu şiddetini artıran zarflar
YOĞUNLAŞTIRICI_ZARFLAR = [
    "çok", "pek", "gayet", "aşırı", "hayli", "epey", "oldukça", "fazlasıyla",
    "iyice", "müthiş", "inanılmaz"
] olsun

// Olumsuzluk tespiti için (duygu analizinde kutup çevirme amaçlı)
OLUMSUZLUK_KELİMELERİ = [
    "değil", "yok", "hayır", "asla", "hiç", "katiyen", "olmaz", "yoktur",
    "değildir"
] olsun

// ──────────────────────────────────────────────────────────────────────────────
// 11. ÜNLEMLER, KISALTMALAR VE NOKTALAMA
// ──────────────────────────────────────────────────────────────────────────────

ÜNLEMLER = [
    "ah", "oh", "vay", "aman", "eyvah", "bravo", "aferin", "maşallah", "yazık",
    "hey", "tüh", "of", "vah", "hadi", "haydi", "hoop", "eyvallah", "pardon"
] olsun

KISALTMALAR = [
    "vb", "vs", "vd", "dr", "prof", "doç", "sn", "tl", "cm", "km", "kg", "gr",
    "mah", "cad", "sok", "no", "tel", "örn", "bkz", "age", "çev", "haz"
] olsun

NOKTALAMA_İŞARETLERİ = [
    ".", ",", "!", "?", ":", ";", "-", "—", "(", ")", "\"", "'", "…", "/", "*"
] olsun
