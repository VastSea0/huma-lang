// ══════════════════════════════════════════════════════════════════════════════
// derin_ogrenme.hb — Hüma Derin Öğrenme Katman & Teknik Kütüphanesi
// Sürüm: 1.0.0
// ══════════════════════════════════════════════════════════════════════════════
//
// İçerik:
//   - dropout_uygula        (vektör bazlı, eğitim/inference toggle)
//   - batch_norm_katman     (sınıf, öğrenilebilir gamma ve beta ile)
//   - flatten               (matris → vektör düzleştirme)
//   - lr_schedule_stepli    (adım bazlı öğrenme hızı azaltma)
//   - lr_schedule_ustel     (üstel azalma: lr * decay^epoch)
//   - erken_durdurma        (EarlyStopping sınıfı, patience ile)
//   - kayip_izle_genis      (epoch, kayıp, doğruluk yazdır)
//
// Bağımlılıklar: Rust built-in (vektor_dropout, matris_batch_norm,
//   matris_duzenle, matris_satir_ortalamalar, matris_satir_varyanslar)
// ══════════════════════════════════════════════════════════════════════════════

"yapay_zeka_temel.hb"'yi yükle

// ─── Dropout ──────────────────────────────────────────────────────────────────

// dropout_uygula(v, oran, egitim) → Vektör
// oran: 0.0 - 0.9 arası; egitim: 1=eğitim modu, 0=inference (devre dışı)
// Inverted dropout uygulanır: çıkan değerler ölçeklenir (beklenti korunur).
dropout_uygula fonksiyon olsun v, oran, egitim alsın {
    vektor_dropout(v, oran, egitim)'i döndür
}

// matris_dropout_uygula(M, oran, egitim) → Matris
matris_dropout_uygula fonksiyon olsun M, oran, egitim alsın {
    matris_dropout(M, oran, egitim)'i döndür
}

// ─── Flatten ──────────────────────────────────────────────────────────────────

// flatten(M) → Vektör [satirlar * sutunlar]
// Çok katmanlı modellerde tam bağlı katmandan önce düzleştirme için kullanılır.
flatten fonksiyon olsun M alsın {
    boyutlar = matris_boyut(M) olsun
    satirlar = boyutlar[0] olsun
    sutunlar = boyutlar[1] olsun
    toplam = satirlar * sutunlar olsun
    // matris_duzenle(M, 1, toplam) → [1 × toplam] matris, sonra satırı vektöre çevir
    duzlestir = matris_duzenle(M, 1, toplam) olsun
    matris_satir_al(duzlestir, 0)'ı döndür
}

// ─── Batch Normalizasyon Katmanı (sınıf) ──────────────────────────────────────

// Kullanım:
//   bn = batch_norm_katman() olsun
//   bn.ilklendir(ozellik_n) olsun
//   x = bn.ileri(x_matrisi) olsun       // eğitim veya inference
//
// gamma ve beta öğrenilebilir parametrelerdir (Adam ile güncellenebilir).
batch_norm_katman sınıf olsun {

    ilklendir fonksiyon olsun ozellik_n alsın {
        kendisi.ozellik_n = ozellik_n olsun
        kendisi.epsilon = 1e-5 olsun
        // gamma başlangıç: tümü 1
        kendisi.gamma = vektor_olustur(ozellik_n, 1.0) olsun
        // beta başlangıç: tümü 0
        kendisi.beta = vektor_olustur(ozellik_n, 0.0) olsun
    }

    // ileri(X) → Batch normalize edilmiş Matris
    // X: [batch_boyutu × ozellik_n] matris
    ileri fonksiyon olsun X alsın {
        matris_batch_norm(X, kendisi.gamma, kendisi.beta, kendisi.epsilon)'i döndür
    }
}

// ─── Öğrenme Hızı Zamanlayıcıları ─────────────────────────────────────────────

// Adım bazlı: her `adim_boyutu` epoch'ta bir `gama` ile çarp
// lr_schedule_stepli(lr, epoch, adim_boyutu, gama) → yeni lr
lr_schedule_stepli fonksiyon olsun lr, epoch, adim_boyutu, gama alsın {
    adim_sayisi = taban_sayı(epoch / adim_boyutu) olsun
    lr * üs(gama, adim_sayisi)'yı döndür
}

// Üstel azalma: her epoch lr * decay oranında azal
// lr_schedule_ustel(lr_0, epoch, azalma) → lr_0 * azalma^epoch
lr_schedule_ustel fonksiyon olsun lr_0, epoch, azalma alsın {
    lr_0 * üs(azalma, epoch)'ı döndür
}

// Cosine Annealing: smooth sinüs tabanlı azalma
// lr_cosine(lr_min, lr_max, epoch, toplam_epoch) → anlık lr
lr_cosine fonksiyon olsun lr_min, lr_max, epoch, toplam_epoch alsın {
    oran = epoch / toplam_epoch olsun
    lr_min + 0.5 * (lr_max - lr_min) * (1.0 + cos(oran * 3.141592653589793))'i döndür
}

// ─── Erken Durdurma (Early Stopping) ─────────────────────────────────────────

// Kullanım:
//   es = erken_durdurma() olsun
//   es.ilklendir(5, 0.001) olsun   // patience=5, min_delta=0.001
//   dur = es.kontrol(epoch_kayip) olsun
//   dur = 1 ise { kır }
erken_durdurma sınıf olsun {

    ilklendir fonksiyon olsun patience, min_delta alsın {
        kendisi.patience = patience olsun
        kendisi.min_delta = min_delta olsun
        kendisi.en_iyi_kayip = 999999.0 olsun
        kendisi.bekleme = 0 olsun
        kendisi.durdu_mu = 0 olsun
    }

    // kontrol(kayip) → 1 (dur) veya 0 (devam)
    kontrol fonksiyon olsun kayip alsın {
        kendisi.durdu_mu = 1 ise { 1'i döndür }

        (kayip < kendisi.en_iyi_kayip - kendisi.min_delta) ise {
            kendisi.en_iyi_kayip = kayip olsun
            kendisi.bekleme = 0 olsun
        } yoksa {
            kendisi.bekleme = kendisi.bekleme + 1 olsun
        }

        (kendisi.bekleme >= kendisi.patience) ise {
            kendisi.durdu_mu = 1 olsun
            1'i döndür
        }
        0'ı döndür
    }

    sifirla fonksiyon olsun {
        kendisi.en_iyi_kayip = 999999.0 olsun
        kendisi.bekleme = 0 olsun
        kendisi.durdu_mu = 0 olsun
    }
}

// ─── Kayıp ve Doğruluk Görselleştirme ────────────────────────────────────────

// kayip_izle_genis(kayiplar, dogruluklar, epoch, toplam_epoch)
// Her 10 epoch'ta bir kapsamlı çıktı verir.
kayip_izle_genis fonksiyon olsun kayiplar, dogruluklar, epoch, toplam_epoch alsın {
    n = uzunluk(kayiplar) olsun
    n = 0 ise { Boş'u döndür }
    son_kayip = kayiplar[n - 1] olsun
    d_n = uzunluk(dogruluklar) olsun
    (d_n > 0) ise {
        son_dogr = dogruluklar[d_n - 1] olsun
        yazdır "Epoch " + epoch + "/" + toplam_epoch + " — Kayıp: " + son_kayip + " — Doğruluk: " + son_dogr
    } yoksa {
        yazdır "Epoch " + epoch + "/" + toplam_epoch + " — Kayıp: " + son_kayip
    }
    Boş'u döndür
}
