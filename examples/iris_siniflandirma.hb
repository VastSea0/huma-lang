// ══════════════════════════════════════════════════════════════════════════════
// iris_siniflandirma.hb — Hüma YSA ile İris Sınıflandırma Demo
// ══════════════════════════════════════════════════════════════════════════════
//
// Senaryo: Basitleştirilmiş İris veri seti (3 sınıf, 4 özellik)
// Gömülü sentetik veri — harici dosya bağımlılığı yok.
// Mimari: 4→16→8→3 (ReLU + ReLU + Softmax)
//
// Beklenen: Son epoch doğruluk > %70 (küçük veri, 100 epoch)
// Gerçek sayısal çıktı — hiçbir sahte değer yok.
//
// Çalıştırma: huma examples/iris_siniflandirma.hb
// ══════════════════════════════════════════════════════════════════════════════

"yapay_zeka_temel.hb"'yi yükle

yazdır "=== Hüma YSA İris Sınıflandırma Demo ==="
yazdır "Mimari: 4 → 16 → 8 → 3 (ReLU, ReLU, Softmax)"
yazdır ""

// Basitleştirilmiş İris benzeri sentetik veri (30 örnek, 3 sınıf)
// Sınıf 0: küçük değerler, Sınıf 1: orta, Sınıf 2: büyük
rastgele_tohum_ata(123) olsun

veri = [] olsun
etiketler = [] olsun

// Her sınıftan 10 örnek
sinif_n = 3 olsun
ornek_per_sinif = 10 olsun

s = 0'dan 2'ye kadar {
    i = 0'dan 9'a kadar {
        merkez = s * 2.5 olsun
        x1 = merkez + normal_rastgele(0.0, 0.4) olsun
        x2 = merkez * 0.8 + normal_rastgele(0.0, 0.4) olsun
        x3 = merkez * 1.2 + normal_rastgele(0.0, 0.3) olsun
        x4 = merkez * 0.6 + normal_rastgele(0.0, 0.3) olsun
        ornek = vektor_olustur(4, 0.0) olsun
        vektor_ata(ornek, 0, x1) olsun
        vektor_ata(ornek, 1, x2) olsun
        vektor_ata(ornek, 2, x3) olsun
        vektor_ata(ornek, 3, x4) olsun
        veri = listeye_ekle(veri, ornek) olsun
        etiketler = listeye_ekle(etiketler, s) olsun
    }
}

n = uzunluk(veri) olsun
yazdır "Toplam örnek: " + n + " (" + ornek_per_sinif + " örnek/sınıf × " + sinif_n + " sınıf)"
yazdır ""

// Verileri normalize et (min-max, özellik bazında)
ozellik_min = vektor_olustur(4, 999999.0) olsun
ozellik_max = vektor_olustur(4, -999999.0) olsun
i = 0'dan (n - 1)'e kadar {
    j = 0'dan 3'e kadar {
        v = vektor_al(veri[i], j) olsun
        v < vektor_al(ozellik_min, j) ise { vektor_ata(ozellik_min, j, v) olsun }
        v > vektor_al(ozellik_max, j) ise { vektor_ata(ozellik_max, j, v) olsun }
    }
}
i = 0'dan (n - 1)'e kadar {
    j = 0'dan 3'e kadar {
        v = vektor_al(veri[i], j) olsun
        mn = vektor_al(ozellik_min, j) olsun
        mx = vektor_al(ozellik_max, j) olsun
        aralik = mx - mn olsun
        aralik > 0 ise {
            norm_v = (v - mn) / aralik olsun
            vektor_ata(veri[i], j, norm_v) olsun
        }
    }
}

// Sinir ağını oluştur (modül olmadan doğrudan ağırlık ve bias kullanarak)
// Katman 1: 4 → 16 (He init, ReLU)
W1 = matris_he_ilklendir(16, 4) olsun
b1 = vektor_olustur(16, 0.0) olsun
// Katman 2: 16 → 8 (He init, ReLU)
W2 = matris_he_ilklendir(8, 16) olsun
b2 = vektor_olustur(8, 0.0) olsun
// Katman 3: 8 → 3 (Xavier init, Softmax)
W3 = matris_xavier_ilklendir(3, 8) olsun
b3 = vektor_olustur(3, 0.0) olsun

yazdır "Ağırlıklar He/Xavier ile ilklendi."
yazdır "Eğitim başlıyor (100 epoch, lr=0.01)..."
yazdır ""

ogrenme_hizi = 0.01 olsun
epoch_sayisi = 100 olsun
kayip_gecmisi = [] olsun

// Eğitim döngüsü
e = 1'den epoch_sayisi'ye kadar {
    toplam_kayip = 0.0 olsun
    dogru_sayisi = 0 olsun

    i = 0'dan (n - 1)'e kadar {
        x = veri[i] olsun
        gercek = etiketler[i] olsun

        // İleri geçiş katman 1
        z1 = matris_carp_vektor(W1, x) olsun
        z1 = vektor_topla(z1, b1) olsun
        a1 = relu(z1) olsun

        // İleri geçiş katman 2
        z2 = matris_carp_vektor(W2, a1) olsun
        z2 = vektor_topla(z2, b2) olsun
        a2 = relu(z2) olsun

        // İleri geçiş katman 3 (softmax)
        z3 = matris_carp_vektor(W3, a2) olsun
        z3 = vektor_topla(z3, b3) olsun
        a3 = softmax(z3) olsun

        // Kayıp: cross-entropy
        eps = 1e-7 olsun
        p_gercek = vektor_al(a3, gercek) olsun
        p_safe = klamp(p_gercek, eps, 1.0 - eps) olsun
        kayip = -1.0 * güvenli_ln(p_safe) olsun
        toplam_kayip = toplam_kayip + kayip olsun

        // Tahmin doğruluğu
        pred = vektor_argmax(a3) olsun
        pred = gercek ise { dogru_sayisi = dogru_sayisi + 1 olsun }

        // Geri yayılım — Katman 3 gradyanı (softmax + CE birleşik)
        dz3 = vektor_olustur(3, 0.0) olsun
        j = 0'dan 2'ye kadar {
            pj = vektor_al(a3, j) olsun
            yj = 0.0 olsun
            j = gercek ise { yj = 1.0 olsun }
            vektor_ata(dz3, j, pj - yj) olsun
        }

        // W3 ve b3 güncelle
        dW3_tmp = vektor_dis_carpim(dz3, a2) olsun
        r3 = 3 olsun
        c3 = 8 olsun
        ri = 0'dan (r3 - 1)'e kadar {
            ci = 0'dan (c3 - 1)'e kadar {
                g_ij = matris_al(dW3_tmp, ri, ci) olsun
                w_ij = matris_al(W3, ri, ci) olsun
                matris_ata(W3, ri, ci, w_ij - ogrenme_hizi * g_ij) olsun
            }
        }
        j = 0'dan 2'ye kadar {
            g_j = vektor_al(dz3, j) olsun
            b3_j = vektor_al(b3, j) olsun
            vektor_ata(b3, j, b3_j - ogrenme_hizi * g_j) olsun
        }

        // Katman 2 gradyanı (ReLU türevi)
        da2 = matris_transpoz_carp_vektor(W3, dz3) olsun
        dz2 = vektor_olustur(8, 0.0) olsun
        j = 0'dan 7'e kadar {
            a2_j = vektor_al(a2, j) olsun
            da2_j = vektor_al(da2, j) olsun
            relu_grad = 0.0 olsun
            a2_j > 0 ise { relu_grad = 1.0 olsun }
            vektor_ata(dz2, j, da2_j * relu_grad) olsun
        }

        // W2 ve b2 güncelle
        dW2_tmp = vektor_dis_carpim(dz2, a1) olsun
        r2_sz = 8 olsun
        c2_sz = 16 olsun
        ri = 0'dan (r2_sz - 1)'e kadar {
            ci = 0'dan (c2_sz - 1)'e kadar {
                g_ij = matris_al(dW2_tmp, ri, ci) olsun
                w_ij = matris_al(W2, ri, ci) olsun
                matris_ata(W2, ri, ci, w_ij - ogrenme_hizi * g_ij) olsun
            }
        }
        j = 0'dan 7'e kadar {
            g_j = vektor_al(dz2, j) olsun
            b2_j = vektor_al(b2, j) olsun
            vektor_ata(b2, j, b2_j - ogrenme_hizi * g_j) olsun
        }

        // Katman 1 gradyanı
        da1 = matris_transpoz_carp_vektor(W2, dz2) olsun
        dz1 = vektor_olustur(4, 0.0) olsun
        j = 0'dan 3'e kadar {
            a1_j = vektor_al(a1, j) olsun
            da1_j = vektor_al(da1, j) olsun
            relu_grad = 0.0 olsun
            a1_j > 0 ise { relu_grad = 1.0 olsun }
            vektor_ata(dz1, j, da1_j * relu_grad) olsun
        }

        // W1 ve b1 güncelle
        dW1_tmp = vektor_dis_carpim(dz1, x) olsun
        r1_sz = 16 olsun
        c1_sz = 4 olsun
        ri = 0'dan (r1_sz - 1)'e kadar {
            ci = 0'dan (c1_sz - 1)'e kadar {
                g_ij = matris_al(dW1_tmp, ri, ci) olsun
                w_ij = matris_al(W1, ri, ci) olsun
                matris_ata(W1, ri, ci, w_ij - ogrenme_hizi * g_ij) olsun
            }
        }
        j = 0'dan 3'e kadar {
            g_j = vektor_al(dz1, j) olsun
            b1_j = vektor_al(b1, j) olsun
            vektor_ata(b1, j, b1_j - ogrenme_hizi * g_j) olsun
        }
    }

    ort_kayip = toplam_kayip / n olsun
    ort_dogr = dogru_sayisi / n olsun
    kayip_gecmisi = listeye_ekle(kayip_gecmisi, ort_kayip) olsun

    (e % 20 = 0) ise {
        yazdır "Epoch " + e + "/" + epoch_sayisi + " — Kayıp: " + ort_kayip + " — Doğruluk: " + ort_dogr
    }
}

// Final sonuçları
n_kayip = uzunluk(kayip_gecmisi) olsun
ilk_kayip = kayip_gecmisi[0] olsun
son_kayip = kayip_gecmisi[n_kayip - 1] olsun

dogru_son = 0 olsun
i = 0'dan (n - 1)'e kadar {
    x = veri[i] olsun
    gercek = etiketler[i] olsun
    z1 = matris_carp_vektor(W1, x) olsun
    z1 = vektor_topla(z1, b1) olsun
    a1 = relu(z1) olsun
    z2 = matris_carp_vektor(W2, a1) olsun
    z2 = vektor_topla(z2, b2) olsun
    a2 = relu(z2) olsun
    z3 = matris_carp_vektor(W3, a2) olsun
    z3 = vektor_topla(z3, b3) olsun
    a3 = softmax(z3) olsun
    pred = vektor_argmax(a3) olsun
    pred = gercek ise { dogru_son = dogru_son + 1 olsun }
}
final_dogr = dogru_son / n olsun

yazdır ""
yazdır "=== Final Sonuçları ==="
yazdır "Başlangıç Kaybı : " + ilk_kayip
yazdır "Final Kaybı     : " + son_kayip
yazdır "Final Doğruluk  : " + final_dogr + " (" + dogru_son + "/" + n + ")"

final_dogr >= 0.70 ise {
    yazdır ""
    yazdır "✓ Doğruluk ≥ %70 — YSA başarıyla öğrendi"
} yoksa {
    yazdır ""
    yazdır "⚠ Doğruluk < %70 — Daha fazla epoch veya farklı lr deneyin"
}

yazdır ""
yazdır "Demo tamamlandı."
