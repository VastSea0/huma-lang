// ═══════════════════════════════════════════════════════════════════
// dizgi.hb — Hüma Dizgi (String) Yardımcı Fonksiyonları
// Sürüm: 1.1.0
// Yazar: Egehan KAHRAMAN
// ═══════════════════════════════════════════════════════════════════
//
// Rust Built-in Bağımlılıklar (interpreter tarafından sağlanır):
//   - içeriyor(metin, aranan)    → metin içinde arama
//   - başlıyor_mu(metin, önek)  → önek kontrolü
//   - bitiyor_mu(metin, sonek)  → sonek kontrolü
//   - kırp(metin)               → boşluk kırpma
//   - küçük_harf(metin)         → küçük harfe çevir
//   - büyük_harf(metin)         → büyük harfe çevir
//   - böl(metin, ayraç)         → parçalara ayır
//   - değiştir(metin, a, b)     → metin değiştirme
//   - dizi_dilim(metin, b, s)   → alt dizgi alma
// ═══════════════════════════════════════════════════════════════════

büyük_mü fonksiyon olsun karakter alsın {
    ((karakter >= "A") ve (karakter <= "Z")) veya (karakter = "Ç") veya (karakter = "Ğ") veya (karakter = "İ") veya (karakter = "Ö") veya (karakter = "Ş") veya (karakter = "Ü")'yi döndür
}

küçük_mü fonksiyon olsun karakter alsın {
    ((karakter >= "a") ve (karakter <= "z")) veya (karakter = "ç") veya (karakter = "ğ") veya (karakter = "ı") veya (karakter = "ö") veya (karakter = "ş") veya (karakter = "ü")'yi döndür
}

boşluk_mu fonksiyon olsun karakter alsın {
    (karakter = " ") veya (karakter = "\n") veya (karakter = "\t")'yi döndür
}

// ─── Geriye Dönük Uyumluluk Alias'ları ─────────────────────────────────
// Bu fonksiyonlar artık Rust built-in'lerine yönlendirir.
// Eski kodların çalışmaya devam etmesini sağlar.

başıyla_mı_başlıyor fonksiyon olsun dizgi, ön_ek alsın {
    başlıyor_mu(dizgi, ön_ek)'i döndür
}

sonuyla_mı_bitiyor fonksiyon olsun dizgi, son_ek alsın {
    bitiyor_mu(dizgi, son_ek)'i döndür
}

// NOT: Aşağıdaki fonksiyonlar Rust built-in olarak mevcuttur:
//   - kırp(metin)              → lib versiyonu kaldırıldı (v1.1.0)
//   - içeriyor(kaynak, aranan) → lib versiyonu kaldırıldı (v1.1.0)
// Eğer eski kodda "içeriyor_mu" kullanıyorsanız, "içeriyor" ile değiştirin.
