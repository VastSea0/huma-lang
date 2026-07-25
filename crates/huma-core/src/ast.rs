use crate::token::Token;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Ifade {
    Bos,
    Sayi(f64),
    Metin(String),
    Dogru,
    Yanlis,
    Degisken(String),
    Bekle(Box<Ifade>),
    IkiliIslem {
        sol: Box<Ifade>,
        operator: Token,
        sag: Box<Ifade>,
    },
    MantıksalDegil(Box<Ifade>),
    Liste(Vec<Ifade>),
    ListeErisim {
        liste: Box<Ifade>,
        indeks: Box<Ifade>,
    },
    NesneErisim {
        nesne: Box<Ifade>,
        ozellik: String,
    },
    #[allow(dead_code)]
    NesneOlustur {
        sinif_adi: String,
        argumanlar: Vec<Ifade>,
    },
    Cagri {
        fonksiyon: Box<Ifade>,
        argumanlar: Vec<Ifade>,
        pos: (usize, usize),
    },
    /// kendisi'nin özellik erişimi
    KendisiErisim {
        ozellik: String,
    },
    /// liste'nin uzunluğu ifadesi
    Uzunluk(Box<Ifade>),
    /// Anonim fonksiyon (Closure)
    FonksiyonIfadesi {
        parametreler: Vec<String>,
        govde: Vec<Komut>,
    },
    Sozluk(Vec<(Ifade, Ifade)>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Komut {
    DegiskenTanimla {
        ad: String,
        deger: Ifade,
    },
    #[allow(dead_code)]
    Atama {
        ad: String,
        deger: Ifade,
    },
    EgerKomutu {
        kosul: Ifade,
        govde: Vec<Komut>,
        degilse_govde: Option<Vec<Komut>>,
    },
    DonguKomutu {
        kosul: Ifade,
        govde: Vec<Komut>,
    },
    FonksiyonTanimla {
        ad: String,
        parametreler: Vec<String>,
        govde: Vec<Komut>,
    },
    DondurKomutu(Ifade),
    YukleKomutu(String),
    SinifTanimla {
        ad: String,
        metotlar: Vec<Komut>,
    },
    YazdirKomutu(Ifade),
    IfadeKomutu(Ifade),
    /// sayılar liste olsun
    ListeOlustur {
        ad: String,
    },
    /// sayılar'a [5]'i ekle
    ListeEkle {
        liste: Ifade,
        deger: Ifade,
    },
    /// sayılar'dan [0]'ı çıkar
    ListeCikar {
        liste: Ifade,
        indeks: Ifade,
    },
    /// dene { } hata var ise { }
    DeneKomutu {
        dene_govde: Vec<Komut>,
        hata_degisken: Option<String>,
        hata_govde: Vec<Komut>,
    },
    /// Nesne alanına atama: kendisi'nin alan = değer olsun
    NesneAlaniAtama {
        nesne: Ifade,
        ozellik: String,
        deger: Ifade,
    },
    /// Aralik döngüsü: i = 0'dan 10'a kadar { ... }
    AralikDongusu {
        degisken: String,
        baslangic: Ifade,
        bitis: Ifade,
        govde: Vec<Komut>,
    },
    /// İçinde bulunulan döngünün sonraki adımına geç.
    Devam,
    /// İçinde bulunulan döngüyü sonlandır.
    Kir,
}
