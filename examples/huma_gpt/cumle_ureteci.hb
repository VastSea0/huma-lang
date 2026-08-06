// ══════════════════════════════════════════════════════════════════════════════
// examples / huma_gpt / cumle_ureteci.hb
// Kelime Düzeyinde Hızlı Türkçe Otoregresif Cümle ve Tek-Kelime Üretici
// ══════════════════════════════════════════════════════════════════════════════

"dosya.hb"'yi yükle
"dizgi.hb"'yi yükle
"rastgele.hb"'yi yükle

// Corpus metnini ve kelime dizisini açılışta 1 kere önbelleğe yükle
corpus_metni_global = dosya_oku("../tor-qurtu/cikti/turkce_llm_corpus.txt")
tum_kelimeler_global = böl(corpus_metni_global, " ")
toplam_k_global = uzunluk(tum_kelimeler_global)

// Gerçek Zamanlı AI Modeli: Bir Sonraki Kelimeyi Tahmin Etme (Yüksek Hızlı)
otoregresif_tek_kelime_tahmin_et fonksiyon olsun mevcut_metin alsın {
    mevcut_kelimeler = böl(kırp(mevcut_metin), " ")
    u_mevcut = uzunluk(mevcut_kelimeler)

    u_mevcut = 0 ise {
        r_idx = r_tamsayı(0, toplam_k_global - 1)
        tum_kelimeler_global[r_idx]'i döndür
    }

    son_kelime = küçük_harf(mevcut_kelimeler[u_mevcut - 1])
    adaylar = []

    // Corpus üzerinde son kelimeden sonra gelen kelimeleri arayıp olasılık adaylarını topla
    c_idx = 0'dan (toplam_k_global - 2)'e kadar {
        küçük_harf(tum_kelimeler_global[c_idx]) = son_kelime ise {
            sonraki = tum_kelimeler_global[c_idx + 1]
            adaylar = listeye_ekle(adaylar, sonraki)
        }
    }

    n_aday = uzunluk(adaylar)

    n_aday > 0 ise {
        secilen_idx = r_tamsayı(0, n_aday - 1)
        adaylar[secilen_idx]'i döndür
    } yoksa {
        r_idx = r_tamsayı(0, toplam_k_global - 1)
        tum_kelimeler_global[r_idx]'i döndür
    }
}

otoregresif_cumle_uret fonksiyon olsun corpus_yolu, istem, hedef_kelime_sayisi alsın {
    "💬 [CÜMLE ÜRETİCİ] İstem: \"" + istem + "\""'u yazdır

    istem_kelimeler = böl(kırp(istem), " ")
    u_istem = uzunluk(istem_kelimeler)

    uretilen_liste = []
    
    i = 0'dan (u_istem - 1)'e kadar {
        uretilen_liste = listeye_ekle(uretilen_liste, istem_kelimeler[i])
    }

    k_step = 0'dan (hedef_kelime_sayisi - 1)'e kadar {
        mevcut_metin_str = birleştir(uretilen_liste, " ")
        sonraki_kelime = otoregresif_tek_kelime_tahmin_et(mevcut_metin_str)
        uretilen_liste = listeye_ekle(uretilen_liste, sonraki_kelime)
    }

    tam_cumle = birleştir(uretilen_liste, " ")
    
    "   ✨ Üretilen Tam Türkçe Cümle / Paragraf:"'ı yazdır
    "   \"" + tam_cumle + "\""'ı yazdır

    tam_cumle'yi döndür
}
