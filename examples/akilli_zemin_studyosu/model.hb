// Gerçek eğitim ve çıkarım katmanı. Sabit tohum aynı modeli tekrar üretir.
"yapay_zeka"'yı yükle

ZEMIN_VERISI = [
    [0.98, 0.94, 0.99], [0.91, 0.88, 0.97], [0.86, 0.82, 0.95],
    [0.79, 0.91, 0.92], [0.88, 0.70, 0.96], [0.74, 0.80, 0.90],
    [0.81, 0.76, 0.86], [0.70, 0.72, 0.94], [0.93, 0.67, 0.91],
    [0.42, 0.32, 0.68], [0.55, 0.18, 0.79], [0.31, 0.71, 0.52],
    [0.60, 0.27, 0.62], [0.48, 0.53, 0.44], [0.67, 0.29, 0.48],
    [0.38, 0.61, 0.57], [0.58, 0.46, 0.51], [0.29, 0.24, 0.83]
] olsun

ZEMIN_ETIKETLERI = [
    [1], [1], [1], [1], [1], [1], [1], [1], [1],
    [0], [0], [0], [0], [0], [0], [0], [0], [0]
] olsun

zemin_modeli_egit fonksiyon olsun epoch_sayisi alsın {
    rastgele_tohum_ata(2026) olsun
    model = sinir_agi() olsun
    model.ilklendir() olsun
    model.katman_ekle(3, 4, "tanh") olsun
    model.katman_ekle(4, 1, "sigmoid") olsun
    model.egit(ZEMIN_VERISI, ZEMIN_ETIKETLERI, epoch_sayisi, 0.025) olsun

    dogru_sayisi = 0 olsun
    i = 0 olsun
    i < uzunluk(ZEMIN_VERISI) olduğu sürece {
        tahmin = model.tahmin_et(listeye_vektor(ZEMIN_VERISI[i])) olsun
        skor = vektor_al(tahmin, 0) olsun
        tahmin_sinifi = 0 olsun
        skor >= 0.5 ise { tahmin_sinifi = 1 olsun }
        tahmin_sinifi = ZEMIN_ETIKETLERI[i][0] ise { dogru_sayisi = dogru_sayisi + 1 olsun }
        i = i + 1 olsun
    }

    kayiplar = model.egitim_kayiplari olsun
    sonuc = {} olsun
    sonuc["model"] = model
    sonuc["epoch"] = epoch_sayisi
    sonuc["ilk_kayıp"] = kayiplar[0]
    sonuc["son_kayıp"] = kayiplar[uzunluk(kayiplar) - 1]
    sonuc["doğruluk"] = dogru_sayisi / uzunluk(ZEMIN_VERISI)
    sonuc'u döndür
}

zemin_skorla fonksiyon olsun model, test_kapsami, gecikme_ms, hata_orani alsın {
    kapsam = test_kapsami / 100.0 olsun
    gecikme_sagligi = 1.0 - (gecikme_ms / 300.0) olsun
    hata_sagligi = 1.0 - (hata_orani / 100.0) olsun

    kapsam < 0 ise { kapsam = 0 olsun }
    kapsam > 1 ise { kapsam = 1 olsun }
    gecikme_sagligi < 0 ise { gecikme_sagligi = 0 olsun }
    gecikme_sagligi > 1 ise { gecikme_sagligi = 1 olsun }
    hata_sagligi < 0 ise { hata_sagligi = 0 olsun }
    hata_sagligi > 1 ise { hata_sagligi = 1 olsun }

    vektor = listeye_vektor([kapsam, gecikme_sagligi, hata_sagligi]) olsun
    tahmin = model.tahmin_et(vektor) olsun
    vektor_al(tahmin, 0)'ı döndür
}
