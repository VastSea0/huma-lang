yükle "lib/nlp.hb"

"\n╔══════════════════════════════════════════════════════╗"'i yazdır
"║       HÜMA NLP v3.0.0 — TAM TEST SÜİTİ              ║"'i yazdır
"╚══════════════════════════════════════════════════════╝"'i yazdır

haber_metni  = "Prof. Dr. Ayşe Kaya, İstanbul Üniversitesi'nde yapay zeka üzerine önemli bir konferans verdi. Konferansa Ankara ve İzmir'den 250 katılımcı geldi. Bilim insanları yeni algoritmalar geliştiriyor ve Türkiye bu alanda hızla ilerliyor." olsun
yorum_poz    = "Bu ürün gerçekten harika! Çok memnun kaldım, kesinlikle herkese tavsiye ederim." olsun
yorum_neg    = "Berbat bir deneyimdi. Çok kötü hizmet aldım, hiç beğenmedim ve bir daha gelmeyeceğim." olsun
yorum_nötr   = "Ürünü aldım ve kullandım. Fatura kesildi." olsun

// ─── TEST 1: TEMİZLEME VE TOKENİZASYON ──────────────────────────────────────
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır
"📌 TEST 1 — Temizleme ve Tokenizasyon"'u yazdır
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır

"» Girdi:"'i yazdır
haber_metni'ni yazdır
temiz = nlp_temizle(haber_metni) olsun
"» Temizlenmiş:"'i yazdır
temiz'i yazdır
tokenler = tokenize(temiz) olsun
"» Token sayısı: " + tokenler'in uzunluğu'nu yazdır
"» Tokenler:"'i yazdır
tokenler'i yazdır

// ─── TEST 2: DURAK KELİME FİLTRELEME ────────────────────────────────────────
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır
"📌 TEST 2 — Durak Kelime Filtreleme"'yi yazdır
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır

kalan = durak_kelime_filtrele(tokenler) olsun
"» Öncesi : " + tokenler'in uzunluğu + " token"'i yazdır
"» Sonrası: " + kalan'ın uzunluğu + " token"'i yazdır
"» Kalan  :"'i yazdır
kalan'ı yazdır

// ─── TEST 3: KÖKLEŞTIRME ─────────────────────────────────────────────────────
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır
"📌 TEST 3 — Kökleştirme (Stemming)"'yi yazdır
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır

test_k = ["geliyor", "gidiyorum", "çalışıyorlar", "koşarak", "üniversitesi", "algoritmaları", "geliştirilmiştir", "seviyorum"] olsun
"» Tek tek kök bulma:"'i yazdır
i = 0 olsun
i < test_k'nin uzunluğu olduğu sürece {
    k = test_k[i] olsun
    "  " + k + "  →  " + stem(k)'i yazdır
    i = i + 1 olsun
}
kökler = toplu_stem(kalan) olsun
"» Haber metninden kökler:"'i yazdır
kökler'i yazdır

// ─── TEST 4: FREKANS ANALİZİ VE SIRALAMA ─────────────────────────────────────
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır
"📌 TEST 4 — Frekans Analizi ve Sıralama"'yı yazdır
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır

frekanslar = kelime_frekansları(kökler) olsun
"» Ham frekans:"'i yazdır
frekanslar'ı yazdır
sıralı = frekans_sırala(frekanslar) olsun
"» Sıralı:"'i yazdır
sıralı'yı yazdır
en3 = en_sık_n(frekanslar, 3) olsun
"» En sık 3:"'i yazdır
en3'ü yazdır

// ─── TEST 5: METİN İSTATİSTİĞİ ────────────────────────────────────────────────
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır
"📌 TEST 5 — Metin İstatistiği"'ni yazdır
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır
metin_istatistik(haber_metni)

// ─── TEST 6: CÜMLE BÖLME ─────────────────────────────────────────────────────
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır
"📌 TEST 6 — Cümle Bölme (Kısaltma Destekli)"'yi yazdır
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır

cümleler = cümle_böl(haber_metni) olsun
"» Bulunan cümle sayısı: " + cümleler'in uzunluğu'nu yazdır
j = 0 olsun
j < cümleler'in uzunluğu olduğu sürece {
    "  [" + j + "] " + cümleler[j]'i yazdır
    j = j + 1 olsun
}

soru_metni = "Bugün hava nasıl? Yarın yağmur yağacak mı! Umarım güneşli olur." olsun
"» Soru/ünlem testi:"'i yazdır
sc = cümle_böl(soru_metni) olsun
k = 0 olsun
k < sc'nin uzunluğu olduğu sürece {
    "  [" + k + "] " + sc[k]'i yazdır
    k = k + 1 olsun
}

// ─── TEST 7: POS ETİKETLEME ───────────────────────────────────────────────────
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır
"📌 TEST 7 — POS Etiketleme"'yi yazdır
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır

pos_metni = "Hızlı kırmızı araba yolda koşarak gidiyor ama duramıyor" olsun
"» Girdi: " + pos_metni'ni yazdır
pos_t = nlp_tokenize(pos_metni) olsun
et    = pos_etiketle(pos_t) olsun
pos_yazdır(et)

"» Haber metninde POS:"'i yazdır
ht = nlp_tokenize(haber_metni) olsun
he = pos_etiketle(ht) olsun
pos_yazdır(he)

// ─── TEST 8: NER ──────────────────────────────────────────────────────────────
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır
"📌 TEST 8 — Varlık İsmi Tanıma (NER)"'yi yazdır
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır

"» Haber metni NER:"'i yazdır
ner1 = ner_etiketle(tokenize(haber_metni)) olsun
ner_yazdır(ner1)

ner_metni2 = "Mehmet ve Zeynep Mart ayında Ankara da bir şirket kurdu" olsun
"» Test 2: " + ner_metni2'yi yazdır
ner2 = ner_etiketle(tokenize(ner_metni2)) olsun
ner_yazdır(ner2)

// ─── TEST 9: DUYGU ANALİZİ ────────────────────────────────────────────────────
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır
"📌 TEST 9 — Duygu Analizi"'ni yazdır
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır

"» Pozitif: " + yorum_poz'u yazdır
duygu_yazdır(yorum_poz)

"» Negatif: " + yorum_neg'i yazdır
duygu_yazdır(yorum_neg)

"» Nötr: " + yorum_nötr'ü yazdır
duygu_yazdır(yorum_nötr)

güçl_test = "Bu film son derece güzeldi ve çok mutlu oldum" olsun
"» Güçlendirici: " + güçl_test'i yazdır
duygu_yazdır(güçl_test)

// ─── TEST 10: BELGE SKORLAMA (TF tabanlı) ─────────────────────────────────────
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır
"📌 TEST 10 — Belge Skorlama (TF tabanlı)"'yi yazdır
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır

b1 = "Yapay zeka teknolojisi hızla gelişiyor ve yeni algoritmalar üretiliyor" olsun
b2 = "Türkiye yapay zeka alanında önemli yatırımlar yapıyor" olsun
b3 = "Algoritmalar ve veri bilimi modern teknolojinin temeli" olsun
belgeler = [b1, b2, b3] olsun
tfidf = tfidf_koleksiyon(belgeler) olsun
"» Belge 1 skorları:"'i yazdır
tfidf[0]'ı yazdır
"» Belge 2 skorları:"'i yazdır
tfidf[1]'i yazdır
"» Belge 3 skorları:"'i yazdır
tfidf[2]'yi yazdır

// ─── TEST 11: METİN BENZERLİĞİ ────────────────────────────────────────────────
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır
"📌 TEST 11 — Metin Benzerliği (ortak kelime sayısı)"'i yazdır
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır

m1 = "Yapay zeka gelecekte çok önemli olacak" olsun
m2 = "Yapay zeka teknolojisi gelecekte kritik rol oynayacak" olsun
m3 = "Bugün hava çok güzeldi ve piknik yaptık" olsun

metin_benzerlik_yazdır("(1,2) benzer konu  ", m1, m2)
metin_benzerlik_yazdır("(1,3) farklı konu  ", m1, m3)
metin_benzerlik_yazdır("(1,1) aynı metin   ", m1, m1)

// ─── TEST 12: N-GRAM ──────────────────────────────────────────────────────────
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır
"📌 TEST 12 — N-Gram Üretimi"'ni yazdır
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır

ng_metni  = "Türk dili zengin ve güzel bir dildir" olsun
ng_tokens = nlp_tokenize(ng_metni) olsun
"» Metin: " + ng_metni'ni yazdır
"» Bigramlar:"'i yazdır
bigram(ng_tokens)'ı yazdır
"» Trigramlar:"'i yazdır
trigram(ng_tokens)'ı yazdır
"» 4-gramlar:"'i yazdır
ngram(ng_tokens, 4)'ü yazdır

// ─── TEST 13: KARAKTER NORMALİZASYONU ────────────────────────────────────────
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır
"📌 TEST 13 — Karakter Normalizasyonu (ASCII)"'ni yazdır
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır

tr_metin = "Şişlideki güzel çiçekler yağmurda büyüdü" olsun
"» Özgün : " + tr_metin'i yazdır
"» ASCII  : " + ascii_normalize(tr_metin)'i yazdır

// ─── TEST 14: ÜNLÜ UYUMU YARDIMCILARI ────────────────────────────────────────
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır
"📌 TEST 14 — Ünlü Uyumu Yardımcıları"'nı yazdır
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır

ü_test = ["okul", "güneş", "kitap", "ülke", "araba", "yüzük"] olsun
i = 0 olsun
i < ü_test'in uzunluğu olduğu sürece {
    kw = ü_test[i] olsun
    "  " + kw + "  →  ünlü: " + kelime_ünlü_sayısı(kw) + "  son: " + son_ünlü(kw) + "  uyum: " + ünlü_uyumu_türü(kw)'u yazdır
    i = i + 1 olsun
}

// ─── TEST 15: TAM PIPELINE ────────────────────────────────────────────────────
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır
"📌 TEST 15 — Tam NLP Pipeline"'u yazdır
"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'i yazdır

pl_metni = "Zeynep İstanbul da yeni bir yazılım şirketi kurdu. Takımı çok başarılı çalışmalar yapıyor ve müşteriler son derece memnun." olsun
"» Girdi: " + pl_metni'ni yazdır
sonuç = nlp_pipeline(pl_metni) olsun
"» Nihai frekans:"'i yazdır
sonuç'u yazdır

// ─── ÖZET ─────────────────────────────────────────────────────────────────────
"╔══════════════════════════════════════════════════════╗"'i yazdır
"║  ✅  Tüm testler başarıyla tamamlandı!               ║"'i yazdır
"╚══════════════════════════════════════════════════════╝"'i yazdır
