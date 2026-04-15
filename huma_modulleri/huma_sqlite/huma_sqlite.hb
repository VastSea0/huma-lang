// huma_sqlite.hb - Hüma Native SQLite Kütüphanesi
// Sürüm: 2.0.0 (Modern Syntax Edition)

Veritabanı sınıf olsun {
    id = 0 olsun
    yol = "" olsun

    kur fonksiyon olsun dosya_yolu alsın {
        kendisi'nin yol'u = dosya_yolu olsun
        kendisi'nin id'i = dosya_yolu ile dahili_sql_bağlan
        
        kendisi'nin id'i == boş ise {
            "Hata: Veritabanına bağlanılamadı: " + dosya_yolu'nu yazdır
        }
    }

    yürüt fonksiyon olsun sql alsın {
        kendisi'nin id'i ile sql'i dahili_sql_yürüt döndür
    }

    sorgula fonksiyon olsun sql alsın {
        kendisi'nin id'i ile sql'i dahili_sql_sorgula döndür
    }
}
