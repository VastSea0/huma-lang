"yapay_zeka_temel.hb"'yi yükle
"nlp_ileri/gomme.hb"'yi yükle


sozluk_b = 1000 olsun
gomme_b = 32 olsun

gomme = gomme_tabakasi() olsun
gomme.ilklendir(sozluk_b, gomme_b) olsun

kelime_vektoru = gomme.token_al(5)
kelime_vektoru'nu yazdır

"  "'yi yazdır
cumle_tokenleri = [10, 20, 15, 25]
cumle_vektoru = gomme.ortalama_havuzla(cumle_tokenleri)
cumle_vektoru'nu yazdır

en_yakinlar = gomme.en_benzer_tokenler(5, 3)
"En benzer 3 kelime: "'ı yazdır
en_yakinlar'ı yazdır

gomme.kaydet("kelime_vektorleri.json")