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

yükle "renkler.hb";

// NOT: dosya_var_mı() artık Rust built-in olarak sağlanır (v1.1.0)
// Eski lib versiyonu kaldırıldı.

güvenli_oku fonksiyon olsun yol alsın {
    icerik = dosya_oku(yol) olsun
    tipi(icerik) = "Boş" ise {
        hata_yaz("Dosya okunamadı: " + yol)
        ""'yi döndür
    }
    icerik'i döndür
}

satırlara_ayır fonksiyon olsun metin alsın {
    satırlar = [] olsun
    gecici = "" olsun
    i = 0 olsun
    boy = uzunluk(metin) olsun
    
    i < boy olduğu sürece {
        c = metin[i] olsun
        (c = "\n") ise {
            satırlar = listeye_ekle(satırlar, gecici) olsun
            gecici = "" olsun
        } yoksa {
            gecici = gecici + c olsun
        }
        i = i + 1 olsun
    }
    uzunluk(gecici) > 0 ise {
        satırlar = listeye_ekle(satırlar, gecici) olsun
    }
    satırlar'ı döndür
}
