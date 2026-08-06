// ══════════════════════════════════════════════════════════════════════════════
// examples / huma_gpt / gpt_model.hb
// Hüma Dili Yapay Sinir Ağı tabanlı GPT Modeli
// ══════════════════════════════════════════════════════════════════════════════

"yapay_zeka"'yı yükle

gpt_modeli_olustur fonksiyon olsun pencere_boyutu, gizli_boyut alsın {
    model = sinir_agi()
    model.ilklendir()

    // Katman 1: Girdi penceresinden gizli boyuta ReLu katmanı
    model.katman_ekle(pencere_boyutu, gizli_boyut, "relu")

    // Katman 2: Gizli boyuttan ikinci gizli katmana ReLu katmanı
    model.katman_ekle(gizli_boyut, gizli_boyut, "relu")

    // Katman 3: Çıkış tahmini katmanı (Sigmoid / Linear Output)
    model.katman_ekle(gizli_boyut, 1, "sigmoid")

    model'i döndür
}

gpt_modeli_egit fonksiyon olsun model, x_veri, y_veri, epoch_sayisi, ogrenme_hizi alsın {
    "🧠 [HÜMA GPT EĞİTİMİ] Model eğitiliyor (Epoch: " + epoch_sayisi + ", Öğrenme Hızı: " + ogrenme_hizi + ")..."'ı yazdır

    model.egit(x_veri, y_veri, epoch_sayisi, ogrenme_hizi)

    "   ✅ Hüma GPT Eğitimi Tamamlandı!"'ı yazdır
    model'i döndür
}
