// Hüma Native GUI Kütüphanesi — v1.0.0
// Dear ImGui (dear-imgui-rs) tabanlı, hızlı ve hafif yerel arayüz oluşturma araçları.
// Kişiselleştirme için bkz: tema_ayarla, tema_olustur, tema_listele.

GUI_SÜRÜM = "1.0.0" olsun

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

// ===================================================================
// KİŞİSELLEŞTİRME (TEMA)
// ===================================================================
// tema_ayarla("gece_mavisi")               -> hazır tema uygula
// tema_ayarla(tema_olustur(1,168,85,247,10.0,8.0)) -> özel tema uygula
// tema_listele()                            -> hazır tema adları listesi
// Hazır temalar: koyu, açık, gece_mavisi, mor_alacakaranlık, orman,
//                gün_batımı, okyanus, kiraz, mono

// ===================================================================
// EK BİLEŞENLER (v1.0.0)
// ===================================================================

// İlerleme çubuğu ekler (değer 0.0-1.0 arası)
// Kullanım: ilerleme_çubuğu_ekle(0.42)
// Kullanım (etiketli): ilerleme_çubuğu_ekle(0.42, "%42")
ilerleme_çubuğu_ekle fonksiyon olsun değer, etiket alsın {
    etiket == boş ise {
        ilerleme_çubuğu(değer) döndür
    } yoksa {
        ilerleme_çubuğu(değer, etiket) döndür
    }
}

// Radyo düğmesi ekler; tıklanırsa 1 döner
// Kullanım: radyo_düğmesi_ekle(seçim, "Seçenek A")
radyo_düğmesi_ekle fonksiyon olsun seçili_mi, metin alsın {
    radyo_düğmesi(seçili_mi, metin) döndür
}

// Açılır liste (combo box) ekler; güncel seçili indeksi döndürür
// Kullanım: açılır_liste_ekle(secili_indeks, ["Kırmızı","Yeşil","Mavi"])
açılır_liste_ekle fonksiyon olsun seçili_indeks, seçenekler alsın {
    açılır_liste(seçili_indeks, seçenekler) döndür
}

// Renk seçici ekler; [r,g,b] listesi döndürür
// Kullanım: renk_seçici_ekle(renk[0], renk[1], renk[2])
renk_seçici_ekle fonksiyon olsun r, g, b alsın {
    renk_seçici(r, g, b) döndür
}

// Tıklanabilir bağlantı ekler; tarayıcıda hedef adresi açar
// Kullanım: bağlantı_ekle("Hüma'yı GitHub'da gör", "https://github.com/...")
bağlantı_ekle fonksiyon olsun metin, adres alsın {
    bağlantı(metin, adres) döndür
}

// Native menü öğesi ekler (açılır_menü_ekle içinde kullanılır)
menü_ögesi_ekle fonksiyon olsun metin alsın {
    menü_ögesi(metin) döndür
}

// Katlanabilir/açılabilir ağaç düğümü (collapsing header) ekler
ağaç_düğümü_ekle fonksiyon olsun başlık, fks alsın {
    ağaç_düğümü(başlık, fks) döndür
}

// Kenarlıklı, başlıksız "kart" paneli ekler (görsel gruplama için)
kart_ekle fonksiyon olsun fks alsın {
    kart(fks) döndür
}

// Gerçek (native) sekme çubuğu oluşturur; içinde sekme_sayfası_ekle çağrıları olmalı
// Kullanım:
//   içerik fonksiyon olsun {
//       sekme_sayfası_ekle("Sayfa 1", sayfa1_fks)
//       sekme_sayfası_ekle("Sayfa 2", sayfa2_fks)
//   }
//   sekme_grubu_ekle("ana_sekmeler", içerik)
sekme_grubu_ekle fonksiyon olsun id, fks alsın {
    sekme_grubu(id, fks) döndür
}

// sekme_grubu_ekle içinde bir sekme sayfası tanımlar; yalnızca aktifken fks çalışır
sekme_sayfası_ekle fonksiyon olsun başlık, fks alsın {
    sekme_sayfası(başlık, fks) döndür
}
