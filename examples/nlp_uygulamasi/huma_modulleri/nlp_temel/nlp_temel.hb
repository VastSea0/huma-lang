// ══════════════════════════════════════════════════════════════════════════════
// nlp_temel.hb — Hüma Türkçe NLP Temel Paketi Ana Giriş Noktası
// Sürüm: 3.2.0 (Nesne Yönelimli ve Modüler Mimari)
// Yazar: Egehan KAHRAMAN
// ══════════════════════════════════════════════════════════════════════════════

"sabitler.hb"'yi yükle
"islemci.hb"'yi yükle
"stemmer.hb"'yi yükle
"analizci.hb"'yi yükle

// ─── Varsayılan Global OOP Örnekleri ─────────────────────────────────────
nlp_islemci = metin_islemci() olsun
nlp_stemmer = kok_bulucu() olsun
nlp_analizci = metin_analizci() olsun