// ══════════════════════════════════════════════════════════════════════════════
// yapay_zeka/yogun_katman.hb — Tam Bağlı (Dense/Linear) Sinir Ağı Katmanı
// Sürüm: 1.0.0
// ══════════════════════════════════════════════════════════════════════════════
//
// Kullanım:
//   katman = yogun_katman() olsun
//   katman.ilklendir(128, 64, "relu") olsun
//   cikis = katman.ileri(giris_vektoru) olsun
//   katman.geri(grad_cikis, ogrenme_hizi) olsun
// ══════════════════════════════════════════════════════════════════════════════

"yapay_zeka_temel.hb"'yi yükle
"optimizor.hb"'yi yükle

yogun_katman sınıf olsun {

    // ─── İlklendirme ──────────────────────────────────────────────────────────
    ilklendir fonksiyon olsun giris_n, cikis_n, aktivasyon alsın {
        kendisi.giris_n = giris_n olsun
        kendisi.cikis_n = cikis_n olsun
        kendisi.aktivasyon = aktivasyon olsun

        // Ağırlık matrisi — He ya da Xavier başlatma
        (aktivasyon = "relu") ise {
            kendisi.W = matris_he_ilklendir(cikis_n, giris_n) olsun
        } yoksa {
            kendisi.W = matris_xavier_ilklendir(cikis_n, giris_n) olsun
        }

        // Yanlılık vektörü — sıfır
        kendisi.b = vektor_olustur(cikis_n, 0.0) olsun

        // Son ileri geçiş girdisi (backprop için sakla)
        kendisi.son_giris = vektor_olustur(giris_n, 0.0) olsun
        kendisi.son_z = vektor_olustur(cikis_n, 0.0) olsun

        // Adam durumları
        kendisi.adam_w = adam_durum_olustur(cikis_n, giris_n) olsun
        kendisi.adam_b = adam_vektor_durum_olustur(cikis_n) olsun
    }

    // ─── İleri Geçiş: z = W*x + b → aktivasyon(z) ────────────────────────────
    ileri fonksiyon olsun giris alsın {
        kendisi.son_giris = giris olsun
        // z = W * x + b
        z = matris_vektor_carp(kendisi.W, giris) olsun
        z = vektor_topla(z, kendisi.b) olsun
        kendisi.son_z = z olsun

        // Aktivasyon uygula
        (kendisi.aktivasyon = "sigmoid") ise {
            n = vektor_uzunluk(z) olsun
            cikis = vektor_olustur(n, 0.0) olsun
            i = 0'dan (n - 1)'e kadar {
                vektor_ata(cikis, i, sigmoid(vektor_al(z, i))) olsun
            }
            cikis'i döndür
        }
        (kendisi.aktivasyon = "relu") ise {
            n = vektor_uzunluk(z) olsun
            cikis = vektor_olustur(n, 0.0) olsun
            i = 0'dan (n - 1)'e kadar {
                vektor_ata(cikis, i, relu(vektor_al(z, i))) olsun
            }
            cikis'i döndür
        }
        (kendisi.aktivasyon = "tanh") ise {
            n = vektor_uzunluk(z) olsun
            cikis = vektor_olustur(n, 0.0) olsun
            i = 0'dan (n - 1)'e kadar {
                vektor_ata(cikis, i, tanh_aktivasyon(vektor_al(z, i))) olsun
            }
            cikis'i döndür
        }
        (kendisi.aktivasyon = "gelu") ise {
            n = vektor_uzunluk(z) olsun
            cikis = vektor_olustur(n, 0.0) olsun
            i = 0'dan (n - 1)'e kadar {
                vektor_ata(cikis, i, gelu(vektor_al(z, i))) olsun
            }
            cikis'i döndür
        }
        (kendisi.aktivasyon = "softmax") ise {
            softmax(z)'yi döndür
        }
        (kendisi.aktivasyon = "log_softmax") ise {
            log_softmax(z)'yi döndür
        }
        // "yok" / diğer → doğrusal (linear)
        z'yi döndür
    }

    // ─── Aktivasyon Türevi ────────────────────────────────────────────────────
    aktivasyon_turevi fonksiyon olsun z_val alsın {
        (kendisi.aktivasyon = "sigmoid") ise {
            s = sigmoid(z_val) olsun
            s * (1.0 - s)'i döndür
        }
        (kendisi.aktivasyon = "relu") ise {
            z_val > 0 ise { 1.0'ı döndür }
            0.0'ı döndür
        }
        (kendisi.aktivasyon = "tanh") ise {
            t = tanh_aktivasyon(z_val) olsun
            1.0 - t * t'yi döndür
        }
        // softmax, log_softmax ve doğrusal — dışarıda zaten kayıp ile birleştirilir
        1.0'ı döndür
    }

    // ─── Geri Yayılım (Backpropagation) ──────────────────────────────────────
    // grad_cikis: sonraki katmandan gelen gradyan vektörü
    // Döndürür: bu katmanın giriş gradyanı (önceki katmana iletilir)
    geri fonksiyon olsun grad_cikis, ogrenme_hizi alsın {
        n_cikis = kendisi.cikis_n olsun
        n_giris = kendisi.giris_n olsun

        // Aktivasyon türevini uygula (element-wise zincir kuralı)
        // dL/dz = dL/da * da/dz
        grad_z = vektor_olustur(n_cikis, 0.0) olsun
        i = 0'dan (n_cikis - 1)'e kadar {
            z_val = vektor_al(kendisi.son_z, i) olsun
            g = vektor_al(grad_cikis, i) olsun
            t = kendisi.aktivasyon_turevi(z_val) olsun
            vektor_ata(grad_z, i, g * t) olsun
        }

        // Ağırlık gradyanı: dL/dW = grad_z ⊗ x^T
        grad_W = matris_olustur(n_cikis, n_giris, 0.0) olsun
        i = 0'dan (n_cikis - 1)'e kadar {
            gz = vektor_al(grad_z, i) olsun
            j = 0'dan (n_giris - 1)'e kadar {
                matris_ata(grad_W, i, j, gz * vektor_al(kendisi.son_giris, j)) olsun
            }
        }

        // Yanlılık gradyanı: dL/db = grad_z
        grad_b = grad_z olsun

        // Giriş gradyanı: dL/dx = W^T * grad_z
        W_transpoz = matris_transpoz(kendisi.W) olsun
        grad_giris = matris_vektor_carp(W_transpoz, grad_z) olsun

        // Adam ile güncelle
        adam_matris_guncelle(kendisi.W, grad_W, kendisi.adam_w, ogrenme_hizi) olsun
        adam_vektor_guncelle(kendisi.b, grad_b, kendisi.adam_b, ogrenme_hizi) olsun

        grad_giris'i döndür
    }
}
