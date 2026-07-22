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
- **Modern CLI + LSP Desteği:** CLI komut seti ve LSP entegrasyonu sayesinde editörde kod tamamlama, hover bilgi kartları ve statik analiz tanılamaları sunulur.

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

## 💻 CLI Komut Referansı
  
Hüma, **Çift Dilli (Bilingual) CLI** mimarisine sahiptir. Tüm komutları hem Türkçe (örn: `çalıştır`, `derle`, `güncelle`) hem de İngilizce (örn: `run`, `build`, `update`) olarak kullanabilirsiniz.

### 1. Çalıştırma ve Etkileşimli Modlar

- `huma çalıştır <hedef>` (Alias: `run`)
    - **Açıklama:** Bir `.hb` kaynak dosyasını veya `huma.json` içinde tanımlı bir betiği (script) çalıştırır.
    - **Varsayılan Dosya Tespiti:** Hedef belirtilmezse dizinde `baslat` veya `start` betiğini çalıştırır, aksi halde `huma.json` dosyasındaki ana giriş noktasını kullanır.
- `huma kabuk` (Alias: `repl`)
    - **Açıklama:** Kodları satır satır denemek ve hızlı prototipleme yapmak için etkileşimli REPL modunu başlatır.
- `huma sına [hedef]` (Alias: `test`)
    - **Açıklama:** Projedeki test dosyalarını çalıştırır. `hedef` verilirse tek dosya veya klasör bazlı çalıştırma yapılır.

### 2. Derleme ve Bytecode İşlemleri

- `huma derle <dosya>` (Alias: `build`)
    - **Açıklama:** Kaynak kodu `.hbc` uzantılı bytecode dosyasına derler. Çıktı adını `-o` parametresi ile belirleyebilirsiniz.
- `huma yürüt <dosya>` (Alias: `exec`)
    - **Açıklama:** Önceden derlenmiş bir `.hbc` bytecode dosyasını Hüma VM üzerinde yüksek performansla çalıştırır.
- `huma üret <dosya>` (Alias: `gen`)
    - **Açıklama:** Hüma kodundan bağımsız çalışabilen, yüksek performanslı Rust kaynak kodu üretir.

### 3. Paket Yönetimi (`paket` veya `package`)

- `huma ilkle` (Alias: `paket ilkle`, `init`)
    - **Açıklama:** Mevcut dizini bir Hüma projesi olarak ilklendirir, `huma.json` dosyasını oluşturur.
- `huma yeni <isim>` (Alias: `paket yeni`, `new`)
    - **Açıklama:** Belirtilen `<isim>` ile yeni bir klasör oluşturur ve içine bir Hüma proje şablonu yerleştirir.
- `huma kur [isim]` (Alias: `paket kur`, `install`, `add`)
    - **Açıklama:** `huma.json` içindeki bağımlılıkları yükler. Bir `[isim]` verilirse o paketi projeye ekler. Native modüller için `--güvenilir` (veya `--trusted`) bayrağı kullanılabilir.
- `huma sil <isim>` (Alias: `paket sil`, `remove`)
    - **Açıklama:** Bir paketi sistemden kaldırır ve `huma.lock` dosyasını günceller.
- `huma liste` (Alias: `paket liste`, `list`)
    - **Açıklama:** Projeye kurulu tüm paketleri ve versiyonlarını listeler.
- `huma paket güncelle` (Alias: `update`)
    - **Açıklama:** Kurulu paketlerin uzak sunucudaki yeni sürümlerini kontrol eder ve günceller.
- `huma paket doğrula` (Alias: `verify`)
    - **Açıklama:** Proje yapısını ve metadataları yayın öncesi kontrol ederek bütünlüğü doğrular.
- `huma paket çalıştır <ad>` (Alias: `run`, `betik`)
    - **Açıklama:** `huma.json` içindeki tanımlı bir betiği çalıştırır (npm run benzeri).

### 4. Bakım ve Bilgi

- `huma güncelle` (Alias: `update`)
    - **Açıklama:** Hüma CLI aracını GitHub üzerinden en son sürüme yükseltir. `--check` bayrağı ile sadece kontrol yapabilirsiniz.
- `huma sürüm` (Alias: `version`)
    - **Açıklama:** Çalışan Hüma binary dosyasının sürüm bilgisini gösterir.

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

// Aralık Döngüsü (kadar)
j = 1'den 5'e kadar {
    "Sayı: " + j'yi yazdır
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

### 🇹🇷 Doğal Dil ve Ek Sistemi (Suffix System)

Hüma, Türkçe'nin eklemeli yapısını destekler. Kesme işareti (`'`) ile ayrılan dilbilgisi ekleri, derleyici tarafından derleme aşamasında (lexical analysis) temizlenir. Bu sayede kodun okunabilirliği artarken çalışma zamanında herhangi bir performans kaybı yaşanmaz.

**Desteklenen Dilbilgisi Ekleri ve Kullanım Şablonları:**

| Durum / Ek Grubu | Desteklenen Tüm Varyasyonlar | Örnek Yazım | Açıklama |
| :--- | :--- | :--- | :--- |
| **Belirtme (-i Hali)** | `'i`, `'ı`, `'u`, `'ü`, `'yi`, `'yı`, `'yu`, `'yü`, `'ni`, `'nı`, `'nu`, `'nü` | `hata'yı yazdır` | Nesneyi vurgular. |
| **Yönelme (-e Hali)** | `'e`, `'a`, `'ye`, `'ya` | `liste'ye ekle` | Hedef veya yön belirtir. |
| **Bulunma (-de Hali)** | `'de`, `'da`, `'te`, `'ta` | `bellek'te tut` | Konum veya durum belirtir. |
| **Ayrılma (-den Hali)** | `'den`, `'dan`, `'ten`, `'tan` | `liste'den çıkar` | Kaynak bildirir. |
| **Çoğul Eki (-lar)** | `'lar`, `'ler` | `sayılar'ı yazdır` | Çokluk belirtir. |
| **İlgi / Tamlama (-nin)** | `'nin`, `'nın`, `'nun`, `'nün`, `'ın`, `'in`, `'un`, `'ün` | `ayarlar'ın sürümü` | Nesne özelliklerine erişim. |
| **Eşitlik Hali (-ce)** | `'ce`, `'ca`, `'çe`, `'ça` | `Türkçe'ce yaz` | "Gibi / olarak" anlamı katar. |
| **Vasıta (-le Hali)** | `'le`, `'la`, `'yle`, `'yla` | `hız'la çalıştır` | Araç veya yöntem belirtir. |
| **İyelik (-si Eki)** | `'si`, `'sı`, `'su`, `'sü`, `'i`, `'ı`, `'u`, `'ü` | `tema'sı`, `adı` | Aitlik/iyelik bildirir. |
| **Sıfat Yapan (-ki)** | `'deki`, `'daki`, `'teki`, `'taki` | `kod'daki hata` | Nitelik belirtir. |
| **Soru Eki (-mi)** | `mi`, `mı`, `mu`, `mü` | `bayrak mi ise` | Koşullarda soru vurgusu. |



```huma
// Ek sistemi sayesinde çok daha akıcı ve okunabilir kodlar yazılır:
ayarlar = { "tema": "koyu" } olsun

// Kötü: yazdır ayarlar.tema
// İyi:
yazdır ayarlar'ın tema'sı;
```

---

## 🧰 CLI (`huma`)

- **Çalıştırma**: `huma run <dosya.hb>`
- **Derleme**: `huma build <dosya.hb> -o cikti.hbc`
- **Bytecode çalıştırma**: `huma exec <dosya.hbc>`
- **REPL**: `huma repl`
- **Test runner**: `huma test [yol]`
  - `yol` verilmezse `tests/` (varsa) taranır
  - Her test dosyası varsayılan olarak **3 saniye** içinde bitmezse hata sayılır (kilitlenmeleri engellemek için)

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
