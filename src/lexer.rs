use crate::token::Token;

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    nin_state: NinState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NinState {
    None,
    AfterNin,
    AfterNinProperty,
}

/// Türkçe karakter kontrolü — identifier'larda kullanılabilecek tüm Türkçe harfler
fn is_turkish_alpha(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}

fn is_turkish_alnum(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

/// Bilinen Türkçe ek kalıpları — suffix stripping için
fn is_turkish_suffix(s: &str) -> bool {
    matches!(s,
        "i" | "ı" | "u" | "ü" | "yi" | "yı" | "yu" | "yü" |
        "ni" | "nı" | "nu" | "nü" |
        "si" | "sı" | "su" | "sü" |
        "a" | "e" | "ya" | "ye" |
        "dan" | "den" | "tan" | "ten" |
        "da" | "de" | "ta" | "te" |
        "lar" | "ler" |
        "ca" | "ce" | "ça" | "çe" |
        "nin" | "nın" | "nun" | "nün" | "in" | "ın" | "un" | "ün" |
        "daki" | "deki" | "taki" | "teki" |
        "la" | "le" | "yla" | "yle"
    )
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            nin_state: NinState::None,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }


    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    pub fn get_pos(&self) -> (usize, usize) {
        (self.line, self.col)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let ch = match self.advance() {
            Some(ch) => ch,
            None => return Token::Son,
        };

        match ch {
            '(' => Token::AcikParantez,
            ')' => Token::KapaliParantez,
            '{' => Token::AcikSuskun,
            '}' => Token::KapaliSuskun,
            '[' => Token::AcikKose,
            ']' => Token::KapaliKose,
            ',' => Token::Virgul,
            ';' => Token::NoktaliVirgul,
            '.' => Token::Nokta,
            '+' => Token::Arti,
            '-' => Token::Eksi,
            '*' => Token::Carpi,
            '/' => {
                if self.peek() == Some('/') {
                    while let Some(c) = self.advance() {
                        if c == '\n' { break; }
                    }
                    self.next_token()
                } else {
                    Token::Bolnu
                }
            }
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::EsitEsittir
                } else {
                    Token::Esittir
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::BuyukEsit
                } else {
                    Token::Buyuktur
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::KucukEsit
                } else {
                    Token::Kucuktur
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::EsitDegil
                } else {
                    Token::Hata("Beklenmeyen karakter: !".to_string())
                }
            }
            '%' => Token::Mod,
            '"' => self.read_string(),
            '\'' | '’' => self.handle_apostrophe(),
            '0'..='9' => self.read_number(ch),
            _ if is_turkish_alpha(ch) => {
                self.read_identifier(ch)
            }
            _ => Token::Hata(format!("Bilinmeyen karakter: {}", ch)),
        }
    }

    /// Kesme işareti sonrası ek yönetimi.
    /// Türkçe suffix (ek) varsa strip eder ve:
    /// - 'nin/'nın → Token::Nin döndürür (nesne erişimi, kendisi'nin)
    /// - İyelik: (X'in Y'si / Y'ı): Token::Iyelik döndürür
    /// - Diğer ekler → yutulur ve bir sonraki token'a geçilir
    fn handle_apostrophe(&mut self) -> Token {
        loop {
            // Kesme işaretinden sonraki eki oku
            let mut suffix = String::new();
            let save_pos = self.pos;
            let save_line = self.line;
            let save_col = self.col;

            while let Some(ch) = self.peek() {
                if is_turkish_alpha(ch) && !ch.is_ascii_digit() {
                    suffix.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }

            if suffix.is_empty() {
                return Token::Hata("Beklenmeyen kesme işareti (ek bulunamadı)".to_string());
            }

            // İyelik eki — sadece `Nin` erişiminden sonra beklenir.
            // Örn: ayarlar'ın tema'sı  /  k1'in yas'ı
            if self.nin_state == NinState::AfterNinProperty
                && matches!(suffix.as_str(), "si" | "sı" | "su" | "sü" | "i" | "ı" | "u" | "ü" | "ni" | "nı" | "nu" | "nü")
            {
                self.nin_state = NinState::None;
                return Token::Iyelik;
            }

            // "nin", "nın", "nun", "nün", "in", "ın", "un", "ün" ekleri → Nin token döndür
            if matches!(suffix.as_str(), "nin" | "nın" | "nun" | "nün" | "in" | "ın" | "un" | "ün") {
                self.nin_state = NinState::AfterNin;
                return Token::Nin;
            }

            // Bilinen bir Türkçe ek mi?
            if is_turkish_suffix(&suffix) {
                // Eki yuttuk. Eğer arkasından başka bir kesme işareti geliyorsa devam et,
                // gelmiyorsa asıl sonraki token'ı döndür.
                self.skip_whitespace();
                if self.peek() == Some('\'') {
                    self.advance(); // Sonraki kesme işaretini yut ve döngüye devam et
                    continue;
                } else {
                    return self.next_token();
                }
            }

            // Bilinmeyen ek — geri al ve hata döndür
            self.pos = save_pos;
            self.line = save_line;
            self.col = save_col;
            return Token::Hata(format!("Bilinmeyen ek: '{}", suffix));
        }
    }

    fn read_string(&mut self) -> Token {
        let mut s = String::new();
        while let Some(ch) = self.advance() {
            if ch == '"' {
                // String kapandıktan sonra kesme işareti + ek varsa strip et
                if self.peek() == Some('\'') {
                    self.advance(); // kesme işaretini yut
                    let mut suffix = String::new();
                    while let Some(sc) = self.peek() {
                        if is_turkish_alpha(sc) && !sc.is_ascii_digit() {
                            suffix.push(sc);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    // Eki yok say, suffix zaten strip edildi
                }
                return Token::Metin(s);
            }
            if ch == '\\' {
                if let Some(next) = self.advance() {
                    match next {
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        'x' => {
                            // Hex escape \x1b
                            let h1 = self.advance().unwrap_or('0');
                            let h2 = self.advance().unwrap_or('0');
                            let hex = format!("{}{}", h1, h2);
                            if let Ok(code) = u8::from_str_radix(&hex, 16) {
                                s.push(code as char);
                            }
                        }
                        _ => { s.push('\\'); s.push(next); }
                    }
                } else {
                    s.push('\\');
                }
            } else {
                s.push(ch);
            }
        }
        Token::Hata("Kapatılmamış metin dizisi".to_string())
    }

    fn read_number(&mut self, first_ch: char) -> Token {
        let mut s = first_ch.to_string();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' {
                s.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        // Sayıdan sonra kesme işareti + ek varsa strip et
        if self.peek() == Some('\'') {
            let save_pos = self.pos;
            let save_line = self.line;
            let save_col = self.col;
            self.advance(); // kesme işaretini yut
            let mut suffix = String::new();
            while let Some(sc) = self.peek() {
                if is_turkish_alpha(sc) && !sc.is_ascii_digit() {
                    suffix.push(sc);
                    self.advance();
                } else {
                    break;
                }
            }
            if !is_turkish_suffix(&suffix) && !matches!(suffix.as_str(), "nin" | "nın" | "nun" | "nün") {
                // Bilinmeyen ek — geri al
                self.pos = save_pos;
                self.line = save_line;
                self.col = save_col;
            }
            // Bilinen ek ise zaten strip ettik
        }

        match s.parse::<f64>() {
            Ok(val) => Token::Sayi(val),
            Err(_) => Token::Hata(format!("Geçersiz sayı: {}", s)),
        }
    }

    fn read_identifier(&mut self, first_ch: char) -> Token {
        let mut s = first_ch.to_string();
        while let Some(ch) = self.peek() {
            if is_turkish_alnum(ch) {
                s.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        
        let tok = match s.as_str() {
            // Yeni Türkçe anahtar kelimeler
            "yazdır" | "yazdir" => Token::Yazdir,
            "olsun" => Token::Olsun,
            "alsın" | "alsin" => Token::Alsin,
            "fonksiyon" => Token::Fonksiyon,
            "sınıf" | "sinif" => Token::Sinif,
            "ise" => Token::Ise,
            "yoksa" => Token::Yoksa,
            "olduğu" | "oldugu" => Token::Oldugu,
            "sürece" | "surece" => Token::Surece,
            "döndür" | "dondur" | "döndur" => Token::Dondur,
            "ve" => Token::Ve,
            "veya" => Token::Veya,
            "değil" | "degil" => Token::Degil,
            "yükle" | "yukle" => Token::Yukle,
            "liste" => Token::ListeAnahtar,
            "ekle" => Token::Ekle,
            "çıkar" | "cikar" => Token::Cikar,
            "uzunluğu" | "uzunlugu" => Token::Uzunlugu,
            "kendisi" => Token::Kendisi,
            "doğru" | "dogru" => Token::Dogru,
            "yanlış" | "yanlis" => Token::Yanlis,
            "dene" => Token::Dene,
            "hata" => Token::HataAnahtar,
            "var" => Token::Var,
            "kadar" => Token::Kadar,
            "mi" | "mı" | "mu" | "mü" => Token::Mi,
            _ => Token::Tanimlayici(s),
        };

        // `X'in Y` yapısında, Nin'den sonra gelen ilk tanımlayıcı "özellik" olarak kabul edilir.
        if self.nin_state == NinState::AfterNin {
            if matches!(tok, Token::Tanimlayici(_)) {
                self.nin_state = NinState::AfterNinProperty;
            } else {
                self.nin_state = NinState::None;
            }
        }

        tok
    }
}
