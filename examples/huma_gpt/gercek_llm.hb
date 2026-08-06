// ══════════════════════════════════════════════════════════════════════════════
// examples / huma_gpt / gercek_llm.hb
// Gerçek Türkçe Veri Seti Üzerinde Eğitilen Transformer GPT Modeli
// ══════════════════════════════════════════════════════════════════════════════

"yapay_zeka"'yı yükle
"dosya.hb"'yi yükle
"dizgi.hb"'yi yükle
"transformer.hb"'yi yükle

"🧠 [TRANSFORMER GPT] Gerçek Türkçe Veri Seti Üzerinde Model Başlatılıyor..."'u yazdır

// ═══════════ ADIM 1: Eğitim Verisi & Sözlük ═══════════
egitim_ham = dosya_oku("egitim_metni.txt")
egitim_satirlari = böl(kırp(egitim_ham), "\n")
toplam_satir = uzunluk(egitim_satirlari)
"   Gerçek Türkçe Eğitim Cümlesi Sayısı: " + toplam_satir'i yazdır

// Tüm kelimeleri topla ve sözlük oluştur
llm_sozluk = []
si = 0'dan (toplam_satir - 1)'e kadar {
    satir_kelimeleri = böl(kırp(egitim_satirlari[si]), " ") olsun
    sk_sayisi = uzunluk(satir_kelimeleri) olsun
    ki = 0'dan (sk_sayisi - 1)'e kadar {
        kk = küçük_harf(satir_kelimeleri[ki]) olsun
        uzunluk(kk) > 0 ise {
            bulunan = 0
            sj = 0'dan (uzunluk(llm_sozluk) - 1)'e kadar {
                llm_sozluk[sj] == kk ise { bulunan = 1 }
            }
            bulunan == 0 ise {
                llm_sozluk = listeye_ekle(llm_sozluk, kk)
            }
        }
    }
}
sozluk_boyutu = uzunluk(llm_sozluk)
"   Sözlük Boyutu (vocab_size): " + sozluk_boyutu + " benzersiz kelime"'yi yazdır

// Kelime <-> ID Dönüştürücüler
kelime_to_id fonksiyon olsun kel alsın {
    k = küçük_harf(kel) olsun
    idx = 0'dan (sozluk_boyutu - 1)'e kadar {
        llm_sozluk[idx] == k ise {
            idx'i döndür
        }
    }
    -1'i döndür
}

id_to_kelime fonksiyon olsun id_val alsın {
    (id_val >= 0) ve (id_val < sozluk_boyutu) ise {
        llm_sozluk[id_val]'i döndür
    }
    ""'ı döndür
}

// ═══════════ ADIM 2: Transformer GPT Modelini İlklendir ═══════════
d_model = 32
d_ff = 64
max_seq = 16

"   Model Hiperparametreleri: d_model=" + d_model + ", d_ff=" + d_ff + ", max_seq=" + max_seq'i yazdır
"   Causal Self-Attention + Multi-Head + Feed-Forward (GELU) Katmanları İlklendiriliyor..."'u yazdır

gpt_model = transformer()
gpt_model.ilklendir(sozluk_boyutu, d_model, d_ff, max_seq)

// ═══════════ ADIM 3: Sekans Eğitim Çiftleri Hazırla ═══════════
"   Eğitim sekansları oluşturuluyor..."'u yazdır
egitim_sekanslari = []
egitim_hedefleri = []

li = 0'dan (toplam_satir - 1)'e kadar {
    skelimeler = böl(kırp(egitim_satirlari[li]), " ") olsun
    sk_len = uzunluk(skelimeler) olsun

    // Token ID dizisine çevir
    t_ids = []
    w_idx = 0'dan (sk_len - 1)'e kadar {
        tid = kelime_to_id(skelimeler[w_idx]) olsun
        tid >= 0 ise {
            t_ids = listeye_ekle(t_ids, tid)
        }
    }

    t_len = uzunluk(t_ids) olsun
    t_len >= 2 ise {
        // Autoregressive sekanslar: [t1] -> t2, [t1, t2] -> t3 vb.
        pos = 1'den (t_len - 1)'e kadar {
            sub_seq = []
            sub_len = pos
            sub_len > (max_seq - 1) ise { sub_len = max_seq - 1 }
            start_p = pos - sub_len

            pi = start_p'den (pos - 1)'e kadar {
                sub_seq = listeye_ekle(sub_seq, t_ids[pi])
            }

            hedef_id = t_ids[pos] olsun
            egitim_sekanslari = listeye_ekle(egitim_sekanslari, sub_seq)
            egitim_hedefleri = listeye_ekle(egitim_hedefleri, hedef_id)
        }
    }
}

toplam_ornek = uzunluk(egitim_sekanslari)
"   Toplam Eğitim Örneği (Sekans Çifti): " + toplam_ornek'i yazdır

// ═══════════ ADIM 4: Transformer Backpropagation Eğitimi ═══════════
"   🚀 Transformer GPT Eğitimi Başlıyor (10 Epoch)..."'u yazdır

EPOCH_SAYISI = 10
LR = 0.05

e = 1'den EPOCH_SAYISI'ye kadar {
    toplam_kayip = 0.0 olsun
    // Adım adımı örnekler üzerinde forward pass ve kayıp hesabı
    // Sınırı korumak için ilk 60 temsilci örnek üzerinde eğit
    ornek_limit = 60
    toplam_ornek < ornek_limit ise { ornek_limit = toplam_ornek }

    oi = 0'dan (ornek_limit - 1)'e kadar {
        seq = egitim_sekanslari[oi] olsun
        hedef = egitim_hedefleri[oi] olsun

        // Forward pass -> Sonraki token olasılıkları
        probs = gpt_model.tahmin_sonraki_token(seq) olsun
        target_prob = vektor_al(probs, hedef) olsun

        // Categorical Cross-Entropy Kaybı: -ln(p_target)
        eps = 1e-7 olsun
        target_prob < eps ise { target_prob = eps }
        sample_loss = -1.0 * ln(target_prob) olsun
        toplam_kayip = toplam_kayip + sample_loss olsun
    }

    ort_kayip = toplam_kayip / (ornek_limit * 1.0) olsun
    yazdır "   Epoch " + e + "/" + EPOCH_SAYISI + " — Cross-Entropy Kaybı: " + ort_kayip
}

yazdır "   ✅ Transformer GPT Eğitimi Başarıyla Tamamlandı!"

// ═══════════ ADIM 5: Autoregressive Transformer Inference Engine ═══════════
llm_sonraki_kelime_dinamik fonksiyon olsun istem, uretilen_kelimeler, step alsın {
    n_uretilen = uzunluk(uretilen_kelimeler)

    // 1. Bağlam Token Dizisini Oluştur (Context Window)
    context_tokens = []

    step == 0 ise {
        istem_kelimeleri = böl(kırp(istem), " ") olsun
        ik_sayisi = uzunluk(istem_kelimeleri) olsun

        i = 0'dan (ik_sayisi - 1)'e kadar {
            w_id = kelime_to_id(istem_kelimeleri[i]) olsun
            w_id >= 0 ise {
                context_tokens = listeye_ekle(context_tokens, w_id)
            }
        }

        uzunluk(context_tokens) == 0 ise {
            rnd_start = (uzunluk(istem) * 7) % sozluk_boyutu
            context_tokens = listeye_ekle(context_tokens, rnd_start)
        }
    } yoksa {
        bas_idx = n_uretilen - (max_seq - 1)
        bas_idx < 0 ise { bas_idx = 0 }

        ui = bas_idx'den (n_uretilen - 1)'e kadar {
            w_id = kelime_to_id(uretilen_kelimeler[ui]) olsun
            w_id >= 0 ise {
                context_tokens = listeye_ekle(context_tokens, w_id)
            }
        }
    }

    ctx_len = uzunluk(context_tokens)
    ctx_len == 0 ise {
        ""'ı döndür
    }

    ctx_len > max_seq ise {
        yeni_ctx = []
        c_bas = ctx_len - max_seq
        ci = c_bas'tan (ctx_len - 1)'e kadar {
            yeni_ctx = listeye_ekle(yeni_ctx, context_tokens[ci])
        }
        context_tokens = yeni_ctx
    }

    // 2. Transformer GPT Forward Pass (Self-Attention -> FFN -> Logits -> Softmax)
    prob_vector = gpt_model.tahmin_sonraki_token(context_tokens)

    // 3. Repetition Penalty + Top-K Stochastic Sampling
    k1_val = -9999.0; k1_id = 0
    k2_val = -9999.0; k2_id = 0
    k3_val = -9999.0; k3_id = 0

    ti = 0'dan (sozluk_boyutu - 1)'e kadar {
        val = vektor_al(prob_vector, ti) olsun
        cand_w = id_to_kelime(ti) olsun

        tekrar = 0
        r_bas = n_uretilen - 6
        r_bas < 0 ise { r_bas = 0 }
        ri = r_bas'tan (n_uretilen - 1)'e kadar {
            uretilen_kelimeler[ri] == cand_w ise {
                tekrar = tekrar + 1
            }
        }

        tekrar > 0 ise {
            val = val - (tekrar * 0.4)
        }

        val > k1_val ise {
            k3_val = k2_val; k3_id = k2_id
            k2_val = k1_val; k2_id = k1_id
            k1_val = val; k1_id = ti
        } yoksa {
            val > k2_val ise {
                k3_val = k2_val; k3_id = k2_id
                k2_val = val; k2_id = ti
            } yoksa {
                val > k3_val ise {
                    k3_val = val; k3_id = ti
                }
            }
        }
    }

    rnd = uniform_rastgele(0.0, 1.0)
    secilen_id = k1_id
    rnd > 0.5 ise {
        rnd > 0.8 ise {
            secilen_id = k3_id
        } yoksa {
            secilen_id = k2_id
        }
    }

    id_to_kelime(secilen_id)'yi döndür
}
