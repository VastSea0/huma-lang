// ═══════════════════════════════════════════════════════════════════
// istatistik.hb — Hüma İstatistik Kütüphanesi
// Sürüm: 1.1.0
// Yazar: Egehan KAHRAMAN
// ═══════════════════════════════════════════════════════════════════
//
// Bağımlılıklar:
//   - matematik.hb → karesi() fonksiyonu
//
// Rust Built-in Bağımlılıklar:
//   - karekök(n)   → kare kök hesaplama (standart_sapma için)
//   - l'in uzunluğu → liste uzunluğu
// ═══════════════════════════════════════════════════════════════════

yükle "matematik.hb";

ortalama fonksiyon olsun d alsın {
    toplam = 0 olsun
    boy = d'nin uzunluğu olsun
    boy = 0 ise { 0'ı döndür }
    
    i = 0 olsun
    i < boy olduğu sürece {
        toplam = toplam + d[i] olsun
        i = i + 1 olsun
    }
    toplam / boy'u döndür
}

en_büyük fonksiyon olsun d alsın {
    boy = d'nin uzunluğu olsun
    boy = 0 ise { 0'ı döndür }
    en_buyuk_deger = d[0] olsun
    i = 1 olsun
    i < boy olduğu sürece {
        d[i] > en_buyuk_deger ise { en_buyuk_deger = d[i] olsun }
        i = i + 1 olsun
    }
    en_buyuk_deger'i döndür
}

en_küçük fonksiyon olsun d alsın {
    boy = d'nin uzunluğu olsun
    boy = 0 ise { 0'ı döndür }
    ek = d[0] olsun
    i = 1 olsun
    i < boy olduğu sürece {
        d[i] < ek ise { ek = d[i] olsun }
        i = i + 1 olsun
    }
    ek'i döndür
}

varyans fonksiyon olsun d alsın {
    boy = d'nin uzunluğu olsun
    boy = 0 ise { 0'ı döndür }
    ort = ortalama(d) olsun
    toplam_kare_fark = 0 olsun
    
    i = 0 olsun
    i < boy olduğu sürece {
        fark = d[i] - ort olsun
        toplam_kare_fark = toplam_kare_fark + karesi(fark) olsun
        i = i + 1 olsun
    }
    toplam_kare_fark / boy'u döndür
}

standart_sapma fonksiyon olsun d alsın {
    karekök(varyans(d))'yi döndür
}
