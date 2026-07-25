// ═══════════════════════════════════════════════════════════════════
// ag_istekleri.hb — Hüma HTTP İstek Kütüphanesi
// Sürüm: 2.0.0
// ═══════════════════════════════════════════════════════════════════

getir fonksiyon olsun url, basliklar alsın {
    "GET" ile url ve boş ve basliklar'ı dahili_istek'i döndür
}

gönder fonksiyon olsun url, veri, basliklar alsın {
    "POST" ile url ve veri ve basliklar'ı dahili_istek'i döndür
}

güncelle fonksiyon olsun url, veri, basliklar alsın {
    "PUT" ile url ve veri ve basliklar'ı dahili_istek'i döndür
}

sil fonksiyon olsun url, basliklar alsın {
    "DELETE" ile url ve boş ve basliklar'ı dahili_istek'i döndür
}

// Nesne tabanlı gelişmiş kullanım
Agİstekleri sınıf olsun {
    getir fonksiyon olsun url, basliklar alsın {
         "GET" ile url ve boş ve basliklar'ı dahili_istek'i döndür
    }
    
    gönder fonksiyon olsun url, veri, basliklar alsın {
        "POST" ile url ve veri ve basliklar'ı dahili_istek'i döndür
    }
    
    güncelle fonksiyon olsun url, veri, basliklar alsın {
        "PUT" ile url ve veri ve basliklar'ı dahili_istek'i döndür
    }
    
    sil fonksiyon olsun url, basliklar alsın {
        "DELETE" ile url ve boş ve basliklar'ı dahili_istek'i döndür
    }
}
