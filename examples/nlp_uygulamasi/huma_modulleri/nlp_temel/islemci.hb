// ══════════════════════════════════════════════════════════════════════════════
// nlp_temel/islemci.hb — Metin Temizleme ve Tokenizasyon (OOP & Modüler)
// ══════════════════════════════════════════════════════════════════════════════

"sabitler.hb"'yi yükle

metin_islemci sınıf olsun {
    temizle fonksiyon olsun metin alsın {
        t = küçük_harf(metin) olsun
        t = değiştir(t, "\t", " ") olsun
        t = değiştir(t, "\r", "") olsun
        t = değiştir(t, "\n", " ") olsun
        res = kırp(t) olsun
        res'i döndür
    }

    tokenize fonksiyon olsun metin alsın {
        t = kendisi.temizle(metin) olsun
        n = uzunluk(t) olsun
        (n = 0) ise {
            r = [] olsun
            r'yi döndür
        }
        tokens = [] olsun
        gecici = "" olsun
        
        i = 0'dan (n - 1)'e kadar {
            c = t[i] olsun
            ((c = " ") veya (c = ",") veya (c = ".") veya (c = "!") veya (c = "?") veya (c = ":") veya (c = ";") veya (c = "(") veya (c = ")") veya (c = "\"") veya (c = "'")) ise {
                (uzunluk(gecici) > 0) ise {
                    tokens = listeye_ekle(tokens, gecici) olsun
                    gecici = "" olsun
                }
            } yoksa {
                gecici = gecici + c olsun
            }
        }
        (uzunluk(gecici) > 0) ise {
            tokens = listeye_ekle(tokens, gecici) olsun
        }
        tokens'ı döndür
    }

    karakter_tokenize fonksiyon olsun metin alsın {
        n = uzunluk(metin) olsun
        (n = 0) ise {
            r = [] olsun
            r'yi döndür
        }
        dizi = [] olsun
        i = 0'dan (n - 1)'e kadar {
            dizi = listeye_ekle(dizi, metin[i]) olsun
        }
        dizi'yi döndür
    }

    durak_mı fonksiyon olsun kelime alsın {
        k = küçük_harf(kelime) olsun
        res = içeriyor(DURAK_LİSTESİ, k) olsun
        res'i döndür
    }

    durak_filtrele fonksiyon olsun tokens alsın {
        n = uzunluk(tokens) olsun
        (n = 0) ise {
            r = [] olsun
            r'yi döndür
        }
        sonuç = [] olsun
        i = 0'dan (n - 1)'e kadar {
            t = tokens[i] olsun
            (kendisi.durak_mı(t)) ise {
                devam
            } yoksa {
                sonuç = listeye_ekle(sonuç, t) olsun
            }
        }
        sonuç'u döndür
    }

    cümle_böl fonksiyon olsun metin alsın {
        n = uzunluk(metin) olsun
        (n = 0) ise {
            r = [] olsun
            r'yi döndür
        }
        cümleler = [] olsun
        gecici = "" olsun
        
        i = 0'dan (n - 1)'e kadar {
            c = metin[i] olsun
            gecici = gecici + c olsun
            ((c = ".") veya (c = "!") veya (c = "?")) ise {
                gecici_kırp = kırp(gecici) olsun
                (uzunluk(gecici_kırp) > 0) ise {
                    cümleler = listeye_ekle(cümleler, gecici_kırp) olsun
                }
                gecici = "" olsun
            }
        }
        kalan = kırp(gecici) olsun
        (uzunluk(kalan) > 0) ise {
            cümleler = listeye_ekle(cümleler, kalan) olsun
        }
        cümleler'i döndür
    }
}

// ─── Geriye Dönük Uyumluluk Sarmalayıcıları ──────────────────────────────

nlp_temizle fonksiyon olsun metin alsın {
    proc = metin_islemci() olsun
    res = proc.temizle(metin) olsun
    res'i döndür
}

tokenize fonksiyon olsun metin alsın {
    proc = metin_islemci() olsun
    res = proc.tokenize(metin) olsun
    res'i döndür
}

nlp_tokenize fonksiyon olsun metin alsın {
    proc = metin_islemci() olsun
    res = proc.tokenize(metin) olsun
    res'i döndür
}

karakter_tokenize fonksiyon olsun metin alsın {
    proc = metin_islemci() olsun
    res = proc.karakter_tokenize(metin) olsun
    res'i döndür
}

durak_mı fonksiyon olsun kelime alsın {
    proc = metin_islemci() olsun
    res = proc.durak_mı(kelime) olsun
    res'i döndür
}

durak_kelime_filtrele fonksiyon olsun tokens alsın {
    proc = metin_islemci() olsun
    res = proc.durak_filtrele(tokens) olsun
    res'i döndür
}

cümle_böl fonksiyon olsun metin alsın {
    proc = metin_islemci() olsun
    res = proc.cümle_böl(metin) olsun
    res'i döndür
}
