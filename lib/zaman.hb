"renkler.hb"'yi yükle

beklet fonksiyon olsun saniye alsın {
    uyut(saniye * 1000)
}

kronometre_başlat fonksiyon olsun {
    zaman()'ı döndür
}

kronometre_bitir fonksiyon olsun başlangıç alsın {
    bitiş = zaman() olsun
    fark = bitiş - başlangıç olsun
    ("Geçen süre: " + fark + " saniye") ile TURKUAZ'ı renkli_yaz
    fark'ı döndür
}
