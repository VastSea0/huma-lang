// huma_sunucu.hb - Hüma Sunucu Kütüphanesi

Yanıt sınıf olsun {
    istek_id = 0 olsun
    
    metin fonksiyon olsun içerik alsın {
        dahili_sunucu_yanitla(kendisi'nin istek_id'i, içerik, 200, "text/plain")
    }

    html fonksiyon olsun içerik alsın {
        dahili_sunucu_yanitla(kendisi'nin istek_id'i, içerik, 200, "text/html; charset=utf-8")
    }
    
    json fonksiyon olsun nesne alsın {
        dahili_sunucu_yanitla(kendisi'nin istek_id'i, nesneden_metine(nesne), 200, "application/json")
    }

    durum fonksiyon olsun kod alsın {
        dahili_sunucu_yanitla(kendisi'nin istek_id'i, "", kod, "text/plain")
    }
}

Sunucu sınıf olsun {
    port = 8080 olsun
    _get_rotaları = metinden_nesneye("{}") olsun
    _post_rotaları = metinden_nesneye("{}") olsun

    kur fonksiyon olsun p alsın {
        kendisi'nin port'u = p olsun
        kendisi'nin _get_rotaları'sı = metinden_nesneye("{}") olsun
        kendisi'nin _post_rotaları'sı = metinden_nesneye("{}") olsun
    }

    getir fonksiyon olsun yol, islem alsın {
        değer_ata(kendisi'nin _get_rotaları'sı, yol, islem)
    }

    gönder fonksiyon olsun yol, islem alsın {
        değer_ata(kendisi'nin _post_rotaları'sı, yol, islem)
    }

    baslat fonksiyon olsun {
        sid = dahili_sunucu_baslat(kendisi'nin port'u)
        sid == boş ise {
            "Hata: Sunucu başlatılamadı!"'ı yazdır
            0'ı döndür
        }
        
        "Hüma Backend Sunucusu " + (kendisi'nin port'u) + " portunda aktif!"'ı yazdır
        
        1 olduğu sürece {
            istek = bekle dahili_sunucu_bekle(sid)
            
            istek != boş ise {
                url = (istek'in url'u)
                metot = (istek'in metot'u)
                
                yanıt = Yanıt()
                yanıt'ın istek_id'i = (istek'in id'i) olsun
                
                metot == "GET" ise {
                    içeriyor(kendisi'nin _get_rotaları'sı, url) ise {
                        islem = değer_al(kendisi'nin _get_rotaları'sı, url)
                        islem(istek, yanıt)
                    } yoksa {
                        dahili_sunucu_yanitla((istek'in id'i), "404 Sayfa Bulunamadı", 404, "text/plain")
                    }
                }
                
                metot == "POST" ise {
                    içeriyor(kendisi'nin _post_rotaları'sı, url) ise {
                        islem = değer_al(kendisi'nin _post_rotaları'sı, url)
                        islem(istek, yanıt)
                    } yoksa {
                        dahili_sunucu_yanitla((istek'in id'i), "404 Sayfa Bulunamadı", 404, "text/plain")
                    }
                }
            }
        }
    }
}
