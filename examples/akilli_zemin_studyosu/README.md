# Akıllı Zemin Stüdyosu

Bu proje Hüma paket yöneticisiyle oluşturuldu. `yapay_zeka` ve `huma_sunucu`
bağımlılıkları paket yöneticisiyle çözülür ve `huma.lock` içinde özetlenir.

Uygulama 18 örnek üzerinde iki katmanlı bir sinir ağı eğitir. Tarayıcıdaki
GUI; test kapsamı, P95 gecikme ve hata oranını Hüma HTTP API’sine gönderir.
Görünen skor, eğitilmiş modelin gerçek ileri geçiş sonucudur.

Proje kökünden kullanım:

```bash
# Yerel monorepo paketlerini açık güven onayıyla kilitle/kur
../../target/release/huma paket kur --güvenilir

# Paketi ve bağımlılık özetlerini doğrula
../../target/release/huma paket doğrula

# Yalnız paket betiği üzerinden çalıştır
../../target/release/huma paket run baslat
```

Arayüz: <http://127.0.0.1:8787>

Test:

```bash
../../target/release/huma paket run test
```

İzinler paket betiğinde açıkça sınırlıdır: yalnız `dosya-okuma` ve
`ağ-sunucu`. Uygulama dış ağa bağlanmaz; model ve GUI yereldir.
