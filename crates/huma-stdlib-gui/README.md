# Karantinadaki eski GUI adaptörü

Bu crate deneysel aynı-süreç GUI prototipidir ve Hüma'nın varsayılan çalışma
alanı, CLI dağıtımı ve güvenlik kabul kapısının parçası değildir.

Bağımlılık zinciri `winit -> sctk-adwaita -> ab_glyph -> ttf-parser` üzerinden
[RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192) ile bakımı
bırakıldığı bildirilen `ttf-parser` paketini içerdiği için karantinaya alınmıştır.
Bu bağımlılık aktif bakımlı bir font ayrıştırıcısına taşınmadan ve adaptör HMI
gibi süreç dışı bir sınıra geçirilmeden genel dağıtımda etkinleştirilmemelidir.
