// Hüma 0.5.2 ve 0.6.0 ile uyumlu, çevrimdışı Türkçe metin analizörü.
"Analiz edilecek metni girin:"'i yazdır
girdi = kırp(oku()) olsun

girdi = "" ise {
    "Hata: Boş metin analiz edilemez."'i yazdır
} yoksa {
    küçük = küçük_harf(girdi) olsun
    temiz = değiştir(küçük, "\t", " ") olsun
    temiz = değiştir(temiz, "  ", " ") olsun
    temiz = değiştir(temiz, "  ", " ") olsun
    temiz = değiştir(temiz, "  ", " ") olsun

    sözcük_sayısı = uzunluk(böl(temiz, " ")) olsun
    sesli_harf = tekrar_sayısı(küçük, "a") + tekrar_sayısı(küçük, "e") olsun
    sesli_harf = sesli_harf + tekrar_sayısı(küçük, "ı") + tekrar_sayısı(küçük, "i") olsun
    sesli_harf = sesli_harf + tekrar_sayısı(küçük, "o") + tekrar_sayısı(küçük, "ö") olsun
    sesli_harf = sesli_harf + tekrar_sayısı(küçük, "u") + tekrar_sayısı(küçük, "ü") olsun

    cümle_sayısı = tekrar_sayısı(girdi, ".") + tekrar_sayısı(girdi, "!") olsun
    cümle_sayısı = cümle_sayısı + tekrar_sayısı(girdi, "?") olsun
    cümle_sayısı = 0 ise {
        cümle_sayısı = 1 olsun
    }

    harfler = değiştir(temiz, " ", "") olsun
    harf_sayısı = uzunluk(harfler) olsun
    ortalama = harf_sayısı / sözcük_sayısı olsun

    "----- METİN RAPORU -----"'nu yazdır
    ("Karakter: " + uzunluk(girdi))'ni yazdır
    ("Harf: " + harf_sayısı)'nı yazdır
    ("Sözcük: " + sözcük_sayısı)'nı yazdır
    ("Cümle: " + cümle_sayısı)'yi yazdır
    ("Sesli harf: " + sesli_harf)'i yazdır
    ("Ortalama sözcük uzunluğu: " + ortalama)'nı yazdır
}
