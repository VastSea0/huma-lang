// ══════════════════════════════════════════════════════════════════════════════
// examples / tor-qurtu / tekillestirici.hb
// Mükerrer Kayıtları (Deduplication) Ayıklama Modülü
// ══════════════════════════════════════════════════════════════════════════════

"dizgi.hb"'yi yükle

// Basit Normalize Etmiş İmzaya Dönüştürme
metin_imzasi_olustur fonksiyon olsun metin alsın {
    t = küçük_harf(kırp(metin))
    t = değiştir(t, " ", "")
    t = değiştir(t, ".", "")
    t = değiştir(t, ",", "")
    t = değiştir(t, ";", "")
    t = değiştir(t, ":", "")
    t = değiştir(t, "-", "")
    t'yi döndür
}

// Cümle Listesinde Daha Önce Var Olup Olmadığını Kontrol Etme
zaten_var_mi fonksiyon olsun imza_listesi, yeni_imza alsın {
    n = uzunluk(imza_listesi)
    i = 0'dan (n - 1)'e kadar {
        imza_listesi[i] = yeni_imza ise {
            1'i döndür
        }
    }
    0'ı döndür
}
