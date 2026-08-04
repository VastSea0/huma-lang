// Akıllı Zemin Stüdyosu — paket yöneticili gerçek AI + yerel web GUI
"model.hb"'yi yükle
"huma_sunucu"'yu yükle

yazdır "╭──────────────────────────────────────────────────────╮"
yazdır "│        HÜMA AKILLI ZEMİN STÜDYOSU                    │"
yazdır "│   Sinir ağı eğitiliyor • GUI hazırlanıyor            │"
yazdır "╰──────────────────────────────────────────────────────╯"

egitim = zemin_modeli_egit(120) olsun
model = egitim["model"] olsun

yazdır ""
yazdır "AI eğitim tamamlandı:"
yazdır "  İlk kayıp : " + egitim["ilk_kayıp"]
yazdır "  Son kayıp : " + egitim["son_kayıp"]
yazdır "  Doğruluk  : %" + (egitim["doğruluk"] * 100)

sunucu = Sunucu() olsun
sunucu.kur(8787)

sunucu.getir("/", fonksiyon olsun istek, cevap alsın {
    cevap.html(dosya_oku("arayuz.html"))
})

sunucu.getir("/api/model", fonksiyon olsun istek, cevap alsın {
    bilgi = {} olsun
    bilgi["motor"] = "Hüma yoğun sinir ağı + geri yayılım + Adam" 
    bilgi["girdi_sayısı"] = 3
    bilgi["eğitim_örneği"] = uzunluk(ZEMIN_VERISI)
    bilgi["epoch"] = egitim["epoch"]
    bilgi["ilk_kayıp"] = egitim["ilk_kayıp"]
    bilgi["son_kayıp"] = egitim["son_kayıp"]
    bilgi["doğruluk"] = egitim["doğruluk"]
    cevap.json(bilgi)
})

sunucu.gönder("/api/tahmin", fonksiyon olsun istek, cevap alsın {
    girdi = metinden_nesneye(istek.gövde) olsun
    test_kapsami = girdi["test_kapsamı"] olsun
    gecikme_ms = girdi["gecikme_ms"] olsun
    hata_orani = girdi["hata_oranı"] olsun

    skor = zemin_skorla(model, test_kapsami, gecikme_ms, hata_orani) olsun
    risk = (1.0 - skor) * 100.0 olsun
    seviye = "KRİTİK" olsun
    renk = "kırmızı" olsun

    skor >= 0.75 ise {
        seviye = "SAĞLAM" olsun
        renk = "yeşil" olsun
    } yoksa skor >= 0.50 ise {
        seviye = "İZLE" olsun
        renk = "sarı" olsun
    }

    sonuc = {} olsun
    sonuc["kararlılık_skoru"] = skor * 100.0
    sonuc["risk"] = risk
    sonuc["seviye"] = seviye
    sonuc["renk"] = renk
    sonuc["açıklama"] = "Sonuç, Hüma içinde eğitilen sinir ağının gerçek ileri geçişinden üretildi."
    cevap.json(sonuc)
})

yazdır ""
yazdır "✓ GUI hazır: http://127.0.0.1:8787"
yazdır "  Durdurmak için Ctrl+C"
sunucu.baslat()
