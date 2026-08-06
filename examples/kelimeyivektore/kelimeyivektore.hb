"yapay_zeka_temel.hb"'yi yükle
"nlp_temel/nlp_temel.hb"'yi yükle
"nlp_ileri/gomme.hb"'yi yükle

yazdır "══════════════════════════════════════════════════"
yazdır "  Hüma — 2.500 Satırlık Gerçek Türkçe Veri Seti ile 2D NLP"
yazdır "══════════════════════════════════════════════════"
yazdır ""

// ─── 1. Gerçek Türkçe Veri Setini Dosyadan Oku ──────────────────────────────
metin = dosya_oku("turkce_veriseti.txt") olsun
yazdır "Gerçek Türkçe Veri Seti (2.491 satır / 369 KB) Yüklendi!"

// ─── 2. Tokenizasyon ve Frekans Sayımı ─────────────────────────────────────
proc = metin_islemci() olsun
tokens_temiz = proc.durak_filtrele(proc.tokenize(metin)) olsun

ham_frekanslar = {} olsun
n_raw = uzunluk(tokens_temiz) olsun

i = 0'dan (n_raw - 1)'e kadar {
    k = tokens_temiz[i] olsun
    ham_frekanslar[k] == Boş ise {
        ham_frekanslar[k] = 1
    } yoksa {
        ham_frekanslar[k] = ham_frekanslar[k] + 1
    }
}

// ─── 3. Anlamlı Kelimeler Sözlüğü (Frekans >= 5 olanlar) ────────────────────
MIN_FREKANS = 5 olsun
sozluk = {} olsun
ters_sozluk = {} olsun
frekanslar = {} olsun
sozluk_boyutu = 0 olsun

i = 0'dan (n_raw - 1)'e kadar {
    k = tokens_temiz[i] olsun
    (ham_frekanslar[k] >= MIN_FREKANS) ise {
        sozluk[k] == Boş ise {
            sozluk[k] = sozluk_boyutu
            ters_sozluk[metne_çevir(sozluk_boyutu)] = k
            frekanslar[k] = ham_frekanslar[k]
            sozluk_boyutu = sozluk_boyutu + 1 olsun
        }
    }
}

yazdır "Süzülen Anlamlı Kelime Sayısı (Frekans >= 5): " + sozluk_boyutu
yazdır ""

// ─── 4. 2D Kelime Gömme Katmanı (X, Y Uzayı) ──────────────────────────────────
GOMME_BOYUTU = 2 olsun
rastgele_tohum_ata(42) olsun
gomme = gomme_tabakasi() olsun
gomme.ilklendir(sozluk_boyutu, GOMME_BOYUTU) olsun

// ─── 5. Skip-gram Çiftleri Oluşturma ──────────────────────────────────────────
PENCERE = 2 olsun
egitim_ciftleri = [] olsun

j = 0'dan (n_raw - 1)'e kadar {
    merkez = tokens_temiz[j] olsun
    merkez_id = sozluk[merkez] olsun
    merkez_id == Boş ise { devam }
    bas = j - PENCERE olsun
    bas < 0 ise { bas = 0 olsun }
    son = j + PENCERE olsun
    son >= n_raw ise { son = n_raw - 1 olsun }
    k = bas'tan son'a kadar {
        k == j ise { devam }
        komsu = tokens_temiz[k] olsun
        komsu_id = sozluk[komsu] olsun
        komsu_id == Boş ise { devam }
        cift = [merkez_id, komsu_id] olsun
        egitim_ciftleri = listeye_ekle(egitim_ciftleri, cift) olsun
    }
}

yazdır "Eğitilecek Çift Sayısı: " + uzunluk(egitim_ciftleri)
yazdır ""

// ─── 6. SGD Eğitimi ─────────────────────────────────────────────────────────
EPOCH = 5 olsun
LR = 0.08 olsun
n_cift = uzunluk(egitim_ciftleri) olsun

yazdır "2.500 satırlık veri seti üzerinde 2D Skip-gram eğitimi başladı..."

e = 1'den EPOCH'a kadar {
    toplam_kayip = 0.0 olsun
    i = 0'dan (n_cift - 1)'e kadar {
        merkez_id = egitim_ciftleri[i][0] olsun
        hedef_id = egitim_ciftleri[i][1] olsun
        
        merkez_v = gomme.token_al(merkez_id) olsun
        hedef_v = gomme.token_al(hedef_id) olsun
        
        skor = sigmoid(ic_carpim(merkez_v, hedef_v)) olsun
        kayip = ikili_capraz_entropi(skor, 1.0) olsun
        toplam_kayip = toplam_kayip + kayip olsun
        
        grad_skalar = skor - 1.0 olsun
        grad_merkez = vektor_skalar_carp(hedef_v, grad_skalar) olsun
        grad_hedef = vektor_skalar_carp(merkez_v, grad_skalar) olsun
        
        gomme.guncelle(merkez_id, grad_merkez, LR) olsun
        gomme.guncelle(hedef_id, grad_hedef, LR) olsun
    }
    
    ort_kayip = toplam_kayip / n_cift olsun
    yazdır "Epoch " + e + "/" + EPOCH + " - Ortalama Kayıp: " + ort_kayip
}

yazdır ""
yazdır "Eğitim Başarıyla Tamamlandı!"
yazdır ""

// ─── 7. En Sık Geçen Kelimelerin 2D Uzaydaki (X, Y) Konumları ───────────────
yazdır "══════════════════════════════════════════════════"
yazdır "  GERÇEK KELİMELERİN 2D UZAYDAKİ KOORDİNATLARI (X, Y)"
yazdır "══════════════════════════════════════════════════"

gosterilen = 0 olsun
i = 0'dan (sozluk_boyutu - 1)'e kadar {
    k_adi = ters_sozluk[metne_çevir(i)] olsun
    frek = frekanslar[k_adi] olsun
    
    frek >= 10 ise {
        pos = gomme.token_al(i) olsun
        pos_l = vektore_liste(pos) olsun
        X = pos_l[0] olsun
        Y = pos_l[1] olsun
        yazdır k_adi + " (frekans: " + frek + ") \t→ X: " + X + " | Y: " + Y
        gosterilen = gosterilen + 1 olsun
        gosterilen >= 30 ise { kır }
    }
}

yazdır ""
gomme.kaydet("kelime_vektorleri.json") olsun