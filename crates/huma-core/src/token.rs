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
