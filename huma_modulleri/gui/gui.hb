// Hüma Native GUI Kütüphanesi
// egui tabanlı yerel arayüz oluşturma araçları

GUI_SÜRÜM = "0.4.0" olsun

gui_sürüm_al fonksiyon olsun {
    GUI_SÜRÜM'ü döndür
}

pencere_oluştur fonksiyon olsun başlık, genişlik, yükseklik, çizim_fks alsın {
    pencere_başlat(başlık, genişlik, yükseklik, çizim_fks)
}

// Birleşik Buton API'sı
// Kullanım Boş: buton_ekle("Metin")
// Kullanım Renkli: buton_ekle("Metin", R, G, B)
// Kullanım Boyutlu: buton_ekle("Metin", W, H)
// Kullanım Full: buton_ekle("Metin", R, G, B, W, H)
buton_ekle fonksiyon olsun metin, p1, p2, p3, p4, p5 alsın {
    p1 == boş ise {
        buton(metin)'i döndür
    } yoksa {
        p2 == boş ise {
            buton(metin)'i döndür
        } yoksa {
            p3 == boş ise {
                buton(metin, p1, p2)'i döndür
            } yoksa {
                p4 == boş ise {
                    buton(metin, p1, p2, p3)'i döndür
                } yoksa {
                    buton(metin, p1, p2, p3, p4, p5)'i döndür
                }
            }
        }
    }
}

// Birleşik Yazı/Etiket API'sı
// Kullanım: yazı_ekle("Metin")
// Kullanım Stil: yazı_ekle("Metin", "kalın") // "kalın", "eğik", "başlık"
// Kullanım Renkli: yazı_ekle("Metin", R, G, B)
yazı_ekle fonksiyon olsun metin, p1, p2, p3 alsın {
    p1 == boş ise {
        etiket(metin)
    } yoksa {
        p2 == boş ise {
            etiket(metin, p1)
        } yoksa {
            etiket(metin, p1, p2, p3)
        }
    }
}

// Metin kutusu ekler. İsteğe bağlı genişlik alabilir.
metin_kutusu_ekle fonksiyon olsun metin, w alsın {
    w == boş ise {
        girdi_alanı(metin)'i döndür
    } yoksa {
        girdi_alanı(metin, w)'i döndür
    }
}

tema_degistir fonksiyon olsun tema alsın {
    tema_ayarla(tema)
}

büyük_metin_kutusu_ekle fonksiyon olsun metin alsın {
    büyük_girdi_alanı(metin)'i döndür
}

kaydırıcı_ekle fonksiyon olsun değer, min, max alsın {
    kaydırıcı(değer, min, max)'ı döndür
}

onay_kutusu_ekle fonksiyon olsun durum, metin alsın {
    onay_kutusu(durum, metin)'i döndür
}

yan_yana_diz fonksiyon olsun fks alsın {
    yan_yana(fks)
}

alt_alta_diz fonksiyon olsun fks alsın {
    alt_alta(fks)
}

ayraç_çiz fonksiyon olsun {
    ayraç()
}

boşluk_bırak fonksiyon olsun miktar alsın {
    boşluk(miktar)
}

sekme_ekle fonksiyon olsun seçili_mi, metin alsın {
    sekme(seçili_mi, metin)'i döndür
}

yüzen_pencere_ekle fonksiyon olsun başlık, açık_mı, fks alsın {
    yüzen_pencere(başlık, açık_mı, fks)'ı döndür
}

menü_çubuğu_ekle fonksiyon olsun fks alsın {
    menü_çubuğu(fks)
}

açılır_menü_ekle fonksiyon olsun başlık, fks alsın {
    açılır_menü(başlık, fks)
}

grup_kutusu_ekle fonksiyon olsun başlık, fks alsın {
    grup_kutusu(başlık, fks)
}

grid_ekle fonksiyon olsun id, fks alsın {
    grid_oluştur(id, fks)
}

yeni_satır_ekle fonksiyon olsun {
    satır_bitir()
}

kaydırılabilir_liste_ekle fonksiyon olsun id, fks alsın {
    kaydırılabilir_alan(id, fks)
}

alan_ayır_ekle fonksiyon olsun w, h, fks alsın {
    alan_ayır(w, h, fks)
}
