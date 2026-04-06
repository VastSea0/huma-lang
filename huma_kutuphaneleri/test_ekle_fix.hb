// test_ekle_fix.hb

TestNesnesi sınıf olsun {
    depo = [] olsun
}

t = TestNesnesi() olsun
"Başlangıç: " + metne_çevir(t.depo)'ı yazdır

// Nesne alanına ekleme (FIX BURADA!)
t.depo'ye [10]'u ekle
t.depo'ye [20]'u ekle
t.depo'ye ["Selam"]'ı ekle

"Sonuç: " + metne_çevir(t.depo)'ı yazdır

t.depo'nin uzunluğu == 3 ise {
    "BAŞARILI: Nesne alanına ekleme çalışıyor."'ı yazdır
} yoksa {
    "HATA: Nesne alanına ekleme çalışmadı!"'ı yazdır
}

// Çıkarma testi
t.depo'den [1]'i çıkar
"Çıkarma sonrası: " + metne_çevir(t.depo)'ı yazdır
t.depo'nin uzunluğu == 2 ise {
    "BAŞARILI: Nesne alanından çıkarma çalışıyor."'ı yazdır
} yoksa {
    "HATA: Nesne alanından çıkarma çalışmadı!"'ı yazdır
}
