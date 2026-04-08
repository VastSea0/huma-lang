// ═══════════════════════════════════════════════════════════════════
// ag_istekleri.hb — Hüma HTTP İstek Kütüphanesi
// Sürüm: 1.1.0
// Yazar: Egehan KAHRAMAN
// GitHub: https://github.com/VastSea0/ag_istekleri
// ═══════════════════════════════════════════════════════════════════
//
// Rust Built-in Bağımlılıklar:
//   - dahili_istek(metot, url, veri, basliklar) → HTTP istek gönderir
// ═══════════════════════════════════════════════════════════════════

getir fonksiyon olsun url, basliklar alsın {
    dahili_istek("GET", url, boş, basliklar)'ı döndür
}

gönder fonksiyon olsun url, veri, basliklar alsın {
    dahili_istek("POST", url, veri, basliklar)'ı döndür
}

güncelle fonksiyon olsun url, veri, basliklar alsın {
    dahili_istek("PUT", url, veri, basliklar)'ı döndür
}

sil fonksiyon olsun url, basliklar alsın {
    dahili_istek("DELETE", url, boş, basliklar)'ı döndür
}

// Nesne tabanlı gelişmiş kullanım
Agİstekleri sınıf olsun {
    getir fonksiyon olsun url, basliklar alsın {
        dahili_istek("GET", url, boş, basliklar)'ı döndür
    }
    
    gönder fonksiyon olsun url, veri, basliklar alsın {
        dahili_istek("POST", url, veri, basliklar)'ı döndür
    }
    
    güncelle fonksiyon olsun url, veri, basliklar alsın {
        dahili_istek("PUT", url, veri, basliklar)'ı döndür
    }
    
    sil fonksiyon olsun url, basliklar alsın {
        dahili_istek("DELETE", url, boş, basliklar)'ı döndür
    }
}
