// golden: yalnız-yorumlayıcı VM yakalanan hatayı metin yerine hata değeri olarak bağlıyor (B-055)
dene {
    (1 / 0) olsun
    "buraya gelinmez"'i yazdır
} yakala hata {
    ("yakalandı: " + hata)'yı yazdır
}
"devam"'ı yazdır
