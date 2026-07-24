// huma-core crate'i "huma" olarak takma adlandırıyoruz,
// böylece bu dosyadaki "use huma::..." importları doğru çalışır.
use huma_core as huma;

use huma::lexer::Lexer;
use huma::parser::Parser;
use huma::interpreter::Yorumlayici;
use std::rc::Rc;
use std::cell::RefCell;

// ─────────────────────────────────────────────
// Yardımcı fonksiyon — kodu çalıştırıp çıktısını döndürür
// ─────────────────────────────────────────────
fn eval(kod: &str) -> String {
    let buffer = Rc::new(RefCell::new(String::new()));
    let mut yorumlayici = Yorumlayici::new().with_output_buffer(Rc::clone(&buffer));

    let lexer = Lexer::new(kod);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    yorumlayici.yorumla(program);

    let sonuc = buffer.borrow().clone();
    sonuc
}

// ─────────────────────────────────────────────
// Temel dil özellikleri — mutlu yol (happy path)
// ─────────────────────────────────────────────

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
        i < 3 olduğu sürece {
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
        dizi = [1, 2, 3] olsun
        dizi[1]'i yazdır
        dizi = listeye_ekle(dizi, 4) olsun
        dizi[3]'ü yazdır
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

// ─────────────────────────────────────────────
// Görev 1 regresyon testleri — üç kritik crash
// ─────────────────────────────────────────────

/// REGRESYON: Derin özyineleme artık panik yerine anlamlı hata döndürmeli.
/// call_depth sıfıra dönmeli ve program paniklememeli.
#[test]
fn test_derin_rekursiyon_hatasi() {
    let kod = r#"
        rekursiyon fonksiyon olsun {
            rekursiyon()
        }
        rekursiyon()
    "#;
    let mut interp = Yorumlayici::new();
    let lexer = Lexer::new(kod);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    // Panik atmamalı
    interp.yorumla(program);
    // Derinlik çıkıştan sonra sıfırlanmış olmalı
    assert_eq!(interp.call_depth, 0);
}

/// REGRESYON: Sayı gibi çağrılamayan bir değer çağrıldığında panik değil,
/// Deger::Hata döndürmeli; program sonlanabilmeli.
#[test]
fn test_cagrilamayan_deger_hatasi() {
    let kod = r#"
        x = 42 olsun
        x()
    "#;
    let mut interp = Yorumlayici::new();
    let lexer = Lexer::new(kod);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    // Panik atmadan tamamlanmalı
    interp.yorumla(program);
}

/// REGRESYON: VM iç içe fonksiyon çağrılarında panik atmamalı.
#[test]
fn test_vm_fonksiyon_cagrisi() {
    let kod = r#"
        yardimci fonksiyon olsun {
            "Merhaba"'yı yazdır
        }
        selamla fonksiyon olsun {
            yardimci()
        }
        selamla()
    "#;
    let lexer = Lexer::new(kod);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse_program();
    let mut derleyici = huma::compiler::Derleyici::new();
    let prog = derleyici.derle(ast);
    let mut vm = huma::vm::VM::new(prog);
    // Panik atmadan çalışmalı
    vm.run();
}

// ─────────────────────────────────────────────
// HTTP isteği — ağ bağlantısı gerektiren entegrasyon testi
// ─────────────────────────────────────────────

#[test]
fn test_bekle_http_istekleri() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let body = "OK";
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });

    let kod = format!(
        r#"
        yanit = bekle dahili_istek("GET", "http://{}/", boş, boş) olsun
        yanit'in içerik'i yazdır
        "#,
        addr
    );

    assert_eq!(eval(&kod).trim(), "OK");
}
