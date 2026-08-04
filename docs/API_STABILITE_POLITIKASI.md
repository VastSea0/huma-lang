# Hüma API ve Uyumluluk Politikası

## Kararlılık düzeyleri

- **Normatif:** Türkçe kaynak grameri, yorumlayıcı semantiği, yapılandırılmış
  tanı zarfı ve HMI v1 kuralları.
- **Sürümlü biçim:** `.hbc` v4 ve paket/imza şemaları. Bilinmeyen sürüm açıkça
  reddedilir; biçimler arasında tahmine dayalı okuma yapılmaz.
- **Deneysel:** bytecode VM destek alt kümesi, Cranelift AOT, Rust crate API'leri
  ve depodaki alan kütüphaneleri.

Yorumlayıcı tek normatif backend'dir. `huma-compiler::backend` destek tablosu bu
durumu makinece okunabilir sabit sözleşme olarak yayımlar. VM ve AOT çağrıları
deneysel olduklarını CLI'da bildirir; desteklenmeyen yapıda sessizce yorumlayıcıya
düşmez.

## SemVer

HMI fonksiyon imzası, etki veya hata kataloğundaki kırıcı değişiklik paket ana
sürümünü artırmalıdır. Otomatik denetim [HMI](HMI.md) belgesindeki kuralları
uygular. 1.0 öncesi Rust crate yüzeyi geriye uyumluluk garantisi taşımaz; buna
karşılık sürümlü dosya/protokol okuyucuları uyumsuz girdiyi her zaman açık hata
ile reddeder.

## Hata ABI'si

Makine tüketimine açık tanılar `DiagnosticEnvelope` şema sürümü, kararlı kod,
kategori, Türkçe ileti, isteğe bağlı kaynak konumu, çağrı izi, güvenli metinsel
ayrıntılar ve yeniden dallanmayan neden zinciri taşır. Yeni isteğe bağlı alanlar
eski v1 zarflarını bozmadan eklenir. Yeni hata kodları eklenebilir; mevcut kodun
anlamı farklı bir hataya taşınamaz. HMI uzak hataları kendi kararlı kod
kataloğuna sahiptir.

## Değişiklik kabulü

Kararlı bir yüzeyi etkileyen değişiklikte ilgili biçim/protokol sürümü,
uyumluluk testi, Türkçe belge ve değişim günlüğü birlikte güncellenir. Performans
ve güvenlik sınırını gevşeten değişiklik ayrıca ölçüm ve gerekçe gerektirir.
