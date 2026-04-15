// ═══════════════════════════════════════════════════════════════════
// liste.hb — Hüma Gelişmiş Liste İşlemleri
// Sürüm: 1.1.0
// Yazar: Egehan KAHRAMAN
// ═══════════════════════════════════════════════════════════════════
//
// Rust Built-in Bağımlılıklar:
//   - uzunluk(liste)         → eleman sayısı
//   - listeye_ekle(l, e)     → yeni liste döndürür
//   - içeriyor(liste, e)     → varlık kontrolü
// ═══════════════════════════════════════════════════════════════════

yazdır_liste fonksiyon olsun d alsın {
    d'yi yazdır
}

içeriyor_mu fonksiyon olsun d, eleman alsın {
    boy = d'nin uzunluğu olsun
    i = 0'dan boy'a kadar {
        d[i] = eleman ise { 1'i döndür }
    }
    0'ı döndür
}

ters_cevir fonksiyon olsun d alsın {
    yeni = [] olsun
    boy = d'nin uzunluğu olsun
    i = boy - 1 olduğu sürece { // Reverse range is not natively supported yet, keep while
        yeni'ye d[i]'yi ekle
        i = i - 1 olsun
        i >= 0 ise { devam } yoksa { kes }
    }
    yeni'yi döndür
}

// Fonksiyonel Araçlar
eşle fonksiyon olsun d, f alsın {
    sonuç = [] olsun
    boy = d'nin uzunluğu olsun
    i = 0'dan boy'a kadar {
        sonuç'a f(d[i])'yi ekle
    }
    sonuç'u döndür
}

filtrele fonksiyon olsun d, f alsın {
    sonuç = [] olsun
    boy = d'nin uzunluğu olsun
    i = 0'dan boy'a kadar {
        eleman = d[i] olsun
        f(eleman) ise {
            sonuç'a eleman'ı ekle
        }
    }
    sonuç'u döndür
}

indirge fonksiyon olsun d, f, başlangıç alsın {
    akümülatör = başlangıç olsun
    boy = d'nin uzunluğu olsun
    i = 0'dan boy'a kadar {
        akümülatör = f(akümülatör, d[i]) olsun
    }
    akümülatör'ü döndür
}

dilimle fonksiyon olsun d, baş, son alsın {
    sonuç = [] olsun
    i = baş'tan son'a kadar {
        sonuç'a d[i]'yi ekle
    }
    sonuç'u döndür
}
