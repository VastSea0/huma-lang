// ══════════════════════════════════════════════════════════════════════════════
// examples / huma_gpt / gercek_llm.hb
// Transformer GPT Mimarisini Kullanan Gerçek Yapay Zeka Çıkarım Modülü
// ══════════════════════════════════════════════════════════════════════════════

"yapay_zeka"'yı yükle
"dosya.hb"'yi yükle
"dizgi.hb"'yi yükle
"transformer.hb"'yi yükle

"🧠 [TRANSFORMER GPT] Gerçek Transformer Mimarisi Başlatılıyor..."'u yazdır

// ═══════════ ADIM 1: Eğitim Verisi & Sözlük ═══════════
egitim_ham = dosya_oku("egitim_metni.txt")
egitim_kelimeleri = böl(kırp(egitim_ham), " ")
toplam_kelime = uzunluk(egitim_kelimeleri)
"   Eğitim verisi kelime sayısı: " + toplam_kelime'yi yazdır

// Benzersiz sözlük
llm_sozluk = []
si = 0'dan (toplam_kelime - 1)'e kadar {
    kk = egitim_kelimeleri[si] olsun
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
sozluk_boyutu = uzunluk(llm_sozluk)
"   Sözlük boyutu (vocab_size): " + sozluk_boyutu'nu yazdır

// Kelime <-> ID Dönüştürücüler
kelime_to_id fonksiyon olsun kel alsın {
    k = küçük_harf(kel) olsun
    idx = 0'dan (sozluk_boyutu - 1)'e kadar {
        küçük_harf(llm_sozluk[idx]) == k ise {
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

// ═══════════ ADIM 2: Transformer GPT Model İlklendirme ═══════════
// d_model=16, d_ff=32, max_seq=16
d_model = 16
d_ff = 32
max_seq = 16

"   Transformer Konfigürasyonu: d_model=" + d_model + ", d_ff=" + d_ff + ", max_seq=" + max_seq'i yazdır
"   Causal Self-Attention + Multi-Head + Feed-Forward (GELU) Katmanları Oluşturuluyor..."'u yazdır

gpt_model = transformer()
gpt_model.ilklendir(sozluk_boyutu, d_model, d_ff, max_seq)

"   ✅ Transformer GPT Mimarisi Hazır!"'ı yazdır

// ═══════════ ADIM 3: Autoregressive Transformer Inference Engine ═══════════
// istem: Kullanıcının girdiği metin
// uretilen_kelimeler: Şimdiye kadar üretilmiş kelimeler
// step: Kaçıncı adım
llm_sonraki_kelime_dinamik fonksiyon olsun istem, uretilen_kelimeler, step alsın {
    n_uretilen = uzunluk(uretilen_kelimeler)

    // 1. Bağlam Token Dizisini Oluştur (Context Window)
    context_tokens = []

    step == 0 ise {
        // İstemdeki kelimeleri sözlük ID'lerine çevir
        istem_kelimeleri = böl(kırp(istem), " ") olsun
        ik_sayisi = uzunluk(istem_kelimeleri) olsun
        
        i = 0'dan (ik_sayisi - 1)'e kadar {
            w_id = kelime_to_id(istem_kelimeleri[i]) olsun
            w_id >= 0 ise {
                context_tokens = listeye_ekle(context_tokens, w_id)
            }
        }

        // Eğer istemdeki hiçbir kelime sözlükte yoksa, varsayılan token ile başla
        uzunluk(context_tokens) == 0 ise {
            rnd_start = (uzunluk(istem) * 7) % sozluk_boyutu
            context_tokens = listeye_ekle(context_tokens, rnd_start)
        }
    } yoksa {
        // Üretilen kelimeleri token ID'lerine dönüştür (son max_seq kadar)
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

    // Context boyutu max_seq'i aşarsa kırp
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
    // En yüksek olasılıklı 3 adayı bul
    k1_val = -9999.0; k1_id = 0
    k2_val = -9999.0; k2_id = 0
    k3_val = -9999.0; k3_id = 0

    ti = 0'dan (sozluk_boyutu - 1)'e kadar {
        val = vektor_al(prob_vector, ti) olsun
        cand_w = id_to_kelime(ti) olsun

        // Son 6 kelimede bu aday geçti mi? (Tekrar cezası)
        tekrar = 0
        r_bas = n_uretilen - 6
        r_bas < 0 ise { r_bas = 0 }
        ri = r_bas'tan (n_uretilen - 1)'e kadar {
            uretilen_kelimeler[ri] == cand_w ise {
                tekrar = tekrar + 1
            }
        }

        tekrar > 0 ise {
            val = val - (tekrar * 0.3)
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

    // Olasılık ağırlıklı rastgele seçim (Temperature Sampling)
    rnd = uniform_rastgele(0.0, 1.0)
    secilen_id = k1_id
    rnd > 0.55 ise {
        rnd > 0.85 ise {
            secilen_id = k3_id
        } yoksa {
            secilen_id = k2_id
        }
    }

    // Seçilen kelimeyi döndür
    id_to_kelime(secilen_id)'yi döndür
}
