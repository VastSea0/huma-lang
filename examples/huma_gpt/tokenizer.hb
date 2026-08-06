// ══════════════════════════════════════════════════════════════════════════════
// examples / huma_gpt / tokenizer.hb
// Hüma Yerleşik BPE (Byte-Pair Encoding) Türkçe Alt-Kelime Tokenizer
// ══════════════════════════════════════════════════════════════════════════════

"dosya.hb"'yi yükle

bpe_tokenizer_egit fonksiyon olsun corpus_yolu, sozluk_boyutu alsın {
    "🔤 [BPE TOKENIZER] Türkçe sözlük eğitiliyor..."'u yazdır
    
    corpus_metni = dosya_oku(corpus_yolu)
    bpe_eğit(corpus_metni, sozluk_boyutu)
    
    "   ✅ BPE Tokenizer Eğitildi! Sözlük Boyutu: " + sozluk_boyutu'nu yazdır
    sozluk_boyutu'nu döndür
}

metni_tokenlestir fonksiyon olsun metin alsın {
    tokenler = bpe_kodla(metin)
    tokenler'i döndür
}

tokenleri_metne_donustur fonksiyon olsun token_ids alsın {
    metin_parcalari = ""
    n = uzunluk(token_ids)
    
    i = 0'dan (n - 1)'e kadar {
        tid = token_ids[i]
        dene {
            p = bpe_çöz([tid])
            metin_parcalari = metin_parcalari + p
        } yakala h {
            // Geçersiz tekli bayt parçalarını atla
        }
    }
    
    metin_parcalari'ni döndür
}
