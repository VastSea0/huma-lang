// golden: yalnız-yorumlayıcı VM indeksli atamayı derlemiyor (B-021)
v = vektor_olustur(2, 0.0) olsun
v[1] = 4 olsun
v[1]'i yazdır
dene { v[0] = "a" olsun } yakala hata { hata'yı yazdır }
