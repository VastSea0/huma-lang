// ══════════════════════════════════════════════════════════════════════════════
// nlp_ileri/gomme.hb — Kelime Gömme (Word Embedding) Katmanı
// Sürüm: 1.0.0
// ══════════════════════════════════════════════════════════════════════════════
//
// Öğrenilebilir kelime vektörleri: her token, yoğun bir vektöre karşılık gelir.
// Word2Vec (Skip-gram), basit language model veya downstream görevler için kullanılır.
// ══════════════════════════════════════════════════════════════════════════════

"yapay_zeka_temel.hb"'yi yükle

gomme_tabakasi sınıf olsun {

    // sozluk_boyutu: kaç farklı token var
    // gomme_boyutu: her token için kaç boyutlu vektör
    ilklendir fonksiyon olsun sozluk_boyutu, gomme_boyutu alsın {
        kendisi.sozluk_boyutu = sozluk_boyutu olsun
        kendisi.gomme_boyutu = gomme_boyutu olsun
        // Embedding matrisi: [sozluk_boyutu × gomme_boyutu]
        // Her satır bir token'ın vektörü
        kendisi.E = matris_he_ilklendir(sozluk_boyutu, gomme_boyutu) olsun
        kendisi.adam_durum = adam_durum_olustur(sozluk_boyutu, gomme_boyutu) olsun
    }

    // token_id için gömme vektörünü getir
    token_al fonksiyon olsun token_id alsın {
        matris_satir_al(kendisi.E, token_id)'yi döndür
    }

    // token_id listesi için gömme matrisini getir
    // Döndürür: [n_token × gomme_boyutu] matris
    ileri fonksiyon olsun token_idler alsın {
        n = uzunluk(token_idler) olsun
        sonuc = matris_olustur(n, kendisi.gomme_boyutu, 0.0) olsun
        i = 0'dan (n - 1)'e kadar {
            tid = token_idler[i] olsun
            satir = kendisi.token_al(tid) olsun
            matris_satir_ata(sonuc, i, satir) olsun
        }
        sonuc'u döndür
    }

    // Ortalama havuzlama — token listesinin ortalamasını al (cümle vektörü)
    ortalama_havuzla fonksiyon olsun token_idler alsın {
        n = uzunluk(token_idler) olsun
        n = 0 ise { vektor_olustur(kendisi.gomme_boyutu, 0.0)'ı döndür }
        toplam = vektor_olustur(kendisi.gomme_boyutu, 0.0) olsun
        i = 0'dan (n - 1)'e kadar {
            v = kendisi.token_al(token_idler[i]) olsun
            toplam = vektor_topla(toplam, v) olsun
        }
        vektor_skalar_carp(toplam, 1.0 / n)'yi döndür
    }

    // Pozisyonel kodlama ekle (sinüs-kosinüs, transformer tarzı)
    pozisyonel_kodla fonksiyon olsun gomme_matrisi, maks_uzunluk alsın {
        M = kendisi.gomme_boyutu olsun
        n = maks_uzunluk olsun
        i = 0'dan (n - 1)'e kadar {
            j = 0'dan (M - 1)'e kadar {
                bolme = üs(10000.0, (2 * taban_sayı(j / 2)) / M) olsun
                j % 2 = 0 ise {
                    pe = sin(i / bolme) olsun
                } yoksa {
                    pe = cos(i / bolme) olsun
                }
                mevcut = matris_al(gomme_matrisi, i, j) olsun
                matris_ata(gomme_matrisi, i, j, mevcut + pe) olsun
            }
        }
        gomme_matrisi'ni döndür
    }

    // Gömme ağırlığını güncelle — sadece kullanılan token'lar için
    guncelle fonksiyon olsun token_id, gradyan, ogrenme_hizi alsın {
        // Satır gradyanını matris gradyanına dönüştür (sadece ilgili satır)
        grad_M = matris_olustur(kendisi.sozluk_boyutu, kendisi.gomme_boyutu, 0.0) olsun
        matris_satir_ata(grad_M, token_id, gradyan) olsun
        adam_matris_guncelle(kendisi.E, grad_M, kendisi.adam_durum, ogrenme_hizi) olsun
    }

    // Benzer token'ları bul — kosinüs benzerliği ile top-k
    en_benzer_tokenler fonksiyon olsun token_id, top_k alsın {
        hedef_v = kendisi.token_al(token_id) olsun
        benzerlikler = [] olsun
        i = 0'dan (kendisi.sozluk_boyutu - 1)'e kadar {
            i = token_id ise { devam }
            v = kendisi.token_al(i) olsun
            sim = kosinus_benzerligi(hedef_v, v) olsun
            benzerlikler = listeye_ekle(benzerlikler, [i, sim]) olsun
        }
        // Basit sıralama — top-k için
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
        sonuc = [] olsun
        son = top_k olsun
        son > m ise { son = m olsun }
        i = 0'dan (son - 1)'e kadar {
            sonuc = listeye_ekle(sonuc, benzerlikler[i]) olsun
        }
        sonuc'u döndür
    }

    // Gömme matrisini JSON'a kaydet
    kaydet fonksiyon olsun yol alsın {
        veri = {} olsun
        veri["sozluk_boyutu"] = kendisi.sozluk_boyutu
        veri["gomme_boyutu"] = kendisi.gomme_boyutu
        satırlar = [] olsun
        i = 0'dan (kendisi.sozluk_boyutu - 1)'e kadar {
            satir_v = matris_satir_al(kendisi.E, i) olsun
            satır_l = vektore_liste(satir_v) olsun
            satırlar = listeye_ekle(satırlar, satır_l) olsun
        }
        veri["E"] = satırlar
        dosya_yaz(yol, nesneden_metine(veri)) olsun
        yazdır "Gömme kaydedildi: " + yol
    }
}
