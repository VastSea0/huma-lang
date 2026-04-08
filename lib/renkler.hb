// ═══════════════════════════════════════════════════════════════════
// renkler.hb — Hüma Terminal Renk Kütüphanesi
// Sürüm: 1.1.0
// Yazar: Egehan KAHRAMAN
// ═══════════════════════════════════════════════════════════════════

SIFIR = "\x1b[0m" olsun
KIRMIZI = "\x1b[31m" olsun
YEŞİL = "\x1b[32m" olsun
SARI = "\x1b[33m" olsun
MAVI = "\x1b[34m" olsun
TURKUAZ = "\x1b[36m" olsun
KALIN = "\x1b[1m" olsun

// Geriye dönük uyumluluk alias'ı
YESIL = YEŞİL olsun

renkli_yaz fonksiyon olsun metin, renk alsın {
    renk + metin + SIFIR'ı yazdır
}

hata_yaz fonksiyon olsun metin alsın {
    KALIN + KIRMIZI + "[HATA] " + SIFIR + metin'i yazdır
}

başarı_yaz fonksiyon olsun metin alsın {
    KALIN + YEŞİL + "[BAŞARI] " + SIFIR + metin'i yazdır
}

uyarı_yaz fonksiyon olsun metin alsın {
    KALIN + SARI + "[UYARI] " + SIFIR + metin'i yazdır
}
