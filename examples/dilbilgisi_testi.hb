// ═══════════════════════════════════════════════════════════════════
// dilbilgisi_testi.hb — Hüma Türkçe Sözdizimi Örneği
// ═══════════════════════════════════════════════════════════════════

"--- Belirtme Eki Testi ---"'ni yazdır
sayılar = [1, 2, 3] olsun
sayılar'ı yazdır

mesaj = "Hüma Türkçe odaklı bir sözdizimi dener." olsun
mesaj'ı yazdır

"--- Aralık Döngüsü (kadar) Testi ---"'yi yazdır
toplam = 0 olsun
i = 1'den 10'a kadar {
    toplam = toplam + i olsun
}
"1'den 10'a kadar toplam: " + toplam'ı yazdır

"--- Soru Eki (mi/mı) Testi ---"'yi yazdır
x = 10 olsun
x > 5 mi ise {
    "x 5'ten büyüktür."'ü yazdır
} yoksa {
    "x 5'ten küçük veya eşittir."'i yazdır
}

"--- Bağımsız İyelik Eki Testi ---"'yi yazdır
test = { "adı": "Hüma" } olsun
test'in adı'nı yazdır           // İlgi + Belirtme
"Uygulama adı: " + test'in adı'nı yazdır

başarı_yaz = fonksiyon olsun m alsın {
    "\x1b[32m[BAŞARI] \x1b[0m" + m'yi yazdır
}

başarı_yaz("Sözdizimi örneği tamamlandı!")
