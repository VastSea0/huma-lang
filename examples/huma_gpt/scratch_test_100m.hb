"yapay_zeka"'yı yükle

yazdır "════════════════════════════════════════════════════════════"
yazdır "  100 MİLYON PARAMETRELİ HÜMA TRANSFORMER LLM MİMARİSİ"
yazdır "════════════════════════════════════════════════════════════"

vocab_size = 16384 olsun
d_model = 768 olsun
d_ff = 3072 olsun
max_seq = 1024 olsun
n_layers = 12 olsun

// Embedding parametreleri
p_emb = (vocab_size * d_model) + (max_seq * d_model) olsun
yazdır "Embedding Katmanı Parametre Sayısı: " + p_emb

// Tek bir Transformer bloğundaki parametreler
p_attn = 4 * (d_model * d_model) olsun
p_ffn = 2 * (d_model * d_ff) olsun
p_layer = p_attn + p_ffn olsun
yazdır "Tek Katman Parametre Sayısı: " + p_layer

// 12 Katman toplamı
p_layers_total = p_layer * n_layers olsun
yazdır "12 Transformer Katmanı Toplamı: " + p_layers_total

// Çıktı Projeksiyon Katmanı
p_out = d_model * vocab_size olsun
yazdır "Çıktı Katmanı Parametre Sayısı: " + p_out

// Genel Toplam
p_toplam = p_emb + p_layers_total + p_out olsun
yazdır "------------------------------------------------------------"
yazdır "TOPLAM MİMARİ PARAMETRE SAYISI: " + p_toplam + " PARAMETRE (~110.8 MİLYON)"
yazdır "════════════════════════════════════════════════════════════"
