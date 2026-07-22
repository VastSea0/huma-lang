// ══════════════════════════════════════════════════════════════════════════════
// nlp_temel/analizci.hb — NLP Metin Analizci, POS, NER & Duygu (OOP & Modüler)
// ══════════════════════════════════════════════════════════════════════════════

"sabitler.hb"'yi yükle
"islemci.hb"'yi yükle
"stemmer.hb"'yi yükle

metin_analizci sınıf olsun {
    pos_etiket fonksiyon olsun kelime alsın {
        k = küçük_harf(kelime) olsun
        
        (içeriyor(FİİL_KÖKLERİ, k)) ise { POS_FİİL'i döndür }
        
        kök = nlp_stemmer.stem(k) olsun
        (içeriyor(FİİL_KÖKLERİ, kök)) ise { POS_FİİL'i döndür }
        
        f_ekler = ["mak", "mek", "ıyor", "iyor", "uyor", "üyor", "acak", "ecek", "mış", "miş", "arak", "erek"] olsun
        n = uzunluk(f_ekler) olsun
        i = 0'dan (n - 1)'e kadar {
            (nlp_stemmer.ek_var_mı(k, f_ekler[i])) ise { POS_FİİL'i döndür }
        }
        
        (içeriyor(DURAK_LİSTESİ, k)) ise {
            ((k = "ve") veya (k = "veya") veya (k = "ama") veya (k = "fakat") veya (k = "lakin") veya (k = "de") veya (k = "da") veya (k = "ki")) ise { POS_BAĞLAÇ'ı döndür }
            ((k = "için") veya (k = "gibi") veya (k = "kadar") veya (k = "göre") veya (k = "ile")) ise { POS_EDAT'ı döndür }
            ((k = "ben") veya (k = "sen") veya (k = "o") veya (k = "biz") veya (k = "siz") veya (k = "onlar") veya (k = "bu") veya (k = "şu") veya (k = "kim")) ise { POS_ZAMİR'i döndür }
        }
        
        POS_İSİM'i döndür
    }

    pos_etiketle fonksiyon olsun tokens alsın {
        n = uzunluk(tokens) olsun
        (n = 0) ise {
            r = [] olsun
            r'yi döndür
        }
        sonuç = [] olsun
        i = 0'dan (n - 1)'e kadar {
            etiket = kendisi.pos_etiket(tokens[i]) olsun
            sonuç = listeye_ekle(sonuç, [tokens[i], etiket]) olsun
        }
        sonuç'u döndür
    }

    büyük_harf_mi fonksiyon olsun karakter alsın {
        res = ((karakter >= "A") ve (karakter <= "Z") veya (karakter = "Ç") veya (karakter = "Ğ") veya (karakter = "İ") veya (karakter = "Ö") veya (karakter = "Ş") veya (karakter = "Ü")) olsun
        res'i döndür
    }

    rakam_mı fonksiyon olsun karakter alsın {
        res = ((karakter >= "0") ve (karakter <= "9")) olsun
        res'i döndür
    }

    sayı_token_mu fonksiyon olsun kelime alsın {
        n = uzunluk(kelime) olsun
        (n = 0) ise {
            r = 0 olsun
            r'yi döndür
        }
        i = 0'dan (n - 1)'e kadar {
            (kendisi.rakam_mı(kelime[i])) ise { } yoksa {
                r = 0 olsun
                r'yi döndür
            }
        }
        r = 1 olsun
        r'yi döndür
    }

    kısaltma_mı fonksiyon olsun kelime alsın {
        res = ((kelime = "prof") veya (kelime = "dr") veya (kelime = "doç") veya (kelime = "av") veya (kelime = "albay") veya (kelime = "yb") veya (kelime = "bb")) olsun
        res'i döndür
    }

    ner_etiket fonksiyon olsun kelime, önceki, önceki_nokta alsın {
        k_küçük = küçük_harf(kelime) olsun
        
        (kendisi.sayı_token_mu(kelime)) ise {
            r = "SAYI" olsun
            r'yi döndür
        }
        
        (içeriyor(AY_LİSTESİ, k_küçük)) ise {
            r = "TARİH" olsun
            r'yi döndür
        }
        
        (içeriyor(KURUM_LİSTESİ, k_küçük)) ise {
            r = "KURUM" olsun
            r'yi döndür
        }
        
        (içeriyor(BİLİNEN_YERLER, k_küçük)) ise {
            r = "YER" olsun
            r'yi döndür
        }
        
        c0 = kelime[0] olsun
        ilk_büyük = kendisi.büyük_harf_mi(c0) olsun
        
        (ilk_büyük) ise {
            (önceki_nokta = 1) ise {
                (kendisi.kısaltma_mı(küçük_harf(önceki))) ise {
                    r = "KİŞİ" olsun
                    r'yi döndür
                }
                r = "O" olsun
                r'yi döndür
            } yoksa {
                ((önceki = "Sayın") veya (önceki = "Prof.") veya (önceki = "Dr.") veya (önceki = "Bay") veya (önceki = "Bayan")) ise {
                    r = "KİŞİ" olsun
                    r'yi döndür
                }
                r = "KİŞİ" olsun
                r'yi döndür
            }
        }
        
        r = "O" olsun
        r'yi döndür
    }

    ner_etiketle fonksiyon olsun tokens alsın {
        n = uzunluk(tokens) olsun
        (n = 0) ise {
            r = [] olsun
            r'yi döndür
        }
        sonuç = [] olsun
        önceki = "" olsun
        önceki_nokta = 1 olsun
        
        i = 0'dan (n - 1)'e kadar {
            t = tokens[i] olsun
            etiket = kendisi.ner_etiket(t, önceki, önceki_nokta) olsun
            sonuç = listeye_ekle(sonuç, [t, etiket]) olsun
            
            önceki = t olsun
            önceki_nokta = 0 olsun
        }
        sonuç'u döndür
    }

    duygu_analiz fonksiyon olsun tokens alsın {
        poz_sayac = 0 olsun
        neg_sayac = 0 olsun
        n = uzunluk(tokens) olsun
        (n = 0) ise {
            sözlük = {} olsun
            sözlük["skor"] = 0.0
            sözlük["etiket"] = "NÖTR"
            sözlük["pozitif_sayı"] = 0
            sözlük["negatif_sayı"] = 0
            sözlük'ü döndür
        }
        
        i = 0'dan (n - 1)'e kadar {
            k = küçük_harf(tokens[i]) olsun
            kök = nlp_stemmer.stem(k) olsun
            
            (içeriyor(POZİTİF_KELİMELER, k) veya içeriyor(POZİTİF_KELİMELER, kök)) ise {
                poz_sayac = poz_sayac + 1 olsun
            }
            (içeriyor(NEGATİF_KELİMELER, k) veya içeriyor(NEGATİF_KELİMELER, kök)) ise {
                neg_sayac = neg_sayac + 1 olsun
            }
        }
        
        toplam = poz_sayac + neg_sayac olsun
        (toplam = 0) ise {
            sözlük = {} olsun
            sözlük["skor"] = 0.0
            sözlük["etiket"] = "NÖTR"
            sözlük["pozitif_sayı"] = 0
            sözlük["negatif_sayı"] = 0
            sözlük'ü döndür
        }
        
        skor = (poz_sayac - neg_sayac) / toplam olsun
        etiket = "NÖTR" olsun
        (skor > 0.15) ise { etiket = "POZİTİF" }
        (skor < -0.15) ise { etiket = "NEGATİF" }
        
        sözlük = {} olsun
        sözlük["skor"] = skor
        sözlük["etiket"] = etiket
        sözlük["pozitif_sayı"] = poz_sayac
        sözlük["negatif_sayı"] = neg_sayac
        sözlük'ü döndür
    }

    frekans_ekle fonksiyon olsun frekanslar, kelime alsın {
        k = küçük_harf(kelime) olsun
        mevcut = frekanslar[k] olsun
        (mevcut = Boş) ise {
            frekanslar[k] = 1
        } yoksa {
            frekanslar[k] = mevcut + 1
        }
        frekanslar'ı döndür
    }

    kelime_frekansları fonksiyon olsun tokens alsın {
        f = {} olsun
        n = uzunluk(tokens) olsun
        (n = 0) ise {
            r = {} olsun
            r'yi döndür
        }
        i = 0'dan (n - 1)'e kadar {
            f = kendisi.frekans_ekle(f, tokens[i]) olsun
        }
        f'yi döndür
    }

    metin_ortak_kelime fonksiyon olsun metin1, metin2 alsın {
        proc = metin_islemci() olsun
        t1 = proc.durak_filtrele(proc.tokenize(metin1)) olsun
        t2 = proc.durak_filtrele(proc.tokenize(metin2)) olsun
        
        n1 = uzunluk(t1) olsun
        n2 = uzunluk(t2) olsun
        (n1 = 0) ise {
            r = [] olsun
            r'yi döndür
        }
        (n2 = 0) ise {
            r = [] olsun
            r'yi döndür
        }
        ortak = [] olsun
        
        i = 0'dan (n1 - 1)'e kadar {
            k1 = t1[i] olsun
            j = 0'dan (n2 - 1)'e kadar {
                k2 = t2[j] olsun
                (k1 = k2) ise {
                    (içeriyor(ortak, k1)) ise { } yoksa {
                        ortak = listeye_ekle(ortak, k1) olsun
                    }
                }
            }
        }
        ortak'ı döndür
    }

    istatistik fonksiyon olsun metin alsın {
        proc = metin_islemci() olsun
        tokens = proc.tokenize(metin) olsun
        cümleler = proc.cümle_böl(metin) olsun
        duraksız = proc.durak_filtrele(tokens) olsun
        
        s = {} olsun
        s["karakter_sayısı"] = uzunluk(metin)
        s["kelime_sayısı"] = uzunluk(tokens)
        s["cümle_sayısı"] = uzunluk(cümleler)
        s["anlamlı_kelime_sayısı"] = uzunluk(duraksız)
        s'yi döndür
    }
}

// ─── Global Örnek ────────────────────────────────────────────────────────
nlp_analizci = metin_analizci() olsun

// ─── Geriye Dönük Uyumluluk Sarmalayıcıları ──────────────────────────────

pos_etiket fonksiyon olsun kelime alsın {
    ma = metin_analizci() olsun
    res = ma.pos_etiket(kelime) olsun
    res'i döndür
}

pos_etiketle fonksiyon olsun tokens alsın {
    ma = metin_analizci() olsun
    res = ma.pos_etiketle(tokens) olsun
    res'i döndür
}

ner_etiketle fonksiyon olsun tokens alsın {
    ma = metin_analizci() olsun
    res = ma.ner_etiketle(tokens) olsun
    res'i döndür
}

duygu_puan fonksiyon olsun tokens alsın {
    ma = metin_analizci() olsun
    res = ma.duygu_analiz(tokens) olsun
    res'i döndür
}

frekans_ekle fonksiyon olsun frekanslar, kelime alsın {
    ma = metin_analizci() olsun
    res = ma.frekans_ekle(frekanslar, kelime) olsun
    res'i döndür
}

kelime_frekansları fonksiyon olsun tokens alsın {
    ma = metin_analizci() olsun
    res = ma.kelime_frekansları(tokens) olsun
    res'i döndür
}

metin_ortak_kelime fonksiyon olsun metin1, metin2 alsın {
    ma = metin_analizci() olsun
    res = ma.metin_ortak_kelime(metin1, metin2) olsun
    res'i döndür
}

metin_istatistik fonksiyon olsun metin alsın {
    ma = metin_analizci() olsun
    res = ma.istatistik(metin) olsun
    res'i döndür
}
