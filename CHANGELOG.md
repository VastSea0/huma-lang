# Değişim Günlüğü

## 0.6.0

### Dil çekirdeği

- Ayrıştırıcı tanıları standart hata nesnelerine taşındı; beklenmeyen ifadeler ve geçersiz atama hedefleri artık sessizce `Boş` üretmiyor.
- Bilimsel sayı gösterimi eklendi; sonlu olmayan literal değerler reddediliyor.
- `devam` ve `kır` döngü komutları eklendi.
- `ve` ve `veya` kısa devreli çalışacak şekilde düzeltildi.
- Liste/sözlük öğesi ataması, sınır ve tip hatalarıyla güvenli hale getirildi.
- Tanımsız değişken, çağrılamayan değer, sıfıra bölme, geçersiz indeks ve modül hataları yapısal çalışma zamanı hatalarına dönüştürüldü.
- `dene/yakala`, yorumlayıcı hatalarını yakalayıp programa devam edebiliyor.

### VM ve AOT

- VM’ye kalan, büyük-eşit, mantıksal işlemler, değil ve uzunluk işlemleri eklendi.
- VM fonksiyon kapsamı ve özyinelemeli Fibonacci sonucu düzeltildi.
- Bytecode derleyici desteklemediği komut ve atama hedeflerini açıkça reddediyor.
- AOT derleyici desteklenmeyen AST için sahte `0` çıktısı üretmek yerine derleme hatası veriyor.

### AI/NLP

- Yoğun katman eğitiminde kullanılan Adam matris/vektör durumları ve güncellemeleri çalışır hale getirildi.
- Bilimsel epsilon literal desteği ve ayrılmış sözcük çakışmaları düzeltildi.
- Sahte güncelleme yapan eski `optimizor.hb` ile yinelenen `katman.hb` kaldırıldı.
- `yapay_zeka.hb`, çalışan `sinir_agi` API’sinin tek giriş noktası oldu.
- TF-IDF → yoğun ağ → geri yayılım → çıkarım örneği uçtan uca doğrulandı.
- Bayt düzeyli BPE, Türkçe UTF-8 metin ve boşlukları kayıpsız kodlayıp geri çözecek şekilde yeniden yazıldı ve sınır kontrolleri eklendi.

### Kalite ve temizlik

- İzlenen bütün `.hb` kaynaklarını ayrıştıran kaynak havuzu testi eklendi.
- Rust testleri çalışma zamanı hatalarını gerçekten başarısızlık sayacak şekilde sertleştirildi.
- Hüma test çerçevesinin sayaç ve açık dönüş davranışı düzeltildi.
- `cargo clippy --workspace --all-targets -- -D warnings` sıfır uyarıya indirildi.
- LSP konum dönüşümü UTF-16/UTF-8 ve emoji öncesi Türkçe tanımlayıcılar için düzeltildi.
- Dosya ve SQLite hataları sessiz boş/başarı değerleri yerine yakalanabilir çalışma zamanı hatalarına dönüştürüldü.
- Güvenli olmayan imzasız CLI güncellemesi ile eksik/kimliği doğrulanmayan uzak paket kurulumu kaldırıldı; yerel paket özeti bütün dosyaları kapsıyor.
- Web sitesi lint ve üretim derlemesi CI kabul kapısına eklendi.
- Eski debug betikleri, yedek kaynak, üretilmiş günlükler, bozuk testler ve geçersiz yayın iş akışları kaldırıldı.
- Belgeler ölçülmemiş performans ve eksiksiz Türkçe/native destek iddialarından arındırıldı.
