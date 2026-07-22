// ═══════════════════════════════════════════════════════════════════
// dosya.hb — Hüma Dosya İşlemleri
// Sürüm: 1.1.0
// Yazar: Egehan KAHRAMAN
// ═══════════════════════════════════════════════════════════════════
//
// Rust Built-in Bağımlılıklar:
//   - dosya_oku(yol)         → dosya içeriğini okur
//   - dosya_yaz(yol, içerik) → dosyaya yazar
//   - dosya_var_mı(yol)      → 1 veya 0 döndürür
// ═══════════════════════════════════════════════════════════════════

"renkler.hb"'yi yükle

// NOT: dosya_var_mı() artık Rust built-in olarak sağlanır (v1.1.0)
// Eski lib versiyonu kaldırıldı.

güvenli_oku fonksiyon olsun yol alsın {
    icerik = yol'u dosya_oku olsun
    tipi(icerik) = "Boş" ise {
        "Dosya okunamadı: " + yol'u hata_yaz
        ""'yi döndür
    }
    icerik'i döndür
}

satırlara_ayır fonksiyon olsun metin alsın {
    satırlar = [] olsun
    gecici = "" olsun
    boy = metin'in uzunluğu olsun
    
    i = 0'dan boy'a kadar {
        c = metin[i] olsun
        (c = "\n") ise {
            satırlar'a gecici'yi ekle
            gecici = "" olsun
        } yoksa {
            (c != "\r") ise {
                gecici = gecici + c olsun
            }
        }
    }
    gecici'nin uzunluğu > 0 ise {
        satırlar'a gecici'yi ekle
    }
    satırlar'ı döndür
}
