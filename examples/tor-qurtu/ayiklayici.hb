// ══════════════════════════════════════════════════════════════════════════════
// examples / tor-qurtu / ayiklayici.hb
// Trafilatura Standartlarında Web Ana Gövde Metin Ayıklayıcı
// ══════════════════════════════════════════════════════════════════════════════

"dizgi.hb"'yi yükle

// 1. Wikipedia Köşeli Parantez ve Dipnot Temizliği [ 1 ], [ değiştir | kaynağı değiştir ]
wikipedia_dipnotlarini_temizle fonksiyon olsun metin alsın {
    sonuc = metin
    
    // Yaygın Wikipedia Şablon Artıklarını Değiştir
    sonuc = değiştir(sonuc, "[ değiştir | kaynağı değiştir ]", "")
    sonuc = değiştir(sonuc, "[ değiştir ]", "")
    sonuc = değiştir(sonuc, "[ kaynak belirtilmeli ]", "")
    sonuc = değiştir(sonuc, "kenar çubuğuna taşı", "")
    sonuc = değiştir(sonuc, "Ana menü", "")
    sonuc = değiştir(sonuc, "İçeriğe atla", "")
    
    // Parantez içi kaynak gösterim kalıntılarını temizleme
    uzun = uzunluk(sonuc)
    t_sonuc = ""
    kose_ici = 0
    
    i = 0'dan (uzun - 1)'e kadar {
        kr = sonuc[i]
        kr = "[" ise {
            kose_ici = 1
        } yoksa {
            kr = "]" ise {
                kose_ici = 0
            } yoksa {
                kose_ici = 0 ise {
                    t_sonuc = t_sonuc + kr
                }
            }
        }
    }
    
    t_sonuc'u döndür
}

// 2. Trafilatura Stil HTML Ana Gövde Metni Ayıklayıcı
trafilatura_ayikla fonksiyon olsun html_metin alsın {
    metin = html_metin
    
    // HTML etiketlerinden hızlı ayıklama (O(N) split)
    parcalar = böl(metin, "<")
    n = uzunluk(parcalar)
    
    temiz_parcalar = []
    temiz_parcalar = listeye_ekle(temiz_parcalar, parcalar[0])
    
    i = 1'dan (n - 1)'e kadar {
        p = parcalar[i]
        içeriyor(p, ">") = 1 ise {
            alt_parcalar = böl(p, ">")
            uzunluk(alt_parcalar) > 1 ise {
                gövde_parça = alt_parcalar[1]
                temiz_parcalar = listeye_ekle(temiz_parcalar, gövde_parça)
            }
        }
    }
    
    ham_birlesik = birleştir(temiz_parcalar, " ")
    
    // HTML Entity Temizlikleri
    t_metin = ham_birlesik
    t_metin = değiştir(t_metin, "&nbsp;", " ")
    t_metin = değiştir(t_metin, "&amp;", "&")
    t_metin = değiştir(t_metin, "&lt;", "<")
    t_metin = değiştir(t_metin, "&gt;", ">")
    t_metin = değiştir(t_metin, "&quot;", "\"")
    t_metin = değiştir(t_metin, "&#39;", "'")
    t_metin = değiştir(t_metin, "\t", " ")
    t_metin = değiştir(t_metin, "\r", " ")
    
    // Wikipedia Dipnot ve Düzenleme Kalıntılarını Temizle
    islenmis = wikipedia_dipnotlarini_temizle(t_metin)
    
    kırp(islenmis)'i döndür
}
