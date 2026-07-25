yükle "gui.hb";

// DURUM DEĞİŞKENLERİ
saat = 0 olsun
dakika = 0 olsun
saniye = 0 olsun
milis = 0 olsun
calisiyor = 0 olsun
tur_listesi = [] olsun

// YARDIMCI DEĞİŞKENLER
ak_tur_index = 0 olsun
ak_tur_metin = "" olsun

sayacı_sıfırla_fks fonksiyon olsun {
    saat = 0
    dakika = 0
    saniye = 0
    milis = 0
    calisiyor = 0
    tur_listesi = []
}

ilerlet_fks fonksiyon olsun {
    calisiyor == 1 ise {
        milis = milis + 1
        milis >= 60 ise {
            milis = 0
            saniye = saniye + 1
            saniye >= 60 ise {
                saniye = 0
                dakika = dakika + 1
                dakika >= 60 ise {
                    dakika = 0
                    saat = saat + 1
                }
            }
        }
    }
}

kontroller_icerik_fks fonksiyon olsun {
    calisiyor == 0 ise {
        buton_ekle("BAŞLAT", 0, 200, 50, 150, 40) ise {
            calisiyor = 1
        }
    } yoksa {
        buton_ekle("DURDUR", 255, 50, 0, 150, 40) ise {
            calisiyor = 0
        }
    }
    
    boşluk_bırak(20.0)
    buton_ekle("TUR EKLE", 0, 150, 255, 120, 40) ise {
        // Zamanı metne çevirmek için parçalı toplama
        t_metin = "" + saat olsun
        t_metin = t_metin + ":"
        t_metin = t_metin + dakika
        t_metin = t_metin + ":"
        t_metin = t_metin + saniye
        t_metin = t_metin + "."
        t_metin = t_metin + milis
        tur_listesi[t_metin] ekle
    }
    
    boşluk_bırak(20.0)
    buton_ekle("SIFIRLA", 150, 150, 150, 120, 40) ise {
        sayacı_sıfırla_fks()
    }
}

turlar_icerik_fks fonksiyon olsun {
    // Liste elemanlarını yazdırma
    idx = 0 olsun
    uz = tur_listesi'nin uzunluğu olsun
    idx < uz olduğu sürece {
        t_no = "" + (idx + 1) olsun
        t_deger = tur_listesi[idx] olsun
        yazı_ekle(t_no + ". Tur: " + t_deger)
        ayraç_çiz()
        idx = idx + 1
    }
}

ana_cizim_fks fonksiyon olsun {
    ilerlet_fks()
    
    yazı_ekle("⏱️ HÜMA KRONOMETRE", "başlık")
    boşluk_bırak(20.0)
    
    // Zaman göstergesini güvenli oluştur
    z_goster = "" + saat olsun
    z_goster = z_goster + ":"
    z_goster = z_goster + dakika
    z_goster = z_goster + ":"
    z_goster = z_goster + saniye
    z_goster = z_goster + "."
    z_goster = z_goster + milis
    
    yazı_ekle(z_goster, "başlık")
    
    boşluk_bırak(20.0)
    ayraç_çiz()
    boşluk_bırak(10.0)
    
    yan_yana_diz(kontroller_icerik_fks)
    
    boşluk_bırak(20.0)
    yazı_ekle("Kaydedilen Turlar:", "kalın")
    ayraç_çiz()
    
    kaydırılabilir_liste_ekle("tur_scroll", turlar_icerik_fks)
}

pencere_oluştur("Hüma Stopwatch", 500.0, 500.0, ana_cizim_fks)
