// huma_sqlite.hb - Hüma Native SQLite Kütüphanesi

Veritabanı sınıf olsun {
    id = 0 olsun
    yol = "" olsun

    kur fonksiyon olsun dosya_yolu alsın {
        kendisi'nin yol'u = dosya_yolu olsun
        kendisi'nin id'i = dahili_sql_bağlan(dosya_yolu)
        
        kendisi'nin id'i == boş ise {
            "Hata: Veritabanına bağlanılamadı: " + dosya_yolu'nu yazdır
        }
    }

    yürüt fonksiyon olsun sql alsın {
        dahili_sql_yürüt(kendisi'nin id'i, sql) döndür
    }

    sorgula fonksiyon olsun sql alsın {
        dahili_sql_sorgula(kendisi'nin id'i, sql) döndür
    }
}
