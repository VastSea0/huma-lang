// ══════════════════════════════════════════════════════════════════════════════
// MÜŞTERİ GERİ BİLDİRİMİ VE YORUM ANALİZ SİSTEMİ
// Hüma Türkçe Doğal Dil İşleme (NLP) Modülü İle Gerçek Dünya Uygulaması
// ══════════════════════════════════════════════════════════════════════════════

"nlp_temel"'i yükle

"╔══════════════════════════════════════════════════════════════════════════════╗"'u yazdır
"║  📊 HÜMA MÜŞTERİ GERİ BİLDİRİM & YORUM ANALİZ ZEKASI (NLP INTELLIGENCE)    ║"'u yazdır
"╚══════════════════════════════════════════════════════════════════════════════╝"'u yazdır
""'ı yazdır

// 1. NLP Servislerini Başlat
islemci  = metin_islemci() olsun
stemmer  = kok_bulucu() olsun
analizci = metin_analizci() olsun

// 2. Müşteri Yorumları Veri Seti (Gerçek Senaryo Örnekleri)
yorum_1 = "Ürün gerçekten harika! Kargo çok hızlı geldi, paketi özenle hazırlamışlar. Çok memnun kaldım, herkese tavsiye ederim." olsun
yorum_2 = "Berbat bir hizmet aldım. Sipariş verdiğim ürün yanlış ve kırık geldi! İade talebine kimse cevap vermiyor, tam bir hayal kırıklığı." olsun
yorum_3 = "Sipariş dün ulaştı. Kutu ambalajı standart. Fiyatına göre performansı normal, ne iyi ne kötü." olsun
yorum_4 = "İstanbul daki teknik destek ekibinden Zeynep Hanım sorunumu çok hızlı çözdü. Ekibe ve firmaya teşekkürler!" olsun
yorum_5 = "Ürün kalitesi çok kötü, kesinlikle almayın. Parama yazık oldu." olsun

yorumlar = [yorum_1, yorum_2, yorum_3, yorum_4, yorum_5] olsun
toplam_yorum = uzunluk(yorumlar) olsun

"📌 TOPLAM İNCELENECEK GERİ BİLDİRİM SAYISI: " + toplam_yorum'u yazdır
"──────────────────────────────────────────────────────────────────────────────"'ı yazdır
""'ı yazdır

// İstatistik Takip Değişkenleri
pozitif_sayisi = 0 olsun
negatif_sayisi = 0 olsun
notr_sayisi    = 0 olsun
tum_kelimeler  = [] olsun

// 3. Her Bir Yorum İçin Detaylı NLP Analizi
i = 0'dan (toplam_yorum - 1)'e kadar {
    y = yorumlar[i] olsun
    no = i + 1 olsun
    
    "📝 [Yorum #" + no + "]: \"" + y + "\""'i yazdır

    // A. Tokenizasyon ve Durak Kelime Filtreleme
    tokenler = islemci.tokenize(y) olsun
    anlamli  = islemci.durak_filtrele(tokenler) olsun

    // Tüm anlamlı kelimeleri genel havuza ekle
    j = 0'dan (uzunluk(anlamli) - 1)'e kadar {
        tum_kelimeler = listeye_ekle(tum_kelimeler, anlamli[j]) olsun
    }

    // B. Duygu Analizi
    duygu = analizci.duygu_analiz(tokenler) olsun
    etiket = duygu["etiket"] olsun
    skor   = duygu["skor"] olsun

    (etiket = "POZİTİF") ise {
        pozitif_sayisi = pozitif_sayisi + 1 olsun
        "   ► Duygu  : ✅ POZİTİF (Skor: +" + skor + ")"'i yazdır
    } yoksa {
        (etiket = "NEGATİF") ise {
            negatif_sayisi = negatif_sayisi + 1 olsun
            "   ► Duygu  : ❌ NEGATİF (Skor: " + skor + ")"'i yazdır
        } yoksa {
            notr_sayisi = notr_sayisi + 1 olsun
            "   ► Duygu  : ➖ NÖTR (Skor: 0)"'ı yazdır
        }
    }

    // C. Öne Çıkan Kökler (Stemming)
    kok_ozet = "" olsun
    k_sayisi = uzunluk(anlamli) olsun
    (k_sayisi > 5) ise { k_sayisi = 5 olsun }
    
    k_idx = 0'dan (k_sayisi - 1)'e kadar {
        kok = stemmer.stem(anlamli[k_idx]) olsun
        (uzunluk(kok_ozet) > 0) ise {
            kok_ozet = kok_ozet + ", " + kok olsun
        } yoksa {
            kok_ozet = kok olsun
        }
    }
    "   ► Kökler : " + kok_ozet'i yazdır
    "------------------------------------------------------------------------------"'i yazdır
}

""'ı yazdır
"=============================================================================="'ı yazdır
"📈 MÜŞTERİ MEMNUNİYET GENEL RAPORU VE ÖZET İSTATİSTİKLER"'i yazdır
"=============================================================================="'ı yazdır

"• Toplam Değerlendirilen Yorum : " + toplam_yorum'u yazdır
"• Pozitif Geri Bildirimler      : " + pozitif_sayisi + " adet"'i yazdır
"• Negatif Geri Bildirimler      : " + negatif_sayisi + " adet"'i yazdır
"• Nötr / Belirsiz Yorumlar      : " + notr_sayisi    + " adet"'i yazdır
""'ı yazdır

// Genel Frekans Analizi
frekans = analizci.kelime_frekansları(tum_kelimeler) olsun
"• En Sık Geçen Anahtar Kelimeler Havuzu:"'i yazdır
frekans'ı yazdır
""'ı yazdır

"╔══════════════════════════════════════════════════════════════════════╗"'u yazdır
"║  🎯 Raporlama Başarıyla Tamamlandı! Tüm veriler canlı analiz edildi. ║"'u yazdır
"╚══════════════════════════════════════════════════════════════════════╝"'u yazdır
