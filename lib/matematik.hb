// ═══════════════════════════════════════════════════════════════════
// matematik.hb — Hüma Temel Matematik Kütüphanesi
// Sürüm: 1.1.0
// Yazar: Egehan KAHRAMAN
// ═══════════════════════════════════════════════════════════════════
//
// Rust Built-in Bağımlılıklar:
//   - karekök(n) → kare kök hesaplama (bu dosyada tanımlı değildir,
//                  interpreter tarafından sağlanır)
// ═══════════════════════════════════════════════════════════════════

PI = 3.141592653589793 olsun
E = 2.718281828459045 olsun

karesi fonksiyon olsun n alsın { n * n'i döndür }
küpü fonksiyon olsun n alsın { n * n * n'i döndür }
mutlak fonksiyon olsun n alsın { n < 0 ise { n * -1'i döndür } n'i döndür }

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
