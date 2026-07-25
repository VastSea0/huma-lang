durum = 1 olsun
toplam = 0 olsun
i = 1'den 200000'e kadar {
    durum = durum * 2 + i olsun
    durum >= 1000000000 ise {
        durum = durum - 1000000000 olsun
    }
    durum >= 1000000000 ise {
        durum = durum - 1000000000 olsun
    }
    toplam = toplam + durum olsun
}
toplam'ı yazdır
