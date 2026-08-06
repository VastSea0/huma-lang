// ══════════════════════════════════════════════════════════════════════════════
// examples / tor-qurtu / birlestirici.hb
// Tekilleştirme, Q&A Soru-Cevap Çiftleri ve LLM Veri Seti Derleme Modülü
// ══════════════════════════════════════════════════════════════════════════════

"dosya.hb"'yi yükle
"nlp_temel"'i yükle
"tekillestirici.hb"'yi yükle

veri_setlerini_birlestir fonksiyon olsun alsın {
    "🔄 [TOR-QURTU BİRLEŞTİRİCİ v2] Ham veriler tekilleştiriliyor ve Q&A çiftleri türetiliyor..."'u yazdır
    "-------------------------------------------------------------------------------------"’i yazdır

    jsonl_hedef = "cikti/turkce_llm_veri_seti.jsonl"
    txt_hedef = "cikti/turkce_llm_corpus.txt"
    qa_hedef = "cikti/turkce_llm_qa_veri_seti.jsonl"

    dosya_yaz(jsonl_hedef, "")
    dosya_yaz(txt_hedef, "")
    dosya_yaz(qa_hedef, "")

    siteler_raw = dosya_oku("siteler.json")
    siteler = metinden_nesneye(siteler_raw)
    toplam_site = uzunluk(siteler)

    toplam_cumle = 0
    toplam_kelime = 0
    toplam_karakter = 0
    mukerrer_sayisi = 0
    qa_sayisi = 0

    gorulen_imzalar = []

    i = 0'dan (toplam_site - 1)'e kadar {
        site = siteler[i]
        site_id = site["id"]
        site_ad = site["ad"]
        ham_yol = "ham_veriler/" + site_id + ".jsonl"

        dosya_var_mı(ham_yol) == 1 ise {
            "  📂 Birleştiriliyor: " + ham_yol'u yazdır
            kayitlar = jsonl_oku(ham_yol)
            kayit_sayisi = uzunluk(kayitlar)

            j = 0'dan (kayit_sayisi - 1)'e kadar {
                kayit = kayitlar[j]
                metin = kayit["metin"]
                
                imza = metin_imzasi_olustur(metin)
                
                zaten_var_mi(gorulen_imzalar, imza) == 0 ise {
                    gorulen_imzalar = listeye_ekle(gorulen_imzalar, imza)
                    
                    // 1. Standart LLM JSONL Kaydı
                    final_kayit = {
                        "id": "tq_llm_" + toplam_cumle,
                        "kaynak": kayit["site_ad"],
                        "kategori": kayit["kategori"],
                        "metin": metin,
                        "kalite_skoru": 0.98
                    }
                    jsonl_yaz(jsonl_hedef, final_kayit)
                    dosya_satir_ekle(txt_hedef, metin)

                    // 2. Otomatik Soru-Cevap (Instruction Q&A) Çifti Türetme
                    uzunluk(metin) > 40 ise {
                        kelimeler = böl(metin, " ")
                        u_k = uzunluk(kelimeler)
                        u_k > 6 ise {
                            konu = site_ad
                            
                            qa_kayit = {
                                "id": "tq_qa_" + qa_sayisi,
                                "soru": konu + " hakkında bilgi verir misiniz?",
                                "cevap": metin
                            }
                            jsonl_yaz(qa_hedef, qa_kayit)
                            qa_sayisi = qa_sayisi + 1
                        }

                        toplam_kelime = toplam_kelime + u_k
                    }

                    toplam_karakter = toplam_karakter + uzunluk(metin)
                    toplam_cumle = toplam_cumle + 1
                } yoksa {
                    mukerrer_sayisi = mukerrer_sayisi + 1
                }
            }
        } yoksa {
            "  ⚠️ UYARI: " + ham_yol + " dosyası bulunamadı, atlanıyor."'ı yazdır
        }
    }

    ""'ı yazdır
    "🎉 [ÜRETİM TAMAMLANDI - GENİŞ TÜRKÇE LLM & Q&A VERİ SETİ]"'yi yazdır
    "   • Nitelikli Benzersiz Cümle : " + toplam_cumle'yi yazdır
    "   • Türetilen Soru-Cevap (Q&A): " + qa_sayisi'yi yazdır
    "   • Elenen Mükerrer Cümle    : " + mukerrer_sayisi'yi yazdır
    "   • Toplam Kelime Sayısı     : " + toplam_kelime'yi yazdır
    "   • Toplam Karakter Sayısı   : " + toplam_karakter'i yazdır
    "   • JSONL Veri Seti Dosyası  : " + jsonl_hedef'i yazdır
    "   • Q&A Soru-Cevap Dosyası   : " + qa_hedef'i yazdır
    "   • Düz Metin Corpus Dosyası : " + txt_hedef'i yazdır
    "-----------------------------------------------------------------------"’i yazdır

    toplam_cumle'yi döndür
}
