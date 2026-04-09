yükle "dizgi.hb";

hecele fonksiyon olsun kelime alsın {
    ünlüler = "aeıioöuüAEIİOÖUÜ" olsun
    konumlar = [] olsun
    i = 0 olsun
    boy = kelime'nin uzunluğu olsun

    // 1. ADIM: Ünlülerin yerlerini tespit et
    i < boy olduğu sürece {
        içeriyor_mu(ünlüler, kelime[i]) ise {
            konumlar'a [i]'yi ekle
        }
        i = i + 1 olsun
    }

    // 2. ADIM: Heceleri parçala
    heceler = [] olsun
    bas = 0 olsun
    k = 0 olsun
    unlu_sayisi = konumlar'ın uzunluğu'su olsun

    unlu_sayisi = 0 ise {
        [kelime]'yi döndür
    }

    k < (unlu_sayisi - 1) olduğu sürece {
        sonraki_unlu = konumlar[k + 1] olsun
        // Turkish hyphenation rule: cut before the last consonant of the next syllable
        kesme_noktası = sonraki_unlu - 1 olsun

        // Metin dilimleme
        parca = "" olsun
        j = bas olsun
        j < kesme_noktası olduğu sürece {
            parca = parca + kelime[j] olsun
            j = j + 1 olsun
        }
        
        heceler'e [parca]'yı ekle
        bas = kesme_noktası olsun
        k = k + 1 olsun
    }

    // Son parçayı ekle
    son_parca = "" olsun
    j = bas olsun
    j < boy olduğu sürece {
        son_parca = son_parca + kelime[j] olsun
        j = j + 1 olsun
    }
    heceler'e [son_parca]'yı ekle
    
    heceler'i döndür
}

// TEST
"--- Heceleme Testi ---"'yi yazdır
kelime = "bilgisayar" olsun
sonuc = hecele(kelime) olsun
(kelime + " -> " + sonuc)'u yazdır
