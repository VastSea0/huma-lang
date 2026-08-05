# Akıllı Zemin Stüdyosu

Bu proje Hüma paket yöneticisiyle oluşturuldu. `yapay_zeka` ve `gui`
bağımlılıkları paket yöneticisiyle çözülür ve `huma.lock` içinde özetlenir.

Uygulama 18 örnek üzerinde iki katmanlı bir sinir ağı eğitir. GUI gerçek bir
native penceredir; slider'lar doğrudan eğitilmiş Hüma modeline girdi verir.

Proje kökünden kullanım:

```bash
# Yerel monorepo paketlerini açık güven onayıyla kilitle/kur
../../target/release/huma paket kur --güvenilir

# Paketi ve bağımlılık özetlerini doğrula
../../target/release/huma paket doğrula

# Yalnız paket betiği üzerinden çalıştır
../../target/release/huma paket run baslat
```

Arayüz: yerel native pencere (Dear ImGui)

Test:

```bash
../../target/release/huma paket run test
```

İzinler paket betiğinde açıkça `gui` ile sınırlıdır. Uygulama ağ sunucusu
kurmaz; model ve pencere yereldir.
