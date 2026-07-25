# Hüma 0.6 Dil Tanımı

Bu belge Hüma 0.6 için kanonik ve normatif sözdizimini tanımlar. Örnek veya web sayfasıyla çelişirse bu belge esas alınır.

## 1. Kaynak metin ve tanımlayıcılar

- Kaynak dosyası UTF-8’dir ve `.hb` uzantısını kullanır.
- Tanımlayıcılar Unicode harf, sayı ve `_` içerebilir; ilk karakter harf veya `_` olmalıdır.
- Türkçe karakterler doğrudan kullanılabilir: `değer`, `öğrenci_sayısı`, `çözüm`.
- Anahtar sözcükler tanımlayıcı olarak kullanılamaz. Özellikle `doğru`, `yanlış`, `liste`, `devam` ve `kır` ayrılmıştır. ASCII eşdeğerleri de ayrılmıştır: `dogru`, `yanlis`, `sinif` gibi.
- `//` ile satır sonuna kadar yorum yazılır.
- Noktalı virgül isteğe bağlıdır.

## 2. Değerler

Kanonik literal ve temel çalışma zamanı değerleri:

```huma
sayı = 42 olsun
ondalık = 1.25e-3 olsun
metin = "Türkçe UTF-8" olsun
evet = doğru olsun
hayır = yanlış olsun
yok = boş olsun
dizi = [1, 2, 3] olsun
sözlük = {"ad": "Hüma", "sürüm": 0.6} olsun
```

`doğru` ve `yanlış` çalışma zamanında sırasıyla `1` ve `0` sayısal değerleriyle temsil edilir. Hüma 0.6 statik tip sistemi sunmaz.

## 3. Atama

Atama cümlesi `olsun` ile biter:

```huma
x = 10 olsun
x = x + 1 olsun
dizi[0] = 20 olsun
sözlük["ad"] = "Yeni ad" olsun
nesne.alan = 3 olsun
```

Atamanın sol tarafı yalnızca değişken, nesne alanı veya indekslenebilir öğe olabilir. `doğru = 1 olsun` gibi ayrılmış sözcük atamaları sözdizimi hatasıdır.

Fonksiyon çağrısından sonra kullanılan `olsun`, çağrının sonucunun bilerek atıldığını belirtir:

```huma
model.egit(veri, etiketler, 10, 0.01) olsun
```

## 4. Ek ayıracı

Kesme işareti, bir Hüma tanımlayıcısını Türkçe görünümlü durum ekinden ayıran programlama dili işaretidir:

```huma
değer'i yazdır
metin'i döndür
1'den 5'e kadar
dizi'nin uzunluğu
```

Lexer şu yüzey biçimlerini kabul eder:

- belirtme: `i ı u ü yi yı yu yü ni nı nu nü`
- yönelme: `a e ya ye`
- bulunma: `da de ta te`
- ayrılma: `dan den tan ten`
- ilgi: `nin nın nun nün in ın un ün`
- diğer ayırıcı biçimler: `si sı su sü lar ler ca ce ça çe daki deki taki teki la le yla yle`

İlgi eki, özellik/uzunluk erişiminde yapısal anlam taşır. Diğer eklerin çoğu ifade sınırını okunur kılan ayıraçlardır ve yeni bir çalışma zamanı değeri üretmez. Bilinmeyen ekler hata verir.

Bu mekanizma, kod gramerini deterministik tutmak için tasarlanmıştır. Sözcüğün son ünlüsüne göre ünlü uyumunu veya doğal Türkçedeki bütün biçimbilim kurallarını otomatik doğrulamaz. Kanonik kaynak kodunda anlam ve ses uyumuna uygun yüzey biçimi seçilmelidir.

## 5. Çıktı ve dönüş

```huma
"Merhaba"'yı yazdır
yazdır "Merhaba"

iki_katı fonksiyon olsun n alsın {
    (n * 2)'yi döndür
}
```

Fonksiyonlarda örtük dönüş yoktur. `döndür` çalışmazsa sonuç `Boş` olur.

## 6. Koşullar ve mantık

```huma
x > 0 ve x < 10 ise {
    "aralıkta"'yı yazdır
} yoksa {
    "aralık dışında"'yı yazdır
}
```

Öncelik yüksekten düşüğe: parantez/erişim/çağrı, tekli `-` ve `değil`, `* / %`, `+ -`, karşılaştırma, eşitlik, `ve`, `veya`.

`ve` ve `veya` kısa devrelidir. Aşağıdaki ifade tanımsız değişkene erişmez:

```huma
yanlış ve tanımsız_değişken
```

Eşitlik için ifadelerde `=` veya `==` kabul edilir. Atama, yalnızca cümle sonundaki `olsun` ile oluşur.

## 7. Döngüler

```huma
i = 0 olsun
i < 10 olduğu sürece {
    i = i + 1 olsun
    i = 4 ise { devam }
    i = 8 ise { kır }
}

j = 1'den 5'e kadar {
    j'yi yazdır
}
```

Aralık iki sınırı da içerir. `devam` ve `kır` yalnızca döngü içinde geçerlidir.

## 8. Fonksiyonlar ve kapsam

```huma
topla fonksiyon olsun a, b alsın {
    (a + b)'yi döndür
}

sonuç = topla(2, 3) olsun
```

Fonksiyon parametreleri ve fonksiyon içinde ilk kez atanan adlar yerel kapsamdadır. Yerel atama aynı adlı parametreyi/yerel değişkeni günceller. Paylaşılan değiştirilebilir durum için nesne, liste veya sözlük kullanılmalıdır.

Azami çağrı derinliği 50’dir; aşılması yakalanabilir çalışma zamanı hatasıdır.

## 9. Sınıflar

```huma
sayaç sınıf olsun {
    değer = 0 olsun

    artır fonksiyon olsun miktar alsın {
        kendisi.değer = kendisi.değer + miktar olsun
        kendisi.değer'i döndür
    }
}

s = sayaç() olsun
s.artır(2)'yi yazdır
```

`kendisi'nin değer'i` biçimi de özellik erişimi için kabul edilir.

## 10. Hata yönetimi

```huma
dene {
    10 / 0'ı yazdır
} yakala hata {
    hata'yı yazdır
}
```

Eski `dene { ... } hata var ise { ... }` biçimi geriye dönük uyumluluk için kabul edilir; kanonik biçim `yakala`dır.

Tanımsız ad, tip uyuşmazlığı, sıfıra bölme, sınır dışı indeks, çağrılamayan değer ve bulunamayan modül hata olarak yayılır. Hata oluşan komuttan sonraki komutlar, hata yakalanmadıkça çalıştırılmaz.

## 11. Modüller

İki kanonik biçim eşdeğerdir:

```huma
yükle "matematik.hb"
"matematik.hb"'yi yükle
```

Arama sırası mevcut dizin, `lib/`, `huma_modulleri/` ve yüklenen modülün dizinidir. Aynı çözülmüş modül bir yorumlayıcı örneğinde bir kez yüklenir.

## 12. Arka uç uyumluluğu

Yorumlayıcı normatif semantiği belirler. Bytecode ve AOT arka uçları destekledikleri alt kümelerde aynı sonucu vermelidir. Bir yapı desteklenmiyorsa derleyici açık hata vermelidir; sessiz bir varsayılan değer üretmesi dil hatası sayılır.
