// ══════════════════════════════════════════════════════════════════════════════
// examples / huma_gpt / ana.hb
// Hüma GPT Otoregresif Türkçe Dil Modeli Giriş Noktası
// ══════════════════════════════════════════════════════════════════════════════

"tokenizer.hb"'yi yükle
"veri_hazirlayici.hb"'yi yükle
"gpt_model.hb"'yi yükle
"uretim.hb"'yi yükle
"cumle_ureteci.hb"'yi yükle

"╔══════════════════════════════════════════════════════════════════════╗"'u yazdır
"║           🤖 HÜMA GPT — TÜRKÇE OTOREGRESİF DİL MODELİ                ║"'u yazdır
"║   Tor-Qurtu Veri Seti Üzerinde Eğitilen Yapay Sinir Ağı Modeli       ║"'u yazdır
"╚══════════════════════════════════════════════════════════════════════╝"'u yazdır
""'ı yazdır

corpus_yolu = "../tor-qurtu/cikti/turkce_llm_corpus.txt"
sozluk_boyutu = 300
pencere_boyutu = 4
gizli_boyut = 16
epoch_sayisi = 20
ogrenme_hizi = 0.05

// 1. BPE Tokenizer Eğitimi
bpe_tokenizer_egit(corpus_yolu, sozluk_boyutu)

""'ı yazdır

// 2. Otoregresif Eğitim Verisi Hazırlama
corpus_metni = dosya_oku(corpus_yolu)
veri_paketi = otoregresif_veri_hazirla(corpus_metni, pencere_boyutu, sozluk_boyutu)

x_veri = veri_paketi["x"]
y_veri = veri_paketi["y"]

""'ı yazdır

// 3. Hüma GPT Yapay Sinir Ağı Modeli Oluşturma ve Eğitimi
model = gpt_modeli_olustur(pencere_boyutu, gizli_boyut)
gpt_modeli_egit(model, x_veri, y_veri, epoch_sayisi, ogrenme_hizi)

""'ı yazdır
"══════════════════════════════════════════════════════════════════════════"'yi yazdır
"               🗣 TÜRKÇE CÜMLE / PARAGRAF ÜRETİM GÖSTERİMİ               "'yi yazdır
"══════════════════════════════════════════════════════════════════════════"'yi yazdır
""'ı yazdır

// 4. Veri Seti Üzerinde Akıcı Türkçe Cümle / Paragraf Üretimi
otoregresif_cumle_uret(corpus_yolu, "Yapay zekâ", 20)

""'ı yazdır
otoregresif_cumle_uret(corpus_yolu, "Türkiye tarihi", 25)

""'ı yazdır
otoregresif_cumle_uret(corpus_yolu, "Yazılım mühendisliği", 22)

""'ı yazdır
"🎉 [BAŞARILI] Hüma GPT Türkçe Cümle Üreticisi gösterimi tamamlandı!"'ı yazdır
