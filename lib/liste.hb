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
    boy = uzunluk(d) olsun
    i = 0 olsun
    i < boy olduğu sürece {
        d[i] = eleman ise { 1'i döndür }
        i = i + 1 olsun
    }
    0'ı döndür
}

ters_cevir fonksiyon olsun d alsın {
    yeni = [] olsun
    i = uzunluk(d) - 1 olsun
    i >= 0 olduğu sürece {
        yeni = listeye_ekle(yeni, d[i]) olsun
        i = i - 1 olsun
    }
    yeni'yi döndür
}

// Fonksiyonel Araçlar
eşle fonksiyon olsun d, f alsın {
    sonuç = [] olsun
    i = 0 olsun
    boy = uzunluk(d) olsun
    i < boy olduğu sürece {
        sonuç = listeye_ekle(sonuç, f(d[i])) olsun
        i = i + 1 olsun
    }
    sonuç'u döndür
}

filtrele fonksiyon olsun d, f alsın {
    sonuç = [] olsun
    i = 0 olsun
    boy = uzunluk(d) olsun
    i < boy olduğu sürece {
        eleman = d[i] olsun
        f(eleman) ise {
            sonuç = listeye_ekle(sonuç, eleman) olsun
        }
        i = i + 1 olsun
    }
    sonuç'u döndür
}

indirge fonksiyon olsun d, f, başlangıç alsın {
    akümülatör = başlangıç olsun
    i = 0 olsun
    boy = uzunluk(d) olsun
    i < boy olduğu sürece {
        akümülatör = f(akümülatör, d[i]) olsun
        i = i + 1 olsun
    }
    akümülatör'ü döndür
}

dilimle fonksiyon olsun d, baş, son alsın {
    sonuç = [] olsun
    i = baş olsun
    i < son olduğu sürece {
        sonuç = listeye_ekle(sonuç, d[i]) olsun
        i = i + 1 olsun
    }
    sonuç'u döndür
}
