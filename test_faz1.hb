// Sözlük Testi
bilgi = {
    "isim": "Hüma",
    "tip": "Programlama Dili",
    "sürüm": 0.6
} olsun

yazdır bilgi'nin isim
yazdır bilgi'nin tip

// Metod testi
yazdır bilgi.getir("sürüm")

// Hata Yönetimi Testi
dene {
    x = 10 / 0 olsun
} yakala hata {
    yazdır "Hata yakalandı!"
    yazdır hata
}

yazdır "Program devam ediyor..."
