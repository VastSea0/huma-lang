// ═══════════════════════════════════════════════════════════════════
// matematik.hb — Hüma Temel Matematik Kütüphanesi
// Sürüm: 2.0.0 (AI/NLP Genişletilmiş)
// Yazar: Egehan KAHRAMAN
// ═══════════════════════════════════════════════════════════════════
//
// Rust Built-in Bağımlılıklar:
//   - karekök(n), üs(a,b), ln(x), log2(x), log10(x)
//   - sin(x), cos(x), tan(x), exp(x)
//   - tavan(x), taban_sayı(x), mutlak_sayı(x), işaret(x)
//   - klamp(x,min,max), sonlu_mu(x)
// ═══════════════════════════════════════════════════════════════════

PI = 3.141592653589793 olsun
E  = 2.718281828459045 olsun
TAU = 6.283185307179586 olsun
SONSUZ = 1.0e308 olsun

// ─── Temel Fonksiyonlar ───────────────────────────────────────────
karesi fonksiyon olsun n alsın { n * n'i döndür }
küpü  fonksiyon olsun n alsın { n * n * n'i döndür }

// Güvenli mutlak değer (mevcut built-in "mutlak_sayı" ile çelişmez)
mutlak fonksiyon olsun n alsın {
    n < 0 ise { n * -1'i döndür }
    n'i döndür
}

kuvvet fonksiyon olsun a, b alsın {
    b = 0 ise { 1'i döndür }
    sonuc = 1 olsun
    i = 1'den b'ye kadar {
        sonuc = sonuc * a olsun
    }
    sonuc'u döndür
}

yuvarla fonksiyon olsun n alsın {
    n < 0 ise {
        poz = n * -1 olsun
        (yuvarla(poz) * -1)'i döndür
    }
    tam = n - (n % 1) olsun
    (n % 1) >= 0.5 ise { (tam + 1)'i döndür }
    tam'ı döndür
}

faktöriyel fonksiyon olsun n alsın {
    n <= 1 ise { 1'i döndür }
    n * faktöriyel(n - 1)'i döndür
}

// ─── Logaritma Yardımcıları ───────────────────────────────────────
// Herhangi bir tabanda logaritma: log_taban(x, taban)
log_taban fonksiyon olsun x, taban alsın {
    ln(x) / ln(taban)'ı döndür
}

// Güvenli log — negatif girişlerde hata vermez
güvenli_ln fonksiyon olsun x alsın {
    x <= 0 ise { -1000.0'ı döndür }
    ln(x)'i döndür
}

// ─── Trigonometri Yardımcıları ────────────────────────────────────
// Dereceden radyana çevrim
derece_radyan fonksiyon olsun derece alsın {
    derece * PI / 180.0'ı döndür
}

// Radyandan dereceye çevrim
radyan_derece fonksiyon olsun radyan alsın {
    radyan * 180.0 / PI'yi döndür
}

// ─── Aralık & İşaret ─────────────────────────────────────────────
// Değeri [min, maks] aralığında tut
aralık_sınırla fonksiyon olsun x, min_val, maks_val alsın {
    klamp(x, min_val, maks_val)'ı döndür
}

// ─── Sayı Teorisi ─────────────────────────────────────────────────
// En büyük ortak bölen (Öklid algoritması)
ebob fonksiyon olsun a, b alsın {
    a = mutlak(a) olsun
    b = mutlak(b) olsun
    b = 0 ise { a'yı döndür }
    ebob(b, a % b)'yi döndür
}

// En küçük ortak kat
ekok fonksiyon olsun a, b alsın {
    a = 0 ise { 0'ı döndür }
    b = 0 ise { 0'ı döndür }
    mutlak(a * b) / ebob(a, b)'yi döndür
}

// Asal mı?
asal_mı fonksiyon olsun n alsın {
    n < 2 ise { 0'ı döndür }
    n = 2 ise { 1'i döndür }
    n % 2 = 0 ise { 0'ı döndür }
    i = 3 olsun
    i * i <= n olduğu sürece {
        n % i = 0 ise { 0'ı döndür }
        i = i + 2 olsun
    }
    1'i döndür
}
