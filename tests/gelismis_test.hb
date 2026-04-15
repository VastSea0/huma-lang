// gelismis_test.hb — Gelişmiş Kütüphane Test Dosyası (Modern Söz Dizimi)

"matematik.hb"'yi yükle
"renkler.hb"'yi yükle
"rastgele.hb"'yi yükle
"dizgi.hb"'yi yükle
"dosya.hb"'yi yükle
"istatistik.hb"'yi yükle

TURKUAZ ile KALIN ile "===== Hüma Gelişmiş Kütüphane Testi =====" renkli_yaz

// 1. Matematik ve İstatistik Testi
veriler = [10, 20, 30, 40, 50] olsun
"Veriler: " + veriler'i yazdır
"Ortalama: " + veriler'in ortalaması'nı yazdır
"Standart Sapma: " + veriler'in standart_sapması'nı yazdır
"144'ün karekökü: " + 144'ün karekökü'nü yazdır

// 2. Dizgi Testi
metin = "   Merhaba Hüma Dünyası!   " olsun
"Orijinal: '" + metin + "'"'yı yazdır
"Kırpılmış: '" + metin'i kırp + "'"'yı yazdır
metin ile "Hüma"'yı içeriyor_mu ise {
    başarı_yaz("Metin 'Hüma' içeriyor.")
}

// 3. Rastgele Testi
"Rastgele Sayı (1-100): " + 1 ile 100'ü r_tamsayı'yı yazdır
sansli_meyve = ["Elma", "Armut", "Kiraz", "Karpuz"]'ı r_seç olsun
"Şanslı Meyveniz: " + sansli_meyve'yi yazdır

// 4. Dosya Sistem Testi
test_dosyasi = "test_gunlugu.txt" olsun
test_icerigi = "Hüma ile dosya yazma testi.\nİkinci satır.\nSon." olsun
test_dosyasi ile test_icerigi'ni dosya_yaz ise {
    başarı_yaz(test_dosyasi + " başarıyla oluşturuldu.")
}

okunan = test_dosyasi'ni güvenli_oku olsun
"Dosya İçeriği:"'ni yazdır
okunan'ı yazdır

satırlar = okunan'ı satırlara_ayır olsun
"Satır Sayısı: " + satırlar'ın uzunluğu'nu yazdır

// 5. Tip Kontrolü
"PI tipi: " + PI'nin tipi'ni yazdır
