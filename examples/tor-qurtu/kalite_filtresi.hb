// ══════════════════════════════════════════════════════════════════════════════
// examples / tor-qurtu / kalite_filtresi.hb
// FineWeb & Gopher Standartlarında Veri Seti Kalite Filtresi
// ══════════════════════════════════════════════════════════════════════════════

"dizgi.hb"'yi yükle

durak_kelimeler_listesi = [
    "ve", "bir", "ile", "bu", "da", "de", "için", "daha", "kadar", "olarak",
    "gibi", "olan", "sonra", "veya", "göre", "her", "ne", "ise", "en", "çok",
    "hem", "fakat", "ancak", "çünkü", "tarafından", "üzerine", "aynı", "içinde"
]

fineweb_kalite_denetimi fonksiyon olsun metin alsın {
    m = kırp(metin)
    u = uzunluk(m)
    
    // 1. Minimum Karakter Sınırı (en az 30 karakter olmalı)
    u < 30 ise {
        0'ı döndür
    }
    
    // 2. Yazılım / Web / Kod Artığı Filtresi
    içeriyor(m, "var ") veya içeriyor(m, "function") veya içeriyor(m, "document.") veya içeriyor(m, "window.") veya içeriyor(m, "wgPage") veya içeriyor(m, "mw-") veya içeriyor(m, "RegExp") veya içeriyor(m, "{") veya içeriyor(m, "}") ise {
        0'ı döndür
    }
    
    // 3. Kelime Sayısı Kontrolü (en az 5 kelime)
    kelimeler = böl(m, " ")
    k_sayisi = uzunluk(kelimeler)
    k_sayisi < 5 ise {
        0'ı döndür
    }
    
    // 4. Ortalama Kelime Uzunluğu Denetimi (3.5 - 14 karakter arası olmalı)
    toplam_k_uzunluk = 0
    k_index = 0'dan (k_sayisi - 1)'e kadar {
        toplam_k_uzunluk = toplam_k_uzunluk + uzunluk(kelimeler[k_index])
    }
    
    ort_uzunluk = 0
    k_sayisi > 0 ise {
        ort_uzunluk = toplam_k_uzunluk / k_sayisi
    }
    
    (ort_uzunluk < 3.5) veya (ort_uzunluk > 14.0) ise {
        0'ı döndür
    }
    
    // 5. FineWeb Türkçe Durak Kelime Oranı Denetimi (Stop-word ratio)
    durak_sayaci = 0
    d_uzunluk = uzunluk(durak_kelimeler_listesi)
    
    w_index = 0'dan (k_sayisi - 1)'e kadar {
        k_kucuk = küçük_harf(kelimeler[w_index])
        
        s_index = 0'dan (d_uzunluk - 1)'e kadar {
            k_kucuk = durak_kelimeler_listesi[s_index] ise {
                durak_sayaci = durak_sayaci + 1
            }
        }
    }
    
    // Durak kelime oranı en az 1 tane bulunmalıdır
    durak_sayaci < 1 ise {
        0'ı döndür
    }
    
    1'i döndür
}
