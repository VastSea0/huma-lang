durum = 1 olsun
toplam = 0 olsun
i = 1'den 200000'e kadar {
    durum = (durum * 1664525 + 1013904223) % 4294967296 olsun
    toplam = (toplam + durum) % 9007199254740881 olsun
}
toplam'ı yazdır
