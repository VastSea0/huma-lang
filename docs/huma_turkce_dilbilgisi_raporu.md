# Hüma Programlama Dili — Türkçe Dilbilgisi Uyumluluk Raporu

Bu rapor, Hüma dilinin Türkçe dilbilgisi eklerini destekleme durumunu, kaynak kod ve dokümantasyon karşılaştırması üzerinden analiz eder.

---

## 1. README Tablosundaki Dilbilgisi Hali Analizi

README'deki tablo 8 dilbilgisi hali / ek grubu içermektedir. Aşağıda her birinin **lexer.rs** ve **parser.rs** kaynak koduyla karşılaştırmalı durumu verilmektedir.

### 1.1 Durum Tablosu

| # | Dilbilgisi Hali | README'de İddia | Kod Gerçeği | Durum |
|---|---|---|---|---|
| 1 | **Belirtme (-i Hali)** `'i`, `'ı`, `'u`, `'ü`, `'yi`, `'yı`, `'yu`, `'yü`, `'ni`, `'nı`, `'nu`, `'nü` | ✅ Desteklendi | `is_turkish_suffix()` içinde: `"i"\|"ı"\|"u"\|"ü"\|"yi"\|"yı"\|"yu"\|"yü"\|"ni"\|"nı"\|"nu"\|"nü"` — **tam liste mevcut** | ✅ **Doğru Çalışıyor** |
| 2 | **Yönelme (-e Hali)** `'e`, `'a`, `'ye`, `'ya` | ✅ Desteklendi | `is_turkish_suffix()` içinde: `"a"\|"e"\|"ya"\|"ye"` — mevcut | ✅ **Doğru Çalışıyor** |
| 3 | **Bulunma (-de Hali)** `'de`, `'da`, `'te`, `'ta` | ✅ Desteklendi | `is_turkish_suffix()` içinde: `"da"\|"de"\|"ta"\|"te"` — mevcut | ✅ **Doğru Çalışıyor** |
| 4 | **Ayrılma (-den Hali)** `'den`, `'dan`, `'ten`, `'tan` | ✅ Desteklendi | `is_turkish_suffix()` içinde: `"dan"\|"den"\|"tan"\|"ten"` — mevcut | ✅ **Doğru Çalışıyor** |
| 5 | **İlgi/Tamlama (-nin)** `'nin`, `'nın`, `'nun`, `'nün`, `'ın`, `'in`, `'un`, `'ün` | ✅ Desteklendi (özellik erişimi) | `handle_apostrophe()`: `"nin"\|"nın"\|"nun"\|"nün"\|"in"\|"ın"\|"un"\|"ün"` → `Token::Nin` yayar. Parser'da `Token::Nin` uygulanmış. | ✅ **Doğru Çalışıyor** (ancak bkz. Sorunlar §3.1) |
| 6 | **Vasıta (-le Hali)** `'le`, `'la`, `'yle`, `'yla` | ✅ Desteklendi | `is_turkish_suffix()` içinde: `"la"\|"le"\|"yla"\|"yle"` — mevcut | ✅ **Strips Edilir Ama...** Parser'da semantik karşılığı **yok** (bkz. §3.2) |
| 7 | **İyelik (-si Eki)** `'si`, `'sı`, `'su`, `'sü`, `'i`, `'ı`, `'u`, `'ü` | ✅ Desteklendi (sahiplik) | `handle_apostrophe()`: `NinState::AfterNinProperty` durumunda `Token::Iyelik` yayılıyor. Ancak **yalnızca `'nin` zincirinden sonra** tetikleniyor. | ⚠️ **Kısmi Destek** (bkz. §3.3) |
| 8 | **Sıfat Yapan (-ki)** `'deki`, `'daki`, `'teki`, `'taki` | ✅ Desteklendi | `is_turkish_suffix()` içinde: `"daki"\|"deki"\|"taki"\|"teki"` — strip edilir | ⚠️ **Strip Edilir Ama Semantik Yok** (bkz. §3.4) |

---

## 2. README'de Yer Almayan Mevcut Özellikler

Kaynak kod üzerinde yapılan incelemede, README tablosunda belirtilmeyen ancak dilde aktif olan şu özellikler tespit edilmiştir:

### 2.1 `kendisi` Anahtar Kelimesi (OOP Self Referansı)
```
// README'de tabloda yer almıyor, ancak token.rs'de tanımlı ve parser/interpreter tam işliyor
islem sınıf olsun {
    topla fonksiyon olsun a alsın {
        kendisi'nin sonuç = a olsun
    }
}
```
- `Token::Kendisi`, `Token::Nin`, `Token::Iyelik` zincirleme çalışır.
- `Ifade::KendisiErisim`, `Komut::NesneAlaniAtama` tam desteklidir.

### 2.2 Türkçe Karaktere Duyarlı NLP Fonksiyonları
```
// interpreter.rs'de built-in olarak tanımlı:
küçük_harf("HÜMA")    // I→ı, İ→i dönüşümü
büyük_harf("hüma")    // ı→I, i→İ dönüşümü
```
- `I/İ` ayrımı ve `ı/i` dönüşümü doğru şekilde uygulanmaktadır. Standart `to_lowercase()` Türkçe'yi yanlış dönüştürdüğünden bu önemli bir özelliktir.

### 2.3 Hata Yakalama `dene` / `hata var ise`
```
dene {
    riskli_islem()
} hata var ise {
    "Hata oluştu"'yu yazdır
}
```
- Token, parser ve interpreter tam desteklidir. README'deki özellikler listesinde yalnızca "zengin sistem kütüphaneleri" olarak geçmektedir; dilbilgisi tablosunda yoktur.

### 2.4 Türkçe Mantık Operatörleri
```
x > 0 ve y > 0 ise { ... }
x > 0 veya y > 0 ise { ... }
değil x ise { ... }
```
- `Token::Ve`, `Token::Veya`, `Token::Degil` tam desteklidir. İngilizce `&&`, `||`, `!` yerine doğal Türkçe kullanımı sağlanmıştır.

### 2.5 Ardışık ve Zincirli Ek Desteği
```
// Bir tanımlayıcıya art arda kesme işareti gelebilir
liste'den[0]'ı çıkar
fonksiyon(arg)'yı yazdır
```
- `handle_apostrophe()` içindeki `loop` yapısı, art arda gelen `'` işaretlerini zincirleme işleyebilmektedir.

### 2.6 Sayı ve String Literal'lere Ek Uygulama
```
0'ı döndür   // sayıya ek
"metin"'i yazdır  // stringe ek
```
- `read_number()` ve `read_string()` fonksiyonları kesme + ek kombinasyonunu desteklemektedir.

### 2.7 Türkçe Eşdeğer `doğru` / `yanlış` Sabitleri
```
bayrak = doğru olsun
bayrak = yanlış olsun
```
- `Token::Dogru`, `Token::Yanlis` → `Ifade::Dogru`, `Ifade::Yanlis` → sırasıyla `1.0` ve `0.0` olarak değerlendirilir.

### 2.8 Unicode-Güvenli String İşleme
```
// dizi_dilim char-bazlıdır, byte-bazlı değil
dizi_dilim("Günaydın", 0, 3)  // "Gün" (byte değil char indeksi)
```
- `interpreter.rs` içindeki `dizi_dilim` built-in fonksiyonu `.chars().collect()` kullanmaktadır.

---

## 3. Eksik veya Hatalı Türkçe Dilbilgisi Uygulamaları

### 3.1 ⚠️ `-nin` Ekinin Karışıklığı: Belirtme ile İlgi Hali Çakışması

**Sorun:** `is_turkish_suffix()` içinde `"in"`, `"ın"`, `"un"`, `"ün"` hem **belirtme hali** (`'i` varyasyonları) hem de **ilgi/tamlama hali** liste içinde çakışıyor.

```rust
// lexer.rs: is_turkish_suffix() içinde
"ni" | "nı" | "nu" | "nü"  // ← Belirtme hali varyasyonu

// AYRICA handle_apostrophe() içinde:
"in" | "ın" | "un" | "ün"  // ← Bu Nin token'ı yayıyor (ilgi hali)
```

**Problem:** `x'in` ifadesinde `'in` eki **ilgi hali** olarak doğru yorumlanır. Ancak `metin'in uzunluğu` yerine `metin'i` yazıldığında lexer `'i` → belirtme, `'in` → ilgi hali olarak yorumlar. Bu tutarlıdır. **Ama** `karakter'i n'yi...` gibi belirli kenar durumlarda çakışma oluşabilir.

**Örnek Hata Senaryosu:**
```
// Bu çalışır (belirtme):
x'i yazdır

// Bu da çalışır (ilgi = Nin token):
liste'nin uzunluğu

// Ama bu HATALI parse edilebilir:
"içinde"'nin içindeki  // 'nin sonrası Nin bekliyor ama 'nin → is_turkish_suffix'te de var mı?
```

`is_turkish_suffix()` içinde `"nin"`, `"nın"`, `"nun"`, `"nün"` **yok** — sadece `handle_apostrophe()`'te ayrı branch'te yakalanıyor. Bu görünürde doğru. Ancak şu durum sorunludur:

```rust
// handle_apostrophe() içinde sıralama:
// 1. NinState::AfterNinProperty ise → İyelik kontrolü
// 2. "nin"/"nın"... eşleşirse → Token::Nin
// 3. is_turkish_suffix() ise → yut
// 4. Bilinmeyense → hata
```

`"in"` ve `"ın"` hem `Token::Nin` yayar (branch 2) hem de teorik olarak belirtme hali sayılabilirdi. Mevcut davranış: `liste'in` yazıldığında `Nin` token yayılır, ki bu **doğal Türkçe açısından hatalıdır** — `'in` belirtme hali varyasyonudur, ilgi hali için `'nin` ya `'ın` kullanılmalıdır. Ancak Türkçe'de `liste'in uzunluğu` yerine `liste'nin uzunluğu` tercih edilir, bu yüzden bu çakışma pratikte nadir oluşur ama dilbilgisel açıdan yanıltıcıdır.

---

### 3.2 ❌ Vasıta Hali (-le) Semantik Karşılığı Yok

**Sorun:** `'le`, `'la`, `'yle`, `'yla` ekleri **sadece strip edilir**, hiçbir semantik anlam taşımaz.

README şunu söylüyor: `hız'la çalıştır` → "Araç veya yöntem belirtir."

Ama `lexer.rs`'de:
```rust
"la" | "le" | "yla" | "yle"  // → is_turkish_suffix = true → EK YUTULUR
```

`parser.rs`'de `-le` hali için hiçbir özel dal yok. Bu ek sadece "sesini kesmek" amacıyla drop ediliyor, semantik anlam üretilmiyor.

**Sonuç:** `hız'la çalıştır` ve `hız çalıştır` tamamen aynı şeyi yapar. Vasıta hali bilgisi kaybolmaktadır.

---

### 3.3 ⚠️ İyelik Ekinin Kısıtlı Bağlamı

**Sorun:** `Token::Iyelik` yalnızca `X'in Y'si` zincirinde tetiklenebilir. Bağımsız kullanım çalışmaz.

```rust
// lexer.rs handle_apostrophe():
if self.nin_state == NinState::AfterNinProperty
    && matches!(suffix.as_str(), "si" | "sı" | "su" | "sü" | "i" | "ı" | "u" | "ü")
{
    return Token::Iyelik;
}
```

`NinState::AfterNinProperty` yalnızca `Token::Nin` yayıldıktan sonra ve bir tanımlayıcı görüldükten sonra set edilir.

**Çalışan:**
```
ayarlar'ın tema'sı yazdır    // ✅ doğru
```

**Çalışmayan:**
```
hesap.toplam'ı yazdır         // buradaki 'ı belirtme hali, OK
kişi'nin ad'ı yazdır          // 'ı → NinState kontrolüne bakıyor, AfterNinProperty false → is_turkish_suffix ile belirtme olarak drop edilir ✅ AMA anlamsal olarak "iyelik" kaybedildi

nesne'si yazdır               // ❌ 'si → NinState::None → is_turkish_suffix'te yok → HATA!
```

`"si"`, `"sı"`, `"su"`, `"sü"` **`is_turkish_suffix()`'te listelenmemiştir**! Bu kritik bir bug'dır:
```rust
fn is_turkish_suffix(s: &str) -> bool {
    matches!(s,
        "i" | "ı" | "u" | "ü" | "yi" | "yı" | "yu" | "yü" |
        "ni" | "nı" | "nu" | "nü" |
        ...
        // "si" | "sı" | "su" | "sü" → YOK!
    )
}
```

`nesne'si` veya tek başına `X'si` yazıldığında lexer:
1. `NinState::AfterNinProperty` değil → İyelik dalı atlanır
2. `is_turkish_suffix("si")` → `false` → **`Token::Hata("Bilinmeyen ek: 'si")`** döner!

---

### 3.4 ⚠️ Sıfat Yapan (-ki) Semantik Karşılığı Yok

**Sorun:** `'deki`, `'daki`, `'teki`, `'taki` ekleri strip edilir ama semantik anlam üretmez.

README şunu söylüyor: `dosya'daki veri` → nitelik veya konum belirtir.

```
dosya'daki veri yazdır
```
Lexer `'daki` ekini siler ve `veri yazdır` gibi devam eder. `dosya` ile `veri` arasındaki ilişki (dosyanın içindeki veri) semantik olarak **hiç işlenmez**.

---

### 3.5 ❌ Çoğul Eki (-lar / -ler) Desteği Yok

Türkçe'nin en temel eklerinden biri olan **çoğul eki** (`-ler`, `-lar`) desteklenmemektedir.

```
// Doğal yazım (şu an ÇALIŞMIYOR):
meyveler'in hepsi yazdır
sayılar'ı sırala
öğrenciler'e mesaj gönder
```

Lexer `'lar` / `'ler` ekini **tanımıyor**:
```rust
fn is_turkish_suffix(s: &str) -> bool {
    matches!(s,
        // "lar" | "ler" → YOK!
    )
}
```
`meyveler'i yazdır` → `'i` strip edilir, `meyveler` token olarak elde edilir (**bu çalışır zaten** çünkü `meyveler` identifier olarak tanımlanmıştır). Ama `meyveler'ler'i` yazmak anlamsız olduğu için bu sorun pratikte çoğunlukla görünmez — sorun şudur: değişken adı `meyve` olup `meyve'leri yazdır` yazmak istendiğinde bu çalışmaz.

---

### 3.6 ❌ Soru Eki (-mi / -mı / -mu / -mü) Desteği Yok

```
// Doğal Türkçe soru ifadesi (ÇALIŞMIYOR):
değer > 5 mi ise { ... }   // mi? sorusu
liste boş mu ise { ... }
```

Türkçe programlama dilinde soru eki önemli bir yapıdır; ancak `mi`, `mı`, `mu`, `mü` token olarak tanımlanmamış ve ek olarak da işlenmiyor.

---

### 3.7 ❌ Emir / İstek Kipi (-sin / -sun / -sın / -sün) Desteği Yok

Türkçe'de emir kipinin doğal karşılığı fonksiyon çağrılarında kullanılabilir:

```
// Doğal Türkçe (şu an desteklenmiyor):
yazdırsın x        // emir kipi çağrı
durdurun döngü     // çoğul emir
```

---

### 3.8 ❌ `-den beri` / `-e kadar` Bileşik Edatları Yok

Döngü koşullarında çok doğal görünen bu yapılar hiç tanımlanmamıştır:

```
// Doğal Türkçe (ÇALIŞMIYOR):
i = 0'dan 10'a kadar { ... }   // for-range döngüsü
```

---

### 3.9 ⚠️ `olduğu sürece` İki Token Gerektirir

Türkçe'de `olduğu sürece` doğal bir bütündür ancak parser bunu **iki ayrı token** olarak (`Token::Oldugu` + `Token::Surece`) işler. Bu:
- Kaynak kodun okunabilirliğini düşürmez ama
- Araya boşluk olmadan yazılmaya (`olduğusürece`) çalışılırsa identifier olarak yorumlanır.
- Daha önemlisi, Türkçe'deki `olduğu` kelimesi farklı bağlamlarda kullanılabilir ancak her seferinde `sürece` ile birleşmesi zorunludur.

---

### 3.10 ❌ Dilek/Şart Kipi (-se / -sa) Yoktur

```
// Doğal Türkçe (ÇALIŞMIYOR):
x büyükse { ... }     // x > ... yerine
liste doluysa { ... }
```

`ise` anahtar kelimesi zaten vardır ama `büyükse`, `doluysa` gibi yapışık kipler tanımlanmamış.

---

### 3.11 ❌ Zarf-Fiil Eki (-arak / -erek) Desteği Yok

```
// Doğal Türkçe zincirleme (ÇALIŞMIYOR):
listeyi sıralayarak yazdır    // filter().sort() zinciri gibi
```

---

### 3.12 ⚠️ `liste'nin uzunluğu` vs `uzunluğu(liste)` Tutarsızlığı

```
// Doğal (çalışıyor):
i < liste'nin uzunluğu olduğu sürece { ... }

// Ama şu da çalışıyor (farklı sözdizim):
i < uzunluk(liste) olduğu sürece { ... }
```

`Token::Uzunlugu` hem postfix hem prefix olarak workaround'larla çalışır ama tutarsız bir API oluşturur.

---

## 4. README Tablosundaki Özellik-Kod Eşleşme Özeti

```
README Tablosu                           Durum
─────────────────────────────────────────────────────────────────
Belirtme (-i Hali)      ────────────→  ✅ Tam Destekleniyor
Yönelme (-e Hali)       ────────────→  ✅ Tam Destekleniyor
Bulunma (-de Hali)      ────────────→  ✅ Strip Edilir (semantik yok)
Ayrılma (-den Hali)     ────────────→  ✅ Strip Edilir (semantik yok)
İlgi/Tamlama (-nin)     ────────────→  ✅ Destekleniyor, ⚠️ 'in/'ın çakışma riski
Vasıta (-le Hali)       ────────────→  ⚠️ Strip Edilir AMA semantik yok
İyelik (-si Eki)        ────────────→  ⚠️ KISITLI: Yalnızca 'nin zincirinde çalışır
                                           'si tek başına → HATA üretir (BUG!)
Sıfat Yapan (-ki)       ────────────→  ⚠️ Strip Edilir AMA semantik yok
```

---

## 5. Hüma'da Olup README'de Görünmeyen (Gizli) Özellikler

| Özellik | Konum | Açıklama |
|---|---|---|
| `kendisi` self referansı | `token.rs`, `parser.rs`, `interpreter.rs` | OOP sınıf içi self erişimi |
| `dene/hata var ise` | `token.rs`, `parser.rs` | try-catch benzeri yapı |
| `değil` mantık operatörü | `token.rs`, `lexer.rs` | Unary NOT |
| Türkçe büyük/küçük harf | `interpreter.rs` | I/İ duyarlı dönüşüm |
| `doğru`/`yanlış` boolean | `token.rs` | Boolean literaller |
| `oku()` built-in | `interpreter.rs` | Kullanıcı girişi okuma |
| `tipi()` built-in | `interpreter.rs` | Tip sorgulama |
| Ardışık ek desteği | `lexer.rs` | `x'ten[0]'ı` gibi zincirleme |
| String'e ek uygulama | `lexer.rs` | `"metin"'i yazdır` |
| Sayıya ek uygulama | `lexer.rs` | `0'ı döndür` |
| Unicode-güvenli dilim | `interpreter.rs` | `dizi_dilim` char-bazlı |

---

## 6. Tam Türkçe Dilbilgisi Uyumu İçin Eklenmesi Gerekenler

### 6.1 KRİTİK: `'si / 'sı / 'su / 'sü` Ekleri `is_turkish_suffix()`'e Eklenmeli

**Dosya:** `src/lexer.rs`

```rust
// MEVCUT (HATALI):
fn is_turkish_suffix(s: &str) -> bool {
    matches!(s,
        "i" | "ı" | "u" | "ü" | "yi" | "yı" | "yu" | "yü" |
        "ni" | "nı" | "nu" | "nü" |
        ...
    )
}

// DÜZELTİLMİŞ:
fn is_turkish_suffix(s: &str) -> bool {
    matches!(s,
        "i" | "ı" | "u" | "ü" | "yi" | "yı" | "yu" | "yü" |
        "ni" | "nı" | "nu" | "nü" |
        "si" | "sı" | "su" | "sü" |          // ← EKLENMELİ
        "a" | "e" | "ya" | "ye" |
        "dan" | "den" | "tan" | "ten" |
        "da" | "de" | "ta" | "te" |
        "nin" | "nın" | "nun" | "nün" |       // ← Not: bunlar Nin token için
        "ın" | "in" | "un" | "ün" |           // ← Not: bunlar Nin token için
        "daki" | "deki" | "taki" | "teki" |
        "la" | "le" | "yla" | "yle" |
        "lar" | "ler" |                        // ← EKLENMELİ (çoğul)
        "ca" | "ce" | "ça" | "çe"             // ← EKLENMELİ (eşitlik hali)
    )
}
```

### 6.2 ÖNEMLİ: Çoğul Eki `-lar / -ler` Desteği

```rust
// src/lexer.rs, is_turkish_suffix() içine:
"lar" | "ler" |

// Kullanım örneği sonrasında çalışacak:
// sayılar'ı yazdır → sayılar (identifier) + belirtme
// meyveler'e ekle → meyveler (identifier) + yönelme
```

### 6.3 ÖNEMLİ: Eşitlik/Denklik Hali `-ca / -ce / -ça / -çe`

```
tamamen → tamamen'ce (tamamen olarak/gibi)
Türkçe'ce → Türkçe dili gibi
```

```rust
// src/lexer.rs, is_turkish_suffix() içine:
"ca" | "ce" | "ça" | "çe" |
```

### 6.4 ÖNEMLİ: For-Range Döngüsü `'den ... 'e kadar`

**Yeni Token:**
```rust
// src/token.rs
Kadar,          // kadar
```

**Lexer:**
```rust
// src/lexer.rs, read_identifier() içine:
"kadar" => Token::Kadar,
```

**Parser:**
```rust
// src/parser.rs — yeni parse metodu
// Sözdizim: i = 0'dan 10'a kadar { ... }
// Bu: i = 0; i <= 10 olduğu sürece { ...; i = i + 1 olsun } anlamına gelir
```

**AST:**
```rust
// src/ast.rs
AralıkDöngüsü {
    degisken: String,
    baslangic: Box<Ifade>,
    bitis: Box<Ifade>,
    govde: Vec<Komut>,
},
```

### 6.5 ORTa: Soru Eki `-mi / -mı / -mu / -mü` Token'ı

```rust
// src/token.rs
Mi,   // mi / mı / mu / mü — soru partikülü

// src/lexer.rs, read_identifier():
"mi" | "mı" | "mu" | "mü" => Token::Mi,

// Kullanım (parser'da):
// liste boş mu ise { ... }
// x > 5 mi ise { ... }
```

### 6.6 ORTA: Vasıta Hali `-le / -la` Semantik Desteği

Şu an `'yla`, `'le` ekleri anlamsızca drop edilmektedir. En azından **zincirleme çağrı** veya **method çağrısı** olarak yorumlanabilir:

```
// Şu an (ÇALIŞMIYOR anlamlı olarak):
hız'la çalıştır         // çalıştır(hız) olarak yorumlanabilirdi

// Öneri: Vasıta hali argüman olarak geçirir
sırala(liste, artan'la)   // artan → modifikatör olarak
```

**Implementasyon:** `Token::Vasıta` eklenip parser'da fonksiyon çağrısının son argümanı olarak yorumlanabilir.

### 6.7 ORTA: Sıfat Yapan `-ki` Anlamlı İşlenmesi

```
// Şu an (çalışmıyor anlamlı olarak):
dosya'daki veri yazdır   // 'daki drop ediliyor, dosya ve veri bağımsız

// Öneri: NesneErisim + string accessor olarak yorumlan
// dosya'daki → dosya["içindeki"] eşdeğeri
```

### 6.8 DÜŞÜK: Bağımsız İyelik Eki Kullanımı

Şu an `X'in Y'si` zinciri dışında `Y'si` formu hata üretir. `is_turkish_suffix()`'e `si/sı/su/sü` eklenmesi (§6.1) bu sorunu çözecektir — ancak semantik anlam kaybı devam eder.

### 6.9 DÜŞÜK: `-arak / -erek` Zarf-Fiil Eki (Method Chaining)

```
// Python'daki list.sort(), Java'daki .stream() gibi metod zinciri:
liste'yi_sıralayarak yazdır   // sırala(liste) → sonucu yazdır

// Sözdizim: ifade'yi işleyerek sonrakiEylem
```

---

## 7. README Tablosunun Güncellenme Önerileri

README tablosuna **başarıyla çalışan** ama eksik olan satırlar eklenmelidir:

| Eklenecek Satır | Varyasyonlar | Örnek | Açıklama |
|---|---|---|---|
| **Eşitlik/Denklik (-ce)** | `'ce`, `'ca`, `'çe`, `'ça` | `Türkçe'ce yaz` | "gibi / olarak" anlamı taşır |
| **Çoğul (-ler)** | `'ler`, `'lar` | `sayılar'ı döndür` | Çoğul yapılı isimlere ek uygulamak |

Ve aşağıdaki satırda **kısmi destek** notunu eklemek önerilir:

- **İyelik (-si Eki):** "Yalnızca `X'in Y'si` zinciri içinde çalışır; bağımsız kullanımda hata üretir." notu eklenmelidir.
- **Vasıta (-le Hali):** "Ek soyulur ancak anlamsal işlev üretmez." notu eklenmelidir.
- **Sıfat Yapan (-ki):** "Ek soyulur ancak anlamsal işlev üretmez." notu eklenmelidir.

---

## 8. Öncelik Sıralaması

| Öncelik | Sorun | Etki | Uygulama Zorluğu |
|---|---|---|---|
| 🔴 KRİTİK | `'si/'sı/'su/'sü` ekleri `is_turkish_suffix()`'e eklensin | Bug düzeltme | Çok Kolay (1 satır) |
| 🔴 KRİTİK | Vasıta hali README'de "anlam üretir" olarak belgelendi ama üretmyor; ya semantik eklensin ya da dokümantasyon düzeltilsin | Doğruluk | Belge: Kolay; Semantik: Orta |
| 🟠 ÖNEMLİ | `'lar` / `'ler` çoğul екі suffix listesine eklensin | Kullanılabilirlik | Çok Kolay |
| 🟠 ÖNEMLİ | `'ca` / `'ce` eşitlik hali suffix listesine eklensin | Kullanılabilirlik | Çok Kolay |
| 🟡 ORTA | For-range döngüsü: `0'dan 10'a kadar` | Kullanılabilirlik | Orta |
| 🟡 ORTA | Soru eki: `mi/mı/mu/mü` token | Dil tamamlığı | Orta |
| 🟢 İYİLEŞTİRME | Vasıta haline semantik anlam kazandır | Dil kalitesi | Zor |
| 🟢 İYİLEŞTİRME | `-deki` / `-daki` sıfat ekine semantik anlam kazandır | Dil kalitesi | Zor |
| 🟢 İYİLEŞTİRME | `-arak/-erek` ile method chaining | İleri özellik | Çok Zor |

---

## 9. Sonuç

Hüma, Türkçe eklemeli yapısını destekleme konusunda **temeli doğru atmıştır**: belirtme, yönelme, ayrılma ve ilgi halleri işlevsel düzeyde çalışmaktadır. Ancak:

1. **Bir kritik hata** vardır: `'si` iyelik eki bağımsız kullanımda hata verir.
2. **Üç ek sınıfı** (vasıta, sıfat yapan, iyelik) ek soyma ötesinde anlam **üretmemektedir** — bu README'nin iddia ettiği ile çelişmektedir.
3. **Üç önemli ek** (`-lar/-ler` çoğul, `-ca/-ce` eşitlik, `-mi/-mı` soru) hiç desteklenmemektedir.
4. Hüma aynı zamanda README'de belgelenmeyen **10+ önemli özelliğe** sahiptir (`kendisi`, `dene/hata var ise`, Türkçe boolean, `değil`, NLP fonksiyonları, vb.).

Dilbilgisi tablosunun güncellenmesi ve özellikle §6.1'deki tek satırlık kritik düzeltmenin yapılması, Hüma'nın dil bütünlüğünü önemli ölçüde artıracaktır.
