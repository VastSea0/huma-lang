use crate::token::Token;
use crate::lexer::Lexer;
use crate::ast::{Ifade, Komut};

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
    current_pos: (usize, usize),
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_pos = lexer.get_pos();
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Self { lexer, current_token, peek_token, current_pos }
    }

    fn next_token(&mut self) {
        self.current_pos = self.lexer.get_pos();
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn error(&self, msg: &str) {
        eprintln!("[Hüma Hatası] Satır {}, Sütun {}: {}", self.current_pos.0, self.current_pos.1, msg);
    }

    fn consume(&mut self, expected: Token) -> bool {
        if self.current_token == expected { self.next_token(); true }
        else {
            let msg = format!("{:?} bekleniyordu ama {:?} geldi", expected, self.current_token);
            self.error(&msg);
            // Hata durumunda ilerlemezsek sonsuz döngü olabilir. 
            // Eğer mevcut token beklenen değilse, onu yutalım ki döngü kırılabilsin.
            if self.current_token != Token::Son { self.next_token(); }
            false 
        }
    }

    pub fn parse_program(&mut self) -> Vec<Komut> {
        let mut komutlar = Vec::new();
        while self.current_token != Token::Son {
            let start_pos = self.current_pos;
            if let Some(komut) = self.parse_komut() { 
                komutlar.push(komut); 
            }
            // Güvenlik: parse_komut bir şey tüketmediyse zorla ilerlet
            if self.current_pos == start_pos && self.current_token != Token::Son {
                eprintln!("[Kritik Ayrıştırıcı Hatası] {}:{} konumunda sonsuz döngü engellendi.", self.current_pos.0, self.current_pos.1);
                self.next_token();
            }
        }
        komutlar
    }

    fn parse_komut(&mut self) -> Option<Komut> {
        match self.current_token.clone() {
            // yükle "dosya.hb"
            Token::Yukle => self.parse_yukle(),

            // dene { } hata var ise { }
            Token::Dene => self.parse_dene(),

            Token::NoktaliVirgul => { self.next_token(); None }

            // Identifier ile başlayan komutlar — özel kalıpları kontrol et
            Token::Tanimlayici(_) => {
                // Peek ile hızlı kontrol: fonksiyon/sınıf/liste tanımı
                match self.peek_token {
                    Token::Fonksiyon => {
                        let ad = if let Token::Tanimlayici(ref s) = self.current_token { s.clone() } else { unreachable!() };
                        self.next_token(); // identifier yut
                        return self.parse_fonksiyon_tanimi(ad);
                    }
                    Token::Sinif => {
                        let ad = if let Token::Tanimlayici(ref s) = self.current_token { s.clone() } else { unreachable!() };
                        self.next_token(); // identifier yut
                        return self.parse_sinif_tanimi(ad);
                    }
                    Token::ListeAnahtar => {
                        let ad = if let Token::Tanimlayici(ref s) = self.current_token { s.clone() } else { unreachable!() };
                        self.next_token(); // identifier yut
                        self.next_token(); // 'liste' yut
                        self.consume(Token::Olsun);
                        if self.current_token == Token::NoktaliVirgul { self.next_token(); }
                        return Some(Komut::ListeOlustur { ad });
                    }
                    _ => {}
                }

                // Genel durum: ifade parse et (= dahil), sonra postfix kontrol
                self.parse_genel_ifade_komutu()
            }

            // kendisi'nin alan = değer olsun
            Token::Kendisi => self.parse_kendisi_komutu(),

            // Diğer tüm token'lar (sayı, metin, parantez, köşeli, doğru, yanlış, değil, eksi vb.)
            _ => self.parse_genel_ifade_komutu(),
        }
    }

    /// Genel ifade tabanlı komut: ifade parse et, ardından postfix kontroller
    fn parse_genel_ifade_komutu(&mut self) -> Option<Komut> {
        let ifade = self.parse_ifade();
        self.parse_ifade_devam(ifade)
    }

    /// İfade parse edildikten sonra postfix kontrolleri yap
    fn parse_ifade_devam(&mut self, ifade: Ifade) -> Option<Komut> {
        // Postfix: uzunluğu
        let ifade = if self.current_token == Token::Uzunlugu {
            self.next_token();
            Ifade::Uzunluk(Box::new(ifade))
        } else {
            ifade
        };

        // olsun → ifadeyi assignment olarak yeniden yorumla
        // x = 5 olsun → DegiskenTanimla { ad: x, deger: 5 }
        // SınıfAdı() olsun → DegiskenTanimla ??? — hayır, o yukarıda identifier seviyesinde
        if self.current_token == Token::Olsun {
            self.next_token();
            if self.current_token == Token::NoktaliVirgul { self.next_token(); }
            // İfadeyi decompose et: IkiliIslem { sol: Degisken(ad), op: Esittir, sag: deger } → DegiskenTanimla
            if let Ifade::IkiliIslem { sol, operator: Token::Esittir, sag } = ifade {
                match *sol {
                    Ifade::Degisken(ad) => return Some(Komut::DegiskenTanimla { ad, deger: *sag }),
                    Ifade::NesneErisim { nesne, ozellik } => {
                        return Some(Komut::NesneAlaniAtama { nesne: *nesne, ozellik, deger: *sag });
                    }
                    Ifade::KendisiErisim { ozellik } => {
                        return Some(Komut::NesneAlaniAtama { nesne: Ifade::Degisken("kendisi".to_string()), ozellik, deger: *sag });
                    }
                    _ => return Some(Komut::IfadeKomutu(Ifade::IkiliIslem { sol, operator: Token::Esittir, sag })),
                }
            }
            // Sadece ifade + olsun (sonucu at)
            return Some(Komut::IfadeKomutu(ifade));
        }

        // Postfix: yazdır
        if self.current_token == Token::Yazdir {
            self.next_token();
            if self.current_token == Token::NoktaliVirgul { self.next_token(); }
            return Some(Komut::YazdirKomutu(ifade));
        }

        // Postfix: döndür
        if self.current_token == Token::Dondur {
            self.next_token();
            if self.current_token == Token::NoktaliVirgul { self.next_token(); }
            return Some(Komut::DondurKomutu(ifade));
        }

        // ifade ise { gövde } [yoksa { gövde }]
        if self.current_token == Token::Ise {
            return self.parse_ise_komutu(ifade);
        }

        // ifade olduğu sürece { gövde }
        if self.current_token == Token::Oldugu {
            return self.parse_oldugu_surece(ifade);
        }

        // Postfix: ekle
        if self.current_token == Token::Ekle {
            self.next_token();
            if self.current_token == Token::NoktaliVirgul { self.next_token(); }
            if let Ifade::ListeErisim { liste, indeks } = ifade {
                return Some(Komut::ListeEkle { liste: *liste, deger: *indeks });
            }
            return Some(Komut::IfadeKomutu(ifade));
        }

        // Postfix: çıkar
        if self.current_token == Token::Cikar {
            self.next_token();
            if self.current_token == Token::NoktaliVirgul { self.next_token(); }
            if let Ifade::ListeErisim { liste, indeks } = ifade {
                return Some(Komut::ListeCikar { liste: *liste, indeks: *indeks });
            }
            return Some(Komut::IfadeKomutu(ifade));
        }

        if self.current_token == Token::NoktaliVirgul { self.next_token(); }
        Some(Komut::IfadeKomutu(ifade))
    }

    /// kendisi'nin alan = değer olsun veya kendisi'nin alan ifadesi
    fn parse_kendisi_komutu(&mut self) -> Option<Komut> {
        // 'kendisi' yi ifade olarak parse et (parse_temel handle edecek)
        let ifade = self.parse_ifade();
        self.parse_ifade_devam(ifade)
    }

    /// isim fonksiyon olsun [param, ... alsın] { gövde }
    fn parse_fonksiyon_tanimi(&mut self, ad: String) -> Option<Komut> {
        self.consume(Token::Fonksiyon);
        self.consume(Token::Olsun);

        let mut params = Vec::new();
        if self.current_token != Token::AcikSuskun {
            loop {
                if let Token::Tanimlayici(ref s) = self.current_token {
                    params.push(s.clone());
                    self.next_token();
                } else {
                    break;
                }
                if self.current_token == Token::Virgul {
                    self.next_token();
                } else {
                    break;
                }
            }
            self.consume(Token::Alsin);
        }

        self.consume(Token::AcikSuskun);
        let govde = self.parse_blok();
        Some(Komut::FonksiyonTanimla { ad, parametreler: params, govde })
    }

    /// isim sınıf olsun { metotlar }
    fn parse_sinif_tanimi(&mut self, ad: String) -> Option<Komut> {
        self.consume(Token::Sinif);
        self.consume(Token::Olsun);
        self.consume(Token::AcikSuskun);

        let mut metotlar = Vec::new();
        while self.current_token != Token::KapaliSuskun && self.current_token != Token::Son {
            if let Some(m) = self.parse_komut() {
                metotlar.push(m);
            }
        }
        self.consume(Token::KapaliSuskun);
        Some(Komut::SinifTanimla { ad, metotlar })
    }


    /// ifade ise { gövde } [yoksa { gövde }]
    fn parse_ise_komutu(&mut self, kosul: Ifade) -> Option<Komut> {
        self.consume(Token::Ise);
        self.consume(Token::AcikSuskun);
        let govde = self.parse_blok();
        let mut degilse_govde = None;
        if self.current_token == Token::Yoksa {
            self.next_token();
            if self.current_token != Token::AcikSuskun {
                let else_kosul = self.parse_ifade();
                if let Some(inner_if) = self.parse_ise_komutu(else_kosul) {
                    degilse_govde = Some(vec![inner_if]);
                }
            } else {
                self.consume(Token::AcikSuskun);
                degilse_govde = Some(self.parse_blok());
            }
        }
        Some(Komut::EgerKomutu { kosul, govde, degilse_govde })
    }

    /// ifade olduğu sürece { gövde }
    fn parse_oldugu_surece(&mut self, kosul: Ifade) -> Option<Komut> {
        self.consume(Token::Oldugu);
        self.consume(Token::Surece);
        self.consume(Token::AcikSuskun);
        let govde = self.parse_blok();
        Some(Komut::DonguKomutu { kosul, govde })
    }

    /// dene { } yakala hata { }
    /// Veya eski: dene { } hata var ise { }
    fn parse_dene(&mut self) -> Option<Komut> {
        self.consume(Token::Dene);
        self.consume(Token::AcikSuskun);
        let dene_govde = self.parse_blok();
        
        let mut hata_degisken = None;
        if self.current_token == Token::Yakala {
            self.next_token();
            if let Token::Tanimlayici(ref s) = self.current_token {
                hata_degisken = Some(s.clone());
                self.next_token();
            }
        } else {
            // Geriye dönük uyumluluk: hata var ise
            self.consume(Token::HataAnahtar);
            self.consume(Token::Var);
            self.consume(Token::Ise);
        }

        self.consume(Token::AcikSuskun);
        let hata_govde = self.parse_blok();
        Some(Komut::DeneKomutu { dene_govde, hata_degisken, hata_govde })
    }

    fn parse_yukle(&mut self) -> Option<Komut> {
        self.next_token();
        if let Token::Metin(ref s) = self.current_token {
            let yol = s.clone();
            self.next_token();
            if self.current_token == Token::NoktaliVirgul { self.next_token(); }
            Some(Komut::YukleKomutu(yol))
        } else { None }
    }

    fn parse_blok(&mut self) -> Vec<Komut> {
        let mut komutlar = Vec::new();
        while self.current_token != Token::KapaliSuskun && self.current_token != Token::Son {
            let start_pos = self.current_pos;
            if let Some(komut) = self.parse_komut() { 
                komutlar.push(komut); 
            }
            // Güvenlik: parse_komut bir şey tüketmediyse zorla ilerlet
            if self.current_pos == start_pos && self.current_token != Token::Son && self.current_token != Token::KapaliSuskun {
                eprintln!("[Kritik Ayrıştırıcı Hatası] {}:{} konumunda sonsuz döngü engellendi.", self.current_pos.0, self.current_pos.1);
                self.next_token();
            }
        }
        self.consume(Token::KapaliSuskun);
        komutlar
    }

    // ─── İfade Ayrıştırma (Expression Parsing) ─────────────────────

    pub fn parse_ifade(&mut self) -> Ifade { self.parse_veya() }

    fn parse_veya(&mut self) -> Ifade {
        let mut sol = self.parse_ve();
        while self.current_token == Token::Veya {
            let op = self.current_token.clone(); self.next_token();
            let sag = self.parse_ve();
            sol = Ifade::IkiliIslem { sol: Box::new(sol), operator: op, sag: Box::new(sag) };
        }
        sol
    }

    fn parse_ve(&mut self) -> Ifade {
        let mut sol = self.parse_esitlik();
        while self.current_token == Token::Ve {
            let op = self.current_token.clone(); self.next_token();
            let sag = self.parse_esitlik();
            sol = Ifade::IkiliIslem { sol: Box::new(sol), operator: op, sag: Box::new(sag) };
        }
        sol
    }

    fn parse_esitlik(&mut self) -> Ifade {
        let mut sol = self.parse_karsilastirma();
        // = operatörü koşulda eşitlik kontrolü yapar, olsun ile birlikte atama olur
        while matches!(self.current_token, Token::EsitEsittir | Token::EsitDegil | Token::Esittir) {
            let op = self.current_token.clone(); self.next_token();
            let sag = self.parse_karsilastirma();
            sol = Ifade::IkiliIslem { sol: Box::new(sol), operator: op, sag: Box::new(sag) };
        }
        sol
    }

    fn parse_karsilastirma(&mut self) -> Ifade {
        let mut sol = self.parse_toplama();
        while matches!(self.current_token, Token::Buyuktur | Token::Kucuktur | Token::BuyukEsit | Token::KucukEsit) {
            let op = self.current_token.clone(); self.next_token();
            let sag = self.parse_toplama();
            sol = Ifade::IkiliIslem { sol: Box::new(sol), operator: op, sag: Box::new(sag) };
        }
        sol
    }

    fn parse_toplama(&mut self) -> Ifade {
        let mut sol = self.parse_carpma();
        while matches!(self.current_token, Token::Arti | Token::Eksi) {
            let op = self.current_token.clone(); self.next_token();
            let sag = self.parse_carpma();
            sol = Ifade::IkiliIslem { sol: Box::new(sol), operator: op, sag: Box::new(sag) };
        }
        sol
    }

    fn parse_carpma(&mut self) -> Ifade {
        let mut sol = self.parse_birincil();
        while matches!(self.current_token, Token::Carpi | Token::Bolnu | Token::Mod) {
            let op = self.current_token.clone(); self.next_token();
            let sag = self.parse_birincil();
            sol = Ifade::IkiliIslem { sol: Box::new(sol), operator: op, sag: Box::new(sag) };
        }
        sol
    }

    fn parse_birincil(&mut self) -> Ifade {
        match self.current_token {
            Token::Degil => { self.next_token(); Ifade::MantıksalDegil(Box::new(self.parse_birincil())) }
            Token::Eksi => {
                self.next_token();
                Ifade::IkiliIslem { sol: Box::new(Ifade::Sayi(0.0)), operator: Token::Eksi, sag: Box::new(self.parse_birincil()) }
            }
            _ => self.parse_temel(),
        }
    }

    fn parse_temel(&mut self) -> Ifade {
        let mut node = match self.current_token.clone() {
            Token::Sayi(n) => { self.next_token(); Ifade::Sayi(n) }
            Token::Metin(ref s) => { let v = Ifade::Metin(s.clone()); self.next_token(); v }
            Token::Tanimlayici(ref s) => {
                let v = Ifade::Degisken(s.clone());
                self.next_token();
                v
            }
            Token::Dogru => { self.next_token(); Ifade::Dogru }
            Token::Yanlis => { self.next_token(); Ifade::Yanlis }
            Token::Kendisi => {
                self.next_token();
                if self.current_token == Token::Nin {
                    self.next_token();
                    if let Token::Tanimlayici(ref s) = self.current_token {
                        let oz = s.clone();
                        self.next_token();
                        Ifade::KendisiErisim { ozellik: oz }
                    } else {
                        Ifade::Degisken("kendisi".to_string())
                    }
                } else {
                    Ifade::Degisken("kendisi".to_string())
                }
            }
            Token::AcikParantez => {
                self.next_token();
                let expr = self.parse_ifade();
                self.consume(Token::KapaliParantez);
                expr
            }
            Token::AcikKose => self.parse_liste(),
            Token::AcikSuskun => self.parse_sozluk(),
            Token::Fonksiyon => self.parse_anonim_fonksiyon(),
            _ => { self.next_token(); Ifade::Bos }
        };

        // Postfix operatörler
        loop {
            match self.current_token {
                Token::AcikParantez => {
                    self.next_token();
                    let mut args = Vec::new();
                    if self.current_token != Token::KapaliParantez {
                        loop {
                            args.push(self.parse_ifade());
                            if self.current_token == Token::Virgul { self.next_token(); } else { break; }
                        }
                    }
                    self.consume(Token::KapaliParantez);
                    node = Ifade::Cagri { fonksiyon: Box::new(node), argumanlar: args };
                }
                Token::AcikKose => {
                    self.next_token();
                    let i = self.parse_ifade();
                    self.consume(Token::KapaliKose);
                    node = Ifade::ListeErisim { liste: Box::new(node), indeks: Box::new(i) };
                }
                Token::Nokta => {
                    self.next_token();
                    if let Token::Tanimlayici(ref s) = self.current_token {
                        let oz = s.clone(); self.next_token();
                        node = Ifade::NesneErisim { nesne: Box::new(node), ozellik: oz };
                    } else { break; }
                }
                Token::Nin => {
                    self.next_token();
                    if self.current_token == Token::Uzunlugu {
                        self.next_token();
                        node = Ifade::Uzunluk(Box::new(node));
                    } else if let Token::Tanimlayici(ref s) = self.current_token {
                        let oz = s.clone(); self.next_token();
                        node = Ifade::NesneErisim { nesne: Box::new(node), ozellik: oz };
                    } else {
                        break;
                    }
                }
                Token::Uzunlugu => {
                    self.next_token();
                    node = Ifade::Uzunluk(Box::new(node));
                }
                _ => break,
            }
        }
        node
    }

    fn parse_liste(&mut self) -> Ifade {
        self.next_token(); // '[' yut
        let mut el = Vec::new();
        if self.current_token != Token::KapaliKose {
            loop {
                el.push(self.parse_ifade());
                if self.current_token == Token::Virgul { self.next_token(); } else { break; }
            }
        }
        self.consume(Token::KapaliKose);
        Ifade::Liste(el)
    }
    
    fn parse_sozluk(&mut self) -> Ifade {
        self.next_token(); // '{' yut
        let mut ciftler = Vec::new();
        if self.current_token != Token::KapaliSuskun {
            loop {
                let anahtar = self.parse_ifade();
                self.consume(Token::IkiNokta);
                let deger = self.parse_ifade();
                ciftler.push((anahtar, deger));
                if self.current_token == Token::Virgul { self.next_token(); } else { break; }
            }
        }
        self.consume(Token::KapaliSuskun);
        Ifade::Sozluk(ciftler)
    }

    fn parse_anonim_fonksiyon(&mut self) -> Ifade {
        self.consume(Token::Fonksiyon);
        self.consume(Token::Olsun);

        let mut params = Vec::new();
        if self.current_token != Token::AcikSuskun {
            loop {
                if let Token::Tanimlayici(ref s) = self.current_token {
                    params.push(s.clone());
                    self.next_token();
                } else {
                    break;
                }
                if self.current_token == Token::Virgul {
                    self.next_token();
                } else {
                    break;
                }
            }
            if self.current_token == Token::Alsin { self.next_token(); }
        }

        self.consume(Token::AcikSuskun);
        let govde = self.parse_blok();
        Ifade::FonksiyonIfadesi { parametreler: params, govde }
    }
}
