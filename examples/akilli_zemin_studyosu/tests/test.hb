"model.hb"'yi yükle

yazdır "Hüma AI paket testi çalışıyor..."
egitim = zemin_modeli_egit(80) olsun

egitim["son_kayıp"] >= egitim["ilk_kayıp"] ise {
    hata("AI eğitimi kaybı azaltmadı") olsun
}

egitim["doğruluk"] < 0.80 ise {
    hata("AI doğruluğu %80 eşiğinin altında") olsun
}

iyi = zemin_skorla(egitim["model"], 95, 20, 0.5) olsun
riskli = zemin_skorla(egitim["model"], 35, 240, 12) olsun

iyi <= riskli ise {
    hata("Model sağlam ve riskli zemini ayıramadı") olsun
}

yazdır "✓ Kayıp azaldı: " + egitim["ilk_kayıp"] + " -> " + egitim["son_kayıp"]
yazdır "✓ Doğruluk: %" + (egitim["doğruluk"] * 100)
yazdır "✓ Sağlam skor: %" + (iyi * 100)
yazdır "✓ Riskli skor: %" + (riskli * 100)
