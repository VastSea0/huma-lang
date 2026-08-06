// ══════════════════════════════════════════════════════════════════════════════
// examples / tor-qurtu / kazici.hb
// Kalite Filtreli Akıllı Web Kazıyıcı (Crawler)
// ══════════════════════════════════════════════════════════════════════════════

"ag_istekleri"'ni yükle
"nlp_temel"'i yükle
"dosya.hb"'yi yükle
"ayiklayici.hb"'yi yükle
"kalite_filtresi.hb"'yi yükle

web_siteleri_kazi fonksiyon olsun alsın {
    "🌐 [TOR-QURTU KAZICI v2] Web siteleri taranıyor ve kalite filtresinden geçiriliyor..."'u yazdır
    "----------------------------------------------------------------------------------"’i yazdır

    siteler_raw = dosya_oku("siteler.json")
    siteler = metinden_nesneye(siteler_raw)
    
    toplam_site = uzunluk(siteler)
    basarili_site = 0
    toplam_nitelikli_cumle = 0

    i = 0'dan (toplam_site - 1)'e kadar {
        site = siteler[i]
        site_id = site["id"]
        site_ad = site["ad"]
        url = site["url"]
        kategori = site["kategori"]

        "  📡 [" + (i + 1) + "/" + toplam_site + "] Taranıyor: " + site_ad + " (" + url + ")"'ı yazdır

        basliklar = {
            "User-Agent": "TorQurtuLLMBot/2.0 (TurkceLLMDatasetPipeline)"
        }
        yanit = bekle getir(url, basliklar)

        durum = yanit["durum"]
        durum = 200 ise {
            ham_html = yanit["içerik"]
            
            temiz_metin = trafilatura_ayikla(ham_html)
            
            ham_dosya_yolu = "ham_veriler/" + site_id + ".jsonl"
            dosya_yaz(ham_dosya_yolu, "")
            
            site_kayit_sayisi = 0
            islemci = metin_islemci()

            // Büyük sayfaları önce paragraf parçalarına böl
            paragraf_parcalari = böl(temiz_metin, "  ")
            p_n = uzunluk(paragraf_parcalari)

            j_p = 0'dan (p_n - 1)'e kadar {
                p_metin = kırp(paragraf_parcalari[j_p])
                uzunluk(p_metin) > 30 ise {
                    cumleler = islemci.cümle_böl(p_metin)
                    cumle_sayisi = uzunluk(cumleler)
                    
                    j = 0'dan (cumle_sayisi - 1)'e kadar {
                        cumle = kırp(cumleler[j])
                        cumle = değiştir(cumle, "\n", " ")
                        cumle = değiştir(cumle, "\r", " ")
                        cumle = kırp(cumle)
                        
                        fineweb_kalite_denetimi(cumle) = 1 ise {
                            kayit = {
                                "site_id": site_id,
                                "site_ad": site_ad,
                                "kategori": kategori,
                                "url": url,
                                "metin": cumle,
                                "karakter_sayisi": uzunluk(cumle)
                            }
                            
                            jsonl_yaz(ham_dosya_yolu, kayit)
                            site_kayit_sayisi = site_kayit_sayisi + 1
                        }
                    }
                }
            }
            
            "     ✅ Tamamlandı: " + site_kayit_sayisi + " yüksek nitelikli cümle saklandı -> " + ham_dosya_yolu'nu yazdır
            basarili_site = basarili_site + 1
            toplam_nitelikli_cumle = toplam_nitelikli_cumle + site_kayit_sayisi
        } yoksa {
            "     ⚠️ Hata: HTTP " + durum + " yanıtı alındı (" + url + ")"'ı yazdır
        }
    }

    ""'ı yazdır
    "📊 [KAZIMA RAPORU] Başarılı Site: " + basarili_site + "/" + toplam_site + " | Toplam Süzülmüş Nitelikli Cümle: " + toplam_nitelikli_cumle'yi yazdır
    toplam_nitelikli_cumle'yi döndür
}
