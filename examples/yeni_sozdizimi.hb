// yeni_sozdizimi.hb — Hüma B ve C seçenekleri testi

"--- Postfix Yükle Testi ---"'yi yazdır
// "standart_lib.hb"'yi yükle // Dosya yoksa hata verebilir, şimdilik yorumda

"--- Option B (ile) Çağrı Testi ---" 'yi yazdır
topla = fonksiyon olsun a, b alsın {
    a + b döndür
}

sonuc = 10 ile 20'yi topla olsun
"10 ile 20'nin toplamı: " + sonuc'u yazdır

"--- Option C (Nesne Metodu) Çağrı Testi ---" 'yi yazdır
araba = {
    "hiz": 0,
    "hizlan": fonksiyon olsun artis alsın {
        kendisi'nin hızı = kendisi'nin hızı + artis olsun
        "Hızlandı: " + kendisi'nin hızı'nı yazdır
    }
} olsun

araba'nın (50) hizlan'ı
"Son hız: " + araba'nın hızı'nı yazdır
