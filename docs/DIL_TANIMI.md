# Hüma 0.6 Dil Tanımı

Bu belge Hüma 0.6 için kanonik ve normatif sözdizimi ile çalışma zamanı
sözleşmesini tanımlar. Örnek veya web sayfasıyla çelişirse bu belge esas alınır.
Makinece okunabilir yüzey grameri [DIL_GRAMERI.ebnf](DIL_GRAMERI.ebnf)
dosyasındadır.

## 1. Kaynak metin ve tanımlayıcılar

- Kaynak dosyası UTF-8’dir ve `.hb` uzantısını kullanır.
- Tek kaynak dosyası en fazla 64 MiB olabilir.
- Tanımlayıcılar Unicode harf, sayı ve `_` içerebilir; ilk karakter harf veya `_` olmalıdır.
- Tanımlayıcılar lexer tarafından Unicode NFC biçimine getirilir. Böylece görsel
  olarak aynı birleşik ve ayrık Unicode yazımları aynı adı belirtir.
- Türkçe karakterler doğrudan kullanılabilir: `değer`, `öğrenci_sayısı`, `çözüm`.
- Anahtar sözcükler tanımlayıcı olarak kullanılamaz. Özellikle `doğru`, `yanlış`, `liste`, `devam` ve `kır` ayrılmıştır. ASCII eşdeğerleri de ayrılmıştır: `dogru`, `yanlis`, `sinif` gibi.
- `//` ile satır sonuna kadar yorum yazılır.
- Noktalı virgül isteğe bağlıdır.
- Metin kaçışları yalnızca `\n`, `\r`, `\t`, `\\`, `\"` ve tam iki onaltılık
  basamaklı `\xNN` biçimleridir. Bilinmeyen veya yarım kaçışlar hatadır.

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

Hüma 0.6'nın tek sayısal türü sonlu IEEE-754 `f64` değeridir. Kaynak literal,
aritmetik işlem veya dış veri dönüşümü `NaN` ya da sonsuz değer üretemez; böyle
bir sonuç yakalanabilir çalışma zamanı hatasıdır. Metinler sayısal işlemlerde
örtük olarak sayıya çevrilmez. Tamsayı hassasiyetinin gerektiği API'ler
`2^53 - 1` sınırını aşan değerleri ayrıca reddetmelidir.

`+`, iki sayıyı toplar. İşlenenlerden en az biri metinse diğer değerin kanonik
gösterimini metne ekler. `-`, `*`, `/` ve `%` yalnızca iki sonlu sayı kabul eder.
Sıralama işlemleri (`<`, `>`, `<=`, `>=`) iki sayı arasında sayısal, iki metin
arasında Unicode kod noktası sırasına dayalı sözlüksel karşılaştırma yapar.
Karışık türler hata verir.

Sözlük anahtarları yalnızca metin olabilir. Metin olmayan anahtarlar örtük
dönüştürülmez veya sessizce atılmaz.

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

İlgi eki özellik/uzunluk erişiminde, yönelme ve ayrılma ekleri doğal liste
işlemlerinde yapısal anlam taşır. Diğer eklerin çoğu ifade sınırını okunur
kılan ayıraçlardır ve yeni bir çalışma zamanı değeri üretmez. Bilinmeyen ekler
hata verir.

Lexer, telaffuzu yazımdan çıkarılabilen adlarda son yazılı ünlüye göre iki ve
dört yönlü ünlü uyumunu, ünlüyle biten köklerde `y`/`n` kaynaştırmasını ve
`ç f h k p s ş t` sonrasında `d → t`, `c → ç` benzeşmesini doğrular. Zincirli
kesme işaretli eklerde oluşan yüzey kökü bir sonraki denetime taşınır.

Tek harfli ad, sayı içeren ad, ünlüsüz kısaltma ve tamamı büyük harfli
kısaltmanın telaffuzu yalnızca yazımdan güvenilir biçimde çıkarılamaz. Bu
sınıflarda lexer yalnızca ekin tanımlı olmasını denetler; sözlük veya telaffuz
verisi olmadan daha güçlü bir doğruluk iddiası yapılamaz. Dolayısıyla Hüma,
tanımlı programlama dili gramerine ve denetlenebilir Türkçe ses kurallarına
uyar; serbest doğal Türkçe için genel amaçlı bir biçimbilim çözümleyicisi
olduğunu iddia etmez.

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

Koşul doğruluğu tek bir çalışma zamanı kuralına sahiptir: `boş`, `0`, boş metin,
boş bayt, boş liste, boş sözlük ve boş vektör yanlıştır. Diğer geçerli değerler
doğrudur. Yorumlayıcı ve VM bu ortak kuralı kullanır.

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

Fonksiyon çağrısındaki argüman sayısı parametre sayısıyla tam eşleşmelidir;
eksik veya fazla argüman çalışma zamanı hatasıdır. Anonim ve iç içe fonksiyonlar
tanımlandıkları andaki sözcüksel kapsamı yakalar. Bu nedenle dış fonksiyon
döndükten sonra da yakalanan değerlere erişebilirler.

Doğal çağrı biçiminde ilk argüman `ile`, sonraki argümanlar `ve` ile ayrılır:

```huma
1 ile 2 ve 3'ü topla3
```

Bu biçimde üst düzey `ve` argüman ayırıcıdır. Bir mantıksal `ve` ifadesini tek
argüman yapmak için parantez gerekir: `(doğru ve yanlış) ile 1'i işle`.
Parantezli `fonksiyon(a, b, c)` biçiminde bu belirsizlik yoktur.

Hüma 0.6 sınıflarında özel kurucu parametresi sözdizimi yoktur. Sınıf çağrısı
argümansız yapılır; alanlar oluşturulduktan sonra açık bir `ilklendir` metodu
çağrılabilir.

Varsayılan azami çağrı derinliği 32’dir; aşılması yakalanabilir çalışma zamanı
hatasıdır. AST yorumlayıcısı ev sahibi Rust yığınını korumak için 32 üst
sınırını aşmaz. VM çağrı frame’lerini kendi yığınında tutar ve aynı varsayılan
çalıştırma sınırını uygular.

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

Listeye doğal ekleme ve indeksle çıkarma biçimleri şöyledir:

```huma
öğeler liste olsun
öğeler'e 1'i ekle
öğeler'e [2, 3]'ü ekle
öğeler'den 0'ı çıkar
```

Eklenecek değer bir listeyse elemanları hedef listeye sırayla eklenir; liste
değilse tek eleman eklenir. Çıkarma indeksi sıfır tabanlı, negatif olmayan
tamsayı olmalıdır.

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

Takma adlı ve açık dışa aktarımlı kanonik biçim:

```huma
// hesap.hb
topla fonksiyon olsun a, b alsın { (a + b)'yi döndür }
topla'yı dışa aktar

// ana.hb
yükle "hesap.hb" olarak hesap
hesap.topla(2, 3)'ü yazdır
```

Arama sırası mevcut dizin, `lib/`, `huma_modulleri/` ve yüklenen modülün
dizinidir. Dosya kimliği kanonik dosya yoludur. Aynı çözülmüş modül bir
yorumlayıcı örneğinde bir kez yüklenir; döngüsel yükleme açık hata verir.
Başarısız yükleme önbelleği ve kısmi ad bağlarını geri alır. Takma adlı modül
yalnızca açık dışa aktarımları gösteren canlı bir ad alanıdır; özel adlar dışarı
sızmaz.

## 12. Arka uç uyumluluğu

Yorumlayıcı normatif semantiği belirler. Bytecode ve AOT arka uçları destekledikleri alt kümelerde aynı sonucu vermelidir. Bir yapı desteklenmiyorsa derleyici açık hata vermelidir; sessiz bir varsayılan değer üretmesi dil hatası sayılır.

Bytecode VM bağımsız çağrı frame’leri, sözcüksel closure ortamları, tam argüman
sayısı denetimi ve kaynak konumlu hata izleri kullanır; fonksiyon gövdelerini AST
yorumlayıcısına geri göndermez. Cranelift AOT arka ucu ayrı ve deneysel sayısal
alt kümedir.

## 13. Yetenek ve güvenlik modeli

Dosya okuma/yazma, ağ istemcisi/sunucusu, süreç, FFI, veritabanı ve GUI
yetenekleri varsayılan olarak kapalıdır. CLI’da yalnız gereken yetenek açıkça
verilir:

```bash
huma run uygulama.hb --izin dosya-okuma --izin ağ-istemci
```

`--tüm-izinler` yalnız güvenilen kod içindir. Yetenek denetimi işletim sistemi
sandbox’ı değildir: izin verilen işlem, Hüma sürecinin işletim sistemi
haklarıyla çalışır. Özellikle FFI yanlış bir C ABI imzasıyla çağrılırsa Rust
tarafındaki denetimlere rağmen ev sahibi süreci çökertebilir. FFI yalnız
`f64()`, `f64(f64)` ve `f64(f64,f64)` açık imzalarını kabul eder; kitaplık
yaşam döngüsü `ffi_yükle`/`ffi_boşalt` ile yönetilir.

## 14. Kaynak sınırları

Varsayılan yürütme sınırları 10.000.000 adım, 32 iç içe çağrı, 16 MiB dil
çıktısı ve tek koleksiyon literalinde 1.000.000 öğedir. Kaynak ve bytecode
payload’u 64 MiB ile sınırlıdır. Dosya, ağ, süreç, SQL, tensor, matris, regex,
tokenizer ve benzeri yerleşikler ayrıca işlemine uygun boyut, süre ve öğe
sınırları uygular. Sınır aşımı sessiz kısaltma değil yakalanabilir hata üretir.
