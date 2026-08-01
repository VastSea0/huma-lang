// ══════════════════════════════════════════════════════════════════════════════
// lojistik_regresyon_demo.hb — Hüma İkili Sınıflandırma Demo
// ══════════════════════════════════════════════════════════════════════════════
//
// Senaryo: İki ayrılabilir Gaussian küme → ikili sınıflandırma
// Beklenen: Doğruluk > %90, F1 > 0.90
// Gerçek sayısal çıktı — hiçbir sahte değer yok.
//
// Çalıştırma: huma examples/lojistik_regresyon_demo.hb
// ══════════════════════════════════════════════════════════════════════════════

"makine_ogrenimi.hb"'yi yükle

yazdır "=== Hüma Lojistik Regresyon + k-NN Demo ==="
yazdır "Senaryo: İki Gaussian küme → ikili sınıflandırma"
yazdır ""

rastgele_tohum_ata(77) olsun

// Sınıf 0: merkez (-2, -2), Sınıf 1: merkez (2, 2)
veri = [] olsun
etiketler = [] olsun
n_sinif = 40  // her sınıftan 40 örnek

i = 0'dan (n_sinif - 1)'e kadar {
    x1 = normal_rastgele(-2.0, 0.8) olsun
    x2 = normal_rastgele(-2.0, 0.8) olsun
    ornek = vektor_olustur(2, 0.0) olsun
    vektor_ata(ornek, 0, x1) olsun
    vektor_ata(ornek, 1, x2) olsun
    veri = listeye_ekle(veri, ornek) olsun
    etiketler = listeye_ekle(etiketler, 0) olsun
}
i = 0'dan (n_sinif - 1)'e kadar {
    x1 = normal_rastgele(2.0, 0.8) olsun
    x2 = normal_rastgele(2.0, 0.8) olsun
    ornek = vektor_olustur(2, 0.0) olsun
    vektor_ata(ornek, 0, x1) olsun
    vektor_ata(ornek, 1, x2) olsun
    veri = listeye_ekle(veri, ornek) olsun
    etiketler = listeye_ekle(etiketler, 1) olsun
}

n_toplam = uzunluk(veri) olsun
yazdır "Toplam örnek: " + n_toplam + " (" + n_sinif + " pozitif, " + n_sinif + " negatif)"

// Eğitim/test bölme (%80 eğitim, %20 test)
bolme = egitim_test_bol(veri, etiketler, 0.8) olsun
X_egitim = bolme["egitim_veri"] olsun
y_egitim = bolme["egitim_etiket"] olsun
X_test = bolme["test_veri"] olsun
y_test = bolme["test_etiket"] olsun
n_egitim = uzunluk(X_egitim) olsun
n_test = uzunluk(X_test) olsun
yazdır "Eğitim: " + n_egitim + " | Test: " + n_test

// ─── Bölüm 1: Lojistik Regresyon ─────────────────────────────────────────────
yazdır ""
yazdır "--- Lojistik Regresyon (200 epoch, lr=0.1) ---"

lr_model = lojistik_regresyon() olsun
lr_model.ilklendir(2) olsun
lr_kayiplar = lr_model.egit(X_egitim, y_egitim, 0.1, 200) olsun

// Test tahmini
lr_tahminler = [] olsun
i = 0'dan (n_test - 1)'e kadar {
    t = lr_model.tahmin_sinif(X_test[i]) olsun
    lr_tahminler = listeye_ekle(lr_tahminler, t) olsun
}

lr_dogr = dogruluk_hesapla(lr_tahminler, y_test) olsun
lr_f1 = f1_skoru(lr_tahminler, y_test) olsun
lr_prec = kesinlik_hesapla(lr_tahminler, y_test) olsun
lr_rec = duyarlilik_hesapla(lr_tahminler, y_test) olsun

n_lr_kayip = uzunluk(lr_kayiplar) olsun
son_lr_kayip = lr_kayiplar[n_lr_kayip - 1] olsun

yazdır "Final BCE Kaybı : " + son_lr_kayip
yazdır "Test Doğruluk   : " + lr_dogr
yazdır "Kesinlik        : " + lr_prec
yazdır "Duyarlılık      : " + lr_rec
yazdır "F1 Skoru        : " + lr_f1

lr_dogr >= 0.90 ise {
    yazdır "✓ Lojistik Reg. doğruluk ≥ %90"
} yoksa {
    yazdır "⚠ Lojistik Reg. doğruluk < %90"
}

// ─── Bölüm 2: k-NN ───────────────────────────────────────────────────────────
yazdır ""
yazdır "--- k-NN Sınıflandırıcı (k=5) ---"

knn = knn_siniflandirici() olsun
knn.ilklendir(5) olsun
knn.egit(X_egitim, y_egitim) olsun

knn_tahminler = knn.toplu_tahmin(X_test) olsun
knn_dogr = dogruluk_hesapla(knn_tahminler, y_test) olsun
knn_f1 = f1_skoru(knn_tahminler, y_test) olsun

yazdır "Test Doğruluk  : " + knn_dogr
yazdır "F1 Skoru       : " + knn_f1

knn_dogr >= 0.85 ise {
    yazdır "✓ k-NN doğruluk ≥ %85"
} yoksa {
    yazdır "⚠ k-NN doğruluk < %85"
}

yazdır ""
yazdır "Demo tamamlandı."
