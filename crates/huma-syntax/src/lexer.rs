use crate::token::Token;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

#[derive(Debug, Clone)]
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    last_token_pos: (usize, usize),
    suffix_stem: Option<String>,
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
    ch.is_alphanumeric() || is_combining_mark(ch) || ch == '_'
}

fn is_apostrophe(ch: char) -> bool {
    matches!(ch, '\'' | '’')
}

/// Bilinen Türkçe ek kalıpları — suffix stripping için
fn is_turkish_suffix(s: &str) -> bool {
    matches!(
        s,
        "i" | "ı"
            | "u"
            | "ü"
            | "yi"
            | "yı"
            | "yu"
            | "yü"
            | "ni"
            | "nı"
            | "nu"
            | "nü"
            | "si"
            | "sı"
            | "su"
            | "sü"
            | "a"
            | "e"
            | "ya"
            | "ye"
            | "dan"
            | "den"
            | "tan"
            | "ten"
            | "da"
            | "de"
            | "ta"
            | "te"
            | "lar"
            | "ler"
            | "ca"
            | "ce"
            | "ça"
            | "çe"
            | "nin"
            | "nın"
            | "nun"
            | "nün"
            | "in"
            | "ın"
            | "un"
            | "ün"
            | "daki"
            | "deki"
            | "taki"
            | "teki"
            | "la"
            | "le"
            | "yla"
            | "yle"
    )
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            last_token_pos: (1, 1),
            suffix_stem: None,
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

    /// En son üretilen token'ın bir tabanlı başlangıç konumu.
    pub fn get_token_pos(&self) -> (usize, usize) {
        self.last_token_pos
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
        if self.nin_state == NinState::AfterNinProperty && !matches!(self.peek(), Some('\'' | '’'))
        {
            self.nin_state = NinState::None;
        }
        self.last_token_pos = (self.line, self.col);

        let ch = match self.advance() {
            Some(ch) => ch,
            None => return Token::Son,
        };
        if !is_apostrophe(ch) {
            self.suffix_stem = None;
        }

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
            ':' => Token::IkiNokta,
            '+' => Token::Arti,
            '-' => Token::Eksi,
            '*' => Token::Carpi,
            '/' => {
                if self.peek() == Some('/') {
                    while let Some(c) = self.advance() {
                        if c == '\n' {
                            break;
                        }
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
            _ if is_turkish_alpha(ch) => self.read_identifier(ch),
            _ => Token::Hata(format!("Bilinmeyen karakter: {}", ch)),
        }
    }

    /// Kesme işareti sonrası ek yönetimi.
    /// Türkçe suffix (ek) varsa strip eder ve:
    /// - 'nin/'nın → Token::Nin döndürür (nesne erişimi, kendisi'nin)
    /// - İyelik: (X'in Y'si / Y'ı): Token::Iyelik döndürür
    /// - Diğer ekler → yutulur ve bir sonraki token'a geçilir
    fn handle_apostrophe(&mut self) -> Token {
        let mut harmony_stem = self.suffix_stem.clone();
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
            if let Some(stem) = &harmony_stem {
                if let Err(error) = crate::morphology::validate_suffix_harmony(stem, &suffix) {
                    return Token::Hata(error.to_string());
                }
            }

            // İyelik eki — sadece `Nin` erişiminden sonra beklenir.
            // Örn: ayarlar'ın tema'sı  /  k1'in yas'ı
            if self.nin_state == NinState::AfterNinProperty
                && matches!(
                    suffix.as_str(),
                    "si" | "sı" | "su" | "sü" | "i" | "ı" | "u" | "ü" | "ni" | "nı" | "nu" | "nü"
                )
            {
                self.nin_state = NinState::None;
                if let Some(stem) = &mut self.suffix_stem {
                    stem.push_str(&suffix);
                }
                return Token::Iyelik;
            }

            // "nin", "nın", "nun", "nün", "in", "ın", "un", "ün" ekleri → Nin token döndür
            if matches!(
                suffix.as_str(),
                "nin" | "nın" | "nun" | "nün" | "in" | "ın" | "un" | "ün"
            ) {
                self.nin_state = NinState::AfterNin;
                return Token::Nin;
            }

            // Yönelme eki doğal liste ekleme ve aralık üst sınırını
            // belirsiz komut bitişlerinden ayırmak için yapısal token taşır.
            if matches!(suffix.as_str(), "a" | "e" | "ya" | "ye") {
                return Token::Yonelme;
            }

            // Ayrılma eki doğal liste çıkarmayı sıradan iki ifade
            // komutundan ayırmak için yapısal token taşır.
            if matches!(suffix.as_str(), "dan" | "den" | "tan" | "ten") {
                return Token::Ayrilma;
            }

            // Bilinen bir Türkçe ek mi?
            if is_turkish_suffix(&suffix) {
                // Eki yuttuk. Eğer arkasından başka bir kesme işareti geliyorsa devam et,
                // gelmiyorsa asıl sonraki token'ı döndür.
                self.skip_whitespace();
                if self.peek().is_some_and(is_apostrophe) {
                    harmony_stem = harmony_stem.map(|mut stem| {
                        stem.push_str(&suffix);
                        stem
                    });
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
                            // Tam olarak iki basamaklı Latin-1/Unicode kaçışı: \x1b
                            let Some(h1) = self.advance() else {
                                return Token::Hata(
                                    "Eksik onaltılık kaçış: iki basamak bekleniyordu".to_string(),
                                );
                            };
                            let Some(h2) = self.advance() else {
                                return Token::Hata(
                                    "Eksik onaltılık kaçış: iki basamak bekleniyordu".to_string(),
                                );
                            };
                            if !h1.is_ascii_hexdigit() || !h2.is_ascii_hexdigit() {
                                return Token::Hata(format!(
                                    "Geçersiz onaltılık kaçış: \\x{}{}",
                                    h1, h2
                                ));
                            }
                            let hex = format!("{}{}", h1, h2);
                            match u8::from_str_radix(&hex, 16) {
                                Ok(code) => s.push(code as char),
                                Err(error) => {
                                    return Token::Hata(format!(
                                        "Geçersiz onaltılık kaçış: \\x{}{} ({})",
                                        h1, h2, error
                                    ));
                                }
                            }
                        }
                        _ => return Token::Hata(format!("Bilinmeyen kaçış dizisi: \\{}", next)),
                    }
                } else {
                    return Token::Hata("Metin sonunda eksik kaçış dizisi".to_string());
                }
            } else {
                s.push(ch);
            }
        }
        Token::Hata("Kapatılmamış metin dizisi".to_string())
    }

    fn read_number(&mut self, first_ch: char) -> Token {
        let mut s = first_ch.to_string();
        while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            if let Some(ch) = self.advance() {
                s.push(ch);
            }
        }

        if self.peek() == Some('.')
            && self
                .input
                .get(self.pos + 1)
                .is_some_and(|ch| ch.is_ascii_digit())
        {
            if let Some(ch) = self.advance() {
                s.push(ch);
            }
            while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                if let Some(ch) = self.advance() {
                    s.push(ch);
                }
            }
        }

        if matches!(self.peek(), Some('e' | 'E')) {
            let exponent_start = self.pos;
            let exponent_marker = self.advance();
            let sign = if matches!(self.peek(), Some('+' | '-')) {
                self.advance()
            } else {
                None
            };
            if self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                if let Some(ch) = exponent_marker {
                    s.push(ch);
                }
                if let Some(ch) = sign {
                    s.push(ch);
                }
                while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                    if let Some(ch) = self.advance() {
                        s.push(ch);
                    }
                }
            } else {
                self.pos = exponent_start;
                self.col = self.col.saturating_sub(1 + usize::from(sign.is_some()));
            }
        }

        match s.parse::<f64>() {
            Ok(val) if val.is_finite() => Token::Sayi(val),
            Ok(_) => Token::Hata(format!("Sonlu olmayan sayı: {}", s)),
            Err(_) => Token::Hata(format!("Geçersiz sayı: {}", s)),
        }
    }

    fn read_identifier(&mut self, first_ch: char) -> Token {
        let mut s = first_ch.to_string();
        while let Some(ch) = self.peek() {
            if is_turkish_alnum(ch) {
                if let Some(next) = self.advance() {
                    s.push(next);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let s = s.nfc().collect::<String>();
        self.suffix_stem = Some(s.clone());
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
            "yakala" => Token::Yakala,
            "hata" => Token::HataAnahtar,
            "var" => Token::Var,
            "kadar" => Token::Kadar,
            "mi" | "mı" | "mu" | "mü" => Token::Mi,
            "ile" => Token::Ile,
            "bekle" => Token::Bekle,
            "çağır" => Token::Cagir,
            "devam" => Token::Devam,
            "kır" | "kir" => Token::Kir,
            "olarak" => Token::Olarak,
            "dışa" | "disa" => Token::Disa,
            "aktar" => Token::Aktar,
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

#[cfg(test)]
mod tests {
    use super::Lexer;
    use crate::token::Token;

    #[test]
    fn token_baslangic_konumlarini_unicode_sutunlariyla_korur() {
        let mut lexer = Lexer::new("değer = 1\n  sonuç = 2");

        assert_eq!(lexer.next_token(), Token::Tanimlayici("değer".into()));
        assert_eq!(lexer.get_token_pos(), (1, 1));
        assert_eq!(lexer.next_token(), Token::Esittir);
        assert_eq!(lexer.get_token_pos(), (1, 7));
        assert_eq!(lexer.next_token(), Token::Sayi(1.0));
        assert_eq!(lexer.get_token_pos(), (1, 9));
        assert_eq!(lexer.next_token(), Token::Tanimlayici("sonuç".into()));
        assert_eq!(lexer.get_token_pos(), (2, 3));
    }

    #[test]
    fn ascii_ve_unicode_kesme_isareti_ayni_tokenlari_uretir() {
        fn tokenize(source: &str) -> Vec<Token> {
            let mut lexer = Lexer::new(source);
            let mut tokens = Vec::new();
            loop {
                let token = lexer.next_token();
                tokens.push(token.clone());
                if token == Token::Son {
                    return tokens;
                }
            }
        }

        assert_eq!(tokenize("değer'i yazdır"), tokenize("değer’i yazdır"));
        assert_eq!(tokenize("\"x\"'in uzunluğu"), tokenize("\"x\"’in uzunluğu"));
        assert_eq!(tokenize("2'nin"), tokenize("2’nin"));
    }

    #[test]
    fn tanimlayicilari_nfc_bicimine_normallestirir() {
        let mut lexer = Lexer::new("deg\u{0306}er");
        assert_eq!(lexer.next_token(), Token::Tanimlayici("değer".into()));
    }

    #[test]
    fn gecersiz_metin_kacislarini_sessizce_kabul_etmez() {
        let mut bilinmeyen = Lexer::new(r#""\q""#);
        assert!(matches!(bilinmeyen.next_token(), Token::Hata(_)));

        let mut eksik_hex = Lexer::new(r#""\xA""#);
        assert!(matches!(eksik_hex.next_token(), Token::Hata(_)));

        let mut bozuk_hex = Lexer::new(r#""\xG0""#);
        assert!(matches!(bozuk_hex.next_token(), Token::Hata(_)));
    }
}
