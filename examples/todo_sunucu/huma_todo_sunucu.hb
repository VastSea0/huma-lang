yükle "huma_sunucu";

// DURUM YÖNETİMİ
dosya_adi = "veriler.json" olsun
// Dosyadan verileri nesne olarak yükleyelim
gorevler = metinden_nesneye(dosya_oku(dosya_adi)) olsun

verileri_kaydet fonksiyon olsun {
    dosya_yaz(dosya_adi, nesneden_metine(gorevler))
}

sunucu = Sunucu() olsun
sunucu.kur(3000)

// 1. ANA SAYFA (HTML SERVİSİ)
sunucu.getir("/", fonksiyon olsun istek, cevap alsın {
    html_kod = dosya_oku("index.html") olsun
    cevap.html(html_kod)
})

// 2. TÜM GÖREVLERİ LİSTELE
sunucu.getir("/api/gorevler", fonksiyon olsun istek, cevap alsın {
    cevap.json(gorevler)
})

// 3. YENİ GÖREV EKLE
sunucu.gönder("/api/ekle", fonksiyon olsun istek, cevap alsın {
    yeni = metinden_nesneye(istek.gövde) olsun
    yeni.id = zaman() olsun // Basit ID üretimi
    yeni.tamamlandı = 0 olsun
    
    gorevler'e [yeni]'yi ekle
    verileri_kaydet()
    
    r = metinden_nesneye("{}") olsun
    r.durum = "başarılı" olsun
    r.gorev = yeni olsun
    cevap.json(r)
})

// 4. GÖREV SİL
sunucu.gönder("/api/sil", fonksiyon olsun istek, cevap alsın {
    body = metinden_nesneye(istek.gövde) olsun
    sil_id = body.id olsun
    
    yeni_list = [] olsun
    i = 0 olsun
    uz = gorevler'in uzunluğu olsun
    
    i < uz olduğu sürece {
        g = gorevler[i] olsun
        g_id = g.id olsun
        g_id != sil_id ise {
            yeni_list'ye [g]'yi ekle
        }
        i = i + 1 olsun
    }
    
    gorevler = yeni_list
    verileri_kaydet()
    
    r = metinden_nesneye("{}") olsun
    r.durum = "başarılı" olsun
    cevap.json(r)
})

sunucu.baslat()