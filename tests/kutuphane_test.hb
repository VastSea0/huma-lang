"matematik.hb"'yi yükle
"renkler.hb"'yi yükle
"zaman.hb"'yi yükle
"liste.hb"'yi yükle

renkli_yaz("--- Hüma Sistem Kütüphaneleri Testi ---", TURKUAZ + KALIN)

// Matematik
"PI: " + PI'yi yazdır
"5'in küpü: " + küpü(5)'i yazdır
"2'nin 10. kuvveti: " + kuvvet(2, 10)'u yazdır

// Renkler
uyarı_yaz("Bu bir sistem uyarısıdır.")
başarı_yaz("Sistem kütüphaneleri başarıyla yüklendi.")

// Liste
meyveler = ["Elma", "Armut", "Muz"] olsun
"Meyveler:"'i yazdır
yazdır_liste(meyveler)

içeriyor_mu(meyveler, "Muz") ise {
    başarı_yaz("Listede Muz bulundu!")
}

"Ters Liste: " + ters_cevir(meyveler)'i yazdır

// Zaman
baslangic = kronometre_başlat() olsun
"Yarım saniye bekleniyor..."'yı yazdır
beklet(0.5)
kronometre_bitir(baslangic)
