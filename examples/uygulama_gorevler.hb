yükle "renkler.hb";
yükle "dosya.hb";
yükle "dizgi.hb";
yükle "liste.hb";

DOSYA = "gorevler.txt" olsun

gorevleri_oku fonksiyon olsun {
    dosya_var_mı(DOSYA) = 0 ise { []'yi döndür }
    icerik = dosya_oku(DOSYA) olsun
    satırlara_ayır(icerik)'i döndür
}

gorevleri_kaydet fonksiyon olsun gorev_listesi alsın {
    metin = "" olsun
    i = 0 olsun
    i < uzunluk(gorev_listesi) olduğu sürece {
        metin = metin + gorev_listesi[i] + "\n" olsun
        i = i + 1 olsun
    }
    dosya_yaz(DOSYA, metin)
}

renkli_yaz("--- HÜMA GÖREV YÖNETİCİSİ ---", TURKUAZ + KALIN)

gorevler = gorevleri_oku() olsun

1 olduğu sürece {
    ""'yi yazdır
    "1. Görevleri Listele"'yi yazdır
    "2. Yeni Görev Ekle"'yi yazdır
    "3. Görev Sil"'yi yazdır
    "4. Çıkış"'yı yazdır
    
    secim = oku("> Seçiminiz: ") olsun
    
    secim = "1" ise {
        uzunluk(gorevler) = 0 ise {
            uyarı_yaz("Henüz görev yok.")
        } yoksa {
            "--- GÖREV LİSTESİ ---"'yi yazdır
            i = 0 olsun
            i < uzunluk(gorevler) olduğu sürece {
                (i + 1) + ". " + gorevler[i]'yi yazdır
                i = i + 1 olsun
            }
        }
    } yoksa secim = "2" ise {
        yeni = oku("Görev metni: ") olsun
        gorevler = listeye_ekle(gorevler, yeni) olsun
        gorevleri_kaydet(gorevler)
        başarı_yaz("Görev eklendi.")
    } yoksa secim = "3" ise {
        no = oku("Silinecek görev no: ") olsun
        yeni_liste = [] olsun
        i = 0 olsun
        hedef = no - 1 olsun
        i < uzunluk(gorevler) olduğu sürece {
            i != hedef ise {
                yeni_liste = listeye_ekle(yeni_liste, gorevler[i]) olsun
            }
            i = i + 1 olsun
        }
        gorevler = yeni_liste olsun
        gorevleri_kaydet(gorevler)
        başarı_yaz("Görev silindi.")
    } yoksa secim = "4" ise {
        "Güle güle!"'yi yazdır
        0'ı döndür
    } yoksa {
        hata_yaz("Geçersiz seçim!")
    }
}
