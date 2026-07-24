// ══════════════════════════════════════════════════════════════════════════════
// yapay_zeka/yapay_zeka.hb — Hüma Yapay Zeka Paketi Ana Giriş Dosyası
// ══════════════════════════════════════════════════════════════════════════════

yükle "katman.hb"
yükle "optimizor.hb"

sınıf YapayZekaMotoru {
    ilklendir fonksiyon olsun {
        kendisi.ad = "Hüma Yapay Zeka v0.1" olsun
    }

    bpe_egit_ve_kodla fonksiyon olsun metin, vocab_boyutu alsın {
        bpe_eğit(metin, vocab_boyutu)
        tokenlar = bpe_kodla(metin) olsun
        tokenlar'ı döndür
    }

    bpe_metne_cevir fonksiyon olsun token_ids alsın {
        metin = bpe_çöz(token_ids) olsun
        metin'i döndür
    }
}
