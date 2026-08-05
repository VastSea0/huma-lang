# Hüma Native GUI adaptörü

Bu crate, `--izin gui` verildiğinde CLI tarafından açılan gerçek yerel pencere
adaptörüdür. Hüma betikleri Dear ImGui tabanlı pencere, düğme, metin ve slider
bileşenlerini aynı süreçte kullanabilir.

Adaptör yalnız GUI capability'si açıkken yerleşikleri kaydeder; varsayılan
çalıştırma hâlâ GUI açmaz. Native pencere yaşam döngüsü `huma run` içindeki
GUI isteği tamamlanana kadar CLI tarafından yönetilir.
