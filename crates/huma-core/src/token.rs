use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Anahtar Kelimeler
    Yazdir,         // yazdır
    Olsun,          // olsun
    Alsin,          // alsın
    Fonksiyon,      // fonksiyon
    Sinif,          // sınıf
    Ise,            // ise
    Yoksa,          // yoksa
    Oldugu,         // olduğu
    Surece,         // sürece
    Dondur,         // döndür
    Ve,             // ve
    Veya,           // veya
    Degil,          // değil
    Yukle,          // yükle
    ListeAnahtar,   // liste
    Ekle,           // ekle
    Cikar,          // çıkar
    Uzunlugu,       // uzunluğu
    Kendisi,        // kendisi
    Dogru,          // doğru
    Yanlis,         // yanlış
    Dene,           // dene
    Yakala,         // yakala
    HataAnahtar,    // hata
    Var,            // var
    Nin,            // 'nin / 'nın (kendisi'nin erişimi)
    Iyelik,         // 'si / 'sı / 'su / 'sü (özellik iyeliği)
    Kadar,          // kadar
    Mi,             // mi / mı / mu / mü
    Ile,            // ile
    Bekle,          // bekle (await)
    Cagir,          // çağır

    // Tanımlayıcılar ve Literaller
    Tanimlayici(String),
    Sayi(f64),
    Metin(String),

    // Operatörler ve Semboller
    Esittir,        // =
    Arti,           // +
    Eksi,           // -
    Carpi,          // *
    Bolnu,          // /
    Buyuktur,       // >
    Kucuktur,       // <
    BuyukEsit,      // >=
    KucukEsit,      // <=
    EsitEsittir,    // ==
    EsitDegil,      // !=
    Mod,            // %
    AcikParantez,   // (
    KapaliParantez, // )
    AcikSuskun,     // {
    KapaliSuskun,   // }
    AcikKose,       // [
    KapaliKose,     // ]
    Virgul,         // ,
    NoktaliVirgul,  // ;
    Nokta,          // .
    IkiNokta,       // :

    // Kontrol
    Hata(String),
    Son,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Token::Yazdir => "yazdır",
            Token::Olsun => "olsun",
            Token::Alsin => "alsın",
            Token::Fonksiyon => "fonksiyon",
            Token::Sinif => "sınıf",
            Token::Ise => "ise",
            Token::Yoksa => "yoksa",
            Token::Oldugu => "olduğu",
            Token::Surece => "sürece",
            Token::Dondur => "döndür",
            Token::Ve => "ve",
            Token::Veya => "veya",
            Token::Degil => "değil",
            Token::Yukle => "yükle",
            Token::ListeAnahtar => "liste",
            Token::Ekle => "ekle",
            Token::Cikar => "çıkar",
            Token::Uzunlugu => "uzunluğu",
            Token::Kendisi => "kendisi",
            Token::Dogru => "doğru",
            Token::Yanlis => "yanlış",
            Token::Dene => "dene",
            Token::Yakala => "yakala",
            Token::HataAnahtar => "hata",
            Token::Var => "var",
            Token::Nin => "'nin/'nın",
            Token::Iyelik => "'si/'sı/'su/'sü",
            Token::Kadar => "kadar",
            Token::Mi => "mi/mı/mu/mü",
            Token::Ile => "ile",
            Token::Bekle => "bekle",
            Token::Cagir => "çağır",
            Token::Tanimlayici(s) => s,
            Token::Sayi(n) => return write!(f, "{}", n),
            Token::Metin(s) => return write!(f, "\"{}\"", s),
            Token::Esittir => "=",
            Token::Arti => "+",
            Token::Eksi => "-",
            Token::Carpi => "*",
            Token::Bolnu => "/",
            Token::Buyuktur => ">",
            Token::Kucuktur => "<",
            Token::BuyukEsit => ">=",
            Token::KucukEsit => "<=",
            Token::EsitEsittir => "==",
            Token::EsitDegil => "!=",
            Token::Mod => "%",
            Token::AcikParantez => "(",
            Token::KapaliParantez => ")",
            Token::AcikSuskun => "{",
            Token::KapaliSuskun => "}",
            Token::AcikKose => "[",
            Token::KapaliKose => "]",
            Token::Virgul => ",",
            Token::NoktaliVirgul => ";",
            Token::Nokta => ".",
            Token::IkiNokta => ":",
            Token::Hata(e) => return write!(f, "Hata({})", e),
            Token::Son => "dosya sonu",
        };
        write!(f, "{}", s)
    }
}
