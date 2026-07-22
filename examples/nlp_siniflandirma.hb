// ══════════════════════════════════════════════════════════════════════════════
// nlp_siniflandirma.hb — Türkçe Metin Sınıflandırma Örneği
// Sinir ağı ile Olumlu/Olumsuz yorum sınıflandırması
// ══════════════════════════════════════════════════════════════════════════════

"yapay_zeka_temel.hb"'yi yükle
"nlp_temel/nlp_temel.hb"'yi yükle
"nlp_ileri/tf_idf.hb"'yi yükle
"yapay_zeka/sinir_agi.hb"'yi yükle

yazdır "══════════════════════════════════════════════════"
yazdır "  Hüma — Türkçe Metin Sınıflandırma Örneği"
yazdır "  Sinir Ağı + TF-IDF + Backpropagation"
yazdır "══════════════════════════════════════════════════"
yazdır ""

// ─── Veri Seti ────────────────────────────────────────────────────────────────
yorumlar = [
    "Bu ürün gerçekten harika, çok memnun kaldım",
    "Mükemmel kalite, kesinlikle tavsiye ederim",
    "Hayal kırıklığı yaşadım, berbat bir ürün",
    "Çok kötü, param boşa gitti",
    "Süper bir deneyimdi, tekrar alacağım",
    "Beklentilerimin çok üzerinde çıktı",
    "Rezalet, hiç beğenmedim",
    "Oldukça memnun kaldım, güzel ürün",
    "İade ettim, işe yaramadı",
    "Mükemmel fiyat performans, tavsiye ederim",
    "Çok kötü bir deneyimdi",
    "Harika, bu fiyata bu kalite inanılmaz"
] olsun

etiketler = [1, 1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1] olsun

// ─── Ön İşleme ────────────────────────────────────────────────────────────────
yazdır "Ön işleme başlıyor..."
proc = metin_islemci() olsun
corpus_tokens = [] olsun
i = 0'dan (uzunluk(yorumlar) - 1)'e kadar {
    tokens = proc.durak_filtrele(proc.tokenize(yorumlar[i])) olsun
    corpus_tokens = listeye_ekle(corpus_tokens, tokens) olsun
}

// ─── TF-IDF Vektörleştirme ────────────────────────────────────────────────────
yazdır "TF-IDF hesaplanıyor..."
sozluk_veri = sozluk_olustur(corpus_tokens) olsun
idf_v = idf_hesapla(corpus_tokens, sozluk_veri) olsun
sozluk_boyutu = sozluk_veri["boyut"] olsun
yazdır "Sözlük boyutu: " + sozluk_boyutu

veri = [] olsun
i = 0'dan (uzunluk(corpus_tokens) - 1)'e kadar {
    v = tfidf_vektoru(corpus_tokens[i], sozluk_veri, idf_v) olsun
    v_liste = vektore_liste(v) olsun
    veri = listeye_ekle(veri, v_liste) olsun
}

// ─── Sinir Ağı Oluşturma ─────────────────────────────────────────────────────
yazdır ""
yazdır "Model oluşturuluyor..."
rastgele_tohum_ata(42) olsun   // Tekrarlanabilirlik için
model = sinir_agi() olsun
model.ilklendir() olsun
model.katman_ekle(sozluk_boyutu, 16, "relu") olsun
model.katman_ekle(16, 8, "relu") olsun
model.katman_ekle(8, 1, "sigmoid") olsun
yazdır "Mimari: " + sozluk_boyutu + " → 16 (ReLU) → 8 (ReLU) → 1 (Sigmoid)"

// Etiketleri tek elemanlı liste olarak sarla (model [1]-boyutlu vektör döndürür)
etiket_vektorleri = [] olsun
i = 0'dan (uzunluk(etiketler) - 1)'e kadar {
    e_v = [etiketler[i] * 1.0] olsun
    etiket_vektorleri = listeye_ekle(etiket_vektorleri, e_v) olsun
}

// ─── Eğitim ───────────────────────────────────────────────────────────────────
yazdır ""
yazdır "Eğitim başlıyor (50 epoch, lr=0.01)..."
model.egit(veri, etiket_vektorleri, 50, 0.01) olsun

// ─── Değerlendirme ────────────────────────────────────────────────────────────
yazdır ""
yazdır "══════════════════════════════════════════════════"
dogru = 0 olsun
toplam = uzunluk(veri) olsun
i = 0'dan (toplam - 1)'e kadar {
    giris = listeye_vektor(veri[i]) olsun
    tahmin_v = model.tahmin_et(giris) olsun
    tahmin_val = vektor_al(tahmin_v, 0) olsun
    tahmin_sinif = 0 olsun
    tahmin_val >= 0.5 ise { tahmin_sinif = 1 olsun }
    etiket = etiketler[i] olsun
    durum = "✗" olsun
    tahmin_sinif = etiket ise {
        durum = "✓" olsun
        dogru = dogru + 1 olsun
    }
    yazdır durum + " " + yorumlar[i] + " → " + tahmin_sinif
}
yazdır "══════════════════════════════════════════════════"
dogruluk = dogru / toplam olsun
yazdır "Eğitim Doğruluğu: " + dogru + "/" + toplam + " (" + (dogruluk * 100) + "%)"

// ─── Model Kaydet ─────────────────────────────────────────────────────────────
model.kaydet("nlp_sinif_model.json") olsun
yazdır ""
yazdır "Örnek test (bilinmeyen metin):"
test_metin = "Bu ürün gerçekten çok güzel, harika kalite" olsun
test_tokens = proc.durak_filtrele(proc.tokenize(test_metin)) olsun
test_v = tfidf_vektoru(test_tokens, sozluk_veri, idf_v) olsun
sonuc = model.tahmin_et(test_v) olsun
skor = vektor_al(sonuc, 0) olsun
etiket = "Olumsuz" olsun
skor >= 0.5 ise { etiket = "Olumlu" olsun }
yazdır "\"" + test_metin + "\""
yazdır "Tahmin: " + etiket + " (güven: " + skor + ")"
