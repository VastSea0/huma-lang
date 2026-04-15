// Hüma Native GUI Kütüphanesi — v0.5.0 (Modern Syntax Edition)
// egui tabanlı yerel arayüz oluşturma araçları

GUI_SÜRÜM = "0.5.0" olsun

gui_sürüm_al fonksiyon olsun {
    GUI_SÜRÜM'ü döndür
}

// Pencere oluşturur
pencere_oluştur fonksiyon olsun başlık, genişlik, yükseklik, çizim_fks alsın {
    başlık ile genişlik ve yükseklik ve çizim_fks'ı pencere_başlat
}

// Buton ekler. Boş parametreler göz ardı edilir.
// Kullanım: "Tamam" ile buton_ekle
// Kullanım Renkli: "Tamam" ile 255 ve 0 ve 0'ı renkli_buton_ekle
buton_ekle fonksiyon olsun metin, p1, p2, p3, p4, p5 alsın {
    p1 == boş ise {
        metin'i buton döndür
    } yoksa {
        p2 == boş ise {
            metin'i buton döndür
        } yoksa {
            p3 == boş ise {
                metin ile p1 ve p2'yi buton döndür
            } yoksa {
                p4 == boş ise {
                    metin ile p1 ve p2 ve p3'ü buton döndür
                } yoksa {
                    metin ile p1 ve p2 ve p3 ve p4 ve p5'i buton döndür
                }
            }
        }
    }
}

// Yazı/Etiket ekler
// Kullanım: "Başlık" ile yazı_ekle
yazı_ekle fonksiyon olsun metin, p1, p2, p3 alsın {
    p1 == boş ise {
        metin'i etiket
    } yoksa {
        p2 == boş ise {
            metin ile p1'i etiket
        } yoksa {
            metin ile p1 ve p2 ve p3'ü etiket
        }
    }
}

// Metin kutusu ekler
metin_kutusu_ekle fonksiyon olsun metin, w alsın {
    w == boş ise {
        metin'i girdi_alanı döndür
    } yoksa {
        metin ile w'yi girdi_alanı döndür
    }
}

tema_degistir fonksiyon olsun tema alsın {
    tema'yı tema_ayarla
}

büyük_metin_kutusu_ekle fonksiyon olsun metin alsın {
    metin'i büyük_girdi_alanı döndür
}

// Kaydırıcı ekler
// Kullanım: değer ile 0 ve 100'ü kaydırıcı_ekle
kaydırıcı_ekle fonksiyon olsun değer, min, max alsın {
    değer ile min ve max'ı kaydırıcı döndür
}

// Onay kutusu ekler
// Kullanım: durum ile "Aktif"'i onay_kutusu_ekle
onay_kutusu_ekle fonksiyon olsun durum, metin alsın {
    durum ile metin'i onay_kutusu döndür
}

yan_yana_diz fonksiyon olsun fks alsın {
    fks'ı yan_yana
}

alt_alta_diz fonksiyon olsun fks alsın {
    fks'ı alt_alta
}

ayraç_çiz fonksiyon olsun {
    ayraç()
}

boşluk_bırak fonksiyon olsun miktar alsın {
    miktar'ı boşluk
}

sekme_ekle fonksiyon olsun seçili_mi, metin alsın {
    seçili_mi ile metin'i sekme döndür
}

yüzen_pencere_ekle fonksiyon olsun başlık, açık_mı, fks alsın {
    başlık ile açık_mı ve fks'ı yüzen_pencere döndür
}

menü_çubuğu_ekle fonksiyon olsun fks alsın {
    fks'ı menü_çubuğu
}

açılır_menü_ekle fonksiyon olsun başlık, fks alsın {
    başlık ile fks'ı açılır_menü
}

grup_kutusu_ekle fonksiyon olsun başlık, fks alsın {
    başlık ile fks'ı grup_kutusu
}

grid_ekle fonksiyon olsun id, fks alsın {
    id ile fks'ı grid_oluştur
}

yeni_satır_ekle fonksiyon olsun {
    satır_bitir()
}

kaydırılabilir_liste_ekle fonksiyon olsun id, fks alsın {
    id ile fks'ı kaydırılabilir_alan
}

alan_ayır_ekle fonksiyon olsun w, h, fks alsın {
    w ile h ve fks'ı alan_ayır
}
