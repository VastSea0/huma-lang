# Hüma web sitesi

Hüma'nın Türkçe/İngilizce tanıtım, belgeleme ve örnek sayfaları. Next.js
uygulamasıdır; dil `app/[lang]/` yol parçasıyla seçilir (`/tr`, `/en`).

## Yapı

- `app/[lang]/page.tsx` — ana sayfa
- `app/[lang]/belgeleme/` — dil başvurusu; normatif kaynak
  [`docs/DIL_TANIMI.md`](../../docs/DIL_TANIMI.md) belgesidir, çelişkide o esastır
- `app/[lang]/ornekler/` — örnek programlar
- `app/components/` — ortak bileşenler
- `app/i18n/` — dil sözlükleri

## Komutlar

```bash
npm ci
npm run dev     # geliştirme sunucusu
npm run lint    # ESLint
npm run build   # üretim derlemesi
```

`www/site` altında değişiklik yapan her commit `npm run lint` ve
`npm run build` kapılarından geçmelidir.
