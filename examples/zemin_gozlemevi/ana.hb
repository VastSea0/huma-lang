// ══════════════════════════════════════════════════════════════════════════════
// Hüma Zemin Gözlemevi
// CSV -> doğrulama -> JSONL denetim izi -> Markdown raporu -> geri okuma
// Yapay zekâ, ağ ve üçüncü taraf hizmet kullanmaz.
// ══════════════════════════════════════════════════════════════════════════════

KAYNAK = "examples/zemin_gozlemevi/bilesenler.csv" olsun
OLAYLAR = "target/huma-zemin-olaylari.jsonl" olsun
RAPOR = "target/huma-zemin-raporu.md" olsun

satirlar = csv_oku(KAYNAK) olsun
toplam_bilesen = uzunluk(satirlar) - 1 olsun
basarili = 0 olsun
butce_asimi = 0 olsun
toplam_test = 0 olsun
en_yavas = "" olsun
en_yavas_ms = 0 olsun

// Her çalıştırma aynı sonucu üretir: eski denetim izini atomik olarak temizle.
dosya_yaz(OLAYLAR, "") olsun

rapor = "# Hüma Zemin Gözlemevi\n\n" olsun
rapor = rapor + "Genel amaçlı dil zemininin tekrar üretilebilir sağlık raporu.\n\n" olsun
rapor = rapor + "| Bileşen | Durum | Test | Ölçüm | Bütçe |\n" olsun
rapor = rapor + "|---|---:|---:|---:|---:|\n" olsun

yazdır ""
yazdır "╭──────────────────────────────────────────────────────────────╮"
yazdır "│                 HÜMA ZEMİN GÖZLEMEVİ                         │"
yazdır "│       CSV • Yetki Sınırı • JSONL • Atomik Rapor              │"
yazdır "╰──────────────────────────────────────────────────────────────╯"
yazdır ""

i = 1 olsun
i < uzunluk(satirlar) olduğu sürece {
    satir = satirlar[i] olsun
    bilesen = satir[0] olsun
    durum = satir[1] olsun
    test_sayisi = sayıya_çevir(satir[2]) olsun
    olcum_ms = sayıya_çevir(satir[3]) olsun
    butce_ms = sayıya_çevir(satir[4]) olsun

    toplam_test = toplam_test + test_sayisi olsun

    durum = "geçti" ise {
        basarili = basarili + 1 olsun
    }

    olcum_ms > butce_ms ise {
        butce_asimi = butce_asimi + 1 olsun
    }

    olcum_ms > en_yavas_ms ise {
        en_yavas_ms = olcum_ms olsun
        en_yavas = bilesen olsun
    }

    kayit = {} olsun
    kayit["bileşen"] = bilesen
    kayit["durum"] = durum
    kayit["test_sayısı"] = test_sayisi
    kayit["ölçüm_ms"] = olcum_ms
    kayit["bütçe_ms"] = butce_ms
    jsonl_yaz(OLAYLAR, kayit) olsun

    durum_simgesi = "✓" olsun
    durum != "geçti" ise { durum_simgesi = "✗" olsun }
    olcum_ms > butce_ms ise { durum_simgesi = "⚠" olsun }

    yazdır "  " + durum_simgesi + " " + bilesen + " — " + test_sayisi + " test, " + olcum_ms + "/" + butce_ms + " ms"

    rapor = rapor + "| " + bilesen + " | " + durum_simgesi + " " + durum + " | " + test_sayisi + " | " + olcum_ms + " ms | " + butce_ms + " ms |\n" olsun
    i = i + 1 olsun
}

basari_orani = (basarili / toplam_bilesen) * 100 olsun

rapor = rapor + "\n## Özet\n\n" olsun
rapor = rapor + "- Sağlıklı bileşen: " + basarili + "/" + toplam_bilesen + "\n" olsun
rapor = rapor + "- Başarı oranı: %" + basari_orani + "\n" olsun
rapor = rapor + "- Toplam doğrulama: " + toplam_test + "\n" olsun
rapor = rapor + "- Bütçe aşımı: " + butce_asimi + "\n" olsun
rapor = rapor + "- En yavaş bileşen: " + en_yavas + " (" + en_yavas_ms + " ms)\n" olsun
dosya_yaz(RAPOR, rapor) olsun

// JSONL çıktısını geri okuyarak tam kayıt sayısını doğrula.
dogrulanan_kayitlar = jsonl_oku(OLAYLAR) olsun

yazdır ""
yazdır "  ────────────────────────────────────────────────────────────"
yazdır "  Sağlıklı       : " + basarili + "/" + toplam_bilesen + " (%" + basari_orani + ")"
yazdır "  Doğrulama      : " + toplam_test
yazdır "  Bütçe aşımı    : " + butce_asimi
yazdır "  JSONL geri oku : " + uzunluk(dogrulanan_kayitlar) + " kayıt"
yazdır "  En yavaş       : " + en_yavas + " — " + en_yavas_ms + " ms"
yazdır "  Rapor           : " + RAPOR
yazdır "  Denetim izi     : " + OLAYLAR
yazdır "  ────────────────────────────────────────────────────────────"
yazdır ""

uzunluk(dogrulanan_kayitlar) != toplam_bilesen ise {
    hata("JSONL geri okuma doğrulaması başarısız") olsun
}

butce_asimi != 0 ise {
    hata("Performans bütçesi aşıldı") olsun
}

yazdır "✓ Zemin sağlıklı; rapor ve denetim izi doğrulandı."
