// ══════════════════════════════════════════════════════════════════════════════
// nlp.hb — Hüma Dili Türkçe NLP Kütüphanesi
// Sürüm: 3.2.0 (Modern Syntax Edition)
// Lisans: MIT
// ══════════════════════════════════════════════════════════════════════════════

"dizgi.hb"'yi yükle
"liste.hb"'yi yükle

// ─── SABİTLER ────────────────────────────────────────────────────────────────

TÜRKÇE_ÜNLÜLER = "aeıioöuüAEIİOÖUÜ" olsun

DURAK_LISTESİ = [
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

ÇEKIM_EKLERİ = [
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
    "dur", "geç", "getir", "götür", "inan", "kazan", "koş", "seç", "tut",
    "uç", "ulaş", "vur", "yürü", "sat", "sev", "say", "sor", "bul", "sil",
    "kullan", "düşün", "konuş", "öğren", "öğret", "bekle", "izle", "dene",
    "çiz", "gönder", "otur", "kalk", "uyan", "uyu", "iç", "ye", "kur",
    "geliş", "geliştir", "ilerle", "katıl", "sun", "üret", "açıkla",
    "değerlendir", "gerçekleştir", "düzenle", "tanıt", "artır"
] olsun

SIFAT_LİSTESİ = [
    "büyük", "küçük", "iyi", "kötü", "güzel", "çirkin", "hızlı", "yavaş",
    "yeni", "eski", "uzun", "kısa", "geniş", "dar", "açık", "kapalı",
    "sıcak", "soğuk", "sert", "yumuşak", "kırmızı", "mavi", "yeşil", "sarı",
    "beyaz", "siyah", "doğru", "yanlış", "kolay", "zor", "güçlü", "zayıf",
    "mutlu", "üzgün", "önemli", "gerekli", "farklı", "aynı", "tek", "çift",
    "yapay", "gerçek", "dijital", "modern", "temel", "kapsamlı", "başarılı"
] olsun

ZARF_LİSTESİ = [
    "hızla", "yavaşça", "erken", "geç", "şimdi", "hemen", "bugün", "yarın",
    "dün", "sabah", "akşam", "gece", "bazen", "nadiren", "yine", "tekrar",
    "neredeyse", "tam", "sadece", "çoktan", "henüz", "artık", "zaten"
] olsun

ZAMİR_LİSTESİ = [
    "ben", "sen", "o", "biz", "siz", "onlar", "bu", "şu", "bunlar", "şunlar",
    "kendi", "hepsi", "kimse", "herkes", "biri", "birisi", "hiçbiri", "bazıları"
] olsun

BAĞLAÇ_LİSTESİ = [
    "ve", "veya", "ama", "fakat", "ancak", "lakin", "oysa", "oysaki",
    "çünkü", "zira", "hem", "ne", "ya", "ki", "dahi", "bile", "hatta",
    "üstelik", "ayrıca", "yoksa", "madem", "eğer"
] olsun

EDAT_LİSTESİ = [
    "için", "ile", "gibi", "kadar", "göre", "karşı", "sonra", "önce",
    "üzere", "doğru", "dek", "değin", "beri", "rağmen", "başka",
    "dışında", "içinde", "üzerinde", "altında", "yanında", "üzerine"
] olsun

NER_KİŞİ  = "KİŞİ" olsun
NER_YER   = "YER" olsun
NER_ORG   = "ORG" olsun
NER_TARİH = "TARİH" olsun
NER_SAYI  = "SAYI" olsun
NER_O     = "O" olsun

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

UNVAN_LİSTESİ = [
    "başkan", "müdür", "bakan", "vali", "kaymakam", "belediye",
    "profesör", "doktor", "uzman", "yetkili", "sözcü", "temsilci",
    "genel", "milli", "ulusal", "cumhurbaşkanı", "başbakan",
    "bakanı", "başkanı", "müdürü", "valisi"
] olsun

BÜYÜK_HARFLER = "ABCÇDEFGĞHIİJKLMNOÖPRSŞTUÜVYZ" olsun
RAKAMLAR      = "0123456789" olsun

POZİTİF_KELİMELER = [
    "güzel", "harika", "mükemmel", "iyi", "seviyorum", "mutlu", "başarılı",
    "memnun", "olağanüstü", "süper", "muhteşem", "enfes", "nefis", "hoş",
    "sevimli", "güçlü", "zeki", "yetenekli", "başarı", "sevmek", "beğenmek",
    "teşekkür", "tebrik", "bravo", "aferin", "umut", "neşe", "sevinç",
    "huzur", "barış", "sevgi", "aşk", "eğlenceli", "ilginç", "faydalı",
    "yararlı", "tavsiye", "öneririm", "rekor", "tamamlandı", "güven"
] olsun

NEGATİF_KELİMELER = [
    "kötü", "berbat", "korkunç", "nefret", "üzgün", "mutsuz",
    "başarısız", "hata", "yanlış", "sorun", "problem", "tehlike",
    "zararlı", "rezalet", "felaket", "facia", "dehşet", "acı",
    "zor", "imkansız", "istemiyorum", "sevmiyorum", "beğenmedim",
    "şikayet", "kaygı", "endişe", "korku", "stres", "yorgun", "bıktım",
    "yandı", "yıkıldı", "mahvoldu", "battı", "çöktü", "hasar", "zarar"
] olsun

GÜÇLENDİRİCİLER = [
    "çok", "derece", "aşırı", "oldukça", "gayet", "epey", "pek",
    "kesinlikle", "gerçekten", "tam", "fazlasıyla", "büyük"
] olsun

KISALTMALAR = [
    "dr", "prof", "doç", "yrd", "arş", "öğr", "muh", "müh",
    "vb", "vs", "bkz", "örn", "no", "tel", "sok", "cad", "blv",
    "apt", "hz", "sr", "st", "mr", "mrs", "ms", "fig", "vol"
] olsun

// ════════════════════════════════════════════════════════════════════════════
// MODÜL 1: TEMİZLEME VE TOKENİZASYON
// ════════════════════════════════════════════════════════════════════════════

nlp_temizle fonksiyon olsun metin alsın {
    sonuç = metin'i küçük_harf olsun
    semboller = [".", ",", ";", ":", "!", "?", "(", ")", "\"", "'", "\u2019", "\u2018", "-", "\n", "\t"] olsun
    n = semboller'in uzunluğu olsun
    i = 0'dan n'e kadar {
        sonuç = sonuç ile semboller[i] ve " " değiştir
    }
    sonuç'u döndür
}

tokenize fonksiyon olsun metin alsın {
    parcalar = metin ile " " böl olsun
    temizler = [] olsun
    n = parcalar'ın uzunluğu olsun
    i = 0'dan n'e kadar {
        tok = parcalar[i]'yi kırp olsun
        tok'un uzunluğu > 0 ise {
            temizler'e tok'u ekle
        }
    }
    temizler'i döndür
}

nlp_tokenize fonksiyon olsun metin alsın {
    metin'i nlp_temizle'yi tokenize'yi döndür
}

karakter_tokenize fonksiyon olsun metin alsın {
     metin ile "" böl'ü döndür
}

// ════════════════════════════════════════════════════════════════════════════
// MODÜL 2: DURAK KELİME FİLTRELEME
// ════════════════════════════════════════════════════════════════════════════

durak_mı fonksiyon olsun kelime alsın {
    küçük = kelime'yi küçük_harf olsun
    DURAK_LISTESİ ile küçük hızlı_içeriyor'u döndür
}

durak_kelime_filtrele fonksiyon olsun tokens alsın {
    sonuç = [] olsun
    n = tokens'ın uzunluğu olsun
    i = 0'dan n'e kadar {
        eleman = tokens[i] olsun
        eleman'ı durak_mı = 0 ise {
            sonuç'a eleman'ı ekle
        }
    }
    sonuç'u döndür
}

// ════════════════════════════════════════════════════════════════════════════
// MODÜL 3: ÜNLÜ UYUMU YARDIMCILARI
// ════════════════════════════════════════════════════════════════════════════

ünlü_mü fonksiyon olsun karakter alsın {
    TÜRKÇE_ÜNLÜLER ile karakter içeriyor'u döndür
}

kelime_ünlü_sayısı fonksiyon olsun kelime alsın {
    sayac = 0 olsun
    n = kelime'nin uzunluğu olsun
    i = 0'dan n'e kadar {
        kelime[i]'yi ünlü_mü ise {
            sayac = sayac + 1 olsun
        }
    }
    sayac'ı döndür
}

son_ünlü fonksiyon olsun kelime alsın {
    bulunan = "" olsun
    devam   = 1 olsun
    boy     = kelime'nin uzunluğu olsun
    i = boy - 1 olduğu sürece {
        devam = 1 ise {
            kelime[i]'yi ünlü_mü ise {
                bulunan = kelime[i] olsun
                devam   = 0 olsun
            }
        }
        i = i - 1 olsun
        i >= 0 ise { devam } yoksa { kes }
    }
    bulunan'ı döndür
}

ünlü_uyumu_türü fonksiyon olsun kelime alsın {
    sü   = kelime'yi son_ünlü olsun
    arkalar = ["a", "ı", "o", "u"] olsun
    arkalar ile sü hızlı_içeriyor ise { "arka"'yı döndür }
    "ön"'ü döndür
}

// ════════════════════════════════════════════════════════════════════════════
// MODÜL 4: KÖKLEŞTIRME (STEMMING)
// ════════════════════════════════════════════════════════════════════════════

ek_var_mı fonksiyon olsun kelime, ek alsın {
    kel_boy = kelime'nin uzunluğu olsun
    ek_boy  = ek'in uzunluğu olsun
    sonuç   = 0 olsun
    ek_boy < kel_boy ise {
        kalan = kel_boy - ek_boy olsun
        son   = kelime ile kalan ve kel_boy dizi_dilim olsun
        son = ek ise {
            sonuç = 1 olsun
        }
    }
    sonuç'u döndür
}

ek_çıkar fonksiyon olsun kelime, ek alsın {
    kelime ile ek ek_var_mı ise {
        boy = kelime'nin uzunluğu - ek'in uzunluğu olsun
        kelime ile 0 ve boy dizi_dilim'i döndür
    }
    kelime'yi döndür
}

stem fonksiyon olsun kelime alsın {
    kök     = kelime'yi küçük_harf olsun
    değişti = 1 olsun
    değişti = 1 olduğu sürece {
        değişti = 0 olsun
        kök'ün uzunluğu > 3 ise {
            ek_sayısı = ÇEKIM_EKLERİ'nin uzunluğu olsun
            i = 0'dan ek_sayısı'na kadar {
                ek      = ÇEKIM_EKLERİ[i] olsun
                ek_boy  = ek'in uzunluğu olsun
                kel_boy = kök'in uzunluğu olsun
                fark    = kel_boy - ek_boy olsun
                fark >= 3 ise {
                    kök ile ek ek_var_mı ise {
                        kök     = kök ile 0 ve fark dizi_dilim olsun
                        değişti = 1 olsun
                        i       = ek_sayısı olsun
                    }
                }
            }
        }
    }
    kök'ü döndür
}

akıllı_stem fonksiyon olsun kelime, ner_etiketi alsın {
    özel_etiketler = [NER_KİŞİ, NER_YER, NER_ORG] olsun
    özel_etiketler ile ner_etiketi hızlı_içeriyor ise {
        kelime'yi küçük_harf'i döndür
    }
    kelime'yi stem'i döndür
}

toplu_stem fonksiyon olsun tokens alsın {
    sonuç = [] olsun
    n = tokens'ın uzunluğu olsun
    i = 0'dan n'e kadar {
        sonuç'a tokens[i]'yi stem'i ekle
    }
    sonuç'u döndür
}

// ════════════════════════════════════════════════════════════════════════════
// MODÜL 5: FREKANS ANALİZİ VE SIRALAMA
// ════════════════════════════════════════════════════════════════════════════

frekans_ekle fonksiyon olsun frekanslar, kelime alsın {
    bulundu = 0 olsun
    n = frekanslar'ın uzunluğu olsun
    i = 0'dan n'e kadar {
        çift = frekanslar[i] olsun
        çift[0] = kelime ise {
            çift[1] = çift[1] + 1 olsun
            bulundu = 1 olsun
            i       = n olsun
        }
    }
    bulundu = 0 ise {
        frekanslar'a [kelime, 1]'i ekle
    }
    frekanslar'ı döndür
}

kelime_frekansları fonksiyon olsun tokens alsın {
    frekanslar = [] olsun
    n = tokens'ın uzunluğu olsun
    i = 0'dan n'e kadar {
        frekanslar = frekanslar ile tokens[i] frekans_ekle olsun
    }
    frekanslar'ı döndür
}

frekans_sırala fonksiyon olsun frekanslar alsın {
    n = frekanslar'ın uzunluğu olsun
    i = 0'dan n'e kadar {
        j = 0'dan (n - i - 1)'e kadar {
            a = frekanslar[j] olsun
            b = frekanslar[j + 1] olsun
            a[1] < b[1] ise {
                frekanslar[j]     = b olsun
                frekanslar[j + 1] = a olsun
            }
        }
    }
    frekanslar'ı döndür
}

en_sık_n fonksiyon olsun frekanslar, n alsın {
    sıralı = frekanslar'ı frekans_sırala olsun
    sonuç  = [] olsun
    top    = sıralı'nın uzunluğu olsun
    top > n ise { top = n olsun }
    i = 0'dan top'a kadar {
        sonuç'a sıralı[i]'yi ekle
    }
    sonuç'u döndür
}

// ════════════════════════════════════════════════════════════════════════════
// MODÜL 6: CÜMLE BÖLME
// ════════════════════════════════════════════════════════════════════════════

kısaltma_mı fonksiyon olsun kelime alsın {
    temiz = kelime'yi küçük_harf olsun
    KISALTMALAR ile temiz hızlı_içeriyor'u döndür
}

cümle_böl fonksiyon olsun metin alsın {
    cümleler = [] olsun
    mevcut   = "" olsun
    n = metin'nin uzunluğu olsun
    i = 0'dan n'e kadar {
        kar  = metin[i] olsun
        sonu = 0 olsun
        kar = "." ise { sonu = 1 olsun }
        kar = "!" ise { sonu = 2 olsun }
        kar = "?" ise { sonu = 2 olsun }

        sonu = 0 ise {
            mevcut = mevcut + kar olsun
        }
        sonu = 1 ise {
            parcalar = mevcut'u kırp ile " " böl olsun
            pn       = parcalar'ın uzunluğu olsun
            son_k    = "" olsun
            pn > 0 ise { son_k = parcalar[pn - 1] olsun }
            son_k'yı kısaltma_mı ise {
                mevcut = mevcut + kar olsun
            } yoksa {
                mevcut  = mevcut + kar olsun
                temiz_c = mevcut'u kırp olsun
                temiz_c'nin uzunluğu > 0 ise { cümleler'e temiz_c'yi ekle }
                mevcut = "" olsun
            }
        }
        sonu = 2 ise {
            mevcut  = mevcut + kar olsun
            temiz_c = mevcut'u kırp olsun
            temiz_c'nin uzunluğu > 0 ise { cümleler'e temiz_c'yi ekle }
            mevcut = "" olsun
        }
    }
    kalan = mevcut'u kırp olsun
    kalan'ın uzunluğu > 0 ise { cümleler'e kalan'ı ekle }
    cümleler'i döndür
}

// ════════════════════════════════════════════════════════════════════════════
// MODÜL 7: POS ETİKETLEME
// ════════════════════════════════════════════════════════════════════════════

pos_etiket fonksiyon olsun kelime alsın {
    k      = kelime'yi küçük_harf olsun
    etiket = POS_İSİM olsun
    buldu  = 0 olsun

    ZAMİR_LİSTESİ ile k hızlı_içeriyor ise { etiket = POS_ZAMİR; buldu = 1 }
    buldu = 0 ise { BAĞLAÇ_LİSTESİ ile k hızlı_içeriyor ise { etiket = POS_BAĞLAÇ; buldu = 1 } }
    buldu = 0 ise { EDAT_LİSTESİ ile k hızlı_içeriyor ise { etiket = POS_EDAT; buldu = 1 } }
    buldu = 0 ise { ZARF_LİSTESİ ile k hızlı_içeriyor ise { etiket = POS_ZARF; buldu = 1 } }
    buldu = 0 ise { SIFAT_LİSTESİ ile k hızlı_içeriyor ise { etiket = POS_SIFAT; buldu = 1 } }
    
    buldu = 0 ise {
        s_ekler = ["lı", "li", "lu", "lü", "sız", "siz", "sal", "sel"] olsun
        i = 0'dan 8'e kadar {
            k ile s_ekler[i] ek_var_mı ise { etiket = POS_SIFAT; buldu = 1; i = 8 }
        }
    }
    
    buldu = 0 ise {
        kök = k'yi stem olsun
        FİİL_KÖKLERİ ile kök hızlı_içeriyor ise { etiket = POS_FİİL; buldu = 1 }
    }
    
    buldu = 0 ise {
        f_ekler = ["mak", "mek", "ıyor", "iyor", "uyor", "üyor", "acak", "ecek", "mış", "miş", "arak", "erek"] olsun
        n = f_ekler'in uzunluğu olsun
        i = 0'dan n'e kadar {
            k ile f_ekler[i] ek_var_mı ise { etiket = POS_FİİL; buldu = 1; i = n }
        }
    }
    etiket'i döndür
}

pos_etiketle fonksiyon olsun tokens alsın {
    sonuç = [] olsun
    n = tokens'ın uzunluğu olsun
    i = 0'dan n'e kadar {
        kelime = tokens[i] olsun
        sonuç'a [kelime, kelime'yi pos_etiket]'i ekle
    }
    sonuç'u döndür
}

// ════════════════════════════════════════════════════════════════════════════
// MODÜL 8: VARLİK İSMİ TANIMA (NER)
// ════════════════════════════════════════════════════════════════════════════

büyük_harf_mi fonksiyon olsun karakter alsın {
    BÜYÜK_HARFLER ile karakter içeriyor'u döndür
}

rakam_mı fonksiyon olsun karakter alsın {
    RAKAMLAR ile karakter içeriyor'u döndür
}

sayı_token_mu fonksiyon olsun kelime alsın {
    n = kelime'nin uzunluğu olsun
    n = 0 ise { 0'ı döndür }
    tümü = 1 olsun
    i = 0'dan n'e kadar {
        kelime[i]'yi rakam_mı = 0 ise { tümü = 0; n = i }
    }
    tümü'yü döndür
}

ner_etiket fonksiyon olsun kelime, önceki, önceki_nokta alsın {
    kelime'yi sayı_token_mu ise { NER_SAYI'yı döndür }
    BİLİNEN_YERLER ile kelime'yi küçük_harf hızlı_içeriyor ise { NER_YER'i döndür }
    AY_LİSTESİ ile kelime'yi küçük_harf hızlı_içeriyor ise { NER_TARİH'i döndür }
    KURUM_LİSTESİ ile kelime'yi küçük_harf hızlı_içeriyor ise { NER_ORG'u döndür }
    UNVAN_LİSTESİ ile kelime'yi küçük_harf hızlı_içeriyor ise { NER_O'yu döndür }

    uzunluk(kelime) > 0 ise {
        kelime[0]'ı büyük_harf_mi ise {
            önceki_nokta = 1 ise { NER_O'yu döndür }
            önceki = NER_KİŞİ ise { NER_KİŞİ'yi döndür }
            önceki = NER_ORG  ise { NER_ORG'u döndür }
            NER_KİŞİ'yi döndür
        }
    }
    NER_O'yu döndür
}

ner_etiketle fonksiyon olsun tokens alsın {
    sonuç        = [] olsun
    önceki       = NER_O olsun
    önceki_nokta = 1 olsun
    n = tokens'ın uzunluğu olsun
    i = 0'dan n'e kadar {
        kelime = tokens[i] olsun
        etiket = ner_etiket(kelime, önceki, önceki_nokta) olsun
        sonuç'a [kelime, etiket]'i ekle
        önceki = etiket olsun
        
        yeni_nokta = 0 olsun
        k_boy = kelime'nin uzunluğu olsun
        k_boy > 0 ise {
            son_harf = kelime[k_boy - 1] olsun
            noktalamalar = [".", "!", "?"] olsun
            noktalamalar ile son_harf hızlı_içeriyor ise { yeni_nokta = 1 }
        }
        önceki_nokta = yeni_nokta olsun
    }
    sonuç'u döndür
}

// ════════════════════════════════════════════════════════════════════════════
// MODÜL 9: DUYGU ANALİZİ
// ════════════════════════════════════════════════════════════════════════════

duygu_puan fonksiyon olsun tokens alsın {
    puan   = 0 olsun
    çarpan = 1 olsun
    n = tokens'ın uzunluğu olsun
    i = 0'dan n'e kadar {
        k = tokens[i]'yi küçük_harf olsun
        GÜÇLENDİRİCİLER ile k hızlı_içeriyor ise { çarpan = 2 } yoksa {
            POZİTİF_KELİMELER ile k hızlı_içeriyor ise { puan = puan + çarpan; çarpan = 1 } yoksa {
                NEGATİF_KELİMELER ile k hızlı_içeriyor ise { puan = puan - çarpan; çarpan = 1 } yoksa {
                    k = "değil" ise { çarpan = -1 }
                }
            }
        }
    }
    puan'ı döndür
}

// ════════════════════════════════════════════════════════════════════════════
// MODÜL 10: METİN BENZERLİĞİ
// ════════════════════════════════════════════════════════════════════════════

metin_ortak_kelime fonksiyon olsun metin1, metin2 alsın {
    t1 = metin1'i nlp_tokenize'yi durak_kelime_filtrele'yi toplu_stem olsun
    t2 = metin2'i nlp_tokenize'yi durak_kelime_filtrele'yi toplu_stem olsun
    ortak = 0 olsun
    n = t1'in uzunluğu olsun
    i = 0'dan n'e kadar {
        t2 ile t1[i] hızlı_içeriyor ise { ortak = ortak + 1 }
    }
    ortak'ı döndür
}

// ════════════════════════════════════════════════════════════════════════════
// MODÜL 11: METİN İSTATİSTİĞİ
// ════════════════════════════════════════════════════════════════════════════

metin_istatistik fonksiyon olsun metin alsın {
    "📊 Metin İstatistiği"'ni yazdır
    "Karakter sayısı : " + metin'in uzunluğu'nu yazdır
    tokens = metin'i nlp_tokenize olsun
    "Kelime sayısı   : " + tokens'ın uzunluğu'nu yazdır
    "Tahmini cümle   : " + metin'i cümle_böl'ün uzunluğu'nu yazdır
}

// ─── FINAL ───

"[nlp.hb v3.2.0] Modern Türkçe NLP kütüphanesi yüklendi ✓"'ü yazdır
