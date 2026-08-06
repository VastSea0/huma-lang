// ══════════════════════════════════════════════════════════════════════════════
// examples / huma_gpt / transformer.hb
// Gerçek Transformer GPT (Self-Attention + FFN) Mimarisi
// ══════════════════════════════════════════════════════════════════════════════

"yapay_zeka"'yı yükle
"dosya.hb"'yi yükle

transformer sınıf olsun {

    // ─── İlklendirme ──────────────────────────────────────────────────────────
    ilklendir fonksiyon olsun vocab_size, d_model, d_ff, max_seq alsın {
        kendisi.vocab_size = vocab_size olsun
        kendisi.d_model = d_model olsun
        kendisi.d_ff = d_ff olsun
        kendisi.max_seq = max_seq olsun

        // 1. Embedding Ağırlıkları
        kendisi.W_token = matris_xavier_ilklendir_builtin(vocab_size, d_model) olsun
        kendisi.W_pos = matris_xavier_ilklendir_builtin(max_seq, d_model) olsun

        // 2. Self-Attention Ağırlıkları (Q, K, V, O Projeksiyonları)
        kendisi.W_Q = matris_xavier_ilklendir_builtin(d_model, d_model) olsun
        kendisi.W_K = matris_xavier_ilklendir_builtin(d_model, d_model) olsun
        kendisi.W_V = matris_xavier_ilklendir_builtin(d_model, d_model) olsun
        kendisi.W_O = matris_xavier_ilklendir_builtin(d_model, d_model) olsun

        // 3. Feed-Forward Network Ağırlıkları
        kendisi.W_ff1 = matris_he_ilklendir_builtin(d_model, d_ff) olsun
        kendisi.W_ff2 = matris_xavier_ilklendir_builtin(d_ff, d_model) olsun

        // 4. Output Head (Unembedding)
        kendisi.W_out = matris_xavier_ilklendir_builtin(d_model, vocab_size) olsun
    }

    // ─── Token + Positional Embedding ─────────────────────────────────────────
    // token_ids: Liste <tamsayı> [seq_len]
    // Çıktı: Matris [seq_len × d_model]
    embed fonksiyon olsun token_ids alsın {
        seq_len = uzunluk(token_ids) olsun
        X = matris_olustur(seq_len, kendisi.d_model, 0.0) olsun

        i = 0'dan (seq_len - 1)'e kadar {
            tid = token_ids[i] olsun
            tok_v = matris_satir_al(kendisi.W_token, tid) olsun
            pos_v = matris_satir_al(kendisi.W_pos, i) olsun
            comb_v = vektor_topla(tok_v, pos_v) olsun
            matris_satir_ata(X, i, comb_v) olsun
        }

        X'i döndür
    }

    // ─── Causal Self-Attention Katmanı ────────────────────────────────────────
    // X: Matris [seq_len × d_model]
    // Çıktı: Matris [seq_len × d_model]
    self_attention fonksiyon olsun X alsın {
        dim = matris_boyutu(X) olsun
        seq_len = dim[0] olsun
        d = kendisi.d_model olsun

        // Q, K, V Matrisleri: X * W_Q, X * W_K, X * W_V
        Q = matris_carp(X, kendisi.W_Q) olsun
        K = matris_carp(X, kendisi.W_K) olsun
        V = matris_carp(X, kendisi.W_V) olsun

        // Scores = (Q * K^T) / sqrt(d_model)
        K_T = matris_transpoz(K) olsun
        Scores = matris_carp(Q, K_T) olsun
        scale = 1.0 / karekök(d * 1.0) olsun
        Scores = matris_skalar_carp(Scores, scale) olsun

        // Causal Masking: Gelecekteki token'lara -10000.0 maske uygula (autoregressive)
        i = 0'dan (seq_len - 1)'e kadar {
            j = (i + 1)'den (seq_len - 1)'e kadar {
                matris_ata(Scores, i, j, -10000.0) olsun
            }
        }

        // Softmax(Scores) -> Attention Olasılıkları [seq_len × seq_len]
        Attn_probs = batch_softmax(Scores) olsun

        // Context = Attn_probs * V -> [seq_len × d_model]
        Context = matris_carp(Attn_probs, V) olsun

        // Output Projection + Residual Connection
        Out = matris_carp(Context, kendisi.W_O) olsun
        Result = matris_topla(Out, X) olsun

        Result'u döndür
    }

    // ─── Feed-Forward Network (FFN) ──────────────────────────────────────────
    // X: Matris [seq_len × d_model]
    // Çıktı: Matris [seq_len × d_model]
    ffn fonksiyon olsun X alsın {
        // H = GELU(X * W_ff1) -> [seq_len × d_ff]
        H1 = matris_carp(X, kendisi.W_ff1) olsun
        H_act = matris_gelu(H1) olsun

        // Out = H_act * W_ff2 -> [seq_len × d_model]
        FFN_out = matris_carp(H_act, kendisi.W_ff2) olsun

        // Residual Connection
        Result = matris_topla(FFN_out, X) olsun

        Result'u döndür
    }

    // ─── Tam Forward Pass ─────────────────────────────────────────────────────
    // token_ids: Liste <tamsayı> [seq_len]
    // Çıktı: Logits Matrisi [seq_len × vocab_size]
    forward fonksiyon olsun token_ids alsın {
        // 1. Embedding
        X = kendisi.embed(token_ids) olsun

        // 2. Transformer Block (Self-Attention + FFN)
        X_attn = kendisi.self_attention(X) olsun
        X_ffn = kendisi.ffn(X_attn) olsun

        // 3. Unembedding Output Projection
        Logits = matris_carp(X_ffn, kendisi.W_out) olsun

        Logits'i döndür
    }

    // ─── Sonraki Token Tahmini (Inference) ────────────────────────────────────
    // token_ids: Liste <tamsayı> [seq_len]
    // Çıktı: Sonraki token'ın olasılık vektörü [vocab_size]
    tahmin_sonraki_token fonksiyon olsun token_ids alsın {
        seq_len = uzunluk(token_ids) olsun
        Logits = kendisi.forward(token_ids) olsun

        // Son pozisyondaki logit vektörünü al
        son_logit_v = matris_satir_al(Logits, seq_len - 1) olsun

        // Softmax uygula -> Olasılık Dağılımı
        prob_v = softmax(son_logit_v) olsun
        prob_v'yi döndür
    }
}
