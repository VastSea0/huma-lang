// ═══════════════════════════════════════════════════════════════════
// birim_test.hb — Hüma Birim Test Çerçevesi
// Sürüm: 1.1.0
// Yazar: Egehan KAHRAMAN
// ═══════════════════════════════════════════════════════════════════

__test_durumu = {"toplam": 0, "başarılı": 0} olsun

test_et fonksiyon olsun ad, f alsın {
    __test_durumu["toplam"] = __test_durumu["toplam"] + 1 olsun
    "[TEST] " + ad + " ..."'yı yazdır
    sonuc = f() olsun
    sonuc ise {
        "  -> BAŞARILI"'yı yazdır
        __test_durumu["başarılı"] = __test_durumu["başarılı"] + 1 olsun
    } yoksa {
        "  -> !!! HATA !!!"'yı yazdır
    }
}

test_raporu fonksiyon olsun {
    "-----------------------------"'yı yazdır
    "Toplam Test: " + __test_durumu["toplam"]'ı yazdır
    "Başarılı: " + __test_durumu["başarılı"]'yı yazdır
    "Başarısız: " + (__test_durumu["toplam"] - __test_durumu["başarılı"])'yı yazdır
    "-----------------------------"'yı yazdır
}

iddia_et fonksiyon olsun beklenen, gelen, mesaj alsın {
    beklenen = gelen ise { 1'i döndür }
    "  Hata: " + mesaj + " (Beklenen: " + beklenen + ", Gelen: " + gelen + ")"'yi yazdır
    0'ı döndür
}
