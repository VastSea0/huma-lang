// golden: yalnız-yorumlayıcı VM modül yüklemeyi derlemiyor (B-021)
yükle "liste.hb";
yükle "birim_test.hb";
yükle "matematik.hb";

test_matematik fonksiyon olsun {
    iddia_et(25, karesi(5), "5'in karesi 25 olmalı")'yı döndür
}

test_liste_esle fonksiyon olsun {
    l = [1, 2, 3] olsun
    iki_kati fonksiyon olsun n alsın { n * 2'yi döndür }
    sonuc = eşle(l, iki_kati) olsun
    iddia_et([2, 4, 6], sonuc, "Eşleme fonksiyonu hatalı")'yı döndür
}

test_liste_filtrele fonksiyon olsun {
    l = [1, 2, 3, 4, 5, 6] olsun
    çift_mi fonksiyon olsun n alsın { (n % 2) = 0'ı döndür }
    sonuc = filtrele(l, çift_mi) olsun
    iddia_et([2, 4, 6], sonuc, "Filtreleme fonksiyonu hatalı")'yı döndür
}

test_et("Kare Testi", test_matematik)
test_et("Liste Eşleme Testi", test_liste_esle)
test_et("Liste Filtreleme Testi", test_liste_filtrele)

test_raporu()
