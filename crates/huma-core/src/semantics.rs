//! Normative runtime semantics shared by every Hüma execution backend.
//!
//! Keeping value rules here prevents the interpreter, bytecode VM and future
//! backends from independently inventing coercion, truthiness and comparison
//! behaviour.

use crate::token::Token;
use crate::value::Deger;
use std::collections::HashSet;
use std::rc::Rc;

const MAX_RUNTIME_TEXT_BYTES: usize = 64 * 1024 * 1024;

/// Return the single normative truthiness result for a Hüma value.
pub fn dogru_mu(deger: &Deger) -> Result<bool, String> {
    let result = match deger {
        Deger::Sayi(n) => *n != 0.0,
        Deger::Metin(s) => !s.is_empty(),
        Deger::Bayt(b) => !b.is_empty(),
        Deger::Liste(l) => !l
            .try_borrow()
            .map_err(|_| "Doğruluk denetiminde liste kullanımda".to_string())?
            .is_empty(),
        Deger::Sozluk(m) => !m
            .try_borrow()
            .map_err(|_| "Doğruluk denetiminde sözlük kullanımda".to_string())?
            .is_empty(),
        Deger::Vektor(v) => !v
            .try_borrow()
            .map_err(|_| "Doğruluk denetiminde vektör kullanımda".to_string())?
            .is_empty(),
        Deger::Matris {
            satirlar, sutunlar, ..
        } => *satirlar != 0 && *sutunlar != 0,
        Deger::Bos => false,
        Deger::Hata(_) => false,
        _ => true,
    };
    Ok(result)
}

fn tur_adi(deger: &Deger) -> &'static str {
    match deger {
        Deger::Sayi(_) => "sayı",
        Deger::Metin(_) => "metin",
        Deger::Bayt(_) => "bayt",
        Deger::Liste(_) => "liste",
        Deger::GorevId(_) => "görev",
        Deger::Bos => "boş",
        Deger::Fonksiyon { .. }
        | Deger::BytecodeFonksiyon { .. }
        | Deger::DahiliFonksiyon(_)
        | Deger::BaglamliDahiliFonksiyon(_) => "fonksiyon",
        Deger::Sinif { .. } => "sınıf",
        Deger::Nesne { .. } => "nesne",
        Deger::Sozluk(_) => "sözlük",
        Deger::Hata(_) => "hata",
        Deger::Vektor(_) => "vektör",
        Deger::Matris { .. } => "matris",
        Deger::Tensor(_) => "tensor",
    }
}

fn sonlu_sayi(islem: &str, sonuc: f64) -> Result<Deger, String> {
    if sonuc.is_finite() {
        Ok(Deger::Sayi(sonuc))
    } else {
        Err(format!("{islem} işleminin sonucu sonlu sayı sınırını aştı"))
    }
}

fn sayisal_cift<'a>(islem: &str, sol: &'a Deger, sag: &'a Deger) -> Result<(f64, f64), String> {
    match (sol, sag) {
        (Deger::Sayi(a), Deger::Sayi(b)) if a.is_finite() && b.is_finite() => Ok((*a, *b)),
        (Deger::Sayi(_), Deger::Sayi(_)) => Err(format!("{islem} işlemi sonlu sayılar gerektirir")),
        _ => Err(format!(
            "{islem} işlemi iki sayı gerektirir; {} ve {} geldi",
            tur_adi(sol),
            tur_adi(sag)
        )),
    }
}

fn hata_degerini_yay(sol: &Deger, sag: &Deger) -> Result<(), String> {
    if let Deger::Hata(message) = sol {
        return Err(message.clone());
    }
    if let Deger::Hata(message) = sag {
        return Err(message.clone());
    }
    Ok(())
}

/// Döngüsel kapsayıcıları da güvenle karşılaştıran normatif eşitlik.
pub fn esit_mi(left: &Deger, right: &Deger) -> Result<bool, String> {
    fn compare(
        left: &Deger,
        right: &Deger,
        active: &mut HashSet<(usize, usize, u8)>,
        depth: usize,
    ) -> Result<bool, String> {
        if depth > 128 {
            return Err("Eşitlik denetiminde azami iç içe değer derinliği aşıldı".to_string());
        }
        match (left, right) {
            (Deger::Sayi(a), Deger::Sayi(b)) => Ok(a == b),
            (Deger::Metin(a), Deger::Metin(b)) => Ok(a == b),
            (Deger::Bayt(a), Deger::Bayt(b)) => Ok(a == b),
            (Deger::GorevId(a), Deger::GorevId(b)) => Ok(a == b),
            (Deger::Bos, Deger::Bos) => Ok(true),
            (Deger::Hata(a), Deger::Hata(b)) => Ok(a == b),
            (Deger::Liste(a), Deger::Liste(b)) => {
                if Rc::ptr_eq(a, b) {
                    return Ok(true);
                }
                let pair = (
                    Rc::as_ptr(a) as *const () as usize,
                    Rc::as_ptr(b) as *const () as usize,
                    1,
                );
                if !active.insert(pair) {
                    return Ok(true);
                }
                let a = a
                    .try_borrow()
                    .map_err(|_| "Eşitlik denetiminde sol liste kullanımda".to_string())?;
                let b = b
                    .try_borrow()
                    .map_err(|_| "Eşitlik denetiminde sağ liste kullanımda".to_string())?;
                let result = if a.len() != b.len() {
                    false
                } else {
                    let mut equal = true;
                    for (left, right) in a.iter().zip(b.iter()) {
                        if !compare(left, right, active, depth + 1)? {
                            equal = false;
                            break;
                        }
                    }
                    equal
                };
                active.remove(&pair);
                Ok(result)
            }
            (Deger::Sozluk(a), Deger::Sozluk(b)) => compare_maps(a, b, active, depth, 2),
            (
                Deger::Nesne {
                    sinif_adi: a_class,
                    alanlar: a,
                    module_kimligi: a_module,
                },
                Deger::Nesne {
                    sinif_adi: b_class,
                    alanlar: b,
                    module_kimligi: b_module,
                },
            ) => {
                if a_class != b_class || a_module != b_module {
                    Ok(false)
                } else {
                    compare_maps(a, b, active, depth, 3)
                }
            }
            (Deger::Vektor(a), Deger::Vektor(b)) => {
                if Rc::ptr_eq(a, b) {
                    return Ok(true);
                }
                let a = a
                    .try_borrow()
                    .map_err(|_| "Eşitlik denetiminde sol vektör kullanımda".to_string())?;
                let b = b
                    .try_borrow()
                    .map_err(|_| "Eşitlik denetiminde sağ vektör kullanımda".to_string())?;
                Ok(*a == *b)
            }
            (
                Deger::Matris {
                    satirlar: ar,
                    sutunlar: ac,
                    veri: a,
                },
                Deger::Matris {
                    satirlar: br,
                    sutunlar: bc,
                    veri: b,
                },
            ) => {
                if ar != br || ac != bc {
                    return Ok(false);
                }
                if Rc::ptr_eq(a, b) {
                    return Ok(true);
                }
                let a = a
                    .try_borrow()
                    .map_err(|_| "Eşitlik denetiminde sol matris kullanımda".to_string())?;
                let b = b
                    .try_borrow()
                    .map_err(|_| "Eşitlik denetiminde sağ matris kullanımda".to_string())?;
                Ok(*a == *b)
            }
            (Deger::Tensor(a), Deger::Tensor(b)) => Ok(a.id == b.id),
            (
                Deger::Fonksiyon { .. }
                | Deger::BytecodeFonksiyon { .. }
                | Deger::DahiliFonksiyon(_)
                | Deger::BaglamliDahiliFonksiyon(_)
                | Deger::Sinif { .. },
                _,
            )
            | (
                _,
                Deger::Fonksiyon { .. }
                | Deger::BytecodeFonksiyon { .. }
                | Deger::DahiliFonksiyon(_)
                | Deger::BaglamliDahiliFonksiyon(_)
                | Deger::Sinif { .. },
            ) => Err("Fonksiyon ve sınıf değerleri eşitlikle karşılaştırılamaz".to_string()),
            _ => Ok(false),
        }
    }

    fn compare_maps(
        left: &Rc<std::cell::RefCell<std::collections::HashMap<String, Deger>>>,
        right: &Rc<std::cell::RefCell<std::collections::HashMap<String, Deger>>>,
        active: &mut HashSet<(usize, usize, u8)>,
        depth: usize,
        kind: u8,
    ) -> Result<bool, String> {
        if Rc::ptr_eq(left, right) {
            return Ok(true);
        }
        let pair = (
            Rc::as_ptr(left) as *const () as usize,
            Rc::as_ptr(right) as *const () as usize,
            kind,
        );
        if !active.insert(pair) {
            return Ok(true);
        }
        let left = left
            .try_borrow()
            .map_err(|_| "Eşitlik denetiminde sol anahtarlı değer kullanımda".to_string())?;
        let right = right
            .try_borrow()
            .map_err(|_| "Eşitlik denetiminde sağ anahtarlı değer kullanımda".to_string())?;
        let result = if left.len() != right.len() {
            false
        } else {
            let mut equal = true;
            for (key, left_value) in left.iter() {
                let Some(right_value) = right.get(key) else {
                    equal = false;
                    break;
                };
                if !compare(left_value, right_value, active, depth + 1)? {
                    equal = false;
                    break;
                }
            }
            equal
        };
        active.remove(&pair);
        Ok(result)
    }

    compare(left, right, &mut HashSet::new(), 0)
}

fn metinleri_birlestir(left: &str, right: &str) -> Result<Deger, String> {
    let length = left
        .len()
        .checked_add(right.len())
        .ok_or_else(|| "Metin birleştirme boyutu taştı".to_string())?;
    if length > MAX_RUNTIME_TEXT_BYTES {
        return Err(format!(
            "Metin birleştirme {} bayt sınırını aşıyor",
            MAX_RUNTIME_TEXT_BYTES
        ));
    }
    let mut output = String::with_capacity(length);
    output.push_str(left);
    output.push_str(right);
    Ok(Deger::Metin(output))
}

/// Evaluate one binary operation according to the normative Hüma value model.
///
/// Numeric operators never coerce text to a number. `+` concatenates only when
/// at least one operand is text. Ordering is defined for number-number and
/// text-text pairs; all other mixed pairs are type errors.
pub fn ikili_islem(operator: &Token, sol: Deger, sag: Deger) -> Result<Deger, String> {
    hata_degerini_yay(&sol, &sag)?;

    match operator {
        Token::Esittir | Token::EsitEsittir => {
            Ok(Deger::Sayi(if esit_mi(&sol, &sag)? { 1.0 } else { 0.0 }))
        }
        Token::EsitDegil => Ok(Deger::Sayi(if esit_mi(&sol, &sag)? { 0.0 } else { 1.0 })),
        Token::Arti => match (sol, sag) {
            (Deger::Sayi(a), Deger::Sayi(b)) if a.is_finite() && b.is_finite() => {
                sonlu_sayi("Toplama", a + b)
            }
            (Deger::Sayi(_), Deger::Sayi(_)) => {
                Err("Toplama işlemi sonlu sayılar gerektirir".to_string())
            }
            (Deger::Metin(a), Deger::Metin(b)) => metinleri_birlestir(&a, &b),
            (Deger::Metin(a), b) => {
                let remaining = MAX_RUNTIME_TEXT_BYTES.saturating_sub(a.len());
                let rendered = b
                    .to_string_limited(remaining)
                    .map_err(|_| "Metin birleştirme sınırı aşıldı".to_string())?;
                metinleri_birlestir(&a, &rendered)
            }
            (a, Deger::Metin(b)) => {
                let remaining = MAX_RUNTIME_TEXT_BYTES.saturating_sub(b.len());
                let rendered = a
                    .to_string_limited(remaining)
                    .map_err(|_| "Metin birleştirme sınırı aşıldı".to_string())?;
                metinleri_birlestir(&rendered, &b)
            }
            (a, b) => Err(format!(
                "Toplama işlemi iki sayı veya en az bir metin gerektirir; {} ve {} geldi",
                tur_adi(&a),
                tur_adi(&b)
            )),
        },
        Token::Eksi => {
            let (a, b) = sayisal_cift("Çıkarma", &sol, &sag)?;
            sonlu_sayi("Çıkarma", a - b)
        }
        Token::Carpi => {
            let (a, b) = sayisal_cift("Çarpma", &sol, &sag)?;
            sonlu_sayi("Çarpma", a * b)
        }
        Token::Bolnu => {
            let (a, b) = sayisal_cift("Bölme", &sol, &sag)?;
            if b == 0.0 {
                Err("Sıfıra bölme hatası".to_string())
            } else {
                sonlu_sayi("Bölme", a / b)
            }
        }
        Token::Mod => {
            let (a, b) = sayisal_cift("Kalan", &sol, &sag)?;
            if b == 0.0 {
                Err("Sıfıra göre kalan hesaplanamaz".to_string())
            } else {
                sonlu_sayi("Kalan", a % b)
            }
        }
        Token::Kucuktur | Token::Buyuktur | Token::KucukEsit | Token::BuyukEsit => {
            let sonuc = match (&sol, &sag) {
                (Deger::Sayi(a), Deger::Sayi(b)) if a.is_finite() && b.is_finite() => {
                    match operator {
                        Token::Kucuktur => a < b,
                        Token::Buyuktur => a > b,
                        Token::KucukEsit => a <= b,
                        Token::BuyukEsit => a >= b,
                        _ => return Err("İç hata: geçersiz karşılaştırma operatörü".to_string()),
                    }
                }
                (Deger::Metin(a), Deger::Metin(b)) => match operator {
                    Token::Kucuktur => a < b,
                    Token::Buyuktur => a > b,
                    Token::KucukEsit => a <= b,
                    Token::BuyukEsit => a >= b,
                    _ => return Err("İç hata: geçersiz karşılaştırma operatörü".to_string()),
                },
                (Deger::Sayi(_), Deger::Sayi(_)) => {
                    return Err("Karşılaştırma sonlu sayılar gerektirir".to_string())
                }
                _ => {
                    return Err(format!(
                        "Karşılaştırma iki sayı veya iki metin gerektirir; {} ve {} geldi",
                        tur_adi(&sol),
                        tur_adi(&sag)
                    ))
                }
            };
            Ok(Deger::Sayi(if sonuc { 1.0 } else { 0.0 }))
        }
        _ => Err(format!("Desteklenmeyen ikili operatör: {operator}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{dogru_mu, esit_mi, ikili_islem};
    use crate::token::Token;
    use crate::value::Deger;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[test]
    fn bos_sozluk_ve_liste_yanlistir() {
        assert!(!dogru_mu(&Deger::Liste(Rc::new(RefCell::new(Vec::new())))).unwrap());
        assert!(!dogru_mu(&Deger::Sozluk(Rc::new(RefCell::new(HashMap::new())))).unwrap());
    }

    #[test]
    fn sayisal_operator_metni_sayiya_cevirmez() {
        let error = ikili_islem(
            &Token::Carpi,
            Deger::Metin("2".to_string()),
            Deger::Sayi(3.0),
        )
        .expect_err("metin örtük olarak sayıya çevrilmemeli");
        assert!(error.contains("iki sayı"));
    }

    #[test]
    fn sonlu_olmayan_aritmetik_sonucu_reddedilir() {
        let error = ikili_islem(&Token::Carpi, Deger::Sayi(f64::MAX), Deger::Sayi(f64::MAX))
            .expect_err("taşan sonuç reddedilmeli");
        assert!(error.contains("sonlu sayı"));
    }

    #[test]
    fn dongusel_listeler_esitlikte_yigin_tasirmadan_karsilastirilir() {
        let left = Rc::new(RefCell::new(Vec::new()));
        let right = Rc::new(RefCell::new(Vec::new()));
        left.borrow_mut().push(Deger::Liste(Rc::clone(&left)));
        right.borrow_mut().push(Deger::Liste(Rc::clone(&right)));
        assert!(esit_mi(
            &Deger::Liste(Rc::clone(&left)),
            &Deger::Liste(Rc::clone(&right))
        )
        .expect("Eş yapılı döngüler karşılaştırılabilmeli"));

        left.borrow_mut().push(Deger::Sayi(1.0));
        right.borrow_mut().push(Deger::Sayi(2.0));
        assert!(!esit_mi(&Deger::Liste(left), &Deger::Liste(right))
            .expect("Farklı döngüler karşılaştırılabilmeli"));
    }

    #[test]
    fn aktif_borclu_koleksiyon_sessiz_sonuca_donusmez() {
        let list = Rc::new(RefCell::new(vec![Deger::Sayi(1.0)]));
        let other = Rc::new(RefCell::new(vec![Deger::Sayi(1.0)]));
        let _borrow = list.borrow_mut();
        assert!(dogru_mu(&Deger::Liste(Rc::clone(&list)))
            .expect_err("Doğruluk borrow çakışmasını bildirmeli")
            .contains("kullanımda"));
        assert!(
            esit_mi(&Deger::Liste(Rc::clone(&list)), &Deger::Liste(other))
                .expect_err("Eşitlik borrow çakışmasını bildirmeli")
                .contains("kullanımda")
        );
    }
}
