use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Ifade {
    Bos,
    Sayi(f64),
    Metin(String),
    Dogru,
    Yanlis,
    Degisken(String),
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
}

#[derive(Debug, Clone, PartialEq)]
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
        hata_govde: Vec<Komut>,
    },
    /// Nesne alanına atama: kendisi'nin alan = değer olsun
    NesneAlaniAtama {
        nesne: Ifade,
        ozellik: String,
        deger: Ifade,
    },
}
