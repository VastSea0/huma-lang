yükle "huma_sunucu";
yükle "huma_sqlite";

// VERİTABANI YÖNETİMİ
vt = Veritabanı() olsun

"Kütüphane veritabanına bağlanılıyor..."'ı yazdır
vt'nin kur'u("kutuphaneler.db")

vt'nin id'i == boş ise {
    "Hata: Veritabanı bağlantısı kurulamadı!"'ı yazdır
} yoksa {
    "Veritabanı bağlantısı başarılı (ID: " + vt'nin id'i + ")"'ı yazdır
}

// Tabloyu oluştur (eğer yoksa)
"Tablo kontrol ediliyor..."'ı yazdır
vt'nin yürüt'ü("CREATE TABLE IF NOT EXISTS kutuphaneler (ad TEXT PRIMARY KEY, aciklama TEXT, yazar TEXT, github TEXT, surum TEXT, durum TEXT, indirme_sayisi INTEGER)")

sunucu = Sunucu() olsun
sunucu.kur(8080)

// 1. ANA SAYFA
sunucu.getir("/", fonksiyon olsun istek, cevap alsın {
    html = dosya_oku("template_index.html") olsun
    cevap.html(html)
})

// 2. KÜTÜPHANE EKLE SAYFASI
sunucu.getir("/ekle", fonksiyon olsun istek, cevap alsın {
    html = dosya_oku("template_ekle.html") olsun
    cevap.html(html)
})

// 3. API - TÜM KÜTÜPHANELER
sunucu.getir("/api/kutuphaneler", fonksiyon olsun istek, cevap alsın {
    veriler = vt'nin sorgula'sı("SELECT * FROM kutuphaneler") olsun
    cevap.json(veriler)
})

// 4. API - YENİ KÜTÜPHANE GÖNDER
sunucu.gönder("/api/gonder", fonksiyon olsun istek, cevap alsın {
    yok = metinden_nesneye(istek.gövde) olsun
    
    // Gelen verileri SQL'e ekle
    ad = değer_al(yok, "ad") olsun
    aciklama = değer_al(yok, "aciklama") olsun
    yazar = değer_al(yok, "yazar") olsun
    github = değer_al(yok, "github") olsun
    surum = değer_al(yok, "surum") olsun
    
    sorgu = "INSERT INTO kutuphaneler (ad, aciklama, yazar, github, surum, durum, indirme_sayisi) VALUES ('" + ad + "', '" + aciklama + "', '" + yazar + "', '" + github + "', '" + surum + "', 'bekliyor', 0)" olsun
    vt'nin yürüt'ü(sorgu)
    
    sonuc = metinden_nesneye("{}") olsun
    değer_ata(sonuc, "durum", "başarılı")
    cevap.json(sonuc)
})

// 5. ADMIN - BEKLEYENLERİ LİSTELE
sunucu.getir("/admin", fonksiyon olsun istek, cevap alsın {
    html = dosya_oku("template_admin.html") olsun
    cevap.html(html)
})

// 6. API - ONAYLA
sunucu.gönder("/api/onayla", fonksiyon olsun istek, cevap alsın {
    body = metinden_nesneye(istek.gövde) olsun
    hedef_ad = değer_al(body, "ad") olsun
    
    sorgu = "UPDATE kutuphaneler SET durum = 'onaylandı' WHERE ad = '" + hedef_ad + "'" olsun
    vt'nin yürüt'ü(sorgu)
    
    sonuc = metinden_nesneye("{}") olsun
    değer_ata(sonuc, "durum", "başarılı")
    cevap.json(sonuc)
})

"Hüma Kütüphane Sunucusu başlatılıyor..."'ı yazdır
sunucu.baslat()