// golden: yalnız-yorumlayıcı VM sınıf tanımını ve nesne alanı atamasını derlemiyor (B-021)
İşlem sınıf olsun {
    topla fonksiyon olsun a, b alsın {
        a + b'yi döndür
    }
}

hesap = İşlem() olsun
"Toplama: " + hesap.topla(10, 20)'yi yazdır
