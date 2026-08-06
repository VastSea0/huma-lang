// ══════════════════════════════════════════════════════════════════════════════
// examples / huma_gpt / uretim.hb
// Otoregresif Türkçe Metin Üreteci (Autoregressive Generation)
// ══════════════════════════════════════════════════════════════════════════════

"tokenizer.hb"'yi yükle

metin_uret fonksiyon olsun model, istem, uretilecek_token_sayisi, pencere_boyutu, sozluk_boyutu alsın {
    "💬 [GPT METİN ÜRETİMİ] İstem: \"" + istem + "\""'u yazdır

    token_ids = metni_tokenlestir(istem)
    
    uzunluk(token_ids) < pencere_boyutu ise {
        eksik = pencere_boyutu - uzunluk(token_ids)
        k = 0'dan (eksik - 1)'e kadar {
            token_ids = listeye_ekle(token_ids, 65)
        }
    }

    uretilen_token_listesi = token_ids

    t_sayi = 0'dan (uretilecek_token_sayisi - 1)'e kadar {
        curr_len = uzunluk(uretilen_token_listesi)
        
        x_dizi = []
        bas = curr_len - pencere_boyutu
        j = 0'dan (pencere_boyutu - 1)'e kadar {
            val = uretilen_token_listesi[bas + j] / sozluk_boyutu
            x_dizi = listeye_ekle(x_dizi, val)
        }

        giris_v = listeye_vektor(x_dizi)
        tahmin_v = model.tahmin_et(giris_v)
        tahmin_val = vektor_al(tahmin_v, 0)

        // Tahmin edilen BPE token ID
        yeni_token_id = yuvarla(tahmin_val * (sozluk_boyutu - 1))
        
        yeni_token_id < 1 ise { yeni_token_id = 65 }
        yeni_token_id >= sozluk_boyutu ise { yeni_token_id = 100 }

        uretilen_token_listesi = listeye_ekle(uretilen_token_listesi, yeni_token_id)
    }

    sonuc_metin = tokenleri_metne_donustur(uretilen_token_listesi)
    
    "   ✨ GPT Tahmini Üretilen Metin: "'i yazdır
    sonuc_metin'i yazdır

    sonuc_metin'i döndür
}
