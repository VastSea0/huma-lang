// Akıllı Zemin Stüdyosu — gerçek native GUI + Hüma sinir ağı
"model.hb"'yi yükle
"gui"'yi yükle

yazdır "Hüma native GUI hazırlanıyor..."
egitim = zemin_modeli_egit(120) olsun
model = egitim["model"] olsun
test_kapsami = 88.0 olsun
gecikme_ms = 55.0 olsun
hata_orani = 1.5 olsun

çizim_fks fonksiyon olsun {
    etiket("HÜMA AKILLI ZEMİN STÜDYOSU", "başlık")
    etiket("Yerel pencere • Hüma YSA + Adam • Gerçek ileri geçiş", "kalın")
    ayraç_çiz()

    etiket("Test kapsamı: " + test_kapsami + "%")
    test_kapsami = kaydırıcı_ekle(test_kapsami, 0, 100)
    etiket("P95 gecikme: " + gecikme_ms + " ms")
    gecikme_ms = kaydırıcı_ekle(gecikme_ms, 1, 300)
    etiket("Hata oranı: " + hata_orani + "%")
    hata_orani = kaydırıcı_ekle(hata_orani, 0, 25)

    skor = zemin_skorla(model, test_kapsami, gecikme_ms, hata_orani) olsun
    etiket("Kararlılık skoru: %" + (skor * 100), "başlık")
    etiket("Tahmini risk: %" + ((1.0 - skor) * 100))
    skor >= 0.75 ise {
        etiket("DURUM: SAĞLAM", 60, 220, 150)
    } yoksa skor >= 0.50 ise {
        etiket("DURUM: İZLE", 240, 190, 70)
    } yoksa {
        etiket("DURUM: KRİTİK", 240, 70, 70)
    }

    ayraç_çiz()
    etiket("Eğitim: " + egitim["epoch"] + " epoch • " + egitim["doğruluk"] * 100 + "% doğruluk")
    etiket("Kayıp: " + egitim["ilk_kayıp"] + " → " + egitim["son_kayıp"])
}

pencere_oluştur("Hüma Akıllı Zemin Stüdyosu", 820.0, 620.0, çizim_fks)
