# Hüma belge sitesi

Bu dizin Hüma'nın Türkçe ve İngilizce web belgelerini içerir. Yetenek
iddiaları kök dizindeki `README.md`, `docs/DIL_TANIMI.md` ve
`docs/DURUM_VE_YOL_HARITASI.md` ile tutarlı olmalıdır.

## Yerel doğrulama

```bash
npm ci
npm run lint
npm run build
```

Site bir tarayıcı yorumlayıcısı içermez. Oyun alanı yalnızca sabit, açıklayıcı
örnekler gösterir. Hüma programlarının gerçek davranışı CLI ve Rust testleriyle
doğrulanır.
