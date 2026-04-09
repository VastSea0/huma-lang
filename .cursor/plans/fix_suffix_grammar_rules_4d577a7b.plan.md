---
name: Fix suffix grammar rules
overview: Hüma’da nesne/özellik erişimini daha doğru Türkçe gramerle ifade etmek için `X'in Y'si` biçimini dil kuralı haline getirip derleyicide/lexer-parser’da uygulamak; ardından tüm test ve doküman örneklerini yeni kurala göre güncellemek.
todos:
  - id: inspect-duplicates
    content: "`crates/huma-core` ve kök `src/` altındaki lexer/parser/token kopyalarının hangisinin gerçek build’de kullanıldığını doğrula; ikisini de güncelleme ihtiyacını netleştir."
    status: completed
  - id: add-iyelik-token
    content: "`Token::Iyelik` ekle ve lexer’da `'sı/'si/'su/'sü` gördüğünde bu token’ı üret."
    status: completed
  - id: enforce-iyelik-after-nin
    content: Parser’da `Nin` sonrası property identifier’dan sonra `Iyelik` zorunlu kuralını uygula; hata mesajını anlaşılır yap.
    status: completed
  - id: update-highlighter
    content: VSCode TextMate grammar regex’ine iyelik eklerini ekle.
    status: completed
  - id: update-docs-tests
    content: README + web docs + test dosyalarında eski `X'in Y` örneklerini `X'in Y'si` biçimine çevir.
    status: completed
  - id: validate
    content: Test/lint/build ile doğrula; en az bir pozitif ve bir negatif örneği koş.
    status: completed
isProject: false
---

## Kapsam ve mevcut durum
- Hüma lexer’ı `'` sonrası ekleri bir whitelist ile **strip** ediyor; yalnızca ilgi eki grubu (`'nin/'ın/...`) `Token::Nin` üreterek parser’da **özellik erişimi** semantiği oluşturuyor.
- Şu an **iyelik eki** (`'sı/'si/'su/'sü`) lexer whitelist’inde yok; bu yüzden dokümandaki `tema'sı` gibi örnekler derleyicide **"Bilinmeyen ek"** hatasına düşüyor.
- Hedef: Özellik erişiminde ana/önerilen kuralı **zorunlu** hale getirmek: `yazdır ayarlar'ın tema'sı;`.

## Tasarım kararı
- `X'in Y` (iyeliksiz) kullanımını **kıran** bir değişiklik olarak kabul edip, `X'in Y'si` biçimini yeni kural yapacağız.
- İyelik eki, artık tamamen “yutulan” bir ek değil; parser’ın doğruladığı bir yapı olacak.

## Uygulama yaklaşımı (derleyici)
- Lexer
  - `crates/huma-core/src/lexer.rs` içinde apostrof sonrası okunan ekler arasına iyelik eklerini ekle: `sı|si|su|sü`.
  - Bu ekler görüldüğünde **yutmak yerine** yeni bir token üret: örn. `Token::Iyelik`.
  - Aynı değişikliği kopya implementasyonda da uygula: `src/lexer.rs`.
  - (Opsiyonel ama pratik) Unicode apostrof `’` desteğini de ekle (kopyala-yapıştır kaynaklı hataları azaltır).
- Token
  - `crates/huma-core/src/token.rs` ve `src/token.rs` içine `Iyelik` token’ını ekle.
- Parser
  - `crates/huma-core/src/parser.rs` ve `src/parser.rs` içinde `Token::Nin` postfix erişiminde:
    - `Nin` sonrası `Tanimlayici(<ozellik>)` okunduktan sonra **hemen** `Token::Iyelik` bekle.
    - Yoksa parse hatası üret (mevcut altyapı nasıl hata veriyorsa ona uygun).

## Uygulama yaklaşımı (editör / syntax highlight)
- `vscode-huma/syntaxes/huma.tmLanguage.json` regex’ine `si|sı|su|sü` ekle.

## Doküman ve örnekleri güncelleme
- Web dokümantasyonu örnekleri:
  - `www/site/app/[locale]/docs/grammar/page.tsx`
  - `www/site/app/[locale]/docs/lists-errors/page.tsx`
  - ve sözlük örnekleri: `www/site/dictionaries/tr.json`, `www/site/dictionaries/en.json` (genitive örnekleri)
- README örnekleri:
  - `README.md`, `README.en.md` (varsa `ayarlar'ın tema` geçen yerleri `ayarlar'ın tema'sı` yap)
- Testler:
  - `tests/test_turkce.hb` başta olmak üzere, repo genelinde `... 'ın <prop>` biçimindeki örnekleri `... 'ın <prop>'<iyelik>` biçimine çevir.

## Doğrulama
- Derleyici testlerini çalıştır (mevcut test komutlarına göre).
- En azından şu iki negatif/pozitif senaryoyu doğrula:
  - Pozitif: `yazdır ayarlar'ın tema'sı;`
  - Negatif: `yazdır ayarlar'ın tema;` artık hata vermeli.

## Riskler / notlar
- Bu değişiklik **kırıcı**: eski sözdizimi çalışmayacak; tüm örnekleri ve testleri güncellemek şart.
- Lexer’ın suffix-whitelist’i ve editor regex’i arasında uyumsuzluk oluşmaması için tek tek aynı listeyi güncelleyeceğiz.