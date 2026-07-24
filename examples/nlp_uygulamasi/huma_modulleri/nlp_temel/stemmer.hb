// ══════════════════════════════════════════════════════════════════════════════
// nlp_temel/stemmer.hb — Türkçe Kök Bulma ve Ek Soyma (OOP & Modüler)
// ══════════════════════════════════════════════════════════════════════════════

"sabitler.hb"'yi yükle

kok_bulucu sınıf olsun {
    ünlü_mü fonksiyon olsun karakter alsın {
        res = içeriyor(TÜRKÇE_ÜNLÜLER, karakter) olsun
        res'i döndür
    }

    kelime_ünlü_sayısı fonksiyon olsun kelime alsın {
        n = uzunluk(kelime) olsun
        (n = 0) ise {
            r = 0 olsun
            r'yi döndür
        }
        sayac = 0 olsun
        i = 0'dan (n - 1)'e kadar {
            c = kelime[i] olsun
            (kendisi.ünlü_mü(c)) ise {
                sayac = sayac + 1 olsun
            }
        }
        sayac'ı döndür
    }

    son_ünlü fonksiyon olsun kelime alsın {
        n = uzunluk(kelime) olsun
        i = n - 1 olsun
        i >= 0 olduğu sürece {
            c = kelime[i] olsun
            (kendisi.ünlü_mü(c)) ise {
                c'yi döndür
            }
            i = i - 1 olsun
        }
        r = "" olsun
        r'yi döndür
    }

    ünlü_uyumu_türü fonksiyon olsun kelime alsın {
        s = kendisi.son_ünlü(kelime) olsun
        ((s = "a") veya (s = "ı") veya (s = "o") veya (s = "u") veya (s = "A") veya (s = "I") veya (s = "O") veya (s = "U")) ise {
            r = "KALIN" olsun
            r'yi döndür
        }
        ((s = "e") veya (s = "i") veya (s = "ö") veya (s = "ü") veya (s = "E") veya (s = "İ") veya (s = "Ö") veya (s = "Ü")) ise {
            r = "İNCE" olsun
            r'yi döndür
        }
        r = "BİLİNMİYOR" olsun
        r'yi döndür
    }

    ek_var_mı fonksiyon olsun kelime, ek alsın {
        k_len = uzunluk(kelime) olsun
        e_len = uzunluk(ek) olsun
        (k_len <= e_len) ise {
            r = 0 olsun
            r'yi döndür
        }
        
        i = 0'dan (e_len - 1)'e kadar {
            c1 = kelime[k_len - e_len + i] olsun
            c2 = ek[i] olsun
            (c1 != c2) ise {
                r = 0 olsun
                r'yi döndür
            }
        }
        r = 1 olsun
        r'yi döndür
    }

    ek_çıkar fonksiyon olsun kelime, ek alsın {
        k_len = uzunluk(kelime) olsun
        e_len = uzunluk(ek) olsun
        (k_len <= e_len) ise { kelime'yi döndür }
        
        sonuç = "" olsun
        n = k_len - e_len olsun
        i = 0'dan (n - 1)'e kadar {
            sonuç = sonuç + kelime[i] olsun
        }
        sonuç'u döndür
    }

    stem fonksiyon olsun kelime alsın {
        k = küçük_harf(kelime) olsun
        k_len = uzunluk(k) olsun
        (k_len <= 2) ise { k'yi döndür }

        n_ekler = uzunluk(ÇEKİM_EKLERİ) olsun
        i = 0'dan (n_ekler - 1)'e kadar {
            ek = ÇEKİM_EKLERİ[i] olsun
            (kendisi.ek_var_mı(k, ek)) ise {
                kök = kendisi.ek_çıkar(k, ek) olsun
                (uzunluk(kök) >= 2) ise {
                    k = kök olsun
                }
            }
        }
        k'yi döndür
    }

    akıllı_stem fonksiyon olsun kelime, ner_etiketi alsın {
        ((ner_etiketi = "KİŞİ") veya (ner_etiketi = "YER") veya (ner_etiketi = "KURUM")) ise {
            kelime'yi döndür
        }
        res = kendisi.stem(kelime) olsun
        res'i döndür
    }

    toplu_stem fonksiyon olsun tokens alsın {
        n = uzunluk(tokens) olsun
        (n = 0) ise {
            r = [] olsun
            r'yi döndür
        }
        sonuç = [] olsun
        i = 0'dan (n - 1)'e kadar {
            sonuç = listeye_ekle(sonuç, kendisi.stem(tokens[i])) olsun
        }
        sonuç'u döndür
    }
}

// ─── Global Örnek ────────────────────────────────────────────────────────
nlp_stemmer = kok_bulucu() olsun

// ─── Geriye Dönük Uyumluluk Sarmalayıcıları ──────────────────────────────

ünlü_mü fonksiyon olsun karakter alsın {
    res = nlp_stemmer.ünlü_mü(karakter) olsun
    res'i döndür
}

kelime_ünlü_sayısı fonksiyon olsun kelime alsın {
    res = nlp_stemmer.kelime_ünlü_sayısı(kelime) olsun
    res'i döndür
}

son_ünlü fonksiyon olsun kelime alsın {
    res = nlp_stemmer.son_ünlü(kelime) olsun
    res'i döndür
}

ünlü_uyumu_türü fonksiyon olsun kelime alsın {
    res = nlp_stemmer.ünlü_uyumu_türü(kelime) olsun
    res'i döndür
}

ek_var_mı fonksiyon olsun kelime, ek alsın {
    res = nlp_stemmer.ek_var_mı(kelime, ek) olsun
    res'i döndür
}

ek_çıkar fonksiyon olsun kelime, ek alsın {
    res = nlp_stemmer.ek_çıkar(kelime, ek) olsun
    res'i döndür
}

stem fonksiyon olsun kelime alsın {
    res = nlp_stemmer.stem(kelime) olsun
    res'i döndür
}

akıllı_stem fonksiyon olsun kelime, ner_etiketi alsın {
    res = nlp_stemmer.akıllı_stem(kelime, ner_etiketi) olsun
    res'i döndür
}

toplu_stem fonksiyon olsun tokens alsın {
    res = nlp_stemmer.toplu_stem(tokens) olsun
    res'i döndür
}
