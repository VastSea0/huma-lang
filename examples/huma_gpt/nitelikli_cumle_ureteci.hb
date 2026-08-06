// ══════════════════════════════════════════════════════════════════════════════
// examples / huma_gpt / nitelikli_cumle_ureteci.hb
// Retrieval-Based Türkçe Bilgi Getirme Motoru & Kelime Akışı Üretici
// ══════════════════════════════════════════════════════════════════════════════

"dosya.hb"'yi yükle
"dizgi.hb"'yi yükle

bilgi_veritabani_raw = dosya_oku("nitelikli_turkce_bilgi.json")
bilgi_veritabani = metinden_nesneye(bilgi_veritabani_raw)

// Kullanıcı istemini analiz edip en uygun cevabı getir (retrieval)
cevap_getir fonksiyon olsun kullanici_istemi alsın {
    m_kucuk = küçük_harf(kırp(kullanici_istemi))

    n_kayit = uzunluk(bilgi_veritabani)
    
    en_iyi_skor = 0
    en_iyi_cevap = "Üzgünüm, bu konuda henüz yeterli bilgim bulunmuyor. Yapay zekâ, tarih, fizik, matematik, yazılım, felsefe veya biyoloji gibi konularda sorular sorabilirsiniz."

    i = 0'dan (n_kayit - 1)'e kadar {
        kayit = bilgi_veritabani[i]
        kaliplar = kayit["soru_kaliplari"]
        n_kalip = uzunluk(kaliplar)
        
        mevcut_skor = 0
        
        j = 0'dan (n_kalip - 1)'e kadar {
            kalip = kaliplar[j]
            içeriyor(m_kucuk, kalip) = 1 ise {
                // Eşleşen kalıbın uzunluğunu skor olarak kullan (daha uzun = daha spesifik)
                kalip_uzunluk = uzunluk(kalip)
                mevcut_skor = mevcut_skor + kalip_uzunluk
            }
        }
        
        mevcut_skor > en_iyi_skor ise {
            en_iyi_skor = mevcut_skor
            en_iyi_cevap = kayit["cevap"]
        }
    }
    
    en_iyi_cevap'ı döndür
}

// Tek kelime akışı: cevabın n. kelimesini döndür
// step = kaçıncı kelimeyi istediğimiz (0-indexed)
cevap_kelime_getir fonksiyon olsun tam_cevap, step alsın {
    kelimeler = böl(tam_cevap, " ")
    n = uzunluk(kelimeler)
    
    step < n ise {
        kelimeler[step]'i döndür
    } yoksa {
        ""'ı döndür
    }
}
