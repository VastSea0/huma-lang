// ══════════════════════════════════════════════════════════════════════════════
// examples / huma_gpt / gercek_llm.hb
// 2. Derece (Trigram) Dinamik Yapay Sinir Ağı Üreteci
// ══════════════════════════════════════════════════════════════════════════════

"yapay_zeka"'yı yükle
"dosya.hb"'yi yükle
"dizgi.hb"'yi yükle

"🧠 [TRİGRAM GENERATIVE LLM] Dinamik Türkçe Dil Modeli başlatılıyor..."'u yazdır

egitim_ham = dosya_oku("egitim_metni.txt")
egitim_kelimeleri = böl(kırp(egitim_ham), " ")
toplam_kelime = uzunluk(egitim_kelimeleri)
"   Eğitim verisi kelime sayısı: " + toplam_kelime'yi yazdır

// Sözlük
llm_sozluk = []
si = 0'dan (toplam_kelime - 1)'e kadar {
    kk = egitim_kelimeleri[si]
    uzunluk(kk) > 0 ise {
        bulunan = 0
        sj = 0'dan (uzunluk(llm_sozluk) - 1)'e kadar {
            llm_sozluk[sj] == kk ise { bulunan = 1 }
        }
        bulunan == 0 ise {
            llm_sozluk = listeye_ekle(llm_sozluk, kk)
        }
    }
}
sozluk_boyutu = uzunluk(llm_sozluk)
"   Sözlük boyutu: " + sozluk_boyutu'nu yazdır

// İstem analiz edip en uygun başlangıç konusunu seçen fonksiyon
baslangic_kelimesi_sec fonksiyon olsun istem alsın {
    k_istem = küçük_harf(kırp(istem))

    içeriyor(k_istem, "yapay") = 1 ise { "Yapay"'ı döndür }
    içeriyor(k_istem, "zeka") = 1 ise { "Yapay"'ı döndür }
    içeriyor(k_istem, "ai") = 1 ise { "Yapay"'ı döndür }
    içeriyor(k_istem, "türkiye") = 1 ise { "Türkiye"'yi döndür }
    içeriyor(k_istem, "tarih") = 1 ise { "Türkiye"'yi döndür }
    içeriyor(k_istem, "atatürk") = 1 ise { "Mustafa"'yı döndür }
    içeriyor(k_istem, "yazılım") = 1 ise { "Yazılım"'ı döndür }
    içeriyor(k_istem, "fizik") = 1 ise { "Bilim"'i döndür }
    içeriyor(k_istem, "bilim") = 1 ise { "Bilim"'i döndür }
    içeriyor(k_istem, "felsefe") = 1 ise { "Felsefe"'yi döndür }
    içeriyor(k_istem, "şiir") = 1 ise { "Şiir"'i döndür }
    içeriyor(k_istem, "siir") = 1 ise { "Şiir"'i döndür }
    içeriyor(k_istem, "selam") = 1 ise { "Merhaba"'yı döndür }
    içeriyor(k_istem, "merhaba") = 1 ise { "Merhaba"'yı döndür }
    içeriyor(k_istem, "teşekkür") = 1 ise { "Teşekkür"'ü döndür }

    // Bilinmeyen veya rastgele istem için dinamik seçim
    rnd_idx = (uzunluk(k_istem) * 7) % sozluk_boyutu
    llm_sozluk[rnd_idx]'i döndür
}

// 2. Derece Trigram Adım Tahmini
llm_sonraki_kelime_dinamik fonksiyon olsun istem, uretilen_kelimeler, step alsın {
    n_uretilen = uzunluk(uretilen_kelimeler)

    // Adım 0: İstem konusuna göre başlangıç kelimesini ver
    step == 0 ise {
        w0 = baslangic_kelimesi_sec(istem)
        w0'ı döndür
    }

    // Adım > 0: Son 2 ya da 1 kelimeye bakarak Trigram adayları topla
    son_w = uretilen_kelimeler[n_uretilen - 1]
    onceki_w = ""
    n_uretilen >= 2 ise {
        onceki_w = uretilen_kelimeler[n_uretilen - 2]
    }

    adaylar = []

    // 1. Öncelik: (onceki_w, son_w) ikilisinden sonra gelen kelimeler (2nd-order Trigram)
    uzunluk(onceki_w) > 0 ise {
        bi = 0'dan (toplam_kelime - 3)'e kadar {
            w1 = egitim_kelimeleri[bi] olsun
            w2 = egitim_kelimeleri[bi + 1] olsun
            w3 = egitim_kelimeleri[bi + 2] olsun

            (küçük_harf(w1) == küçük_harf(onceki_w)) ve (küçük_harf(w2) == küçük_harf(son_w)) ise {
                adaylar = listeye_ekle(adaylar, w3)
            }
        }
    }

    // 2. Yedek: Eğer Trigram adayı çıkmadıysa son_w sonrası gelen kelimeler (Bigram fallback)
    uzunluk(adaylar) == 0 ise {
        bi = 0'dan (toplam_kelime - 2)'e kadar {
            w1 = egitim_kelimeleri[bi] olsun
            w2 = egitim_kelimeleri[bi + 1] olsun

            küçük_harf(w1) == küçük_harf(son_w) ise {
                adaylar = listeye_ekle(adaylar, w2)
            }
        }
    }

    n_aday = uzunluk(adaylar)

    // Eğer hiç aday kalmadıysa dur (EOS)
    n_aday == 0 ise {
        ""'ı döndür
    }

    // Filtreleme: Son üretilen kelimelerde bu aday var mı? (Tekrar cezası)
    temiz_adaylar = []
    ai = 0'dan (n_aday - 1)'e kadar {
        cand = adaylar[ai] olsun
        gecerli = 1
        ui = 0'dan (n_uretilen - 1)'e kadar {
            küçük_harf(uretilen_kelimeler[ui]) == küçük_harf(cand) ise {
                gecerli = 0
            }
        }
        gecerli == 1 ise {
            temiz_adaylar = listeye_ekle(temiz_adaylar, cand)
        }
    }

    n_temiz = uzunluk(temiz_adaylar)

    n_temiz > 0 ise {
        r_idx = rastgele_tamsayi(0, n_temiz - 1)
        temiz_adaylar[r_idx]'i döndür
    } yoksa {
        r_idx = rastgele_tamsayi(0, n_aday - 1)
        adaylar[r_idx]'i döndür
    }
}
