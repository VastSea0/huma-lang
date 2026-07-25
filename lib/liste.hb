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
        yeni = listeye_ekle(yeni, d[i]) olsun
        i = i - 1 olsun
    }
    yeni'yi döndür
}

// Fonksiyonel Araçlar
eşle fonksiyon olsun d, f alsın {
    sonuç = [] olsun
    boy = d'nin uzunluğu olsun
    boy = 0 ise { sonuç'u döndür }
    i = 0'dan (boy - 1)'e kadar {
        sonuç = listeye_ekle(sonuç, f(d[i])) olsun
    }
    sonuç'u döndür
}

filtrele fonksiyon olsun d, f alsın {
    sonuç = [] olsun
    boy = d'nin uzunluğu olsun
    boy = 0 ise { sonuç'u döndür }
    i = 0'dan (boy - 1)'e kadar {
        eleman = d[i] olsun
        f(eleman) ise {
            sonuç = listeye_ekle(sonuç, eleman) olsun
        }
    }
    sonuç'u döndür
}

indirge fonksiyon olsun d, f, başlangıç alsın {
    akümülatör = başlangıç olsun
    boy = d'nin uzunluğu olsun
    boy = 0 ise { akümülatör'ü döndür }
    i = 0'dan (boy - 1)'e kadar {
        akümülatör = f(akümülatör, d[i]) olsun
    }
    akümülatör'ü döndür
}

dilimle fonksiyon olsun d, baş, son alsın {
    sonuç = [] olsun
    i = baş'tan son'a kadar {
        sonuç = listeye_ekle(sonuç, d[i]) olsun
    }
    sonuç'u döndür
}
