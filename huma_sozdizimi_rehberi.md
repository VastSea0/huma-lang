# Hüma Programlama Dili — Sözdizimi (Syntax) Doğru ve Yanlış Kullanım Kılavuzu

Bu belge, **Hüma Programlama Dili** (`.hb`) ile kod yazarken yapılan yaygın sözdizimi (syntax) hatalarını ve bunların doğru Türkçe kullanım şekillerini karşılaştırmalı bir tablo halinde sunmaktadır.

---

## 📋 Sözdizimi Karşılaştırma Tablosu

| Konu / Kategori | ❌ Yanlış Kullanım (Wrong) | ✅ Doğru Kullanım (True) | Açıklama ve Hata Nedeni |
| :--- | :--- | :--- | :--- |
| **Değişken Atama** | `x = 10` | `x = 10 olsun` | Değişken atamalarının sonunda **`olsun`** kelimesi zorunludur. |
| **Değişken Atama** | `let x = 10;` <br> `var x = 10;` | `x = 10 olsun` | Hüma'da `let`, `var`, `const` gibi İngilizce sözcükler yoktur. |
| **Değişken Atama** | `olsun x = 10` | `x = 10 olsun` | `olsun` ifadesi atama cümlesinin **sonuna** yazılır. |
| **Ekrana Yazdırma** | `print("Merhaba")` <br> `console.log("Merhaba")` | `"Merhaba"'yı yazdır;` <br> `yazdır "Merhaba";` | Ekrana yazdırma komutu **`yazdır`**dır. Nesne eki (`'yı`, `'yi`) kullanımı önerilir. |
| **Koşul (If)** | `if (x > 5) { ... }` | `x > 5 ise { ... }` | İngilizce `if` yerine **`ise`** kullanılır ve koşulun **sonuna** yazılır. |
| **Koşul (Else)** | `else { ... }` | `yoksa { ... }` | İngilizce `else` yerine **`yoksa`** kullanılır. |
| **Koşul (Soru Eki)** | `if (x > 5 == true)` | `x > 5 mi ise { ... }` | Koşul vurgusu için Türkçe soru eki (**`mi` / `mı` / `mu` / `mü`**) eklenebilir. |
| **Döngü (While)** | `while (i < 10) { ... }` | `i < 10 olduğu sürece { ... }` | `while` yerine **`olduğu sürece`** deyimi koşul ifadesinin **sonuna** getirilir. |
| **Döngü (For / Range)** | `for i in 1..10 { ... }` | `i = 1'den 10'a kadar { ... }` | Aralık döngülerinde **`... 'den ... 'e kadar`** kalıbı kullanılır. |
| **Fonksiyon Tanımlama** | `function topla(a, b) { ... }` <br> `def topla(a, b):` | `topla = fonksiyon olsun a, b alsın { ... }` | Fonksiyonlar **`fonksiyon olsun [parametreler] alsın`** yapısıyla tanımlanır. |
| **Fonksiyon Parametresiz** | `fonksiyon selam() { ... }` | `selam = fonksiyon olsun { ... }` | Parametresiz fonksiyonlarda `alsın` kısmına gerek yoktur. |
| **Değer Döndürme** | `return a + b;` | `(a + b)'yi döndür;` <br> `a + b döndür` | `return` yerine **`döndür`** anahtar kelimesi kullanılır. |
| **Fonksiyon Çağırma** | `topla.çağır(5, 10)` | `sonuc = topla(5, 10) olsun` <br> `sonuc = 5 ile 10'u topla'yı çağır olsun` | Standart parantezli çağrı veya `ile ... 'ı çağır` doğal dil formatı kullanılır. |
| **Sınıf (Class)** | `class Araba { ... }` | `Araba = sınıf olsun { ... }` | `class` yerine **`sınıf olsun`** kullanılır. |
| **Sınıf İçi Self/This** | `this.hız = 100;` <br> `self.hiz = 100` | `kendisi'nin hız'ı = 100 olsun` | Sınıf içi nitelik erişimlerinde `this`/`self` yerine **`kendisi`** kullanılır. |
| **Nesne Örnekleme** | `araba = new Araba()` | `araba = Araba() olsun` | `new` operatörü yoktur; nesne doğrudan sınıf adı çağrılarak türetilir. |
| **Mantıksal VE** | `x > 0 && y > 0` | `x > 0 ve y > 0 ise { ... }` | `&&` yerine **`ve`** kelimesi kullanılır. |
| **Mantıksal VEYA** | `a > 0 \|\| b > 0` | `a > 0 veya b > 0 ise { ... }` | `\|\|` yerine **`veya`** kelimesi kullanılır. |
| **Mantıksal DEĞİL** | `!aktif` | `değil aktif ise { ... }` | `!` yerine **`değil`** kelimesi kullanılır. |
| **Mantıksal Sabitler** | `durum = true` <br> `durum = false` | `durum = doğru olsun` <br> `durum = yanlış olsun` | `true`/`false` yerine **`doğru`** ve **`yanlış`** kullanılır. |
| **Liste Tanımlama** | `array(1, 2, 3)` | `sayılar = [1, 2, 3] olsun` | Listeler köşeli parantez `[...]` ile tanımlanır. |
| **Listeye Eleman Ekleme** | `sayılar.push(4)` | `sayılar'a [4]'ü ekle;` <br> `ekle sayılar 4;` | `push()` yerine **`... 'e [...]'ü ekle`** veya `ekle` deyimi kullanılır. |
| **Listedeneden Çıkarma** | `sayılar.remove(0)` | `sayılar'dan [0]'ı çıkar;` | İndeks bazlı silmede **`... 'den [...]'ı çıkar`** kalıbı kullanılır. |
| **Liste Uzunluğu** | `sayılar.length` <br> `len(sayılar)` | `u = sayılar'ın uzunluğu olsun` | Uzunluk alma işleminde **`... 'in uzunluğu`** tamlaması kullanılır. |
| **Sözlük (Map / Dict)** | `kullanıcı = { ad: "Hüma" }` | `kullanıcı = { "ad": "Hüma" } olsun` | Sözlük anahtarları çift tırnaklı metin literal (`"ad"`) olmalıdır. |
| **Dilbilgisi Ek Kullanımı** | `sayıyı yazdır` | `sayı'yı yazdır` | Eklerin derleyici tarafından temizlenmesi için kesme işareti (**`'`**) şarttır. Kesmesiz yazımda `sayıyı` ayrı bir değişken adı sanılır. |
| **Modül Yükleme** | `import "matematik.hb"` <br> `require("matematik.hb")` | `yükle "matematik.hb";` <br> `"matematik.hb"'yi yükle;` | Modül yüklemek için **`yükle`** komutu kullanılır. |
| **Hata Yakalama** | `try { ... } catch (e) { ... }` | `dene { ... } hata var ise { ... }` | `try/catch` yerine **`dene { ... } hata var ise { ... }`** yapısı kullanılır. |

---

## 💡 Detaylı Örnekler ve Yan Yana Karşılaştırmalar

### 1. Koşul ve Döngü Yapısı

❌ **Yanlış (İngilizce/C Stili):**
```huma
if (sayi > 0) {
    print("Pozitif");
} else {
    print("Negatif veya Sıfır");
}

while (i < 5) {
    i = i + 1;
}
```

✅ **Doğru (Hüma Stili):**
```huma
sayi > 0 ise {
    "Pozitif"'i yazdır;
} yoksa {
    "Negatif veya Sıfır"'ı yazdır;
}

i = 0 olsun
i < 5 olduğu sürece {
    i = i + 1 olsun
}
```

---

### 2. Fonksiyon ve Nesne Yönelim (OOP)

❌ **Yanlış (Hatalı Tanımlamalar):**
```huma
function hesapla(a, b) {
    return a + b;
}

class Hesaplayici {
    this.deger = 0;
}
h = new Hesaplayici();
```

✅ **Doğru (Hüma Standartları):**
```huma
hesapla = fonksiyon olsun a, b alsın {
    (a + b)'yi döndür
}

Hesaplayici = sınıf olsun {
    deger = 0 olsun
    
    ekle = fonksiyon olsun miktar alsın {
        kendisi'nin deger'i = kendisi'nin deger'i + miktar olsun
    }
}

h = Hesaplayici() olsun
```

---

### 3. Türkçe Ek Sistemi (Suffixes)

Hüma'da tanımlayıcılara gelen ekler kesme işareti (`'`) ile ayrılmalıdır. Kesme işareti koyulmazsa derleyici sözcüğü tek parça yeni bir değişken adı olarak değerlendirir.

❌ **Yanlış:**
```huma
sayı = 50 olsun
yazdır sayıyı  // Hata: "sayıyı" isimli tanımlayıcı bulunamadı!
```

✅ **Doğru:**
```huma
sayı = 50 olsun
sayı'yı yazdır // Başarılı: 'yı eki temizlenir, "sayı" değişkeni yazdırılır.
```
