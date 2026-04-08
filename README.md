# Hüma Programlama Dili 

Hüma, modern yazılım geliştirme prensiplerini Türkçe doğal dil yapısıyla birleştiren, yüksek performanslı ve güvenli bir programlama dilidir. Rust diliyle geliştirilmiş olup, hem yorumlanabilir (tree-walking) hem de bytecode üzerinden derlenebilir (VM) bir mimariye sahiptir.

[**Click here for the English version of README.**](README.en.md)

## 🚀 Öne Çıkan Özellikler

- **Tamamen Doğal Türkçe Sözdizimi:** Kod yazarken Türkçe konuşma dilinin akışını (olsun, ise, yoksa, olduğu sürece vb.) kullanabilirsiniz.
- **Esnek Ek Yönetimi (Suffix):** `liste'yi yazdır`, `x'ten çıkar` gibi ifadelerdeki ekler otomatik olarak temizlenir, en doğal yazıma imkan tanır.
- **Hibrit Çalışma Modu:** İsterseniz doğrudan yorumlayın, isterseniz bytecode'a derleyip sanal makinede (VM) koşturun.
- **Bağımsız İkili Dosyalar (Standalone):** Kodlarınızı derleyip herhangi bir bağımlılık olmadan çalışabilen native binary'lere dönüştürebilirsiniz.
- **Zengin Sistem Kütüphaneleri:** Matematik, NLP, terminal renklendirme, zaman yönetimi, liste araçları ve birim test çerçevesi hazırdır.
- **Güvenli Paket Yönetimi:** SHA-256 bütünlük doğrulaması, path traversal koruması ve mono-repo (alt dizin) desteği ile güvenli modül yönetimi (v0.5.2).
- **İki Dilli CLI (Bilingual CLI):** Tüm komutları hem Türkçe hem İngilizce olarak kullanabilirsiniz (Örn: `run` veya `çalıştır`, `build` veya `derle`).
- **Modern IDE Desteği:** Kod yazmanız için hem Native (GTK) hem de Web tabanlı modern editörler sunulur.

---

## 🛠️ Kurulum ve Derleme

Hüma'yı derlemek için sisteminizde [Rust](https://www.rust-lang.org/) kurulu olmalıdır.

```bash
git clone https://github.com/VastSea0/huma-lang.git
cd huma-lang
cargo build --release
```

Derleme sonrası çalıştırılabilir dosya `target/release/huma` konumunda olacaktır.

---

## 💻 Kullanım Rehberi

### 1. Etkileşimli Mod (REPL)

Doğrudan komut satırından kod denemek için:

```bash
huma repl
```

### 2. Dosya Çalıştırma (Yorumlayıcı)

Bir `.hb` dosyasını çalıştırmak için (`run` veya `çalıştır`):

```bash
huma çalıştır program.hb
# veya
huma run program.hb
```

### 3. Bytecode Modu (Performans)

Kodunuzu önce bytecode'a derleyip (`build`/`derle`) sonra Sanal Makine üzerinde yürütebilirsiniz (`exec`/`yürüt`):

```bash
# Derle
huma derle program.hb -o cikti.hbc
# Yürüt
huma yürüt cikti.hbc
```

### 4. Paket Yöneticisi ve Betikler

Hüma, modern bir paket yöneticisine sahiptir (`paket` veya `package`):

```bash
# Projeyi ilklendir
huma paket ilkle
# Bağımlılıkları kur
huma paket kur
# Yeni bağımlılık ekle
huma paket kur ag_istekleri
# Projedeki bir betiği çalıştır (npm run gibi)
huma çalıştır baslat
```

---

## 📖 Dil Referansı

### Temel Sözdizimi

```huma
// Değişken Tanımlama ve Atama
x = 10 olsun
isim = "Hüma" olsun

// Matematiksel İşlemler
toplam = x + 5 * 2 olsun

// Koşullu İfadeler (ise / yoksa)
x > 5 ise {
    "Sayı büyüktür."'ü yazdır;
} yoksa {
    "Sayı küçüktür."'ü yazdır;
}

// Döngüler (olduğu sürece)
i = 0 olsun
i < 5 olduğu sürece {
    "Sıra: " + i'yi yazdır;
    i = i + 1 olsun
}
```

### Fonksiyonlar ve Sınıflar

```huma
yükle "matematik.hb";

merhaba_de fonksiyon olsun ad alsın {
    "Merhaba " + ad + "!"'i döndür
}

sonuç = merhaba_de("Dünya") olsun
sonuç'u yazdır;

islem sınıf olsun {
    toplama fonksiyon olsun a, b alsın {
        a + b'yi döndür
    }
}
hesap = islem() olsun
hesap.toplama(10, 20)'yi yazdır;
```

### Listeler

```huma
meyveler = ["Elma", "Armut"] olsun
meyveler'e ["Muz"]'u ekle;
meyveler'den [0]'ı çıkar; // İndeks bazlı silme

i = 0 olsun
i < meyveler'in uzunluğu olduğu sürece {
    meyveler[i]'yi yazdır;
    i = i + 1 olsun
}
```

---

## 📚 Sistem Kütüphaneleri (`lib/`)

- **`matematik.hb`**: `karesi(n)`, `küpü(n)`, `kuvvet(a, b)`, `faktöriyel(n)`
- **`renkler.hb`**: `renkli_yaz(m, r)`, `başarı_yaz(m)`, `hata_yaz(m)`
- **`zaman.hb`**: `beklet(s)`, `kronometre_başlat()`, `kronometre_bitir(b)`
- **`liste.hb`**: `eşle(d, f)`, `filtrele(d, f)`, `indirge(d, f, b)`
- **`birim_test.hb`**: `test_et(ad, f)`, `iddia_et(a, b, m)`

---

## 📜 Lisans

Bu proje MIT Lisansı ile lisanslanmıştır.
