içeriyor_mu fonksiyon olsun dizgi, aranan alsın {
    i = 0 olsun
    boy = dizgi'nin uzunluğu olsun
    aranan_boy = aranan'ın uzunluğu olsun
    
    i <= (boy - aranan_boy) olduğu sürece {
        j = 0 olsun
        eşleşti = 1 olsun
        j < aranan_boy olduğu sürece {
            dizgi[i + j] != aranan[j] ise {
                eşleşti = 0 olsun
                j = aranan_boy olsun
            }
            j = j + 1 olsun
        }
        eşleşti ise { 
            ("Eşleşti! i=" + i) yazdır
            1'i döndür 
        }
        i = i + 1 olsun
    }
    0'ı döndür
}

ünlüler = "aei" olsun
res = içeriyor_mu(ünlüler, "i") olsun
("Test res: " + res) yazdır
