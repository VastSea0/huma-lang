use huma::lexer::Lexer;
use huma::parser::Parser;
use huma::interpreter::Yorumlayici;
use std::rc::Rc;
use std::cell::RefCell;

fn eval(kod: &str) -> String {
    let output_buffer = Rc::new(RefCell::new(String::new()));
    let mut yorumlayici = Yorumlayici::new().with_output_buffer(Rc::clone(&output_buffer));
    
    let tarayici = Lexer::new(kod);
    let mut ayristirici = Parser::new(tarayici);
    let program = ayristirici.parse_program();
    
    yorumlayici.yorumla(program);
    
    let res = output_buffer.borrow().clone();
    res
}

#[test]
fn test_degisken_atama_ve_okuma() {
    let kod = r#"
        sayi = 42 olsun
        sayi'yı yazdır
    "#;
    assert_eq!(eval(kod).trim(), "42");
}

#[test]
fn test_matematiksel_islemler() {
    let kod = r#"
        sonuc = (10 + 5) * 2 - 4 / 2 olsun
        sonuc'u yazdır
    "#;
    assert_eq!(eval(kod).trim(), "28");
}

#[test]
fn test_kosullu_ifadeler() {
    let kod = r#"
        a = 10 olsun
        a > 5 ise {
            "Buyuk"'u yazdır
        }
    "#;
    assert_eq!(eval(kod).trim(), "Buyuk");
}

#[test]
fn test_donguler() {
    let kod = r#"
        i = 0 olsun
        i < 3 iken {
            i'yi yazdır
            i = i + 1 olsun
        }
    "#;
    let out = eval(kod);
    let mut lines = out.lines();
    assert_eq!(lines.next(), Some("0"));
    assert_eq!(lines.next(), Some("1"));
    assert_eq!(lines.next(), Some("2"));
}

#[test]
fn test_fonksiyonlar() {
    let kod = r#"
        topla fonksiyon olsun a, b alsın {
            a + b'yi döndür
        }
        sonuc = topla(5, 7) olsun
        sonuc'u yazdır
    "#;
    assert_eq!(eval(kod).trim(), "12");
}

#[test]
fn test_listeler() {
    let kod = r#"
        liste = [1, 2, 3] olsun
        liste[1]'i yazdır
        listeye_ekle(liste, 4)
        liste[3]'ü yazdır
    "#;
    let out = eval(kod);
    let mut lines = out.lines();
    assert_eq!(lines.next(), Some("2"));
    assert_eq!(lines.next(), Some("4"));
}

#[test]
fn test_siniflar() {
    let kod = r#"
        kisi sınıf olsun {
            yas = 20 olsun
            buyu fonksiyon olsun {
                kendisi'nin yas'ı = kendisi'nin yas'ı + 1 olsun
            }
        }
        k1 = kisi() olsun
        k1.buyu()
        k1'in yas'ı yazdır
    "#;
    assert_eq!(eval(kod).trim(), "21");
}
