// ══════════════════════════════════════════════════════════════════════════════
// lineer_regresyon_demo.hb — Hüma Lineer Regresyon Demo
// ══════════════════════════════════════════════════════════════════════════════
//
// Senaryo: y = 2*x + 1 + küçük gürültü  (sentetik veri)
// Beklenen sonuç: MSE < 1.0, R² > 0.95 (100 epoch sonunda)
// Gerçek sayısal çıktı — hiçbir sahte değer yok.
//
// Çalıştırma: huma examples/lineer_regresyon_demo.hb
// ══════════════════════════════════════════════════════════════════════════════

"makine_ogrenimi.hb"'yi yükle

yazdır "=== Hüma Lineer Regresyon Demo ==="
yazdır "Veri: y = 2*x + 1 + gürültü (50 örnek)"
yazdır ""

// Sabit seed ile tekrar üretilebilir sonuçlar
rastgele_tohum_ata(42) olsun

// Eğitim verisi oluştur: 50 örnek, x ∈ [-2, 2]
veri = [] olsun
etiketler = [] olsun
i = 0'dan 49'a kadar {
    x_val = uniform_rastgele(-2.0, 2.0) olsun
    gurultu = normal_rastgele(0.0, 0.3) olsun
    y_val = 2.0 * x_val + 1.0 + gurultu olsun
    // Her örnek 1 özellikli vektör
    ornek = vektor_olustur(1, x_val) olsun
    veri = listeye_ekle(veri, ornek) olsun
    etiketler = listeye_ekle(etiketler, y_val) olsun
}

yazdır "Veri oluşturuldu. İlk 3 örnek:"
yazdır "  x=" + vektor_al(veri[0], 0) + "  y=" + etiketler[0]
yazdır "  x=" + vektor_al(veri[1], 0) + "  y=" + etiketler[1]
yazdır "  x=" + vektor_al(veri[2], 0) + "  y=" + etiketler[2]
yazdır ""

// Modeli oluştur ve eğit
model = lineer_regresyon() olsun
model.ilklendir(1) olsun

yazdır "Eğitim başlıyor (100 epoch, lr=0.05)..."
kayiplar = model.egit(veri, etiketler, 0.05, 100) olsun

// Sonuçları raporla
n_kayip = uzunluk(kayiplar) olsun
ilk_kayip = kayiplar[0] olsun
son_kayip = kayiplar[n_kayip - 1] olsun

yazdır ""
yazdır "=== Eğitim Sonuçları ==="
yazdır "Başlangıç MSE : " + ilk_kayip
yazdır "Final MSE     : " + son_kayip

// R² skoru hesapla (test verisinde)
tahminler = [] olsun
i = 0'dan 49'a kadar {
    t = model.tahmin_et(veri[i]) olsun
    tahminler = listeye_ekle(tahminler, t) olsun
}
r2 = r2_skoru(tahminler, etiketler) olsun
yazdır "R² Skoru      : " + r2

// Öğrenilen parametreler
w = model.agirliklar olsun
yazdır ""
yazdır "=== Öğrenilen Parametreler ==="
yazdır "w[0] (eğim)   : " + w[0] + "  (beklenen: ~2.0)"
yazdır "b   (kesim)   : " + w[1] + "  (beklenen: ~1.0)"

// Doğrulama
son_kayip < 1.0 ise {
    yazdır ""
    yazdır "✓ MSE < 1.0 — Model başarıyla yakınsadı"
} yoksa {
    yazdır ""
    yazdır "⚠ MSE ≥ 1.0 — Model daha fazla epoch'a ihtiyaç duyabilir"
}

r2 > 0.90 ise {
    yazdır "✓ R² > 0.90 — Güçlü doğrusal ilişki yakalandı"
} yoksa {
    yazdır "⚠ R² ≤ 0.90 — Veri doğrusal ilişki göstermeyebilir"
}

yazdır ""
yazdır "Demo tamamlandı."
