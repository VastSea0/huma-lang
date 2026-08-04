// ══════════════════════════════════════════════════════════════════════════════
// makine_ogrenimi.hb — Hüma Klasik Makine Öğrenimi Kütüphanesi
// Sürüm: 1.0.0
// ══════════════════════════════════════════════════════════════════════════════
//
// Sınıflar:
//   - lineer_regresyon      (gradient descent, MSE loss)
//   - lojistik_regresyon    (gradient descent, BCE loss, sigmoid)
//   - knn_siniflandirici    (k-NN, Öklit mesafe, çoğunluk oylaması)
//
// Fonksiyonlar:
//   - dogruluk_hesapla      (yüzde doğruluk)
//   - kesinlik_hesapla      (precision, ikili)
//   - duyarlilik_hesapla    (recall, ikili)
//   - f1_skoru              (F1 score, ikili)
//   - r2_skoru              (R² determinasyon katsayısı, regresyon için)
//   - karmasiklik_matrisi   (2×2 confusion matrix, ikili sınıflandırma)
//
// Bağımlılıklar: yapay_zeka_temel.hb (MSE, ikili_capraz_entropi)
//   Rust built-in: oklid_mesafe, sigmoid, vektor_olustur
// ══════════════════════════════════════════════════════════════════════════════

"yapay_zeka_temel.hb"'yi yükle

// ═══════════════════════════════════════════════════════════════════════════
// 1. Lineer Regresyon — Gradient Descent
// ═══════════════════════════════════════════════════════════════════════════
//
// Kullanım:
//   model = lineer_regresyon() olsun
//   model.ilklendir(ozellik_sayisi) olsun
//   model.egit(X_liste, y_liste, lr, epoch) olsun   // her örnek: X_liste[i] = vektör
//   tahmin = model.tahmin_et(x_vektoru) olsun
//
// X_liste: liste<vektör>, y_liste: liste<sayı>
// ─────────────────────────────────────────────────────────────────────────────
lineer_regresyon sınıf olsun {

    ilklendir fonksiyon olsun ozellik_n alsın {
        kendisi.n = ozellik_n olsun
        // Sıfır başlangıcı — bias dahil (son eleman)
        kendisi.agirliklar = vektor_olustur(ozellik_n + 1, 0.0) olsun
        kendisi.kayip_gecmisi = [] olsun
    }

    // tahmin_et(x) → y_hat (skaler)
    // x: ozellik_n uzunluklu vektör (bias eklenmeden verilir)
    tahmin_et fonksiyon olsun x alsın {
        n = kendisi.n olsun
        toplam = kendisi.agirliklar[n] olsun   // bias
        i = 0'dan (n - 1)'e kadar {
            toplam = toplam + kendisi.agirliklar[i] * x[i] olsun
        }
        toplam'ı döndür
    }

    // egit(X, y, lr, epochlar) → kayıp geçmişi listesi
    egit fonksiyon olsun X, y, lr, epochlar alsın {
        n = uzunluk(X) olsun
        n = 0 ise { []'i döndür }
        n_w = kendisi.n olsun

        e = 0 olsun
        e < epochlar olduğu sürece {
            // Gradient hesapla
            grad = vektor_olustur(n_w + 1, 0.0) olsun
            toplam_kayip = 0.0 olsun

            i = 0'dan (n - 1)'e kadar {
                y_hat = kendisi.tahmin_et(X[i]) olsun
                hata = y_hat - y[i] olsun
                toplam_kayip = toplam_kayip + hata * hata olsun

                // Gradient: dL/dw_j = hata * x_j; dL/db = hata
                j = 0'dan (n_w - 1)'e kadar {
                    grad[j] = grad[j] + hata * X[i][j] olsun
                }
                grad[n_w] = grad[n_w] + hata olsun
            }

            // Parametre güncelle (SGD — batch gradient)
            j = 0'dan n_w'e kadar {
                kendisi.agirliklar[j] = kendisi.agirliklar[j] - lr * grad[j] / n olsun
            }

            mse = toplam_kayip / n olsun
            kendisi.kayip_gecmisi = listeye_ekle(kendisi.kayip_gecmisi, mse) olsun
            e = e + 1 olsun
        }
        kendisi.kayip_gecmisi'ni döndür
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Lojistik Regresyon — Binary Cross-Entropy + Gradient Descent
// ═══════════════════════════════════════════════════════════════════════════
//
// Kullanım:
//   model = lojistik_regresyon() olsun
//   model.ilklendir(ozellik_n) olsun
//   model.egit(X, y, lr, epochlar) olsun
//   olasılik = model.tahmin_olasilik(x) olsun   // 0-1 arası
//   sinif = model.tahmin_sinif(x) olsun         // 0 veya 1
// ─────────────────────────────────────────────────────────────────────────────
lojistik_regresyon sınıf olsun {

    ilklendir fonksiyon olsun ozellik_n alsın {
        kendisi.n = ozellik_n olsun
        kendisi.agirliklar = vektor_olustur(ozellik_n + 1, 0.0) olsun
        kendisi.kayip_gecmisi = [] olsun
    }

    // ham_skor(x) → lineer bileşim (sigmoid öncesi)
    _ham_skor fonksiyon olsun x alsın {
        n = kendisi.n olsun
        toplam = kendisi.agirliklar[n] olsun
        i = 0'dan (n - 1)'e kadar {
            toplam = toplam + kendisi.agirliklar[i] * x[i] olsun
        }
        toplam'ı döndür
    }

    // tahmin_olasilik(x) → [0, 1] sigmoid çıktısı
    tahmin_olasilik fonksiyon olsun x alsın {
        z = kendisi._ham_skor(x) olsun
        sigmoid(z)'i döndür
    }

    // tahmin_sinif(x) → 0 veya 1 (eşik 0.5)
    tahmin_sinif fonksiyon olsun x alsın {
        p = kendisi.tahmin_olasilik(x) olsun
        p >= 0.5 ise { 1'i döndür }
        0'ı döndür
    }

    // egit(X, y, lr, epochlar) → kayıp listesi
    egit fonksiyon olsun X, y, lr, epochlar alsın {
        n = uzunluk(X) olsun
        n = 0 ise { []'i döndür }
        n_w = kendisi.n olsun

        e = 0 olsun
        e < epochlar olduğu sürece {
            grad = vektor_olustur(n_w + 1, 0.0) olsun
            toplam_kayip = 0.0 olsun

            i = 0'dan (n - 1)'e kadar {
                p = kendisi.tahmin_olasilik(X[i]) olsun
                hata = p - y[i] olsun

                // BCE: binary cross entropy gradient = (p - y) * x_j
                j = 0'dan (n_w - 1)'e kadar {
                    grad[j] = grad[j] + hata * X[i][j] olsun
                }
                grad[n_w] = grad[n_w] + hata olsun

                // Kayıp: -[y*log(p) + (1-y)*log(1-p)]
                eps = 1e-7 olsun
                p_safe = klamp(p, eps, 1.0 - eps) olsun
                kayip = (y[i] * -1.0 * güvenli_ln(p_safe)) - ((1.0 - y[i]) * güvenli_ln(1.0 - p_safe)) olsun
                toplam_kayip = toplam_kayip + kayip olsun
            }

            j = 0'dan n_w'e kadar {
                kendisi.agirliklar[j] = kendisi.agirliklar[j] - lr * grad[j] / n olsun
            }

            ortalama_kayip = toplam_kayip / n olsun
            kendisi.kayip_gecmisi = listeye_ekle(kendisi.kayip_gecmisi, ortalama_kayip) olsun
            e = e + 1 olsun
        }
        kendisi.kayip_gecmisi'ni döndür
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. k-NN Sınıflandırıcı — Öklit Mesafe + Çoğunluk Oylaması
// ═══════════════════════════════════════════════════════════════════════════
//
// Kullanım:
//   knn = knn_siniflandirici() olsun
//   knn.ilklendir(3) olsun         // k=3
//   knn.egit(X_egitim, y_egitim) olsun
//   sinif = knn.tahmin_et(x_test) olsun
// ─────────────────────────────────────────────────────────────────────────────
knn_siniflandirici sınıf olsun {

    ilklendir fonksiyon olsun k alsın {
        kendisi.k = k olsun
        kendisi.X_egitim = [] olsun
        kendisi.y_egitim = [] olsun
    }

    // egit(X, y) → eğitim verisini depola (lazy — k-NN bellekten öğrenir)
    egit fonksiyon olsun X, y alsın {
        kendisi.X_egitim = X olsun
        kendisi.y_egitim = y olsun
    }

    // tahmin_et(x) → tahmin edilen sınıf (en yaygın komşu etiketi)
    tahmin_et fonksiyon olsun x alsın {
        n = uzunluk(kendisi.X_egitim) olsun
        n = 0 ise { 0'ı döndür }

        // Her eğitim örneğine mesafeyi hesapla
        mesafeler = [] olsun
        i = 0'dan (n - 1)'e kadar {
            d = oklid_mesafe(x, kendisi.X_egitim[i]) olsun
            cift = {} olsun
            cift["mesafe"] = d
            cift["etiket"] = kendisi.y_egitim[i]
            mesafeler = listeye_ekle(mesafeler, cift) olsun
        }

        // Mesafeye göre sırala (bubble sort — küçük veri setleri için yeterli)
        m = uzunluk(mesafeler) olsun
        i = 0'dan (m - 2)'e kadar {
            j = 0'dan (m - i - 2)'e kadar {
                mesafeler[j]["mesafe"] > mesafeler[j + 1]["mesafe"] ise {
                    tmp = mesafeler[j] olsun
                    mesafeler[j] = mesafeler[j + 1] olsun
                    mesafeler[j + 1] = tmp olsun
                }
            }
        }

        // k komşunun oylama sayısını bul (etiket → sayı sözlüğü)
        oylar = {} olsun
        k_gercek = kendisi.k olsun
        k_gercek > m ise { k_gercek = m olsun }

        i = 0'dan (k_gercek - 1)'e kadar {
            etiket = mesafeler[i]["etiket"] olsun
            etiket_str = "" + etiket olsun
            mevcut = oylar[etiket_str] olsun
            mevcut = Boş ise { mevcut = 0 olsun }
            oylar[etiket_str] = mevcut + 1 olsun
        }

        // En yüksek oyu bul
        en_iyi_etiket = kendisi.y_egitim[0] olsun
        en_iyi_oy = -1 olsun
        i = 0'dan (k_gercek - 1)'e kadar {
            etiket = mesafeler[i]["etiket"] olsun
            etiket_str = "" + etiket olsun
            oy = oylar[etiket_str] olsun
            oy > en_iyi_oy ise {
                en_iyi_oy = oy olsun
                en_iyi_etiket = etiket olsun
            }
        }

        en_iyi_etiket'i döndür
    }

    // toplu_tahmin(X_liste) → tahmin listesi
    toplu_tahmin fonksiyon olsun X_liste alsın {
        n = uzunluk(X_liste) olsun
        tahminler = [] olsun
        i = 0'dan (n - 1)'e kadar {
            t = kendisi.tahmin_et(X_liste[i]) olsun
            tahminler = listeye_ekle(tahminler, t) olsun
        }
        tahminler'i döndür
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Metrik Fonksiyonları
// ═══════════════════════════════════════════════════════════════════════════

// Doğruluk — tahminler ve gerçek etiketler listesi
// dogruluk_hesapla([0,1,1], [0,1,0]) → 0.666...
dogruluk_hesapla fonksiyon olsun tahminler, gercekler alsın {
    n = uzunluk(tahminler) olsun
    n = 0 ise { 0.0'ı döndür }
    dogru_sayisi = 0 olsun
    i = 0'dan (n - 1)'e kadar {
        tahminler[i] = gercekler[i] ise { dogru_sayisi = dogru_sayisi + 1 olsun }
    }
    dogru_sayisi / n'yi döndür
}

// Kesinlik (Precision) — ikili sınıflandırma, pozitif sınıf=1
kesinlik_hesapla fonksiyon olsun tahminler, gercekler alsın {
    n = uzunluk(tahminler) olsun
    gp = 0 olsun   // gerçek pozitif
    yp = 0 olsun   // yanlış pozitif
    i = 0'dan (n - 1)'e kadar {
        tahminler[i] = 1 ise {
            gercekler[i] = 1 ise { gp = gp + 1 olsun } yoksa { yp = yp + 1 olsun }
        }
    }
    (gp + yp) = 0 ise { 0.0'ı döndür }
    gp / (gp + yp)'yi döndür
}

// Duyarlılık (Recall) — ikili sınıflandırma, pozitif sınıf=1
duyarlilik_hesapla fonksiyon olsun tahminler, gercekler alsın {
    n = uzunluk(tahminler) olsun
    gp = 0 olsun   // gerçek pozitif
    gn = 0 olsun   // yanlış negatif
    i = 0'dan (n - 1)'e kadar {
        gercekler[i] = 1 ise {
            tahminler[i] = 1 ise { gp = gp + 1 olsun } yoksa { gn = gn + 1 olsun }
        }
    }
    (gp + gn) = 0 ise { 0.0'ı döndür }
    gp / (gp + gn)'yi döndür
}

// F1 Skoru — Precision ve Recall harmonik ortalaması
f1_skoru fonksiyon olsun tahminler, gercekler alsın {
    p = kesinlik_hesapla(tahminler, gercekler) olsun
    r = duyarlilik_hesapla(tahminler, gercekler) olsun
    (p + r) = 0 ise { 0.0'ı döndür }
    2.0 * p * r / (p + r)'yi döndür
}

// R² Determinasyon Katsayısı — regresyon modeli değerlendirme
// 1.0 = mükemmel tahmin, 0.0 = ortalama kadar iyi, negatif = kötü model
r2_skoru fonksiyon olsun tahminler, gercekler alsın {
    n = uzunluk(gercekler) olsun
    n = 0 ise { 0.0'ı döndür }

    // Gerçek değerlerin ortalaması
    toplam = 0.0 olsun
    i = 0'dan (n - 1)'e kadar {
        toplam = toplam + gercekler[i] olsun
    }
    ortalama = toplam / n olsun

    // SS_tot ve SS_res
    ss_tot = 0.0 olsun
    ss_res = 0.0 olsun
    i = 0'dan (n - 1)'e kadar {
        fark_tot = gercekler[i] - ortalama olsun
        fark_res = gercekler[i] - tahminler[i] olsun
        ss_tot = ss_tot + fark_tot * fark_tot olsun
        ss_res = ss_res + fark_res * fark_res olsun
    }

    ss_tot = 0 ise { 1.0'ı döndür }
    1.0 - ss_res / ss_tot'u döndür
}
