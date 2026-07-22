// ══════════════════════════════════════════════════════════════════════════════
// yapay_zeka_temel.hb — Hüma Yapay Zeka Temel Kütüphanesi
// Sürüm: 1.0.0
// Yazar: Egehan KAHRAMAN
// ══════════════════════════════════════════════════════════════════════════════
//
// Rust Built-in Bağımlılıklar:
//   Blok A: üs, ln, exp, tavan, taban_sayı, klamp
//   Blok B: sigmoid, relu, tanh_aktivasyon, gelu, softmax, log_softmax
//   Blok C: vektor_olustur, ic_carpim, vektor_norm, kosinus_benzerligi vb.
//   Blok D: matris_olustur, matris_carp, matris_transpoz vb.
//   Blok F: normal_rastgele, uniform_rastgele, rastgele_tohum_ata, vektor_karistir
// ══════════════════════════════════════════════════════════════════════════════

"matematik.hb"'yi yükle

// ─── Veri Hazırlama ───────────────────────────────────────────────────────────

// Listeyi [0,1] aralığına normalize et (min-max ölçeklendirme)
min_maks_normalize fonksiyon olsun veri alsın {
    n = uzunluk(veri) olsun
    n = 0 ise { []'i döndür }
    en_k = veri[0] olsun
    en_b = veri[0] olsun
    i = 1'den (n - 1)'e kadar {
        veri[i] < en_k ise { en_k = veri[i] olsun }
        veri[i] > en_b ise { en_b = veri[i] olsun }
    }
    aralik = en_b - en_k olsun
    aralik = 0 ise { []'i döndür }
    sonuc = [] olsun
    i = 0'dan (n - 1)'e kadar {
        norm = (veri[i] - en_k) / aralik olsun
        sonuc = listeye_ekle(sonuc, norm) olsun
    }
    sonuc'u döndür
}

// Z-skoru standardizasyonu (ortalama=0, std=1)
z_skorla fonksiyon olsun veri alsın {
    "istatistik.hb"'yi yükle
    n = uzunluk(veri) olsun
    n = 0 ise { []'i döndür }
    ort = ortalama(veri) olsun
    std = standart_sapma(veri) olsun
    std = 0 ise { []'i döndür }
    sonuc = [] olsun
    i = 0'dan (n - 1)'e kadar {
        z = (veri[i] - ort) / std olsun
        sonuc = listeye_ekle(sonuc, z) olsun
    }
    sonuc'u döndür
}

// Eğitim/Test bölme — oran: 0.0-1.0 arası eğitim oranı
egitim_test_bol fonksiyon olsun veri, etiketler, egitim_orani alsın {
    n = uzunluk(veri) olsun
    egitim_n = tavan(n * egitim_orani) olsun
    egitim_veri = [] olsun
    egitim_etiket = [] olsun
    test_veri = [] olsun
    test_etiket = [] olsun
    i = 0'dan (n - 1)'e kadar {
        i < egitim_n ise {
            egitim_veri = listeye_ekle(egitim_veri, veri[i]) olsun
            egitim_etiket = listeye_ekle(egitim_etiket, etiketler[i]) olsun
        } yoksa {
            test_veri = listeye_ekle(test_veri, veri[i]) olsun
            test_etiket = listeye_ekle(test_etiket, etiketler[i]) olsun
        }
    }
    sonuc = {} olsun
    sonuc["egitim_veri"] = egitim_veri
    sonuc["egitim_etiket"] = egitim_etiket
    sonuc["test_veri"] = test_veri
    sonuc["test_etiket"] = test_etiket
    sonuc'u döndür
}

// Mini-batch oluşturucu — (veri, etiketler, batch_boyutu) → batch listesi
mini_batch_olustur fonksiyon olsun veri, etiketler, batch_boyutu alsın {
    n = uzunluk(veri) olsun
    n = 0 ise { []'i döndür }
    batchler = [] olsun
    i = 0 olsun
    i < n olduğu sürece {
        son = i + batch_boyutu olsun
        son > n ise { son = n olsun }
        batch_v = [] olsun
        batch_e = [] olsun
        j = i'den (son - 1)'e kadar {
            batch_v = listeye_ekle(batch_v, veri[j]) olsun
            batch_e = listeye_ekle(batch_e, etiketler[j]) olsun
        }
        batch = {} olsun
        batch["veri"] = batch_v
        batch["etiketler"] = batch_e
        batchler = listeye_ekle(batchler, batch) olsun
        i = i + batch_boyutu olsun
    }
    batchler'i döndür
}

// ─── Metrik Fonksiyonları ────────────────────────────────────────────────────

// Doğruluk hesapla — ikili sınıflandırma (eşik: 0.5)
dogruluk_hesapla fonksiyon olsun tahminler, gercekler alsın {
    n = uzunluk(tahminler) olsun
    n = 0 ise { 0.0'ı döndür }
    dogru = 0 olsun
    i = 0'dan (n - 1)'e kadar {
        tahmin_sinif = 0 olsun
        tahminler[i] >= 0.5 ise { tahmin_sinif = 1 olsun }
        tahmin_sinif = gercekler[i] ise { dogru = dogru + 1 olsun }
    }
    dogru / n'yi döndür
}

// MSE — Ortalama Kare Hata
mse fonksiyon olsun tahminler, gercekler alsın {
    n = uzunluk(tahminler) olsun
    n = 0 ise { 0.0'ı döndür }
    toplam = 0.0 olsun
    i = 0'dan (n - 1)'e kadar {
        fark = tahminler[i] - gercekler[i] olsun
        toplam = toplam + (fark * fark) olsun
    }
    toplam / n'yi döndür
}

// MAE — Ortalama Mutlak Hata
mae fonksiyon olsun tahminler, gercekler alsın {
    n = uzunluk(tahminler) olsun
    n = 0 ise { 0.0'ı döndür }
    toplam = 0.0 olsun
    i = 0'dan (n - 1)'e kadar {
        fark = mutlak_sayı(tahminler[i] - gercekler[i]) olsun
        toplam = toplam + fark olsun
    }
    toplam / n'yi döndür
}

// İkili çapraz entropi kaybı — güvenli hesaplama
ikili_capraz_entropi fonksiyon olsun tahmin, gercek alsın {
    eps = 1e-7 olsun
    p = klamp(tahmin, eps, 1.0 - eps) olsun
    (gercek * -1.0 * güvenli_ln(p)) - ((1.0 - gercek) * güvenli_ln(1.0 - p))'yı döndür
}

// Kategorik çapraz entropi — log_softmax çıktısı ve tek-sıcak etiket
kategorik_capraz_entropi fonksiyon olsun log_olasıliklar, gercek_sinif alsın {
    log_olasıliklar[gercek_sinif] * -1.0'ı döndür
}

// ─── Kayıp İzleme ─────────────────────────────────────────────────────────────

kaybı_izle fonksiyon olsun kayiplar, epoch, toplam_epoch alsın {
    n = uzunluk(kayiplar) olsun
    n = 0 ise { Boş'u döndür }
    son_kayip = kayiplar[n - 1] olsun
    yazdır "Epoch " + epoch + "/" + toplam_epoch + " — Kayıp: " + son_kayip
    Boş'u döndür
}

// ─── Xavier / He Ağırlık İlklendirme ─────────────────────────────────────────

// Xavier ilklendirmesi — sigmoid/tanh için önerilen
xavier_ilklendir fonksiyon olsun giris_n, cikis_n alsın {
    sinir = karekök(6.0 / (giris_n + cikis_n)) olsun
    uniform_rastgele(-sinir, sinir)'i döndür
}

// He ilklendirmesi — ReLU için önerilen
he_ilklendir fonksiyon olsun giris_n alsın {
    std = karekök(2.0 / giris_n) olsun
    normal_rastgele(0.0, std)'yi döndür
}

// Matris için Xavier ilklendirmesi
matris_xavier_ilklendir fonksiyon olsun satirlar, sutunlar alsın {
    M = matris_olustur(satirlar, sutunlar) olsun
    sinir = karekök(6.0 / (satirlar + sutunlar)) olsun
    i = 0'dan (satirlar - 1)'e kadar {
        j = 0'dan (sutunlar - 1)'e kadar {
            val = uniform_rastgele(-sinir, sinir) olsun
            matris_ata(M, i, j, val) olsun
        }
    }
    M'yi döndür
}

// Matris için He ilklendirmesi
matris_he_ilklendir fonksiyon olsun satirlar, sutunlar alsın {
    M = matris_olustur(satirlar, sutunlar) olsun
    std = karekök(2.0 / satirlar) olsun
    i = 0'dan (satirlar - 1)'e kadar {
        j = 0'dan (sutunlar - 1)'e kadar {
            val = normal_rastgele(0.0, std) olsun
            matris_ata(M, i, j, val) olsun
        }
    }
    M'yi döndür
}
