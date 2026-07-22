# Hüma Sistem Kütüphaneleri

Hüma ile birlikte gelen standart kütüphanelerin detaylı kullanım rehberi.

> **Not:** Tüm kütüphaneler `yükle "kütüphane_adı.hb"` komutuyla yüklenir.
> Harici modüller ise `yükle "modül_adı"` ile `huma_modulleri/` dizininden yüklenir.

---

## Gömülü Kütüphaneler (`lib/`)

### 1. Matematik (`matematik.hb`)

Temel matematiksel sabitler ve fonksiyonlar içerir.

- **Sabitler:** `PI`, `E`
- **`karesi(n)`**, **`küpü(n)`**: n sayısının karesini/küpünü alır.
- **`mutlak(n)`**: Sayının mutlak değerini döner.
- **`kuvvet(a, b)`**: a^b hesaplar.
- **`yuvarla(n)`**: En yakın tam sayıya yuvarlar.
- **`faktöriyel(n)`**: n! hesaplar.
- *Rust Built-in:* `karekök(n)` bu dosyada tanımlı değildir, interpreter tarafından sağlanır.

### 2. Renkler (`renkler.hb`)

Terminal çıktılarını renklendirmek için kullanılır.

- **Sabitler:** `KIRMIZI`, `YEŞİL`, `SARI`, `MAVI`, `TURKUAZ`, `KALIN`, `SIFIR`
  - *Not:* Eski `YESIL` adı geriye dönük uyumluluk için alias olarak korunmuştur.
- **`renkli_yaz(metin, renk)`**: Belirtilen renkte metin yazdırır.
- **`başarı_yaz(metin)`**, **`hata_yaz(metin)`**, **`uyarı_yaz(metin)`**: Renkli etiketli çıktılar.

### 3. Zaman (`zaman.hb`)

- **`beklet(saniye)`**: Programı durdurur.
- **`kronometre_başlat()`**, **`kronometre_bitir(başlangıç)`**: Süre ölçümü.

### 4. Liste Araçları (`liste.hb`)

- **`yazdır_liste(liste)`**: Listeyi güzel formatta yazar.
- **`içeriyor_mu(liste, eleman)`**: Varlık kontrolü.
- **`ters_cevir(liste)`**: Listeyi tersine döndürür.
- **`eşle(liste, f)`**: Her elemana f fonksiyonunu uygular (Map).
- **`filtrele(liste, f)`**: f koşuluna uyanları seçer (Filter).
- **`indirge(liste, f, başlangıç)`**: Liste elemanlarını tek bir değere indirger (Reduce).
- **`dilimle(liste, baş, son)`**: Alt liste alır.
- *Rust Built-in:* `uzunluk()`, `listeye_ekle()`, `içeriyor()` interpreter tarafından sağlanır.

### 5. Dizgi (`dizgi.hb`)

- **`büyük_mü(karakter)`**, **`küçük_mü(karakter)`**, **`boşluk_mu(karakter)`**: Karakter kontrolleri.
- **`başıyla_mı_başlıyor(dizgi, ön_ek)`** → `başlıyor_mu()` alias'ıdır.
- **`sonuyla_mı_bitiyor(dizgi, son_ek)`** → `bitiyor_mu()` alias'ıdır.
- *Kaldırılan fonksiyonlar (v1.1.0):* `kırp()` ve `içeriyor_mu()` Rust built-in ile çakıştığı için kaldırılmıştır. Yerlerine `kırp()` ve `içeriyor()` built-in'lerini kullanın.

### 6. Rastgele (`rastgele.hb`)

- **`r_tamsayı(min, max)`**: Aralıklı rastgele tam sayı.
- **`r_seç(liste)`**: Listeden rastgele eleman seçer.
- **`r_karıştır(liste)`**: Listeyi rastgele karıştırır.

### 7. Dosya (`dosya.hb`)

- **`güvenli_oku(yol)`**: Hata vermeden dosya okumaya çalışır.
- **`satırlara_ayır(metin)`**: Metni satır listesine çevirir.
- *Not:* `dosya_var_mı()` artık Rust built-in olarak sağlanır, lib versiyonu kaldırılmıştır (v1.1.0).

### 8. İstatistik (`istatistik.hb`)

- **`ortalama(liste)`**, **`en_büyük(liste)`**, **`en_küçük(liste)`**
- **`varyans(liste)`**, **`standart_sapma(liste)`**
- *Bağımlılıklar:* `matematik.hb` (`karesi()`), Rust built-in `karekök()`.

### 9. Birim Test (`birim_test.hb`)

- **`test_et(ad, f)`**: Test çalıştırır.
- **`iddia_et(beklenen, gelen, mesaj)`**: Eşitlik kontrolü (assertion).
- **`test_raporu()`**: Sonuç özetini yazar.

### 10. NLP — Türkçe Doğal Dil İşleme (`nlp.hb`)

26KB'lık kapsamlı Türkçe NLP kütüphanesi. Tokenizasyon, stemming, POS etiketleme, NER, duygu analizi ve metin benzerliği.

---

## Harici Modüller (`huma_modulleri/`)

Bu modüller `huma kur <paket_adı>` ile kurulabilir veya `yükle "<paket_adı>"` ile kullanılabilir.

### huma_sunucu
HTTP sunucu kütüphanesi. GET/POST route tanımlama, JSON/HTML yanıt.
- **GitHub:** [VastSea0/huma-lang](https://github.com/VastSea0/huma-lang)
- **Sürüm:** 1.0.0

### huma_sqlite
SQLite veritabanı desteği. Sınıf tabanlı sorgulama API'si.
- **GitHub:** [VastSea0/huma-lang](https://github.com/VastSea0/huma-lang)
- **Sürüm:** 1.0.0

### ag_istekleri
HTTP istek kütüphanesi. GET, POST, PUT, DELETE desteği.
- **GitHub:** [VastSea0/ag_istekleri](https://github.com/VastSea0/ag_istekleri)
- **Sürüm:** 1.1.0

### nlp_temel
Türkçe NLP modülü. `nlp.hb` kütüphanesini yükleyen wrapper.
- **GitHub:** [VastSea0/huma-lang](https://github.com/VastSea0/huma-lang)
- **Sürüm:** 3.1.0

### gui
Native GUI kütüphanesi. egui tabanlı masaüstü arayüz araçları.
- **GitHub:** [VastSea0/huma-lang](https://github.com/VastSea0/huma-lang)
- **Sürüm:** 0.4.0
- *Not:* Yalnızca GUI modunda (Tauri/egui) çalışır.

---

## Rust Built-in Fonksiyonlar

Bu fonksiyonlar interpreter tarafından otomatik olarak sağlanır ve her zaman mevcuttur:

| Fonksiyon | Açıklama |
|-----------|----------|
| `uzunluk(x)` | Metin/liste uzunluğu |
| `oku()` | Kullanıcıdan girdi al |
| `uyut(ms)` | Milisaniye bekle |
| `zaman()` | Epoch timestamp |
| `listeye_ekle(l, e)` | Listeye eleman ekle |
| `karekök(n)` | Kare kök |
| `rastgele()` | 0-1 arası rastgele sayı |
| `dosya_oku(yol)` | Dosya oku |
| `dosya_yaz(yol, i)` | Dosya yaz |
| `dosya_var_mı(yol)` | Dosya varlık kontrolü |
| `tipi(x)` | Değer tipini döndür |
| `küçük_harf(m)` | Küçük harfe çevir |
| `büyük_harf(m)` | Büyük harfe çevir |
| `böl(m, a)` | Metni parçala |
| `birleştir(l, a)` | Listeyi birleştir |
| `değiştir(m, a, b)` | Metin değiştir |
| `kırp(m)` | Boşluk kırp |
| `içeriyor(k, a)` | İçerik kontrolü |
| `başlıyor_mu(m, ö)` | Önek kontrolü |
| `bitiyor_mu(m, s)` | Sonek kontrolü |
| `dizi_dilim(m, b, s)` | Alt dizgi al |
| `sayıya_çevir(m)` | Metni sayıya çevir |
| `metne_çevir(n)` | Sayıyı metne çevir |
| `nesneden_metine(n)` | JSON serialize |
| `metinden_nesneye(m)` | JSON deserialize |
