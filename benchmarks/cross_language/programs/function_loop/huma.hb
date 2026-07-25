karıştır fonksiyon olsun durum, sıra alsın {
    ((durum * 48271 + sıra) % 2147483647)'yi döndür
}

durum = 1 olsun
i = 1'den 100000'e kadar {
    durum = karıştır(durum, i) olsun
}
durum'u yazdır
