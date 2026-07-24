// ══════════════════════════════════════════════════════════════════════════════
// ÖRNEK NLP UYGULAMASI (Hüma Paket Yöneticisi & OOP NLP Modülü)
// ══════════════════════════════════════════════════════════════════════════════

// 1. Hüma paket yöneticisi ile kurulan nlp_temel modülünü yükle
"nlp_temel"'i yükle

"╔══════════════════════════════════════════════════════════════════════╗"'u yazdır
"║       🤖 HÜMA DOĞAL DİL İŞLEME (NLP) ÖRNEK UYGULAMASI                ║"'u yazdır
"╚══════════════════════════════════════════════════════════════════════╝"'u yazdır
""'ı yazdır

// 2. OOP Sınıf Nesnelerini Oluştur
islemci  = metin_islemci() olsun
stemmer  = kok_bulucu() olsun
analizci = metin_analizci() olsun

// Örnek Haber Metni
metin = "Prof. Dr. Ayşe Kaya İstanbul Üniversitesi'nde yapay zeka üzerine harika bir konuşma yaptı. Konferansa Ankara ve İzmir'den çok sayıda araştırmacı katıldı!" olsun

"📌 ÖRNEK GİRDİ METNİ:"'i yazdır
metin'i yazdır
""'ı yazdır

// ── 1. Metin Temizleme ve Tokenizasyon ─────────────────────────────────────
temiz = islemci.temizle(metin) olsun
"1️⃣ Temizlenmiş Metin:"'i yazdır
temiz'i yazdır
""'ı yazdır

tokenler = islemci.tokenize(metin) olsun
"2️⃣ Tokenizasyon (Kelime Sayısı: " + uzunluk(tokenler) + "):"'yi yazdır
tokenler'i yazdır
""'ı yazdır

// ── 2. Durak Kelimeleri Filtreleme ──────────────────────────────────────────
anlamli_kelimeler = islemci.durak_filtrele(tokenler) olsun
"3️⃣ Durak Kelime Filtreleme Sonrası (Kalan: " + uzunluk(anlamli_kelimeler) + "):"'yi yazdır
anlamli_kelimeler'i yazdır
""'ı yazdır

// ── 3. Kökleştirme (Stemming) ────────────────────────────────────────────────
kokler = [] olsun
i = 0'dan (uzunluk(anlamli_kelimeler) - 1)'e kadar {
    k = stemmer.stem(anlamli_kelimeler[i]) olsun
    kokler = listeye_ekle(kokler, k) olsun
}
"4️⃣ Kökleştirilmiş Kelimeler (Stemming):"'yi yazdır
kokler'i yazdır
""'ı yazdır

// ── 4. Duygu Analizi (Sentiment Analysis) ──────────────────────────────────
duygu = analizci.duygu_analiz(tokenler) olsun
"5️⃣ Duygu Analizi Sonucu:"'yi yazdır
"   • Etiket: " + duygu["etiket"]'yi yazdır
"   • Skor  : " + duygu["skor"]'yi yazdır
""'ı yazdır

// ── 5. Cümle Bölme ─────────────────────────────────────────────────────────
cumleler = islemci.cümle_böl(metin) olsun
"6️⃣ Cümle Bölme (Bulunan Cümle Sayısı: " + uzunluk(cumleler) + "):"'yi yazdır
j = 0'dan (uzunluk(cumleler) - 1)'e kadar {
    "   [" + j + "] " + cumleler[j]'yi yazdır
}
""'ı yazdır

"✅ NLP Analizi Başarıyla Tamamlandı!"'ı yazdır
