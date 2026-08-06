// ══════════════════════════════════════════════════════════════════════════════
// nlp_ileri/tf_idf.hb — TF-IDF ve BoW Vektör Temsili
// Sürüm: 1.0.0
// ══════════════════════════════════════════════════════════════════════════════

"yapay_zeka_temel.hb"'yi yükle

// ─── Bag of Words (BoW) ───────────────────────────────────────────────────────

// Corpus'tan sözlük oluştur
sozluk_olustur fonksiyon olsun corpus_tokens alsın {
    sozluk = {} olsun
    idx = 0 olsun
    n_belge = uzunluk(corpus_tokens) olsun
    i = 0'dan (n_belge - 1)'e kadar {
        tokens = corpus_tokens[i] olsun
        m = uzunluk(tokens) olsun
        j = 0'dan (m - 1)'e kadar {
            kelime = tokens[j] olsun
            mevcut = sozluk[kelime] olsun
            mevcut = Boş ise {
                sozluk[kelime] = idx
                idx = idx + 1 olsun
            }
        }
    }
    sonuc = {} olsun
    sonuc["sozluk"] = sozluk
    sonuc["boyut"] = idx
    sonuc'u döndür
}

// Tek belgeyi BoW vektörüne çevir
bow_vektoru fonksiyon olsun tokens, sozluk_veri alsın {
    boyut = sozluk_veri["boyut"] olsun
    sozluk = sozluk_veri["sozluk"] olsun
    v = vektor_olustur(boyut, 0.0) olsun
    n = uzunluk(tokens) olsun
    i = 0'dan (n - 1)'e kadar {
        idx = sozluk[tokens[i]] olsun
        idx = Boş ise { devam }
        mevcut = vektor_al(v, idx) olsun
        vektor_ata(v, idx, mevcut + 1.0) olsun
    }
    v'yi döndür
}

// ─── TF (Term Frequency) ─────────────────────────────────────────────────────

tf_hesapla fonksiyon olsun tokens, sozluk_veri alsın {
    v = bow_vektoru(tokens, sozluk_veri) olsun
    toplam = vektor_uzunluk(v) olsun
    // Normalize: tf(t) = count(t) / toplam_kelime
    n_token = uzunluk(tokens) olsun
    n_token = 0 ise { v'yi döndür }
    boyut = sozluk_veri["boyut"] olsun
    i = 0'dan (boyut - 1)'e kadar {
        tf = vektor_al(v, i) / n_token olsun
        vektor_ata(v, i, tf) olsun
    }
    v'yi döndür
}

// ─── IDF (Inverse Document Frequency) ────────────────────────────────────────

idf_hesapla fonksiyon olsun corpus_tokens, sozluk_veri alsın {
    boyut = sozluk_veri["boyut"] olsun
    sozluk = sozluk_veri["sozluk"] olsun
    n_belge = uzunluk(corpus_tokens) olsun
    belge_frekanslari = vektor_olustur(boyut, 0.0) olsun

    i = 0'dan (n_belge - 1)'e kadar {
        tokens = corpus_tokens[i] olsun
        // Bu belgede görülen token'ları tekil say
        goruldu = {} olsun
        m = uzunluk(tokens) olsun
        j = 0'dan (m - 1)'e kadar {
            kelime = tokens[j] olsun
            goruldu_mu = goruldu[kelime] olsun
            goruldu_mu = Boş ise {
                goruldu[kelime] = 1
                idx = sozluk[kelime] olsun
                idx = Boş ise { devam }
                mevcut = vektor_al(belge_frekanslari, idx) olsun
                vektor_ata(belge_frekanslari, idx, mevcut + 1.0) olsun
            }
        }
    }

    // idf(t) = ln(N / (1 + df(t))) + 1  (smooth IDF)
    idf_v = vektor_olustur(boyut, 0.0) olsun
    i = 0'dan (boyut - 1)'e kadar {
        df = vektor_al(belge_frekanslari, i) olsun
        idf_val = güvenli_ln(n_belge / (1.0 + df)) + 1.0 olsun
        vektor_ata(idf_v, i, idf_val) olsun
    }
    idf_v'yi döndür
}

// ─── TF-IDF Birleştirme ────────────────────────────────────────────────────

tfidf_vektoru fonksiyon olsun tokens, sozluk_veri, idf_v alsın {
    tf_v = tf_hesapla(tokens, sozluk_veri) olsun
    vektor_carpi(tf_v, idf_v)'yi döndür
}

// Tüm corpus için TF-IDF matrisi
tfidf_matrisi fonksiyon olsun corpus_tokens alsın {
    sozluk_veri = sozluk_olustur(corpus_tokens) olsun
    idf_v = idf_hesapla(corpus_tokens, sozluk_veri) olsun
    n_belge = uzunluk(corpus_tokens) olsun
    boyut = sozluk_veri["boyut"] olsun
    M = matris_olustur(n_belge, boyut, 0.0) olsun
    i = 0'dan (n_belge - 1)'e kadar {
        satir = tfidf_vektoru(corpus_tokens[i], sozluk_veri, idf_v) olsun
        matris_satir_ata(M, i, satir) olsun
    }
    sonuc = {} olsun
    sonuc["matris"] = M
    sonuc["sozluk"] = sozluk_veri
    sonuc["idf"] = idf_v
    sonuc'u döndür
}

// ─── Kosinüs Benzerliği ile Belge Sıralama ────────────────────────────────

en_benzer_belgeler fonksiyon olsun sorgu_tokens, corpus_tokens, top_k alsın {
    sozluk_veri = sozluk_olustur(corpus_tokens) olsun
    idf_v = idf_hesapla(corpus_tokens, sozluk_veri) olsun
    sorgu_v = tfidf_vektoru(sorgu_tokens, sozluk_veri, idf_v) olsun
    n = uzunluk(corpus_tokens) olsun
    benzerlikler = [] olsun
    i = 0'dan (n - 1)'e kadar {
        belge_v = tfidf_vektoru(corpus_tokens[i], sozluk_veri, idf_v) olsun
        sim = kosinus_benzerligi(sorgu_v, belge_v) olsun
        benzerlikler = listeye_ekle(benzerlikler, [i, sim]) olsun
    }
    // Basit sıralama (bubble sort — küçük corpus için yeterli)
    m = uzunluk(benzerlikler) olsun
    i = 0'dan (m - 2)'ye kadar {
        j = 0'dan (m - i - 2)'ye kadar {
            benzerlikler[j][1] < benzerlikler[j + 1][1] ise {
                gecici = benzerlikler[j] olsun
                benzerlikler[j] = benzerlikler[j + 1] olsun
                benzerlikler[j + 1] = gecici olsun
            }
        }
    }
    // Top-k döndür
    sonuc = [] olsun
    son = top_k olsun
    son > m ise { son = m olsun }
    i = 0'dan (son - 1)'e kadar {
        sonuc = listeye_ekle(sonuc, benzerlikler[i]) olsun
    }
    sonuc'u döndür
}
