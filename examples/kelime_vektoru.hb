// ══════════════════════════════════════════════════════════════════════════════
// kelime_vektoru.hb — Türkçe Kelime Vektörü Eğitimi (Word2Vec Skip-gram Benzeri)
// ══════════════════════════════════════════════════════════════════════════════

"yapay_zeka_temel.hb"'yi yükle
"nlp_temel/nlp_temel.hb"'yi yükle
"nlp_ileri/gomme.hb"'yi yükle

yazdır "══════════════════════════════════════════════════"
yazdır "  Hüma — Türkçe Kelime Vektörü Eğitimi"
yazdır "  Skip-gram benzeri, 32 boyutlu gömme"
yazdır "══════════════════════════════════════════════════"
yazdır ""

// ─── Mini Türkçe Corpus ───────────────────────────────────────────────────────
corpus = [
    "Türkiye büyük ve güzel bir ülkedir",
    "Ankara Türkiye nin başkentidir",
    "İstanbul Türkiye nin en büyük şehridir",
    "Türkçe eklemeli bir dildir",
    "Dil öğrenmek zihin için faydalıdır",
    "Yapay zeka dil işlemede kullanılır",
    "Makine öğrenmesi büyük veri gerektirir",
    "Sinir ağları insan beyninden esinlenmiştir",
    "Kelimeler anlamlarıyla birbirine bağlıdır",
    "Metin sınıflandırma NLP görevidir"
] olsun

// ─── Tokenizasyon ve Sözlük Oluşturma ────────────────────────────────────────
yazdır "Corpus işleniyor..."
proc = metin_islemci() olsun
tum_tokens = [] olsun
belge_tokens = [] olsun
i = 0'dan (uzunluk(corpus) - 1)'e kadar {
    tokens = proc.durak_filtrele(proc.tokenize(corpus[i])) olsun
    belge_tokens = listeye_ekle(belge_tokens, tokens) olsun
    j = 0'dan (uzunluk(tokens) - 1)'e kadar {
        tum_tokens = listeye_ekle(tum_tokens, tokens[j]) olsun
    }
}

// Benzersiz sözlük oluştur
sozluk = {} olsun
ters_sozluk = {} olsun
sozluk_boyutu = 0 olsun
i = 0'dan (uzunluk(tum_tokens) - 1)'e kadar {
    kelime = tum_tokens[i] olsun
    mevcut = sozluk[kelime] olsun
    mevcut = Boş ise {
        sozluk[kelime] = sozluk_boyutu
        id_str = metne_çevir(sozluk_boyutu) olsun
        ters_sozluk[id_str] = kelime
        sozluk_boyutu = sozluk_boyutu + 1 olsun
    }
}
yazdır "Sözlük boyutu: " + sozluk_boyutu

// ─── Gömme Tabakası Oluşturma ─────────────────────────────────────────────────
GOMME_BOYUTU = 32 olsun
yazdır "32 boyutlu gömme tabakası oluşturuluyor..."
rastgele_tohum_ata(123) olsun
gomme = gomme_tabakasi() olsun
gomme.ilklendir(sozluk_boyutu, GOMME_BOYUTU) olsun

// ─── Skip-gram Eğitim Çiftleri ────────────────────────────────────────────────
// Pencere boyutu = 2: her kelime etrafındaki 2 kelimeyi hedef al
PENCERE = 2 olsun
egitim_ciftleri = [] olsun
i = 0'dan (uzunluk(belge_tokens) - 1)'e kadar {
    tokens = belge_tokens[i] olsun
    n = uzunluk(tokens) olsun
    j = 0'dan (n - 1)'e kadar {
        merkez = tokens[j] olsun
        merkez_id = sozluk[merkez] olsun
        merkez_id = Boş ise { devam }
        bas = j - PENCERE olsun
        bas < 0 ise { bas = 0 olsun }
        son = j + PENCERE olsun
        son >= n ise { son = n - 1 olsun }
        k = bas'tan son'a kadar {
            k = j ise { devam }
            komsu = tokens[k] olsun
            komsu_id = sozluk[komsu] olsun
            komsu_id = Boş ise { devam }
            cift = [merkez_id, komsu_id] olsun
            egitim_ciftleri = listeye_ekle(egitim_ciftleri, cift) olsun
        }
    }
}
yazdır "Eğitim çifti sayısı: " + uzunluk(egitim_ciftleri)

// ─── Basit Skip-gram Eğitim (SGD) ─────────────────────────────────────────────
EPOCH = 20 olsun
LR = 0.05 olsun
yazdır ""
yazdır "Eğitim başlıyor (" + EPOCH + " epoch)..."

e = 1'den EPOCH'a kadar {
    toplam_kayip = 0.0 olsun
    n_cift = uzunluk(egitim_ciftleri) olsun
    i = 0'dan (n_cift - 1)'e kadar {
        merkez_id = egitim_ciftleri[i][0] olsun
        hedef_id = egitim_ciftleri[i][1] olsun
        // İleri geçiş: merkez gömme * hedef gömme → skalar skor
        merkez_v = gomme.token_al(merkez_id) olsun
        hedef_v = gomme.token_al(hedef_id) olsun
        skor = sigmoid(ic_carpim(merkez_v, hedef_v)) olsun
        // Gerçek = 1 (pozitif çift)
        kayip = ikili_capraz_entropi(skor, 1.0) olsun
        toplam_kayip = toplam_kayip + kayip olsun
        // Gradyan — sigmoid'in BCE gradyanı: (sigma - y)
        grad_skalar = skor - 1.0 olsun
        // Her iki gömme için gradyan güncelle
        grad_merkez = vektor_skalar_carp(hedef_v, grad_skalar) olsun
        grad_hedef = vektor_skalar_carp(merkez_v, grad_skalar) olsun
        gomme.guncelle(merkez_id, grad_merkez, LR) olsun
        gomme.guncelle(hedef_id, grad_hedef, LR) olsun
    }
    ort_kayip = toplam_kayip / n_cift olsun
    (e % 5 = 0) ise {
        yazdır "Epoch " + e + "/" + EPOCH + " — Ortalama Kayıp: " + ort_kayip
    }
}

// ─── Benzer Kelimeler ─────────────────────────────────────────────────────────
yazdır ""
yazdır "══════════════════════════════════════════════════"
yazdır "Öğrenilen kelime ilişkileri (en yakın komşular):"
yazdır "══════════════════════════════════════════════════"

test_kelimeleri = ["türkiye", "dil", "yapay", "kelimeler"] olsun
i = 0'dan (uzunluk(test_kelimeleri) - 1)'e kadar {
    kelime = test_kelimeleri[i] olsun
    kid = sozluk[kelime] olsun
    kid = Boş ise { devam }
    benzerler = gomme.en_benzer_tokenler(kid, 3) olsun
    benzer_metinler = "" olsun
    j = 0'dan (uzunluk(benzerler) - 1)'e kadar {
        bid = benzerler[j][0] olsun
        bid_str = metne_çevir(bid) olsun
        b_kelime = ters_sozluk[bid_str] olsun
        b_sim = benzerler[j][1] olsun
        benzer_metinler = benzer_metinler + b_kelime + "(" + b_sim + ") " olsun
    }
    yazdır "'" + kelime + "' → " + benzer_metinler
}

// ─── Kaydet ───────────────────────────────────────────────────────────────────
gomme.kaydet("kelime_vektorleri.json") olsun
yazdır ""
yazdır "Kelime vektörleri kelime_vektorleri.json dosyasına kaydedildi."
