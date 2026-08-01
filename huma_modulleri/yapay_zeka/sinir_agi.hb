// ══════════════════════════════════════════════════════════════════════════════
// yapay_zeka/sinir_agi.hb — Yüksek Seviyeli Sinir Ağı API
// Sürüm: 1.0.0
// ══════════════════════════════════════════════════════════════════════════════
//
// Kullanım:
//   model = sinir_agi() olsun
//   model.ilklendir() olsun
//   model.katman_ekle(128, 64, "relu") olsun
//   model.katman_ekle(64, 2, "softmax") olsun
//   model.egit(veri, etiketler, 100, 0.001) olsun
//   tahmin = model.tahmin_et(ornek) olsun
//   model.kaydet("model.json") olsun
// ══════════════════════════════════════════════════════════════════════════════

"kayip.hb"'yi yükle
"yogun_katman.hb"'yi yükle

sinir_agi sınıf olsun {

    ilklendir fonksiyon olsun {
        kendisi.katmanlar = [] olsun
        kendisi.katman_sayisi = 0 olsun
        kendisi.egitim_kayiplari = [] olsun
    }

    // Yeni yoğun katman ekle
    katman_ekle fonksiyon olsun giris_n, cikis_n, aktivasyon alsın {
        k = yogun_katman() olsun
        k.ilklendir(giris_n, cikis_n, aktivasyon) olsun
        kendisi.katmanlar = listeye_ekle(kendisi.katmanlar, k) olsun
        kendisi.katman_sayisi = kendisi.katman_sayisi + 1 olsun
    }

    // İleri geçiş — tüm katmanlardan sırayla
    tahmin_et fonksiyon olsun giris alsın {
        x = giris olsun
        i = 0'dan (kendisi.katman_sayisi - 1)'e kadar {
            k = kendisi.katmanlar[i] olsun
            x = k.ileri(x) olsun
        }
        x'i döndür
    }

    // Geri yayılım — tüm katmanlardan ters sırayla
    geri_yayil fonksiyon olsun grad, ogrenme_hizi alsın {
        g = grad olsun
        i = kendisi.katman_sayisi - 1 olsun
        i >= 0 olduğu sürece {
            k = kendisi.katmanlar[i] olsun
            g = k.geri(g, ogrenme_hizi) olsun
            i = i - 1 olsun
        }
        g'yi döndür
    }

    // Tek örnek üzerinde eğitim adımı (MSE kaybı ile)
    egitim_adimi fonksiyon olsun giris, gercek_cikis, ogrenme_hizi alsın {
        tahmin = kendisi.tahmin_et(giris) olsun
        // MSE kaybı
        n = vektor_uzunluk(tahmin) olsun
        kayip = 0.0 olsun
        grad = vektor_olustur(n, 0.0) olsun
        i = 0'dan (n - 1)'e kadar {
            t = vektor_al(tahmin, i) olsun
            g = vektor_al(gercek_cikis, i) olsun
            kayip = kayip + mse_kayip(t, g) olsun
            vektor_ata(grad, i, mse_gradyan(t, g)) olsun
        }
        kayip = kayip / n olsun
        kendisi.geri_yayil(grad, ogrenme_hizi) olsun
        kayip'i döndür
    }

    // Toplu eğitim — epoch sayısı kadar tekrar
    egit fonksiyon olsun veri, etiketler, epoch_sayisi, ogrenme_hizi alsın {
        n = uzunluk(veri) olsun
        e = 1'den epoch_sayisi'ye kadar {
            epoch_kayip = 0.0 olsun
            i = 0'dan (n - 1)'e kadar {
                giris_v = listeye_vektor(veri[i]) olsun
                gercek_v = listeye_vektor(etiketler[i]) olsun
                k = kendisi.egitim_adimi(giris_v, gercek_v, ogrenme_hizi) olsun
                epoch_kayip = epoch_kayip + k olsun
            }
            epoch_kayip = epoch_kayip / n olsun
            kendisi.egitim_kayiplari = listeye_ekle(kendisi.egitim_kayiplari, epoch_kayip) olsun
            (e % 10 = 0) ise {
                yazdır "Epoch " + e + "/" + epoch_sayisi + " — Kayıp: " + epoch_kayip
            }
        }
    }

    // Doğruluk hesapla — skalar çıkışlı model için (0/1 sınıflandırma)
    dogruluk_degerlendir fonksiyon olsun veri, etiketler alsın {
        n = uzunluk(veri) olsun
        dogru_sayisi = 0 olsun
        i = 0'dan (n - 1)'e kadar {
            giris_v = listeye_vektor(veri[i]) olsun
            tahmin = kendisi.tahmin_et(giris_v) olsun
            tahmin_val = vektor_al(tahmin, 0) olsun
            gercek = etiketler[i] olsun
            tahmin_sinif = 0 olsun
            tahmin_val >= 0.5 ise { tahmin_sinif = 1 olsun }
            tahmin_sinif = gercek ise { dogru_sayisi = dogru_sayisi + 1 olsun }
        }
        dogru_sayisi / n'yi döndür
    }

    // Çok-sınıflı doğruluk değerlendirmesi (softmax çıkışlı modeller için)
    // veri: liste<vektör>, etiketler: liste<tamsayı> (0-tabanlı sınıf indeksi)
    dogruluk_cok_sinif fonksiyon olsun veri, etiketler alsın {
        n = uzunluk(veri) olsun
        n = 0 ise { 0.0'ı döndür }
        dogru = 0 olsun
        i = 0'dan (n - 1)'e kadar {
            giris_v = listeye_vektor(veri[i]) olsun
            tahmin = kendisi.tahmin_et(giris_v) olsun
            // En yüksek aktivasyonlu çıkış sınıfı
            tahmin_sinif = vektor_argmax(tahmin) olsun
            gercek_sinif = etiketler[i] olsun
            tahmin_sinif = gercek_sinif ise { dogru = dogru + 1 olsun }
        }
        dogru / n'yi döndür
    }

    // argmax_tahmin(giris) → tahmin edilen sınıf indeksi (skaler)
    // Doğrudan sınıf numarası döndürür — softmax olasılıkları değil
    argmax_tahmin fonksiyon olsun giris alsın {
        t = kendisi.tahmin_et(giris) olsun
        vektor_argmax(t)'ı döndür
    }

    // Tek örnek üzerinde cross-entropy eğitim adımı (softmax son katman)
    // gercek_sinif: 0-tabanlı sınıf indeksi (skaler)
    egitim_adimi_ce fonksiyon olsun giris, gercek_sinif, ogrenme_hizi alsın {
        tahmin = kendisi.tahmin_et(giris) olsun
        n_sinif = vektor_uzunluk(tahmin) olsun

        // Kategorik cross-entropy gradyanı: (p_i - y_i)
        // gercek sınıf için: grad = p - 1, diğerleri için: grad = p - 0
        grad = vektor_olustur(n_sinif, 0.0) olsun
        kayip = 0.0 olsun
        j = 0'dan (n_sinif - 1)'e kadar {
            p = vektor_al(tahmin, j) olsun
            j_kopya = j + 0 olsun      // döngü değişkenini kopyala
            y = 0.0 olsun
            j_kopya = gercek_sinif ise { y = 1.0 olsun }
            grad_j = p - y olsun
            vektor_ata(grad, j, grad_j) olsun
            // Sadece gercek_sinif'teki kayıp hesaplanır
            j_kopya = gercek_sinif ise {
                eps = 1e-7 olsun
                p_safe = klamp(p, eps, 1.0 - eps) olsun
                kayip = -1.0 * güvenli_ln(p_safe) olsun
            }
        }
        kendisi.geri_yayil(grad, ogrenme_hizi) olsun
        kayip'i döndür
    }

    // Model kaydet — ağırlıkları JSON formatında dosyaya yazar
    kaydet fonksiyon olsun yol alsın {
        model_sozluk = {} olsun
        model_sozluk["katman_sayisi"] = kendisi.katman_sayisi
        i = 0'dan (kendisi.katman_sayisi - 1)'e kadar {
            k = kendisi.katmanlar[i] olsun
            katman_anahtar = "katman_" + i olsun
            katman_veri = {} olsun
            katman_veri["giris_n"] = k.giris_n
            katman_veri["cikis_n"] = k.cikis_n
            katman_veri["aktivasyon"] = k.aktivasyon
            // W matrisini satır-satır kaydet
            w_satırlar = [] olsun
            r = k.cikis_n olsun
            j = 0'dan (r - 1)'e kadar {
                satir_v = matris_satir_al(k.W, j) olsun
                satir_l = vektore_liste(satir_v) olsun
                w_satırlar = listeye_ekle(w_satırlar, satir_l) olsun
            }
            katman_veri["W"] = w_satırlar
            katman_veri["b"] = vektore_liste(k.b)
            model_sozluk[katman_anahtar] = katman_veri
        }
        json_metin = nesneden_metine(model_sozluk) olsun
        dosya_yaz(yol, json_metin) olsun
        yazdır "Model kaydedildi: " + yol
    }

    // Model yükle — JSON dosyasından ağırlıkları okur
    model_yukle fonksiyon olsun yol alsın {
        icerik = dosya_oku(yol) olsun
        icerik = Boş ise { yazdır "Hata: Model dosyası bulunamadı: " + yol }
        veri = metinden_nesneye(icerik) olsun
        katman_n = veri["katman_sayisi"] olsun
        kendisi.katmanlar = [] olsun
        kendisi.katman_sayisi = katman_n olsun
        i = 0'dan (katman_n - 1)'e kadar {
            katman_anahtar = "katman_" + i olsun
            kv = veri[katman_anahtar] olsun
            k = yogun_katman() olsun
            k.ilklendir(kv["giris_n"], kv["cikis_n"], kv["aktivasyon"]) olsun
            // W ağırlıklarını yükle
            w_satırlar = kv["W"] olsun
            r = kv["cikis_n"] olsun
            c = kv["giris_n"] olsun
            j = 0'dan (r - 1)'e kadar {
                satir = w_satırlar[j] olsun
                satir_v = listeye_vektor(satir) olsun
                matris_satir_ata(k.W, j, satir_v) olsun
            }
            // b yanlılıklarını yükle
            b_liste = kv["b"] olsun
            b_boyut = uzunluk(b_liste) olsun
            j = 0'dan (b_boyut - 1)'e kadar {
                vektor_ata(k.b, j, b_liste[j]) olsun
            }
            kendisi.katmanlar = listeye_ekle(kendisi.katmanlar, k) olsun
        }
        yazdır "Model yüklendi: " + yol
    }
}
