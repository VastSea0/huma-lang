// Rastgele İşlemler

r_tamsayı fonksiyon olsun min, max alsın {
    r = rastgele() olsun
    sonuc = min + (r * (max - min + 1)) olsun
    sonuc - (sonuc % 1)'i döndür
}

r_seç fonksiyon olsun d alsın {
    boy = uzunluk(d) olsun
    boy = 0 ise { Boş'u döndür }
    idx = r_tamsayı(0, boy - 1) olsun
    d[idx]'i döndür
}

r_karıştır fonksiyon olsun d alsın {
    yeni = d olsun
    boy = uzunluk(d) olsun
    i = 0 olsun
    i < boy olduğu sürece {
        j = r_tamsayı(0, boy - 1) olsun
        gecici = yeni[i] olsun
        yeni[i] = yeni[j] olsun
        yeni[j] = gecici olsun
        i = i + 1 olsun
    }
    yeni'yi döndür
}
