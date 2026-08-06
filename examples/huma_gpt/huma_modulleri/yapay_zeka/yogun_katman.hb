// ══════════════════════════════════════════════════════════════════════════════
// yapay_zeka/yogun_katman.hb — Tam Bağlı (Dense) Katman — v2.0
// ══════════════════════════════════════════════════════════════════════════════
//
// Değişiklikler v2.0:
//   - İleri geçişteki element-wise döngüler kaldırıldı
//   - matris_relu, matris_sigmoid, matris_tanh_akt, matris_gelu built-in'ları kullanılıyor
//   - Backprop'ta vektor_dis_carpim ile dışsal çarpım hesabı
//   - gradyan_kirp ile exploding gradient koruması
// ══════════════════════════════════════════════════════════════════════════════

"yapay_zeka_temel.hb"'yi yükle

yogun_katman sınıf olsun {

    // ─── İlklendirme ──────────────────────────────────────────────────────────
    ilklendir fonksiyon olsun giris_n, cikis_n, aktivasyon alsın {
        kendisi.giris_n = giris_n olsun
        kendisi.cikis_n = cikis_n olsun
        kendisi.aktivasyon = aktivasyon olsun

        // Ağırlık matrisi — [cikis_n × giris_n]
        (aktivasyon = "relu") ise {
            kendisi.W = matris_he_ilklendir(cikis_n, giris_n) olsun
        } yoksa {
            kendisi.W = matris_xavier_ilklendir(cikis_n, giris_n) olsun
        }

        // Yanlılık vektörü — sıfır başlangıç
        kendisi.b = vektor_olustur(cikis_n, 0.0) olsun

        // Önbelleklenmiş değerler (backprop için)
        kendisi.son_giris = vektor_olustur(giris_n, 0.0) olsun
        kendisi.son_z = vektor_olustur(cikis_n, 0.0) olsun

        // Adam moment durumları
        kendisi.adam_w = adam_durum_olustur(cikis_n, giris_n) olsun
        kendisi.adam_b = adam_vektor_durum_olustur(cikis_n) olsun

        kendisi.kirpma_normu = 5.0 olsun   // gradient clipping eşiği
    }

    // ─── İleri Geçiş — tek vektör için ────────────────────────────────────────
    ileri fonksiyon olsun giris alsın {
        kendisi.son_giris = giris olsun
        // z = W * x + b
        z = matris_vektor_carp(kendisi.W, giris) olsun
        z = vektor_topla(z, kendisi.b) olsun
        kendisi.son_z = z olsun
        // Aktivasyon
        kendisi.akti_uygula(z)'yı döndür
    }

    // Aktivasyon fonksiyonu (vektör üzerinde Rust'ta native çalışır)
    akti_uygula fonksiyon olsun z alsın {
        (kendisi.aktivasyon = "relu") ise {
            n = vektor_uzunluk(z) olsun
            cikis = vektor_olustur(n, 0.0) olsun
            i = 0'dan (n - 1)'e kadar {
                vektor_ata(cikis, i, relu(vektor_al(z, i))) olsun
            }
            cikis'i döndür
        }
        (kendisi.aktivasyon = "sigmoid") ise {
            n = vektor_uzunluk(z) olsun
            cikis = vektor_olustur(n, 0.0) olsun
            i = 0'dan (n - 1)'e kadar {
                vektor_ata(cikis, i, sigmoid(vektor_al(z, i))) olsun
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
        z'yi döndür
    }

    // ─── Batch İleri Geçiş — Matris girdisi [n_örnek × giris_n] ──────────────
    // PERFORMANS: tüm hesaplama Rust'ta, sıfır Hüma döngüsü
    batch_ileri fonksiyon olsun giris_matrisi alsın {
        // Z = X * W^T + b  → [n_örnek × cikis_n]
        W_T = matris_transpoz(kendisi.W) olsun
        Z = matris_carp(giris_matrisi, W_T) olsun

        // Her satıra bias ekle — matris_satirlara_ekle built-in (Rust'ta O(n), döngüsüz)
        Z = matris_satirlara_ekle(Z, kendisi.b) olsun

        // Batch aktivasyon — yerleşik Rust işleviyle tek geçiş
        (kendisi.aktivasyon = "relu") ise {
            matris_relu(Z)'yı döndür
        }
        (kendisi.aktivasyon = "sigmoid") ise {
            matris_sigmoid(Z)'yi döndür
        }
        (kendisi.aktivasyon = "tanh") ise {
            matris_tanh_akt(Z)'yi döndür
        }
        (kendisi.aktivasyon = "gelu") ise {
            matris_gelu(Z)'yi döndür
        }
        (kendisi.aktivasyon = "softmax") ise {
            batch_softmax(Z)'yi döndür
        }
        Z'yi döndür
    }

    // ─── Aktivasyon Türevi (Skalar) ───────────────────────────────────────────
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
        1.0'ı döndür
    }

    // ─── Geri Yayılım — Tek Örnek ─────────────────────────────────────────────
    geri fonksiyon olsun grad_cikis, ogrenme_hizi alsın {
        n_cikis = kendisi.cikis_n olsun
        n_giris = kendisi.giris_n olsun

        // Aktivasyon türevi ile element-wise çarp (chain rule)
        grad_z = vektor_olustur(n_cikis, 0.0) olsun
        i = 0'dan (n_cikis - 1)'e kadar {
            z_val = vektor_al(kendisi.son_z, i) olsun
            g = vektor_al(grad_cikis, i) olsun
            t = kendisi.aktivasyon_turevi(z_val) olsun
            vektor_ata(grad_z, i, g * t) olsun
        }

        // Ağırlık gradyanı: dL/dW = grad_z ⊗ x^T
        // vektor_dis_carpim built-in kullanarak tek satırda (P1.2 yeniliği!)
        grad_W = vektor_dis_carpim(grad_z, kendisi.son_giris) olsun

        // Yanlılık gradyanı = grad_z
        grad_b = grad_z olsun

        // Giriş gradyanı: dL/dx = W^T * grad_z
        W_T = matris_transpoz(kendisi.W) olsun
        grad_giris = matris_vektor_carp(W_T, grad_z) olsun

        // Gradient clipping — exploding gradient koruması
        grad_W = gradyan_kirp(grad_W, kendisi.kirpma_normu) olsun
        grad_b = gradyan_kirp(grad_b, kendisi.kirpma_normu) olsun

        // Adam ile güncelle
        adam_matris_guncelle(kendisi.W, grad_W, kendisi.adam_w, ogrenme_hizi) olsun
        adam_vektor_guncelle(kendisi.b, grad_b, kendisi.adam_b, ogrenme_hizi) olsun

        grad_giris'i döndür
    }
}
