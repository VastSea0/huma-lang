// ══════════════════════════════════════════════════════════════════════════════
// yapay_zeka/katman.hb — Tam Bağlantılı (Dense / Linear) Nöral Katman
// ══════════════════════════════════════════════════════════════════════════════

sınıf TamBaglantiliKatman {
    in_dim = 0
    out_dim = 0
    w = 0
    b = 0

    katman_olustur fonksiyon olsun in_dim, out_dim, ilk_w_liste, ilk_b_liste alsın {
        kendisi.in_dim = in_dim olsun
        kendisi.out_dim = out_dim olsun
        kendisi.w = tensor_olustur(in_dim, out_dim, ilk_w_liste, 1) olsun
        kendisi.b = tensor_olustur(1, out_dim, ilk_b_liste, 1) olsun
        kendisi'yi döndür
    }

    ileri fonksiyon olsun x alsın {
        w = kendisi.w olsun
        b = kendisi.b olsun
        out = tensor_matris_carp(x, w) olsun
        y = tensor_relu(out) olsun
        y'yi döndür
    }

    w_al fonksiyon olsun { kendisi.w'yi döndür }
    b_al fonksiyon olsun { kendisi.b'yi döndür }
}
