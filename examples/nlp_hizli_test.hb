// Türkçe durak kelime filtresi için hızlı doğrulama.
"nlp_temel"'i yükle

"İçeriyor testi:"'i yazdır
metin = " | bir | bu | " olsun
arama = " | bir | " olsun
sonuç = içeriyor(metin, arama) olsun
"içeriyor: " + sonuç'u yazdır

arama2 = " | xyz | " olsun
sonuç2 = içeriyor(metin, arama2) olsun
"içeriyor yok: " + sonuç2'yi yazdır

"durak_mı testi:"'i yazdır
d1 = durak_mı("bir") olsun
"bir durak mı: " + d1'i yazdır
d2 = durak_mı("kitap") olsun
"kitap durak mı: " + d2'yi yazdır

"filtreleme testi:"'i yazdır
kelimeler = ["bir", "kitap", "bu", "güzel"] olsun
sonuc = [] olsun
i = 0 olsun
i < uzunluk(kelimeler) olduğu sürece {
    durum = durak_mı(kelimeler[i]) olsun
    "  " + kelimeler[i] + " → durak: " + durum'u yazdır
    durum = 0 ise {
        sonuc = listeye_ekle(sonuc, kelimeler[i]) olsun
    }
    i = i + 1 olsun
}
"Kalan: " + birleştir(sonuc, " ")'i yazdır
"Bitti!"'i yazdır
