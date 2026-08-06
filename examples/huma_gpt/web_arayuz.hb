// ══════════════════════════════════════════════════════════════════════════════
// examples / huma_gpt / web_arayuz.hb
// Hüma GPT Web Sunucusu — 2. Derece Trigram Dinamik Üreteç API
// ══════════════════════════════════════════════════════════════════════════════

"huma_sunucu.hb"'yi yükle
"gercek_llm.hb"'yi yükle

""'ı yazdır
"🌐 Sunucu başlatılıyor..."'u yazdır

sunucu = Sunucu()
sunucu.kur(8080)

// URL percent-decode yardımcısı (Türkçe karakterler)
url_coz fonksiyon olsun m alsın {
    m = değiştir(m, "+", " ")
    m = değiştir(m, "%20", " ")
    m = değiştir(m, "%C3%B6", "ö")
    m = değiştir(m, "%c3%b6", "ö")
    m = değiştir(m, "%C3%BC", "ü")
    m = değiştir(m, "%c3%bc", "ü")
    m = değiştir(m, "%C3%A7", "ç")
    m = değiştir(m, "%c3%a7", "ç")
    m = değiştir(m, "%C5%9F", "ş")
    m = değiştir(m, "%c5%9f", "ş")
    m = değiştir(m, "%C4%9F", "ğ")
    m = değiştir(m, "%c4%9f", "ğ")
    m = değiştir(m, "%C4%B1", "ı")
    m = değiştir(m, "%c4%b1", "ı")
    m = değiştir(m, "%C3%A2", "â")
    m = değiştir(m, "%c3%a2", "â")
    m = değiştir(m, "%C4%B0", "İ")
    m = değiştir(m, "%c4%b0", "İ")
    m'yi döndür
}

// 1. Ana Sayfa
sunucu.getir("/", fonksiyon olsun istek, yanıt alsın {
    html_icerik = dosya_oku("web/index.html")
    yanıt.html(html_icerik)
})

// 2. Gerçek Zamanlı Trigram Kelime Çıkarım API (/api/next_word?metin=...&step=...)
sunucu.getir("/api/next_word", fonksiyon olsun istek, yanıt alsın {
    url_metin = istek["url"]

    istem = ""
    step = 0

    içeriyor(url_metin, "metin=") = 1 ise {
        p1 = böl(url_metin, "metin=")
        uzunluk(p1) > 1 ise {
            ham = p1[1]
            içeriyor(ham, "&step=") = 1 ise {
                p2 = böl(ham, "&step=")
                istem = url_coz(p2[0])
                step = sayıya_çevir(p2[1])
            } yoksa {
                istem = url_coz(ham)
            }
        }
    }

    // 0. adımdan step adımlığına kadar kelimeleri sırayla üret (Dinamik Markov Zinciri)
    uretilenler = []
    s = 0
    yeni_kelime = ""

    s <= step olduğu sürece {
        yeni_kelime = llm_sonraki_kelime_dinamik(istem, uretilenler, s)
        uzunluk(yeni_kelime) > 0 ise {
            uretilenler = listeye_ekle(uretilenler, yeni_kelime)
            s = s + 1
        } yoksa {
            // Sonlandı (EOS)
            s = step + 999
        }
    }

    is_done = 0
    uzunluk(yeni_kelime) == 0 ise {
        is_done = 1
    }

    cevap = {
        "word": yeni_kelime,
        "done": is_done
    }

    yanıt.json(cevap)
})

"   ✅ http://localhost:8080 adresinde dinleniyor"'u yazdır
sunucu.baslat()
