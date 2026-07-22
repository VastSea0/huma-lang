// ══════════════════════════════════════════════════════════════════════════════
// yapay_zeka/optimizor.hb — Gradient Descent Optimizörleri
// Sürüm: 1.0.0
// ══════════════════════════════════════════════════════════════════════════════
// Fonksiyonlar matris/vektör nesneleri üzerinde yerinde (in-place) çalışır.
// ══════════════════════════════════════════════════════════════════════════════

"yapay_zeka_temel.hb"'yi yükle

// ─── SGD (Stochastic Gradient Descent) ────────────────────────────────────────
// matris_agirliklar: Matris, grad_matris: Matris, ogrenme_hizi: Sayı
sgd_matris_guncelle fonksiyon olsun agirliklar, gradyan, ogrenme_hizi alsın {
    boyut = matris_boyutu(agirliklar) olsun
    r = boyut[0] olsun
    c = boyut[1] olsun
    i = 0'dan (r - 1)'e kadar {
        j = 0'dan (c - 1)'e kadar {
            w = matris_al(agirliklar, i, j) olsun
            g = matris_al(gradyan, i, j) olsun
            matris_ata(agirliklar, i, j, w - ogrenme_hizi * g) olsun
        }
    }
    agirliklar'ı döndür
}

// vektör yanlılıklar için SGD
sgd_vektor_guncelle fonksiyon olsun yanlilik, gradyan, ogrenme_hizi alsın {
    n = vektor_uzunluk(yanlilik) olsun
    i = 0'dan (n - 1)'e kadar {
        w = vektor_al(yanlilik, i) olsun
        g = vektor_al(gradyan, i) olsun
        vektor_ata(yanlilik, i, w - ogrenme_hizi * g) olsun
    }
    yanlilik'ı döndür
}

// ─── Adam Optimizör ────────────────────────────────────────────────────────────
// adam_durum_olustur — matris ağırlıkları için Adam durumu (m, v, t)
adam_durum_olustur fonksiyon olsun satirlar, sutunlar alsın {
    durum = {} olsun
    durum["m"] = matris_olustur(satirlar, sutunlar, 0.0)
    durum["v"] = matris_olustur(satirlar, sutunlar, 0.0)
    durum["t"] = 0
    durum'u döndür
}

adam_vektor_durum_olustur fonksiyon olsun boyut alsın {
    durum = {} olsun
    durum["m"] = vektor_olustur(boyut, 0.0)
    durum["v"] = vektor_olustur(boyut, 0.0)
    durum["t"] = 0
    durum'u döndür
}

// Adam güncelleme — matris ağırlıkları
// beta1=0.9, beta2=0.999, eps=1e-8 (standart değerler)
adam_matris_guncelle fonksiyon olsun agirliklar, gradyan, durum, ogrenme_hizi alsın {
    beta1 = 0.9 olsun
    beta2 = 0.999 olsun
    eps = 1e-8 olsun
    durum["t"] = durum["t"] + 1 olsun
    t = durum["t"] olsun
    m = durum["m"] olsun
    v = durum["v"] olsun
    boyut = matris_boyutu(agirliklar) olsun
    r = boyut[0] olsun
    c = boyut[1] olsun
    i = 0'dan (r - 1)'e kadar {
        j = 0'dan (c - 1)'e kadar {
            g = matris_al(gradyan, i, j) olsun
            m_val = beta1 * matris_al(m, i, j) + (1.0 - beta1) * g olsun
            v_val = beta2 * matris_al(v, i, j) + (1.0 - beta2) * g * g olsun
            matris_ata(m, i, j, m_val) olsun
            matris_ata(v, i, j, v_val) olsun
            m_hat = m_val / (1.0 - üs(beta1, t)) olsun
            v_hat = v_val / (1.0 - üs(beta2, t)) olsun
            w = matris_al(agirliklar, i, j) olsun
            matris_ata(agirliklar, i, j, w - ogrenme_hizi * m_hat / (karekök(v_hat) + eps)) olsun
        }
    }
    agirliklar'ı döndür
}

// Adam güncelleme — vektör yanlılıkları
adam_vektor_guncelle fonksiyon olsun yanlilik, gradyan, durum, ogrenme_hizi alsın {
    beta1 = 0.9 olsun
    beta2 = 0.999 olsun
    eps = 1e-8 olsun
    durum["t"] = durum["t"] + 1 olsun
    t = durum["t"] olsun
    m = durum["m"] olsun
    v = durum["v"] olsun
    n = vektor_uzunluk(yanlilik) olsun
    i = 0'dan (n - 1)'e kadar {
        g = vektor_al(gradyan, i) olsun
        m_val = beta1 * vektor_al(m, i) + (1.0 - beta1) * g olsun
        v_val = beta2 * vektor_al(v, i) + (1.0 - beta2) * g * g olsun
        vektor_ata(m, i, m_val) olsun
        vektor_ata(v, i, v_val) olsun
        m_hat = m_val / (1.0 - üs(beta1, t)) olsun
        v_hat = v_val / (1.0 - üs(beta2, t)) olsun
        w = vektor_al(yanlilik, i) olsun
        vektor_ata(yanlilik, i, w - ogrenme_hizi * m_hat / (karekök(v_hat) + eps)) olsun
    }
    yanlilik'ı döndür
}
