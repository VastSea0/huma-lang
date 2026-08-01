yükle "gui.hb";

// Hüma Native GUI v1.0.0 tanıtım uygulaması — Dear ImGui tabanlı
// Kişiselleştirme (tema), yeni bileşenler ve gerçek sekme çubuğu gösterimi.

gui_ver = gui_sürüm_al() olsun
("Kullanılan GUI Sürümü: " + gui_ver)'i yazdır

// Global durum değişkenleri
isim = "Hüma Geliştiricisi" olsun
yaş = 20 olsun
onay_verdi_mi = 0 olsun
sayaç = 0 olsun
pencere_acik = 0 olsun
ilerleme = 0.35 olsun
secili_tema_indeks = 0 olsun
temalar = tema_listele() olsun
renk = [168, 85, 247] olsun
secili_meyve = 0 olsun
meyveler = ["Elma", "Armut", "Muz", "Çilek"] olsun

// ===================
// SAYFA 1 (Profil)
// ===================

yas_bilgisi fonksiyon olsun {
    yazı_ekle("Seçilen Yaş: " + yaş)
    yaş > 50 ise {
        yazı_ekle(" (Yarım asrı devirmişsiniz!)", "kalın")
    }
}

profil_sayfasi fonksiyon olsun {
    yazı_ekle("PROFİL BİLGİLERİ", "başlık")
    boşluk_bırak(8.0)

    isim = metin_kutusu_ekle(isim, 300.0)

    boşluk_bırak(5.0)
    yazı_ekle("Merhaba " + isim + "!", 0, 150, 255)

    boşluk_bırak(5.0)
    ayraç_çiz()
    boşluk_bırak(5.0)

    yaş = kaydırıcı_ekle(yaş, 0, 100)
    yan_yana_diz(yas_bilgisi)

    boşluk_bırak(10.0)
    bağlantı_ekle("Hüma projesini GitHub'da görüntüle", "https://github.com/VastSea0/huma-lang")
}

// ===================
// SAYFA 2 (Kişiselleştirme)
// ===================

tema_secici fonksiyon olsun {
    secili_tema_indeks = açılır_liste_ekle(secili_tema_indeks, temalar)
    boşluk_bırak(8.0)
    buton_ekle("Bu Temayı Uygula", 140.0, 32.0) ise {
        tema_ayarla(temalar[secili_tema_indeks])
    }
}

ozel_tema_uygula fonksiyon olsun {
    renk = renk_seçici_ekle(renk[0], renk[1], renk[2])
    boşluk_bırak(8.0)
    buton_ekle("Özel Temayı Uygula", 160.0, 32.0) ise {
        ozel_tema = tema_olustur(1, renk[0], renk[1], renk[2], 10.0, 8.0) olsun
        tema_ayarla(ozel_tema)
    }
}

kisisellestirme_sayfasi fonksiyon olsun {
    yazı_ekle("HAZIR TEMALAR", "başlık")
    boşluk_bırak(6.0)
    ağaç_düğümü_ekle("Hazır temalardan seç", tema_secici)

    boşluk_bırak(10.0)
    ağaç_düğümü_ekle("Kendi temanı oluştur", ozel_tema_uygula)

    boşluk_bırak(12.0)
    onay_verdi_mi = onay_kutusu_ekle(onay_verdi_mi, "Geliştirici İstatistiklerine İzin Ver")
    onay_verdi_mi == 1 ise {
        yazı_ekle("Teşekkürler, anonim veriler arka planda toplanıyor.", "eğik")
    }
}

// ===================
// SAYFA 3 (Bileşenler)
// ===================

buton_islemleri fonksiyon olsun {
    buton_ekle("Sayacı Artır", 50, 200, 50, 150.0, 40.0) ise {
        sayaç = sayaç + 1
        ilerleme = ilerleme + 0.05
        ilerleme > 1.0 ise { ilerleme = 0.0 }
    }
    buton_ekle("Sıfırla", 255, 100, 100, 100.0, 40.0) ise {
        sayaç = 0
        ilerleme = 0.0
    }
}

bilesenler_sayfasi fonksiyon olsun {
    yazı_ekle("YENİ BİLEŞENLER", "başlık")
    boşluk_bırak(8.0)

    kart_ekle(buton_islemleri)
    boşluk_bırak(8.0)
    yazı_ekle("Sayaç: " + sayaç, "kalın")
    ilerleme_çubuğu_ekle(ilerleme)

    boşluk_bırak(12.0)
    secili_meyve = açılır_liste_ekle(secili_meyve, meyveler)
    yazı_ekle("Seçilen meyve: " + meyveler[secili_meyve])

    boşluk_bırak(10.0)
    buton_ekle("Yüzen Pencereyi Aç", 200, 40) ise {
        pencere_acik = 1
    }
}

// ===================
// YÜZEN PENCERE
// ===================

pencere_icerigi fonksiyon olsun {
    yazı_ekle("Dikkat!", "başlık")
    yazı_ekle("Ben bir yüzen (floating) pencereyim!", 255, 100, 100)
    yazı_ekle("Mevcut Sayaç: " + sayaç)

    buton_ekle("Beni Kapat") ise {
        pencere_acik = 0
    }
}

// ===================
// ÜST MENÜ
// ===================

dosya_menusu fonksiyon olsun {
    menü_ögesi_ekle("Kaydet") ise {
        "Kaydet'e tıklandı."'nı yazdır
    }
}

ust_menu fonksiyon olsun {
    açılır_menü_ekle("Dosya", dosya_menusu)
}

// ===================
// ANA ÇİZİM DÖNGÜSÜ (gerçek sekme çubuğu ile)
// ===================

sekmeler_fks fonksiyon olsun {
    sekme_sayfası_ekle("Profil", profil_sayfasi)
    sekme_sayfası_ekle("Kişiselleştirme", kisisellestirme_sayfasi)
    sekme_sayfası_ekle("Bileşenler", bilesenler_sayfasi)
}

çizim_fks fonksiyon olsun {
    menü_çubuğu_ekle(ust_menu)
    boşluk_bırak(5.0)

    "huma_ana_sekmeler" ile sekmeler_fks'ı sekme_grubu_ekle

    pencere_acik == 1 ise {
        pencere_acik = yüzen_pencere_ekle("Uyarı Ekranı", pencere_acik, pencere_icerigi)
    }
}

pencere_oluştur("Hüma Native GUI v1.0.0", 640.0, 520.0, çizim_fks)
