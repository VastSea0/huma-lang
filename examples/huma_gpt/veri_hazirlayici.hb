// ══════════════════════════════════════════════════════════════════════════════
// examples / huma_gpt / veri_hazirlayici.hb
// Otoregresif Çift Dizilim Veri Hazırlayıcı (X_t -> Y_t+1)
// ══════════════════════════════════════════════════════════════════════════════

"tokenizer.hb"'yi yükle

otoregresif_veri_hazirla fonksiyon olsun corpus_metni, pencere_boyutu, sozluk_boyutu alsın {
    "📊 [VERİ HAZIRLAYICI] Token dizileri ve otoregresif matrisler oluşturuluyor..."'u yazdır

    token_ids = metni_tokenlestir(corpus_metni)
    toplam_token = uzunluk(token_ids)

    "   • Corpus Toplam BPE Token Sayısı: " + toplam_token'i yazdır

    ornekler_x = []
    etiketler_y = []

    sinir = toplam_token - pencere_boyutu - 1
    sinir > 100 ise {
        sinir = 100
    }

    i = 0'dan sinir'e kadar {
        x_dizi = []
        j = 0'dan (pencere_boyutu - 1)'e kadar {
            val = token_ids[i + j] / sozluk_boyutu
            x_dizi = listeye_ekle(x_dizi, val)
        }

        hedef_token_id = token_ids[i + pencere_boyutu]
        hedef_val = hedef_token_id / sozluk_boyutu
        y_dizi = [hedef_val]

        ornekler_x = listeye_ekle(ornekler_x, x_dizi)
        etiketler_y = listeye_ekle(etiketler_y, y_dizi)
    }

    "   ✅ Hazırlanan Otoregresif Örnek Sayısı: " + uzunluk(ornekler_x)'i yazdır

    paket = {
        "x": ornekler_x,
        "y": etiketler_y,
        "tokenler": token_ids
    }

    paket'i döndür
}
