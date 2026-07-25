yükle "birim_test.hb";
yükle "matematik.hb";
yükle "liste.hb";
yükle "dizgi.hb";

// ═══════════════════════════════════════════════════════════════════
// HÜMA PROGRAMLAMA DİLİ KAPSAMLI TEST SÜİTİ
// ═══════════════════════════════════════════════════════════════════

// --- KATEGORİ 1: Değişkenler ve Veri Tipleri ---
test_et("1.1 Sayı Veri Tipi", fonksiyon olsun {
    x = 42 olsun
    y = -15.5 olsun
    sonuc = iddia_et(42, x, "Tam sayı atama") ve iddia_et(-15.5, y, "Odalıklı sayı atama") olsun
    sonuc'u döndür
})

test_et("1.2 Metin Veri Tipi", fonksiyon olsun {
    metin = "Hüma Dili" olsun
    iddia_et("Hüma Dili", metin, "Metin atama")'yı döndür
})

test_et("1.3 Mantıksal Veri Tipi", fonksiyon olsun {
    b1 = doğru olsun
    b2 = yanlış olsun
    sonuc = iddia_et(doğru, b1, "Mantıksal doğru") ve iddia_et(yanlış, b2, "Mantıksal yanlış") olsun
    sonuc'u döndür
})

test_et("1.4 Boş Değer", fonksiyon olsun {
    b = boş olsun
    iddia_et(boş, b, "Boş değer atama")'yı döndür
})

// --- KATEGORİ 2: Operatörler ve Aritmetik ---
test_et("2.1 Aritmetik Operatörler", fonksiyon olsun {
    t = 10 + 5 olsun
    c = 20 - 8 olsun
    c2 = 4 * 3 olsun
    b = 20 / 4 olsun
    m = 17 % 5 olsun
    
    sonuc = iddia_et(15, t, "Toplama") ve
    iddia_et(12, c, "Çıkarma") ve 
    iddia_et(12, c2, "Çarpma") ve 
    iddia_et(5, b, "Bölme") ve 
    iddia_et(2, m, "Mod alma") olsun
    sonuc'u döndür
})

test_et("2.2 Mantıksal ve Karşılaştırma Operatörleri", fonksiyon olsun {
    k1 = (10 > 5) olsun
    k2 = (3 == 3) olsun
    k3 = (4 != 4) olsun
    k4 = (doğru ve doğru) olsun
    k5 = (doğru veya yanlış) olsun
    k6 = değil yanlış olsun
    
    sonuc = iddia_et(doğru, k1, "Büyüktür") ve
    iddia_et(doğru, k2, "Eşittir") ve 
    iddia_et(yanlış, k3, "Eşit değildir") ve 
    iddia_et(doğru, k4, "Ve operatörü") ve 
    iddia_et(doğru, k5, "Veya operatörü") ve 
    iddia_et(doğru, k6, "Değil operatörü") olsun
    sonuc'u döndür
})

// --- KATEGORİ 3: Türkçe Doğal Dil Ek Sistemi (Suffixes) ---
test_et("3.1 Türkçe Ek Temizleme", fonksiyon olsun {
    sayı = 100 olsun
    
    // Kesme işaretli eklerin değişken ismi olarak çözümlenmesi
    a = sayı'yı olsun
    b = sayı'dan olsun
    c = sayı'ya olsun
    d = sayı'da olsun
    
    sonuc = iddia_et(100, a, "Belirtme eki ('yı)") ve
    iddia_et(100, b, "Ayrılma eki ('dan)") ve
    iddia_et(100, c, "Yönelme eki ('ya)") ve
    iddia_et(100, d, "Bulunma eki ('da)") olsun
    sonuc'u döndür
})

// --- KATEGORİ 4: Akış Kontrolü ve Koşullar ---
test_et("4.1 Koşullu İfadeler (ise / yoksa)", fonksiyon olsun {
    değer = 15 olsun
    sonuç = "" olsun
    
    değer > 10 ise {
        sonuç = "büyük" olsun
    } yoksa {
        sonuç = "küçük" olsun
    }
    
    iddia_et("büyük", sonuç, "ise bloğu çalıştı")'yı döndür
})

// --- KATEGORİ 5: Döngüler ---
test_et("5.1 'olduğu sürece' Döngüsü", fonksiyon olsun {
    toplam = 0 olsun
    i = 1 olsun
    i <= 5 olduğu sürece {
        toplam = toplam + i olsun
        i = i + 1 olsun
    }
    iddia_et(15, toplam, "Sürece döngüsü ile 1'den 5'e toplam")'ı döndür
})

test_et("5.2 'kadar' Aralık Döngüsü", fonksiyon olsun {
    toplam = 0 olsun
    k = 1'den 5'e kadar {
        toplam = toplam + k olsun
    }
    iddia_et(15, toplam, "Kadar döngüsü ile 1'den 5'e toplam")'ı döndür
})

// --- KATEGORİ 6: Fonksiyonlar ve Özyineleme ---
faktöriyel_hesapla fonksiyon olsun n alsın {
    n <= 1 ise {
        1'i döndür
    } yoksa {
        (n * faktöriyel_hesapla(n - 1))'i döndür
    }
}

test_et("6.1 Özyinelemeli Fonksiyon (Recursion)", fonksiyon olsun {
    f5 = faktöriyel_hesapla(5) olsun
    iddia_et(120, f5, "5! = 120")'yi döndür
})

// --- KATEGORİ 7: Nesne Yönelim (Sınıflar & Metotlar) ---
araba sınıf olsun {
    hız = 0 olsun
    
    hızlan fonksiyon olsun miktar alsın {
        kendisi'nin hız'ı = kendisi'nin hız'ı + miktar olsun
        kendisi'nin hız'ı'nı döndür
    }
}

test_et("7.1 Sınıf ve Metot Çağrısı", fonksiyon olsun {
    a1 = araba() olsun
    a1.hızlan(50)
    son_hız = a1.hızlan(20) olsun
    iddia_et(70, son_hız, "Sınıf metodu hız artışı")'ni döndür
})

// --- KATEGORİ 8: Listeler ve Sözlükler ---
test_et("8.1 Liste Operasyonları", fonksiyon olsun {
    meyveler = ["Elma", "Armut"] olsun
    meyveler'e ["Muz"]'u ekle
    uzunluk = meyveler'in uzunluğu olsun
    iddia_et(3, uzunluk, "Liste eleman ekleme ve uzunluk")'u döndür
})

test_et("8.2 Sözlük Operasyonları", fonksiyon olsun {
    kullanıcı = { "ad": "Hüma", "yaş": 2.5 } olsun
    ad = kullanıcı["ad"] olsun
    iddia_et("Hüma", ad, "Sözlük anahtar erişimi")'yi döndür
})

// --- KATEGORİ 9: Standart Kütüphaneler ---
test_et("9.1 Matematik Kütüphanesi", fonksiyon olsun {
    k = karesi(4) olsun
    kuv = kuvvet(2, 3) olsun
    sonuc = iddia_et(16, k, "karesi(4)") ve iddia_et(8, kuv, "kuvvet(2, 3)") olsun
    sonuc'u döndür
})

test_et("9.2 Dizgi Kütüphanesi", fonksiyon olsun {
    d = büyük_harf("hüma") olsun
    iddia_et("HÜMA", d, "büyük_harf('hüma')")'yi döndür
})

// Test sonuç raporunu yazdır
test_raporu()
