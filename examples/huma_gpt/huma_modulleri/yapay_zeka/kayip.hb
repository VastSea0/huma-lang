// ══════════════════════════════════════════════════════════════════════════════
// yapay_zeka/kayip.hb — Kayıp Fonksiyonları
// Sürüm: 1.0.0
// ══════════════════════════════════════════════════════════════════════════════

"yapay_zeka_temel.hb"'yi yükle

// İkili Çapraz Entropi Kaybı (Binary Cross-Entropy)
// tahmin: [0,1] skalar, gercek: 0 veya 1
ikce_kayip fonksiyon olsun tahmin, gercek alsın {
    ikili_capraz_entropi(tahmin, gercek)'i döndür
}

// Kategorik Çapraz Entropi (Categorical Cross-Entropy)
// log_olasiliklar: log_softmax çıktısı (Vektor), gercek_sinif: indeks
kce_kayip fonksiyon olsun log_olasiliklar, gercek_sinif alsın {
    vektor_al(log_olasiliklar, gercek_sinif) * -1.0'ı döndür
}

// Ortalama Kare Hata Kaybı (MSE)
mse_kayip fonksiyon olsun tahmin, gercek alsın {
    fark = tahmin - gercek olsun
    fark * fark'ı döndür
}

// MSE gradyanı — backprop için
mse_gradyan fonksiyon olsun tahmin, gercek alsın {
    2.0 * (tahmin - gercek)'yi döndür
}

// Sigmoid + Binary Cross-Entropy gradyanı (birleştirilmiş, numerik kararlı)
ikce_gradyan fonksiyon olsun tahmin, gercek alsın {
    tahmin - gercek'i döndür
}

// Batch MSE — liste üzerinde ortalama
batch_mse fonksiyon olsun tahminler, gercekler alsın {
    mse(tahminler, gercekler)'yi döndür
}
