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
    içeriyor(d, eleman)'ı döndür
}

ters_cevir fonksiyon olsun d alsın {
    boy = d'nin uzunluğu olsun
    boy = 0 ise { []'i döndür }
    yeni = [] olsun
    i = boy - 1 olsun
    i >= 0 olduğu sürece {
        yeni = yeni'ye d[i]'yi ekle olsun
        i = i - 1 olsun
    }
    yeni'yi döndür
}

// Fonksiyonel Araçlar
eşle fonksiyon olsun d, f alsın {
    sonuç = [] olsun
    boy = d'nin uzunluğu olsun
    i = 0'dan boy'a kadar {
        sonuç = sonuç'a f(d[i])'yi ekle olsun
    }
    sonuç'u döndür
}

filtrele fonksiyon olsun d, f alsın {
    sonuç = [] olsun
    boy = d'nin uzunluğu olsun
    i = 0'dan boy'a kadar {
        eleman = d[i] olsun
        f(eleman) ise {
            sonuç = sonuç'a eleman'ı ekle olsun
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
        sonuç = sonuç'a d[i]'yi ekle olsun
    }
    sonuç'u döndür
}
