use crate::ast::{Ifade, Komut};
use crate::builtin_files;
use crate::error::{HumaError, HumaResult, RuntimeDiagnostic, SourceSpan, StackFrame};
use crate::gc::Gc;
use crate::gc::HeapSweepGuard;
use crate::token::Token;
use crate::value::{BuiltinRuntime, Deger};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use regex::Regex;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as FmtWrite;
use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use unicode_normalization::UnicodeNormalization;

const EN_BUYUK_GUVENLI_SAYISAL_KIMLIK: u64 = (1_u64 << 53) - 1;
const EN_FAZLA_DOSYA_BYTES: usize = 64 * 1024 * 1024;
const EN_FAZLA_BUILTIN_OGE: usize = 1_000_000;
const EN_FAZLA_DESEN_BYTES: usize = 1024 * 1024;
const EN_FAZLA_GIRDI_BYTES: usize = 1024 * 1024;
const EN_FAZLA_SAYISAL_ISLEM: usize = 100_000_000;
// AST yorumlayıcısı çağrıları Rust yığını üzerinde yürütür. Bu sınır, hata
// denetimi devreye girmeden önce ev sahibi sürecin yığın taşmasıyla kapanmasını
// önler. Daha derin çağrılar yığın tabanlı VM'de çalıştırılmalıdır.
const INTERPRETER_MAX_CALL_DEPTH: usize = 32;

const EN_FAZLA_TENSOR_ELEMANI: usize = 10_000_000;

fn boyut_dogrula(deger: f64, islem: &str, sifira_izin_ver: bool) -> Result<usize, String> {
    let alt_sinir = if sifira_izin_ver { 0.0 } else { 1.0 };
    if !deger.is_finite() || deger.fract() != 0.0 || deger < alt_sinir {
        let nitelik = if sifira_izin_ver {
            "negatif olmayan tamsayı"
        } else {
            "pozitif tamsayı"
        };
        return Err(format!("{islem}: boyut {nitelik} olmalı"));
    }
    if deger > EN_FAZLA_TENSOR_ELEMANI as f64 {
        return Err(format!(
            "{islem}: boyut güvenlik sınırını ({EN_FAZLA_TENSOR_ELEMANI}) aşıyor"
        ));
    }
    Ok(deger as usize)
}

fn eleman_sayisi_dogrula(satirlar: usize, sutunlar: usize, islem: &str) -> Result<usize, String> {
    let eleman_sayisi = satirlar
        .checked_mul(sutunlar)
        .ok_or_else(|| format!("{islem}: boyut çarpımı taştı"))?;
    if eleman_sayisi > EN_FAZLA_TENSOR_ELEMANI {
        return Err(format!(
            "{islem}: {eleman_sayisi} eleman güvenlik sınırını \
             ({EN_FAZLA_TENSOR_ELEMANI}) aşıyor"
        ));
    }
    Ok(eleman_sayisi)
}

fn indeks_dogrula(deger: f64, uzunluk: usize, islem: &str) -> Result<usize, String> {
    if !deger.is_finite() || deger < 0.0 || deger.fract() != 0.0 {
        return Err(format!("{islem}: indeks negatif olmayan tamsayı olmalı"));
    }
    if deger >= uzunluk as f64 {
        return Err(format!(
            "{islem}: indeks sınır dışında: {deger} (uzunluk {uzunluk})"
        ));
    }
    Ok(deger as usize)
}

fn yetenek_hatasi(capability: crate::capability::Capability, operation: &str) -> Option<Deger> {
    crate::capability::require(capability, operation)
        .err()
        .map(Deger::Hata)
}

fn read_file_limited(path: impl AsRef<Path>, operation_name: &str) -> Result<Vec<u8>, String> {
    let path = path.as_ref();
    let file = std::fs::File::open(path)
        .map_err(|error| format!("{operation_name}: '{}': {}", path.display(), error))?;
    if let Ok(metadata) = file.metadata() {
        if metadata.len() > EN_FAZLA_DOSYA_BYTES as u64 {
            return Err(format!(
                "{operation_name}: dosya {} baytlık güvenlik sınırını aşıyor",
                EN_FAZLA_DOSYA_BYTES
            ));
        }
    }
    let mut bytes = Vec::new();
    file.take((EN_FAZLA_DOSYA_BYTES as u64) + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("{operation_name}: '{}': {}", path.display(), error))?;
    if bytes.len() > EN_FAZLA_DOSYA_BYTES {
        return Err(format!(
            "{operation_name}: dosya {} baytlık güvenlik sınırını aşıyor",
            EN_FAZLA_DOSYA_BYTES
        ));
    }
    Ok(bytes)
}

fn read_utf8_file_limited(path: impl AsRef<Path>, operation_name: &str) -> Result<String, String> {
    String::from_utf8(read_file_limited(path, operation_name)?)
        .map_err(|error| format!("{operation_name}: dosya geçerli UTF-8 değil: {error}"))
}

struct LimitedBuffer {
    bytes: Vec<u8>,
    limit: usize,
}

struct LimitedText {
    text: String,
    limit: usize,
}

impl LimitedText {
    fn new(limit: usize) -> Self {
        Self {
            text: String::new(),
            limit,
        }
    }
}

impl std::fmt::Write for LimitedText {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        let next_size = self
            .text
            .len()
            .checked_add(text.len())
            .ok_or(std::fmt::Error)?;
        if next_size > self.limit {
            return Err(std::fmt::Error);
        }
        self.text.push_str(text);
        Ok(())
    }
}

impl LimitedBuffer {
    fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
        }
    }
}

impl Write for LimitedBuffer {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let next_size = self
            .bytes
            .len()
            .checked_add(buffer.len())
            .ok_or_else(|| io::Error::new(io::ErrorKind::FileTooLarge, "çıktı boyutu taştı"))?;
        if next_size > self.limit {
            return Err(io::Error::new(
                io::ErrorKind::FileTooLarge,
                "çıktı güvenlik sınırını aşıyor",
            ));
        }
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn json_serialize_limited(
    value: &serde_json::Value,
    pretty: bool,
    operation_name: &str,
) -> Result<String, String> {
    let mut output = LimitedBuffer::new(EN_FAZLA_DOSYA_BYTES);
    let result = if pretty {
        serde_json::to_writer_pretty(&mut output, value)
    } else {
        serde_json::to_writer(&mut output, value)
    };
    result.map_err(|error| format!("{operation_name}: JSON yazılamadı: {error}"))?;
    String::from_utf8(output.bytes)
        .map_err(|error| format!("{operation_name}: JSON çıktısı UTF-8 değil: {error}"))
}

fn display_value_limited(value: &Deger, operation_name: &str) -> Result<String, String> {
    value
        .to_string_limited(EN_FAZLA_DOSYA_BYTES)
        .map_err(|error| format!("{operation_name}: {error}"))
}

fn replacement_output_size(
    text: &str,
    pattern: &str,
    replacement: &str,
    match_count: usize,
    operation_name: &str,
) -> Result<usize, String> {
    let removed = pattern
        .len()
        .checked_mul(match_count)
        .ok_or_else(|| format!("{operation_name}: çıktı boyutu hesabı taştı"))?;
    let added = replacement
        .len()
        .checked_mul(match_count)
        .ok_or_else(|| format!("{operation_name}: çıktı boyutu hesabı taştı"))?;
    let result = text
        .len()
        .checked_sub(removed)
        .and_then(|size| size.checked_add(added))
        .ok_or_else(|| format!("{operation_name}: çıktı boyutu hesabı taştı"))?;
    if result > EN_FAZLA_DOSYA_BYTES {
        return Err(format!(
            "{operation_name}: çıktı {} bayt sınırını aşıyor",
            EN_FAZLA_DOSYA_BYTES
        ));
    }
    Ok(result)
}

fn compile_regex(pattern: &str, operation_name: &str) -> Result<Regex, String> {
    if pattern.len() > EN_FAZLA_DESEN_BYTES {
        return Err(format!(
            "{operation_name}: desen {} bayt sınırını aşıyor",
            EN_FAZLA_DESEN_BYTES
        ));
    }
    Regex::new(pattern).map_err(|error| format!("{operation_name}: geçersiz desen — {error}"))
}

fn validate_regex_text(text: &str, operation_name: &str) -> Result<(), String> {
    if text.len() > EN_FAZLA_DOSYA_BYTES {
        Err(format!(
            "{operation_name}: metin {} bayt sınırını aşıyor",
            EN_FAZLA_DOSYA_BYTES
        ))
    } else {
        Ok(())
    }
}

fn unary_numeric_builtin(
    args: Vec<Deger>,
    operation_name: &str,
    operation: fn(f64) -> f64,
) -> Deger {
    match args.as_slice() {
        [Deger::Sayi(value)] if value.is_finite() => {
            let result = operation(*value);
            if result.is_finite() {
                Deger::Sayi(result)
            } else {
                Deger::Hata(format!(
                    "{operation_name}: işlem sonlu olmayan sonuç üretti"
                ))
            }
        }
        [Deger::Sayi(_)] => Deger::Hata(format!("{operation_name}: sayı sonlu olmalıdır")),
        [other] => Deger::Hata(format!(
            "{operation_name}: sayı bekleniyordu; {} geldi",
            other
        )),
        _ => Deger::Hata(format!(
            "{operation_name}: tam olarak 1 argüman bekleniyordu; {} geldi",
            args.len()
        )),
    }
}

fn positive_unary_numeric_builtin(
    args: Vec<Deger>,
    operation_name: &str,
    operation: fn(f64) -> f64,
) -> Deger {
    match args.as_slice() {
        [Deger::Sayi(value)] if value.is_finite() && *value > 0.0 => {
            let result = operation(*value);
            if result.is_finite() {
                Deger::Sayi(result)
            } else {
                Deger::Hata(format!(
                    "{operation_name}: işlem sonlu olmayan sonuç üretti"
                ))
            }
        }
        [Deger::Sayi(_)] => Deger::Hata(format!("{operation_name}: pozitif ve sonlu sayı gerekir")),
        [other] => Deger::Hata(format!(
            "{operation_name}: sayı bekleniyordu; {} geldi",
            other
        )),
        _ => Deger::Hata(format!(
            "{operation_name}: tam olarak 1 argüman bekleniyordu; {} geldi",
            args.len()
        )),
    }
}

fn value_to_finite_vector(value: &Deger, operation_name: &str) -> Result<Vec<f64>, String> {
    let values = match value {
        Deger::Vektor(vector) => vector
            .try_borrow()
            .map_err(|_| format!("{operation_name}: vektör kullanımda"))?
            .clone(),
        Deger::Liste(list) => {
            let borrowed = list
                .try_borrow()
                .map_err(|_| format!("{operation_name}: liste kullanımda"))?;
            let mut values = Vec::with_capacity(borrowed.len());
            for (index, value) in borrowed.iter().enumerate() {
                match value {
                    Deger::Sayi(number) if number.is_finite() => values.push(*number),
                    Deger::Sayi(_) => {
                        return Err(format!(
                            "{operation_name}: {index}. eleman sonlu sayı olmalıdır"
                        ))
                    }
                    other => {
                        return Err(format!(
                            "{operation_name}: {index}. eleman sayı olmalıdır; {} geldi",
                            other
                        ))
                    }
                }
            }
            values
        }
        other => {
            return Err(format!(
                "{operation_name}: vektör veya sayı listesi bekleniyordu; {} geldi",
                other
            ))
        }
    };
    if values.iter().any(|value| !value.is_finite()) {
        return Err(format!(
            "{operation_name}: bütün vektör elemanları sonlu olmalıdır"
        ));
    }
    Ok(values)
}

fn value_to_finite_matrix(
    value: &Deger,
    operation_name: &str,
) -> Result<(usize, usize, Vec<f64>), String> {
    let Deger::Matris {
        satirlar,
        sutunlar,
        veri,
    } = value
    else {
        return Err(format!(
            "{operation_name}: matris bekleniyordu; {} geldi",
            value
        ));
    };
    let expected = eleman_sayisi_dogrula(*satirlar, *sutunlar, operation_name)?;
    let values = veri
        .try_borrow()
        .map_err(|_| format!("{operation_name}: matris kullanımda"))?;
    if values.len() != expected {
        return Err(format!("{operation_name}: bozuk matris veri boyutu"));
    }
    if values.iter().any(|value| !value.is_finite()) {
        return Err(format!(
            "{operation_name}: bütün matris elemanları sonlu olmalıdır"
        ));
    }
    Ok((*satirlar, *sutunlar, values.clone()))
}

fn insert_keyed_value(
    values: &Gc<HashMap<String, Deger>>,
    key: String,
    value: Deger,
    maximum_items: usize,
    operation_name: &str,
) -> Result<(), String> {
    let mut values = values
        .try_borrow_mut()
        .map_err(|_| format!("{operation_name}: hedef kullanımda"))?;
    if !values.contains_key(&key) && values.len() >= maximum_items {
        return Err(format!(
            "{operation_name}: öğe sınırı aşıldı: {}",
            maximum_items
        ));
    }
    values.insert(key, value);
    Ok(())
}

fn get_keyed_value(
    values: &Gc<HashMap<String, Deger>>,
    key: &str,
    operation_name: &str,
) -> Result<Option<Deger>, String> {
    values
        .try_borrow()
        .map_err(|_| format!("{operation_name}: hedef kullanımda"))
        .map(|values| values.get(key).cloned())
}

fn unary_matrix_builtin(
    args: Vec<Deger>,
    operation_name: &str,
    operation: fn(f64) -> f64,
) -> Deger {
    let [value] = args.as_slice() else {
        return Deger::Hata(format!(
            "{operation_name}: tam olarak 1 argüman bekleniyordu; {} geldi",
            args.len()
        ));
    };
    let (rows, columns, values) = match value_to_finite_matrix(value, operation_name) {
        Ok(matrix) => matrix,
        Err(error) => return Deger::Hata(error),
    };
    let result = values.into_iter().map(operation).collect::<Vec<_>>();
    if result.iter().any(|value| !value.is_finite()) {
        return Deger::Hata(format!(
            "{operation_name}: işlem sonlu olmayan sonuç üretti"
        ));
    }
    Deger::Matris {
        satirlar: rows,
        sutunlar: columns,
        veri: Gc::from_cell(RefCell::new(result)),
    }
}

fn binary_matrix_builtin(
    args: Vec<Deger>,
    operation_name: &str,
    operation: fn(f64, f64) -> f64,
) -> Deger {
    let [left, right] = args.as_slice() else {
        return Deger::Hata(format!(
            "{operation_name}: tam olarak 2 argüman bekleniyordu; {} geldi",
            args.len()
        ));
    };
    let (left_rows, left_columns, left_values) = match value_to_finite_matrix(left, operation_name)
    {
        Ok(matrix) => matrix,
        Err(error) => return Deger::Hata(error),
    };
    let (right_rows, right_columns, right_values) =
        match value_to_finite_matrix(right, operation_name) {
            Ok(matrix) => matrix,
            Err(error) => return Deger::Hata(error),
        };
    if left_rows != right_rows || left_columns != right_columns {
        return Deger::Hata(format!("{operation_name}: boyutlar eşit olmalıdır"));
    }
    let result = left_values
        .into_iter()
        .zip(right_values)
        .map(|(left, right)| operation(left, right))
        .collect::<Vec<_>>();
    if result.iter().any(|value| !value.is_finite()) {
        return Deger::Hata(format!(
            "{operation_name}: işlem sonlu olmayan sonuç üretti"
        ));
    }
    Deger::Matris {
        satirlar: left_rows,
        sutunlar: left_columns,
        veri: Gc::from_cell(RefCell::new(result)),
    }
}

fn adam_matris_durumu(args: Vec<Deger>) -> Deger {
    let (satirlar, sutunlar) = match args.as_slice() {
        [Deger::Sayi(r), Deger::Sayi(c)] => {
            let satirlar = match boyut_dogrula(*r, "adam_durum_olustur", false) {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            let sutunlar = match boyut_dogrula(*c, "adam_durum_olustur", false) {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            (satirlar, sutunlar)
        }
        [_, _] => {
            return Deger::Hata("adam_durum_olustur: pozitif tamsayı boyutlar gerekir".to_string());
        }
        _ => {
            return Deger::Hata(format!(
                "adam_durum_olustur: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            ));
        }
    };
    let eleman_sayisi = match eleman_sayisi_dogrula(satirlar, sutunlar, "adam_durum_olustur") {
        Ok(deger) => deger,
        Err(hata) => return Deger::Hata(hata),
    };
    let mut durum = HashMap::new();
    durum.insert(
        "m".to_string(),
        Deger::Matris {
            satirlar,
            sutunlar,
            veri: Gc::from_cell(RefCell::new(vec![0.0; eleman_sayisi])),
        },
    );
    durum.insert(
        "v".to_string(),
        Deger::Matris {
            satirlar,
            sutunlar,
            veri: Gc::from_cell(RefCell::new(vec![0.0; eleman_sayisi])),
        },
    );
    durum.insert("adim".to_string(), Deger::Sayi(0.0));
    Deger::Sozluk(Gc::from_cell(RefCell::new(durum)))
}

fn adam_vektor_durumu(args: Vec<Deger>) -> Deger {
    let boyut = match args.as_slice() {
        [Deger::Sayi(n)] => match boyut_dogrula(*n, "adam_vektor_durum_olustur", false) {
            Ok(deger) => deger,
            Err(hata) => return Deger::Hata(hata),
        },
        [_] => {
            return Deger::Hata(
                "adam_vektor_durum_olustur: pozitif tamsayı boyut gerekir".to_string(),
            )
        }
        _ => {
            return Deger::Hata(format!(
                "adam_vektor_durum_olustur: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            ));
        }
    };
    let mut durum = HashMap::new();
    durum.insert(
        "m".to_string(),
        Deger::Vektor(Gc::from_cell(RefCell::new(vec![0.0; boyut]))),
    );
    durum.insert(
        "v".to_string(),
        Deger::Vektor(Gc::from_cell(RefCell::new(vec![0.0; boyut]))),
    );
    durum.insert("adim".to_string(), Deger::Sayi(0.0));
    Deger::Sozluk(Gc::from_cell(RefCell::new(durum)))
}

fn adam_matris_guncelle(args: Vec<Deger>) -> Deger {
    let [weights @ Deger::Matris {
        veri: weights_cell, ..
    }, gradient @ Deger::Matris { .. }, Deger::Sozluk(state), Deger::Sayi(learning_rate)] =
        args.as_slice()
    else {
        return if args.len() == 4 {
            Deger::Hata(
                "adam_matris_guncelle: iki matris, Adam durumu ve öğrenme hızı gerekir".to_string(),
            )
        } else {
            Deger::Hata(format!(
                "adam_matris_guncelle: tam olarak 4 argüman bekleniyordu; {} geldi",
                args.len()
            ))
        };
    };
    if !learning_rate.is_finite() || *learning_rate <= 0.0 {
        return Deger::Hata(
            "adam_matris_guncelle: öğrenme hızı pozitif ve sonlu olmalı".to_string(),
        );
    }
    let (rows, columns, weights_values) =
        match value_to_finite_matrix(weights, "adam_matris_guncelle") {
            Ok(matrix) => matrix,
            Err(error) => return Deger::Hata(error),
        };
    let (gradient_rows, gradient_columns, gradient_values) =
        match value_to_finite_matrix(gradient, "adam_matris_guncelle") {
            Ok(matrix) => matrix,
            Err(error) => return Deger::Hata(error),
        };
    if rows != gradient_rows || columns != gradient_columns {
        return Deger::Hata(
            "adam_matris_guncelle: ağırlık ve gradyan boyutları eşit olmalı".to_string(),
        );
    }
    let (m_value, v_value, current_step) = {
        let map = match state.try_borrow() {
            Ok(map) => map,
            Err(_) => {
                return Deger::Hata("adam_matris_guncelle: durum kullanımda".to_string());
            }
        };
        let current_step = match map.get("adim") {
            Some(Deger::Sayi(step)) if step.is_finite() && *step >= 0.0 && step.fract() == 0.0 => {
                *step
            }
            _ => {
                return Deger::Hata(
                    "adam_matris_guncelle: durumdaki adım negatif olmayan tamsayı olmalı"
                        .to_string(),
                )
            }
        };
        (map.get("m").cloned(), map.get("v").cloned(), current_step)
    };
    let (
        Some(m_value @ Deger::Matris { veri: m_cell, .. }),
        Some(v_value @ Deger::Matris { veri: v_cell, .. }),
    ) = (&m_value, &v_value)
    else {
        return Deger::Hata("adam_matris_guncelle: bozuk optimizör durumu".to_string());
    };
    let (m_rows, m_columns, m_values) =
        match value_to_finite_matrix(m_value, "adam_matris_guncelle") {
            Ok(matrix) => matrix,
            Err(error) => return Deger::Hata(error),
        };
    let (v_rows, v_columns, v_values) =
        match value_to_finite_matrix(v_value, "adam_matris_guncelle") {
            Ok(matrix) => matrix,
            Err(error) => return Deger::Hata(error),
        };
    if (m_rows, m_columns) != (rows, columns) || (v_rows, v_columns) != (rows, columns) {
        return Deger::Hata("adam_matris_guncelle: durum boyutu uyuşmuyor".to_string());
    }
    let next_step = current_step + 1.0;
    if !next_step.is_finite() {
        return Deger::Hata("adam_matris_guncelle: adım sayacı taştı".to_string());
    }
    let beta1: f64 = 0.9;
    let beta2: f64 = 0.999;
    let epsilon = 1e-8;
    let correction1 = 1.0 - beta1.powf(next_step);
    let correction2 = 1.0 - beta2.powf(next_step);
    let mut next_weights = Vec::with_capacity(weights_values.len());
    let mut next_m = Vec::with_capacity(weights_values.len());
    let mut next_v = Vec::with_capacity(weights_values.len());
    for index in 0..weights_values.len() {
        let m = beta1 * m_values[index] + (1.0 - beta1) * gradient_values[index];
        let v = beta2 * v_values[index]
            + (1.0 - beta2) * gradient_values[index] * gradient_values[index];
        let weight = weights_values[index]
            - *learning_rate * (m / correction1) / ((v / correction2).sqrt() + epsilon);
        if !m.is_finite() || !v.is_finite() || !weight.is_finite() {
            return Deger::Hata(
                "adam_matris_guncelle: güncelleme sonlu olmayan sonuç üretti".to_string(),
            );
        }
        next_m.push(m);
        next_v.push(v);
        next_weights.push(weight);
    }
    let mut state_map = match state.try_borrow_mut() {
        Ok(map) => map,
        Err(_) => return Deger::Hata("adam_matris_guncelle: durum kullanımda".to_string()),
    };
    let mut weights_output = match weights_cell.try_borrow_mut() {
        Ok(values) => values,
        Err(_) => {
            return Deger::Hata("adam_matris_guncelle: ağırlık matrisi kullanımda".to_string())
        }
    };
    let mut m_output = match m_cell.try_borrow_mut() {
        Ok(values) => values,
        Err(_) => return Deger::Hata("adam_matris_guncelle: m matrisi kullanımda".to_string()),
    };
    let mut v_output = match v_cell.try_borrow_mut() {
        Ok(values) => values,
        Err(_) => return Deger::Hata("adam_matris_guncelle: v matrisi kullanımda".to_string()),
    };
    weights_output.copy_from_slice(&next_weights);
    m_output.copy_from_slice(&next_m);
    v_output.copy_from_slice(&next_v);
    state_map.insert("adim".to_string(), Deger::Sayi(next_step));
    Deger::Bos
}

fn adam_vektor_guncelle(args: Vec<Deger>) -> Deger {
    let [weights @ Deger::Vektor(weights_cell), gradient @ Deger::Vektor(_), Deger::Sozluk(state), Deger::Sayi(learning_rate)] =
        args.as_slice()
    else {
        return if args.len() == 4 {
            Deger::Hata(
                "adam_vektor_guncelle: iki vektör, Adam durumu ve öğrenme hızı gerekir".to_string(),
            )
        } else {
            Deger::Hata(format!(
                "adam_vektor_guncelle: tam olarak 4 argüman bekleniyordu; {} geldi",
                args.len()
            ))
        };
    };
    if !learning_rate.is_finite() || *learning_rate <= 0.0 {
        return Deger::Hata(
            "adam_vektor_guncelle: öğrenme hızı pozitif ve sonlu olmalı".to_string(),
        );
    }
    let weights_values = match value_to_finite_vector(weights, "adam_vektor_guncelle") {
        Ok(values) => values,
        Err(error) => return Deger::Hata(error),
    };
    let gradient_values = match value_to_finite_vector(gradient, "adam_vektor_guncelle") {
        Ok(values) => values,
        Err(error) => return Deger::Hata(error),
    };
    if weights_values.len() != gradient_values.len() {
        return Deger::Hata(
            "adam_vektor_guncelle: ağırlık ve gradyan boyutları eşit olmalı".to_string(),
        );
    }
    let (m_value, v_value, current_step) = {
        let map = match state.try_borrow() {
            Ok(map) => map,
            Err(_) => {
                return Deger::Hata("adam_vektor_guncelle: durum kullanımda".to_string());
            }
        };
        let current_step = match map.get("adim") {
            Some(Deger::Sayi(step)) if step.is_finite() && *step >= 0.0 && step.fract() == 0.0 => {
                *step
            }
            _ => {
                return Deger::Hata(
                    "adam_vektor_guncelle: durumdaki adım negatif olmayan tamsayı olmalı"
                        .to_string(),
                )
            }
        };
        (map.get("m").cloned(), map.get("v").cloned(), current_step)
    };
    let (Some(m_value @ Deger::Vektor(m_cell)), Some(v_value @ Deger::Vektor(v_cell))) =
        (&m_value, &v_value)
    else {
        return Deger::Hata("adam_vektor_guncelle: bozuk optimizör durumu".to_string());
    };
    let m_values = match value_to_finite_vector(m_value, "adam_vektor_guncelle") {
        Ok(values) => values,
        Err(error) => return Deger::Hata(error),
    };
    let v_values = match value_to_finite_vector(v_value, "adam_vektor_guncelle") {
        Ok(values) => values,
        Err(error) => return Deger::Hata(error),
    };
    if weights_values.len() != m_values.len() || weights_values.len() != v_values.len() {
        return Deger::Hata("adam_vektor_guncelle: durum boyutu uyuşmuyor".to_string());
    }
    let next_step = current_step + 1.0;
    if !next_step.is_finite() {
        return Deger::Hata("adam_vektor_guncelle: adım sayacı taştı".to_string());
    }
    let beta1: f64 = 0.9;
    let beta2: f64 = 0.999;
    let epsilon = 1e-8;
    let correction1 = 1.0 - beta1.powf(next_step);
    let correction2 = 1.0 - beta2.powf(next_step);
    let mut next_weights = Vec::with_capacity(weights_values.len());
    let mut next_m = Vec::with_capacity(weights_values.len());
    let mut next_v = Vec::with_capacity(weights_values.len());
    for index in 0..weights_values.len() {
        let m = beta1 * m_values[index] + (1.0 - beta1) * gradient_values[index];
        let v = beta2 * v_values[index]
            + (1.0 - beta2) * gradient_values[index] * gradient_values[index];
        let weight = weights_values[index]
            - *learning_rate * (m / correction1) / ((v / correction2).sqrt() + epsilon);
        if !m.is_finite() || !v.is_finite() || !weight.is_finite() {
            return Deger::Hata(
                "adam_vektor_guncelle: güncelleme sonlu olmayan sonuç üretti".to_string(),
            );
        }
        next_m.push(m);
        next_v.push(v);
        next_weights.push(weight);
    }
    let mut state_map = match state.try_borrow_mut() {
        Ok(map) => map,
        Err(_) => return Deger::Hata("adam_vektor_guncelle: durum kullanımda".to_string()),
    };
    let mut weights_output = match weights_cell.try_borrow_mut() {
        Ok(values) => values,
        Err(_) => {
            return Deger::Hata("adam_vektor_guncelle: ağırlık vektörü kullanımda".to_string())
        }
    };
    let mut m_output = match m_cell.try_borrow_mut() {
        Ok(values) => values,
        Err(_) => return Deger::Hata("adam_vektor_guncelle: m vektörü kullanımda".to_string()),
    };
    let mut v_output = match v_cell.try_borrow_mut() {
        Ok(values) => values,
        Err(_) => return Deger::Hata("adam_vektor_guncelle: v vektörü kullanımda".to_string()),
    };
    weights_output.copy_from_slice(&next_weights);
    m_output.copy_from_slice(&next_m);
    v_output.copy_from_slice(&next_v);
    state_map.insert("adim".to_string(), Deger::Sayi(next_step));
    Deger::Bos
}

pub struct Yorumlayici {
    pub global_degiskenler: HashMap<String, Deger>,
    pub yerel_scopes: Vec<HashMap<String, Deger>>,
    pub donus_degeri: Option<Deger>,
    pub yuklenen_dosyalar: HashSet<String>,
    yuklenmekte_olan_dosyalar: HashSet<String>,
    module_namespaces: HashMap<String, HashMap<String, Deger>>,
    module_environments: HashMap<String, HashMap<String, Deger>>,
    active_exports: Vec<HashSet<String>>,
    active_module_bindings: Vec<HashSet<String>>,
    active_module_calls: Vec<String>,
    pub arama_yolları: Vec<String>,
    pub output_buffer: Option<Rc<RefCell<String>>>,
    pub call_depth: usize,
    runtime_errors: Vec<RuntimeDiagnostic>,
    current_location: Option<SourceSpan>,
    call_stack: Vec<StackFrame>,
    dongu_kontrolu: Option<DonguKontrolu>,
    dongu_derinligi: usize,
    limits: crate::limits::ExecutionLimits,
    executed_steps: u64,
    output_bytes: usize,
    task_awaiter: Option<fn(u64) -> Deger>,
    // En son bırakılmalı: diğer alanların tuttuğu heap kökleri düştükten
    // sonra sahipsiz döngüleri toplar.
    _heap_sweep: HeapSweepGuard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DonguKontrolu {
    Devam,
    Kir,
}

pub fn varsayilan_global_degiskenler() -> HashMap<String, Deger> {
    let mut globals = HashMap::new();
    globals.insert("boş".to_string(), Deger::Bos);
    globals.insert("Boş".to_string(), Deger::Bos);
    globals.insert(
        "adam_durum_olustur".to_string(),
        Deger::DahiliFonksiyon(adam_matris_durumu),
    );
    globals.insert(
        "adam_vektor_durum_olustur".to_string(),
        Deger::DahiliFonksiyon(adam_vektor_durumu),
    );
    globals.insert(
        "adam_matris_guncelle".to_string(),
        Deger::DahiliFonksiyon(adam_matris_guncelle),
    );
    globals.insert(
        "adam_vektor_guncelle".to_string(),
        Deger::DahiliFonksiyon(adam_vektor_guncelle),
    );
    globals.insert(
        "uzunluk".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(value)] => Deger::Sayi(value.chars().count() as f64),
            [Deger::Bayt(value)] => Deger::Sayi(value.len() as f64),
            [Deger::Liste(value)] => match value.try_borrow() {
                Ok(value) => Deger::Sayi(value.len() as f64),
                Err(_) => Deger::Hata("uzunluk: liste kullanımda".to_string()),
            },
            [Deger::Sozluk(value)] => match value.try_borrow() {
                Ok(value) => Deger::Sayi(value.len() as f64),
                Err(_) => Deger::Hata("uzunluk: sözlük kullanımda".to_string()),
            },
            [Deger::Vektor(value)] => match value.try_borrow() {
                Ok(value) => Deger::Sayi(value.len() as f64),
                Err(_) => Deger::Hata("uzunluk: vektör kullanımda".to_string()),
            },
            [other] => Deger::Hata(format!(
                "uzunluk: metin, bayt, liste, sözlük veya vektör bekleniyordu; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "uzunluk: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );
    globals.insert(
        "oku".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let prompt = match args.as_slice() {
                [] => None,
                [Deger::Metin(prompt)] if prompt.len() <= 4096 => Some(prompt),
                [Deger::Metin(_)] => {
                    return Deger::Hata("oku: istem 4096 baytı aşamaz".to_string())
                }
                [other] => {
                    return Deger::Hata(format!("oku: istem metin olmalıdır; {} geldi", other))
                }
                _ => {
                    return Deger::Hata(format!(
                        "oku: 0 veya 1 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                }
            };
            if let Some(prompt) = prompt {
                print!("{}", prompt);
                if let Err(error) = io::stdout().flush() {
                    return Deger::Hata(format!("oku: istem yazılamadı: {error}"));
                }
            }
            let mut input = String::new();
            let stdin = io::stdin();
            let mut limited = stdin.lock().take((EN_FAZLA_GIRDI_BYTES as u64) + 1);
            match limited.read_line(&mut input) {
                Ok(_) if input.len() <= EN_FAZLA_GIRDI_BYTES => {
                    Deger::Metin(input.trim_end_matches(['\r', '\n']).to_string())
                }
                Ok(_) => Deger::Hata(format!(
                    "oku: girdi {} bayt sınırını aşıyor",
                    EN_FAZLA_GIRDI_BYTES
                )),
                Err(error) => Deger::Hata(format!("oku: girdi okunamadı: {error}")),
            }
        }),
    );
    globals.insert(
        "uyut".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Sayi(ms)]
                if ms.is_finite() && *ms >= 0.0 && ms.fract() == 0.0 && *ms <= 60_000.0 =>
            {
                if *ms != 0.0 {
                    thread::sleep(Duration::from_millis(*ms as u64));
                }
                Deger::Bos
            }
            [Deger::Sayi(_)] => Deger::Hata(
                "uyut: milisaniye 0..60000 aralığında negatif olmayan bir tamsayı olmalıdır"
                    .to_string(),
            ),
            [other] => Deger::Hata(format!("uyut: sayı bekleniyordu; {} geldi", other)),
            _ => Deger::Hata(format!(
                "uyut: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );
    globals.insert(
        "zaman".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if !args.is_empty() {
                return Deger::Hata(format!(
                    "zaman: argüman beklenmiyordu; {} geldi",
                    args.len()
                ));
            }
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(duration) => Deger::Sayi(duration.as_secs_f64()),
                Err(error) => Deger::Hata(format!("zaman: sistem saati hatası: {}", error)),
            }
        }),
    );
    globals.insert(
        "listeye_ekle".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Liste(list), value] => {
                let borrowed = match list.try_borrow() {
                    Ok(values) => values,
                    Err(_) => return Deger::Hata("listeye_ekle: liste kullanımda".to_string()),
                };
                if borrowed.len() >= EN_FAZLA_BUILTIN_OGE {
                    return Deger::Hata(format!(
                        "listeye_ekle: liste {} öğelik güvenlik sınırına ulaştı",
                        EN_FAZLA_BUILTIN_OGE
                    ));
                }
                let mut yeni = borrowed.clone();
                yeni.push(value.clone());
                Deger::Liste(Gc::from_cell(RefCell::new(yeni)))
            }
            [other, _] => Deger::Hata(format!(
                "listeye_ekle: ilk argüman liste olmalıdır; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "listeye_ekle: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );
    globals.insert(
        "karekök".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Sayi(value)] if value.is_finite() && *value >= 0.0 => Deger::Sayi(value.sqrt()),
            [Deger::Sayi(_)] => {
                Deger::Hata("karekök: sonlu ve negatif olmayan sayı gerekir".to_string())
            }
            [other] => Deger::Hata(format!("karekök: sayı bekleniyordu; {} geldi", other)),
            _ => Deger::Hata(format!(
                "karekök: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );
    globals.insert(
        "rastgele".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if !args.is_empty() {
                return Deger::Hata(format!(
                    "rastgele: argüman beklenmiyordu; {} geldi",
                    args.len()
                ));
            }
            Deger::Sayi(rand::thread_rng().gen::<f64>())
        }),
    );
    // JSON Fonksiyonları
    globals.insert(
        "nesneden_metine".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [deger] => match deger.to_json_checked() {
                Ok(json) => match json_serialize_limited(&json, true, "nesneden_metine") {
                    Ok(text) => Deger::Metin(text),
                    Err(error) => Deger::Hata(error),
                },
                Err(error) => Deger::Hata(format!("nesneden_metine: {error}")),
            },
            _ => Deger::Hata(format!(
                "nesneden_metine: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );
    globals.insert(
        "metinden_nesneye".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(metin)] if metin.len() <= EN_FAZLA_DOSYA_BYTES => {
                match serde_json::from_str::<serde_json::Value>(metin) {
                    Ok(json) => Deger::from_json_checked(&json)
                        .unwrap_or_else(|error| Deger::Hata(format!("metinden_nesneye: {error}"))),
                    Err(error) => Deger::Hata(format!("metinden_nesneye: geçersiz JSON: {error}")),
                }
            }
            [Deger::Metin(_)] => Deger::Hata(format!(
                "metinden_nesneye: girdi {} bayt sınırını aşıyor",
                EN_FAZLA_DOSYA_BYTES
            )),
            [other] => Deger::Hata(format!(
                "metinden_nesneye: metin bekleniyordu; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "metinden_nesneye: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );
    globals.insert(
        "ortam_değişkeni".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(anahtar)] => {
                if let Some(error) =
                    yetenek_hatasi(crate::capability::Capability::Process, "ortam_değişkeni")
                {
                    return error;
                }
                match std::env::var(anahtar) {
                    Ok(value) => Deger::Metin(value),
                    Err(std::env::VarError::NotPresent) => Deger::Bos,
                    Err(error) => Deger::Hata(format!(
                        "ortam_değişkeni: '{}' okunamadı: {}",
                        anahtar, error
                    )),
                }
            }
            [other] => Deger::Hata(format!(
                "ortam_değişkeni: metin anahtar bekleniyordu; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "ortam_değişkeni: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // ── NLP / Metin İşleme Built-in Fonksiyonları ──────────────────────────

    // küçük_harf(metin) → Türkçe-farkında küçük harf dönüşümü
    globals.insert(
        "küçük_harf".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(s)] => {
                let mut output = LimitedText::new(EN_FAZLA_DOSYA_BYTES);
                for character in s.chars() {
                    let result = match character {
                        'I' => output.write_str("ı"),
                        'İ' => output.write_str("i"),
                        _ => character
                            .to_lowercase()
                            .try_for_each(|lower| output.write_char(lower)),
                    };
                    if result.is_err() {
                        return Deger::Hata(format!(
                            "küçük_harf: çıktı {} bayt sınırını aşıyor",
                            EN_FAZLA_DOSYA_BYTES
                        ));
                    }
                }
                Deger::Metin(output.text)
            }
            [other] => Deger::Hata(format!("küçük_harf: metin bekleniyordu; {} geldi", other)),
            _ => Deger::Hata(format!(
                "küçük_harf: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // büyük_harf(metin) → Türkçe-farkında büyük harf dönüşümü
    globals.insert(
        "büyük_harf".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(s)] => {
                let mut output = LimitedText::new(EN_FAZLA_DOSYA_BYTES);
                for character in s.chars() {
                    let result = match character {
                        'ı' => output.write_str("I"),
                        'i' => output.write_str("İ"),
                        _ => character
                            .to_uppercase()
                            .try_for_each(|upper| output.write_char(upper)),
                    };
                    if result.is_err() {
                        return Deger::Hata(format!(
                            "büyük_harf: çıktı {} bayt sınırını aşıyor",
                            EN_FAZLA_DOSYA_BYTES
                        ));
                    }
                }
                Deger::Metin(output.text)
            }
            [other] => Deger::Hata(format!("büyük_harf: metin bekleniyordu; {} geldi", other)),
            _ => Deger::Hata(format!(
                "büyük_harf: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // böl(metin, ayraç) → Liste döndürür
    globals.insert(
        "böl".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(metin), Deger::Metin(ayrac)] => {
                let parcalar: Vec<Deger> = if ayrac.is_empty() {
                    metin
                        .chars()
                        .map(|character| Deger::Metin(character.to_string()))
                        .collect()
                } else {
                    metin
                        .split(ayrac.as_str())
                        .map(|part| Deger::Metin(part.to_string()))
                        .collect()
                };
                if parcalar.len() > EN_FAZLA_BUILTIN_OGE {
                    return Deger::Hata(format!(
                        "böl: parça sayısı {} öğelik güvenlik sınırını aşıyor",
                        EN_FAZLA_BUILTIN_OGE
                    ));
                }
                Deger::Liste(Gc::from_cell(RefCell::new(parcalar)))
            }
            [_, _] => Deger::Hata("böl: iki argüman da metin olmalıdır".to_string()),
            _ => Deger::Hata(format!(
                "böl: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // birleştir(liste, ayraç) → birleştirilmiş metin
    globals.insert(
        "birleştir".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (list, separator) = match args.as_slice() {
                [Deger::Liste(list)] => (list, ""),
                [Deger::Liste(list), Deger::Metin(separator)] => (list, separator.as_str()),
                [_, _] => {
                    return Deger::Hata(
                        "birleştir: liste ve isteğe bağlı metin ayıracı gerekir".to_string(),
                    )
                }
                _ => {
                    return Deger::Hata(format!(
                        "birleştir: 1 veya 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                }
            };
            let borrowed = match list.try_borrow() {
                Ok(values) => values,
                Err(_) => return Deger::Hata("birleştir: liste kullanımda".to_string()),
            };
            if borrowed.len() > EN_FAZLA_BUILTIN_OGE {
                return Deger::Hata(format!(
                    "birleştir: liste {} öğelik güvenlik sınırını aşıyor",
                    EN_FAZLA_BUILTIN_OGE
                ));
            }
            let mut output = LimitedText::new(EN_FAZLA_DOSYA_BYTES);
            for (index, value) in borrowed.iter().enumerate() {
                if index > 0 && output.write_str(separator).is_err() {
                    return Deger::Hata(format!(
                        "birleştir: çıktı {} bayt sınırını aşıyor",
                        EN_FAZLA_DOSYA_BYTES
                    ));
                }
                match value {
                    Deger::Metin(text) => {
                        if output.write_str(text).is_err() {
                            return Deger::Hata(format!(
                                "birleştir: çıktı {} bayt sınırını aşıyor",
                                EN_FAZLA_DOSYA_BYTES
                            ));
                        }
                    }
                    other => {
                        return Deger::Hata(format!(
                            "birleştir: liste yalnızca metin içermelidir; {} geldi",
                            other
                        ))
                    }
                }
            }
            Deger::Metin(output.text)
        }),
    );

    // değiştir(metin, aranan, yeni) → yeni metin
    globals.insert(
        "değiştir".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(text), Deger::Metin(pattern), Deger::Metin(replacement)] => {
                let match_count = if pattern.is_empty() {
                    text.chars().count().saturating_add(1)
                } else {
                    text.matches(pattern.as_str()).count()
                };
                if let Err(error) =
                    replacement_output_size(text, pattern, replacement, match_count, "değiştir")
                {
                    return Deger::Hata(error);
                }
                Deger::Metin(text.replace(pattern.as_str(), replacement.as_str()))
            }
            [_, _, _] => Deger::Hata("değiştir: üç argüman da metin olmalıdır".to_string()),
            _ => Deger::Hata(format!(
                "değiştir: tam olarak 3 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // kırp(metin) → baştaki ve sondaki boşlukları sil
    globals.insert(
        "kırp".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(text)] => Deger::Metin(text.trim().to_string()),
            [other] => Deger::Hata(format!("kırp: metin bekleniyordu; {} geldi", other)),
            _ => Deger::Hata(format!(
                "kırp: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // tekrar_sayısı(metin, aranan) → kaç kez geçiyor
    globals.insert(
        "tekrar_sayısı".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(text), Deger::Metin(pattern)] => {
                if pattern.is_empty() {
                    Deger::Sayi(0.0)
                } else {
                    Deger::Sayi(text.matches(pattern.as_str()).count() as f64)
                }
            }
            [_, _] => Deger::Hata("tekrar_sayısı: iki metin argümanı gerekir".to_string()),
            _ => Deger::Hata(format!(
                "tekrar_sayısı: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // sayıya_çevir(metin) → Sayı değerine dönüştür
    globals.insert(
        "sayıya_çevir".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(text)] => match text.trim().parse::<f64>() {
                Ok(number) if number.is_finite() => Deger::Sayi(number),
                Ok(_) => Deger::Hata("sayıya_çevir: sonuç sonlu olmalıdır".to_string()),
                Err(error) => Deger::Hata(format!(
                    "sayıya_çevir: '{}' geçerli bir sayı değil: {}",
                    text, error
                )),
            },
            [Deger::Sayi(number)] if number.is_finite() => Deger::Sayi(*number),
            [Deger::Sayi(_)] => Deger::Hata("sayıya_çevir: sayı sonlu olmalıdır".to_string()),
            [other] => Deger::Hata(format!(
                "sayıya_çevir: metin veya sayı bekleniyordu; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "sayıya_çevir: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // metne_çevir(değer) → Metin değerine dönüştür
    globals.insert(
        "metne_çevir".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [value] => match display_value_limited(value, "metne_çevir") {
                Ok(text) => Deger::Metin(text),
                Err(error) => Deger::Hata(error),
            },
            _ => Deger::Hata(format!(
                "metne_çevir: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // ascii_kodu(karakter) — geriye uyumlu ad; bir Unicode kod noktası döndürür.
    globals.insert(
        "ascii_kodu".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(text)] => {
                let mut characters = text.chars();
                let Some(character) = characters.next() else {
                    return Deger::Hata("ascii_kodu: metin boş olamaz".to_string());
                };
                if characters.next().is_some() {
                    return Deger::Hata(
                        "ascii_kodu: tam olarak bir Unicode karakteri gerekir".to_string(),
                    );
                }
                Deger::Sayi(character as u32 as f64)
            }
            [other] => Deger::Hata(format!("ascii_kodu: metin bekleniyordu; {} geldi", other)),
            _ => Deger::Hata(format!(
                "ascii_kodu: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // karakterden(kod) → Unicode karakterini metin olarak döndür
    globals.insert(
        "karakterden".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Sayi(number)]
                if number.is_finite()
                    && *number >= 0.0
                    && number.fract() == 0.0
                    && *number <= char::MAX as u32 as f64 =>
            {
                match char::from_u32(*number as u32) {
                    Some(character) => Deger::Metin(character.to_string()),
                    None => Deger::Hata(format!(
                        "karakterden: {} geçerli bir Unicode kod noktası değil",
                        number
                    )),
                }
            }
            [Deger::Sayi(_)] => Deger::Hata(
                "karakterden: 0 ile 1114111 arasında sonlu bir tamsayı gerekir".to_string(),
            ),
            [other] => Deger::Hata(format!("karakterden: sayı bekleniyordu; {} geldi", other)),
            _ => Deger::Hata(format!(
                "karakterden: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // içeriyor(metin_veya_liste_veya_nesne, aranan) → 1 veya 0
    globals.insert(
        "içeriyor".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(text), Deger::Metin(pattern)] => {
                Deger::Sayi(if text.contains(pattern.as_str()) {
                    1.0
                } else {
                    0.0
                })
            }
            [Deger::Liste(list), target] => {
                let values = match list.try_borrow() {
                    Ok(values) => values,
                    Err(_) => return Deger::Hata("içeriyor: liste kullanımda".to_string()),
                };
                for item in values.iter() {
                    match crate::semantics::esit_mi(item, target) {
                        Ok(true) => return Deger::Sayi(1.0),
                        Ok(false) => {}
                        Err(error) => return Deger::Hata(format!("içeriyor: {error}")),
                    }
                }
                Deger::Sayi(0.0)
            }
            [Deger::Nesne { alanlar, .. }, Deger::Metin(key)] => match alanlar.try_borrow() {
                Ok(fields) => Deger::Sayi(if fields.contains_key(key) { 1.0 } else { 0.0 }),
                Err(_) => Deger::Hata("içeriyor: nesne kullanımda".to_string()),
            },
            [Deger::Sozluk(map), Deger::Metin(key)] => match map.try_borrow() {
                Ok(values) => Deger::Sayi(if values.contains_key(key) { 1.0 } else { 0.0 }),
                Err(_) => Deger::Hata("içeriyor: sözlük kullanımda".to_string()),
            },
            [container, _] => Deger::Hata(format!(
                "içeriyor: metin, liste, nesne veya sözlük bekleniyordu; {} geldi",
                container
            )),
            _ => Deger::Hata(format!(
                "içeriyor: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );
    globals.insert(
        "değer_al".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Nesne { alanlar, .. }, Deger::Metin(key)] => match alanlar.try_borrow() {
                Ok(fields) => fields.get(key).cloned().unwrap_or(Deger::Bos),
                Err(_) => Deger::Hata("değer_al: nesne kullanımda".to_string()),
            },
            [Deger::Sozluk(map), Deger::Metin(key)] => match map.try_borrow() {
                Ok(values) => values.get(key).cloned().unwrap_or(Deger::Bos),
                Err(_) => Deger::Hata("değer_al: sözlük kullanımda".to_string()),
            },
            [container, Deger::Metin(_)] => Deger::Hata(format!(
                "değer_al: ilk argüman nesne veya sözlük olmalıdır; {} geldi",
                container
            )),
            [_, other] => Deger::Hata(format!(
                "değer_al: anahtar metin olmalıdır; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "değer_al: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "değer_ata".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Nesne { alanlar, .. }, Deger::Metin(key), value] => {
                match alanlar.try_borrow_mut() {
                    Ok(mut fields) => {
                        if !fields.contains_key(key) && fields.len() >= EN_FAZLA_BUILTIN_OGE {
                            return Deger::Hata(
                                "değer_ata: nesne alan güvenlik sınırına ulaştı".to_string(),
                            );
                        }
                        fields.insert(key.clone(), value.clone());
                        Deger::Sayi(1.0)
                    }
                    Err(_) => Deger::Hata("değer_ata: nesne kullanımda".to_string()),
                }
            }
            [Deger::Sozluk(map), Deger::Metin(key), value] => match map.try_borrow_mut() {
                Ok(mut values) => {
                    if !values.contains_key(key) && values.len() >= EN_FAZLA_BUILTIN_OGE {
                        return Deger::Hata(
                            "değer_ata: sözlük öğe güvenlik sınırına ulaştı".to_string(),
                        );
                    }
                    values.insert(key.clone(), value.clone());
                    Deger::Sayi(1.0)
                }
                Err(_) => Deger::Hata("değer_ata: sözlük kullanımda".to_string()),
            },
            [container, Deger::Metin(_), _] => Deger::Hata(format!(
                "değer_ata: ilk argüman nesne veya sözlük olmalıdır; {} geldi",
                container
            )),
            [_, other, _] => Deger::Hata(format!(
                "değer_ata: anahtar metin olmalıdır; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "değer_ata: tam olarak 3 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "hızlı_içeriyor".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Liste(list), target] => {
                let values = match list.try_borrow() {
                    Ok(values) => values,
                    Err(_) => return Deger::Hata("hızlı_içeriyor: liste kullanımda".to_string()),
                };
                for value in values.iter() {
                    match crate::semantics::esit_mi(value, target) {
                        Ok(true) => return Deger::Sayi(1.0),
                        Ok(false) => {}
                        Err(error) => return Deger::Hata(format!("hızlı_içeriyor: {error}")),
                    }
                }
                Deger::Sayi(0.0)
            }
            [other, _] => Deger::Hata(format!(
                "hızlı_içeriyor: ilk argüman liste olmalıdır; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "hızlı_içeriyor: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "tipi".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [value] => match value {
                Deger::Sayi(_) => Deger::Metin("sayı".to_string()),
                Deger::Metin(_) => Deger::Metin("metin".to_string()),
                Deger::Liste(_) => Deger::Metin("liste".to_string()),
                Deger::Sozluk(_) => Deger::Metin("sözlük".to_string()),
                Deger::Fonksiyon { .. }
                | Deger::BytecodeFonksiyon { .. }
                | Deger::DahiliFonksiyon(_)
                | Deger::BaglamliDahiliFonksiyon(_) => Deger::Metin("fonksiyon".to_string()),
                Deger::Nesne { sinif_adi, .. } => Deger::Metin(sinif_adi.clone()),
                Deger::Sinif { ad, .. } => Deger::Metin(format!("sınıf_{}", ad)),
                Deger::Bayt(_) => Deger::Metin("bayt".to_string()),
                Deger::GorevId(_) => Deger::Metin("görev".to_string()),
                Deger::Bos => Deger::Metin("boş".to_string()),
                Deger::Hata(_) => Deger::Metin("hata".to_string()),
                Deger::Vektor(vector) => {
                    let length = match vector.try_borrow() {
                        Ok(values) => values.len(),
                        Err(_) => return Deger::Hata("tipi: vektör kullanımda".to_string()),
                    };
                    Deger::Metin(format!("vektör[{}]", length))
                }
                Deger::Matris {
                    satirlar, sutunlar, ..
                } => Deger::Metin(format!("matris[{}×{}]", satirlar, sutunlar)),
                Deger::Harici(value) => Deger::Metin(value.type_label()),
            },
            _ => Deger::Hata(format!(
                "tipi: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "başlıyor_mu".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(text), Deger::Metin(prefix)] => {
                Deger::Sayi(if text.starts_with(prefix.as_str()) {
                    1.0
                } else {
                    0.0
                })
            }
            [_, _] => Deger::Hata("başlıyor_mu: iki metin argümanı gerekir".to_string()),
            _ => Deger::Hata(format!(
                "başlıyor_mu: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "bitiyor_mu".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(text), Deger::Metin(suffix)] => {
                Deger::Sayi(if text.ends_with(suffix.as_str()) {
                    1.0
                } else {
                    0.0
                })
            }
            [_, _] => Deger::Hata("bitiyor_mu: iki metin argümanı gerekir".to_string()),
            _ => Deger::Hata(format!(
                "bitiyor_mu: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "dizi_dilim".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(text), Deger::Sayi(start), Deger::Sayi(end)]
                if start.is_finite()
                    && end.is_finite()
                    && *start >= 0.0
                    && *end >= 0.0
                    && start.fract() == 0.0
                    && end.fract() == 0.0
                    && *start <= usize::MAX as f64
                    && *end <= usize::MAX as f64 =>
            {
                let characters = text.chars().collect::<Vec<_>>();
                let start = *start as usize;
                let end = *end as usize;
                if start > end || end > characters.len() {
                    Deger::Hata(format!(
                        "dizi_dilim: geçerli aralık 0..{} iken {}..{} istendi",
                        characters.len(),
                        start,
                        end
                    ))
                } else {
                    Deger::Metin(characters[start..end].iter().collect())
                }
            }
            [Deger::Metin(_), Deger::Sayi(_), Deger::Sayi(_)] => Deger::Hata(
                "dizi_dilim: başlangıç ve bitiş negatif olmayan sonlu tamsayılar olmalıdır"
                    .to_string(),
            ),
            [_, _, _] => {
                Deger::Hata("dizi_dilim: metin, başlangıç ve bitiş sayıları gerekir".to_string())
            }
            _ => Deger::Hata(format!(
                "dizi_dilim: tam olarak 3 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    let cli_args: Vec<Deger> = std::env::args().map(Deger::Metin).collect();
    globals.insert(
        "argümanlar".to_string(),
        Deger::Liste(Gc::from_cell(RefCell::new(cli_args))),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // BLOK A — Genişletilmiş Matematik Built-in'leri
    // ═══════════════════════════════════════════════════════════════════════

    globals.insert(
        "üs".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Sayi(base), Deger::Sayi(exponent)]
                if base.is_finite() && exponent.is_finite() =>
            {
                let result = base.powf(*exponent);
                if result.is_finite() {
                    Deger::Sayi(result)
                } else {
                    Deger::Hata("üs: işlem sonlu olmayan sonuç üretti".to_string())
                }
            }
            [Deger::Sayi(_), Deger::Sayi(_)] => {
                Deger::Hata("üs: iki sayı da sonlu olmalıdır".to_string())
            }
            [_, _] => Deger::Hata("üs: iki sayı argümanı gerekir".to_string()),
            _ => Deger::Hata(format!(
                "üs: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "ln".to_string(),
        Deger::DahiliFonksiyon(|args| positive_unary_numeric_builtin(args, "ln", f64::ln)),
    );

    globals.insert(
        "log2".to_string(),
        Deger::DahiliFonksiyon(|args| positive_unary_numeric_builtin(args, "log2", f64::log2)),
    );

    globals.insert(
        "log10".to_string(),
        Deger::DahiliFonksiyon(|args| positive_unary_numeric_builtin(args, "log10", f64::log10)),
    );

    globals.insert(
        "sin".to_string(),
        Deger::DahiliFonksiyon(|args| unary_numeric_builtin(args, "sin", f64::sin)),
    );

    globals.insert(
        "cos".to_string(),
        Deger::DahiliFonksiyon(|args| unary_numeric_builtin(args, "cos", f64::cos)),
    );

    globals.insert(
        "tan".to_string(),
        Deger::DahiliFonksiyon(|args| unary_numeric_builtin(args, "tan", f64::tan)),
    );

    globals.insert(
        "exp".to_string(),
        Deger::DahiliFonksiyon(|args| unary_numeric_builtin(args, "exp", f64::exp)),
    );

    globals.insert(
        "tavan".to_string(),
        Deger::DahiliFonksiyon(|args| unary_numeric_builtin(args, "tavan", f64::ceil)),
    );

    globals.insert(
        "taban_sayı".to_string(),
        Deger::DahiliFonksiyon(|args| unary_numeric_builtin(args, "taban_sayı", f64::floor)),
    );

    globals.insert(
        "mutlak_sayı".to_string(),
        Deger::DahiliFonksiyon(|args| unary_numeric_builtin(args, "mutlak_sayı", f64::abs)),
    );

    globals.insert(
        "işaret".to_string(),
        Deger::DahiliFonksiyon(|args| unary_numeric_builtin(args, "işaret", f64::signum)),
    );

    globals.insert(
        "sonlu_mu".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Sayi(value)] => Deger::Sayi(if value.is_finite() { 1.0 } else { 0.0 }),
            [other] => Deger::Hata(format!("sonlu_mu: sayı bekleniyordu; {} geldi", other)),
            _ => Deger::Hata(format!(
                "sonlu_mu: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "klamp".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Sayi(value), Deger::Sayi(minimum), Deger::Sayi(maximum)]
                if value.is_finite()
                    && minimum.is_finite()
                    && maximum.is_finite()
                    && minimum <= maximum =>
            {
                Deger::Sayi(value.clamp(*minimum, *maximum))
            }
            [Deger::Sayi(_), Deger::Sayi(_), Deger::Sayi(_)] => Deger::Hata(
                "klamp: sayılar sonlu ve alt sınır üst sınırdan küçük/eşit olmalıdır".to_string(),
            ),
            [_, _, _] => Deger::Hata("klamp: üç sayı argümanı gerekir".to_string()),
            _ => Deger::Hata(format!(
                "klamp: tam olarak 3 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // BLOK B — Aktivasyon Fonksiyonları & ML Primitifleri
    // ═══════════════════════════════════════════════════════════════════════

    globals.insert(
        "sigmoid".to_string(),
        Deger::DahiliFonksiyon(|args| {
            unary_numeric_builtin(args, "sigmoid", |value| {
                if value >= 0.0 {
                    1.0 / (1.0 + (-value).exp())
                } else {
                    let exponential = value.exp();
                    exponential / (1.0 + exponential)
                }
            })
        }),
    );

    globals.insert(
        "relu".to_string(),
        Deger::DahiliFonksiyon(|args| unary_numeric_builtin(args, "relu", |value| value.max(0.0))),
    );

    globals.insert(
        "tanh_aktivasyon".to_string(),
        Deger::DahiliFonksiyon(|args| unary_numeric_builtin(args, "tanh_aktivasyon", f64::tanh)),
    );

    // GELU — Gaussian Error Linear Unit (tanh approximation)
    globals.insert(
        "gelu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            unary_numeric_builtin(args, "gelu", |value| {
                0.5 * value
                    * (1.0
                        + (0.7978845608028654 * (value + 0.044715 * value * value * value)).tanh())
            })
        }),
    );

    // softmax(vektor) — her iki vektör tipi de kabul edilir
    globals.insert(
        "softmax".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [value] = args.as_slice() else {
                return Deger::Hata(format!(
                    "softmax: tam olarak 1 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let values = match value_to_finite_vector(value, "softmax") {
                Ok(values) => values,
                Err(error) => return Deger::Hata(error),
            };
            let Some(maximum) = values.iter().copied().reduce(f64::max) else {
                return Deger::Hata("softmax: boş vektör kabul edilmez".to_string());
            };
            let exponentials = values
                .iter()
                .map(|value| (value - maximum).exp())
                .collect::<Vec<_>>();
            let sum = exponentials.iter().sum::<f64>();
            if !sum.is_finite() || sum <= 0.0 {
                return Deger::Hata("softmax: geçersiz normalizasyon toplamı".to_string());
            }
            let result = exponentials
                .iter()
                .map(|value| value / sum)
                .collect::<Vec<_>>();
            Deger::Vektor(Gc::from_cell(RefCell::new(result)))
        }),
    );

    // log_softmax — numerically stable log-softmax
    globals.insert(
        "log_softmax".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [value] = args.as_slice() else {
                return Deger::Hata(format!(
                    "log_softmax: tam olarak 1 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let values = match value_to_finite_vector(value, "log_softmax") {
                Ok(values) => values,
                Err(error) => return Deger::Hata(error),
            };
            let Some(maximum) = values.iter().copied().reduce(f64::max) else {
                return Deger::Hata("log_softmax: boş vektör kabul edilmez".to_string());
            };
            let shifted_sum = values
                .iter()
                .map(|value| (value - maximum).exp())
                .sum::<f64>();
            if !shifted_sum.is_finite() || shifted_sum <= 0.0 {
                return Deger::Hata("log_softmax: geçersiz normalizasyon toplamı".to_string());
            }
            let log_sum_exp = shifted_sum.ln() + maximum;
            let result = values
                .iter()
                .map(|value| value - log_sum_exp)
                .collect::<Vec<_>>();
            if result.iter().any(|value| !value.is_finite()) {
                return Deger::Hata("log_softmax: sonlu olmayan sonuç".to_string());
            }
            Deger::Vektor(Gc::from_cell(RefCell::new(result)))
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // BLOK C — Vektör Operasyonları
    // ═══════════════════════════════════════════════════════════════════════

    globals.insert(
        "vektor_olustur".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(n), Deger::Sayi(deger)] = args.as_slice() else {
                return if args.len() == 2 {
                    Deger::Hata(
                        "vektor_olustur: boyut ve başlangıç değeri sayı olmalıdır".to_string(),
                    )
                } else {
                    Deger::Hata(format!(
                        "vektor_olustur: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            let boyut = match boyut_dogrula(*n, "vektor_olustur", true) {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            if !deger.is_finite() {
                return Deger::Hata("vektor_olustur: başlangıç değeri sonlu olmalı".to_string());
            }
            Deger::Vektor(Gc::from_cell(RefCell::new(vec![*deger; boyut])))
        }),
    );

    globals.insert(
        "vektor_uzunluk".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Vektor(vector)] => match vector.try_borrow() {
                Ok(values) => Deger::Sayi(values.len() as f64),
                Err(_) => Deger::Hata("vektor_uzunluk: vektör kullanımda".to_string()),
            },
            [Deger::Liste(list)] => match list.try_borrow() {
                Ok(values) => Deger::Sayi(values.len() as f64),
                Err(_) => Deger::Hata("vektor_uzunluk: liste kullanımda".to_string()),
            },
            [other] => Deger::Hata(format!(
                "vektor_uzunluk: vektör veya liste bekleniyordu; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "vektor_uzunluk: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "ic_carpim".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [left, right] = args.as_slice() else {
                return Deger::Hata(format!(
                    "ic_carpim: tam olarak 2 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let left = match value_to_finite_vector(left, "ic_carpim") {
                Ok(values) => values,
                Err(error) => return Deger::Hata(error),
            };
            let right = match value_to_finite_vector(right, "ic_carpim") {
                Ok(values) => values,
                Err(error) => return Deger::Hata(error),
            };
            if left.len() != right.len() {
                return Deger::Hata("ic_carpim: vektör boyutları eşit olmalı".to_string());
            }
            let result = left
                .iter()
                .zip(right.iter())
                .map(|(left, right)| left * right)
                .sum::<f64>();
            if result.is_finite() {
                Deger::Sayi(result)
            } else {
                Deger::Hata("ic_carpim: sonlu olmayan sonuç".to_string())
            }
        }),
    );

    globals.insert(
        "vektor_norm".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [value] = args.as_slice() else {
                return Deger::Hata(format!(
                    "vektor_norm: tam olarak 1 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let values = match value_to_finite_vector(value, "vektor_norm") {
                Ok(values) => values,
                Err(error) => return Deger::Hata(error),
            };
            let result = values.iter().map(|value| value * value).sum::<f64>().sqrt();
            if result.is_finite() {
                Deger::Sayi(result)
            } else {
                Deger::Hata("vektor_norm: sonlu olmayan sonuç".to_string())
            }
        }),
    );

    globals.insert(
        "vektor_birim".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [value] = args.as_slice() else {
                return Deger::Hata(format!(
                    "vektor_birim: tam olarak 1 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let values = match value_to_finite_vector(value, "vektor_birim") {
                Ok(values) => values,
                Err(error) => return Deger::Hata(error),
            };
            let norm = values.iter().map(|value| value * value).sum::<f64>().sqrt();
            if !norm.is_finite() || norm == 0.0 {
                return Deger::Hata(
                    "vektor_birim: sıfır veya sonlu olmayan vektör normalize edilemez".to_string(),
                );
            }
            let result = values.iter().map(|value| value / norm).collect();
            Deger::Vektor(Gc::from_cell(RefCell::new(result)))
        }),
    );

    globals.insert(
        "kosinus_benzerligi".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [left, right] = args.as_slice() else {
                return Deger::Hata(format!(
                    "kosinus_benzerligi: tam olarak 2 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let left = match value_to_finite_vector(left, "kosinus_benzerligi") {
                Ok(values) => values,
                Err(error) => return Deger::Hata(error),
            };
            let right = match value_to_finite_vector(right, "kosinus_benzerligi") {
                Ok(values) => values,
                Err(error) => return Deger::Hata(error),
            };
            if left.len() != right.len() {
                return Deger::Hata("kosinus_benzerligi: boyutlar eşit olmalı".to_string());
            }
            let dot = left
                .iter()
                .zip(right.iter())
                .map(|(left, right)| left * right)
                .sum::<f64>();
            let left_norm = left.iter().map(|value| value * value).sum::<f64>().sqrt();
            let right_norm = right.iter().map(|value| value * value).sum::<f64>().sqrt();
            if left_norm == 0.0 || right_norm == 0.0 {
                return Deger::Hata(
                    "kosinus_benzerligi: sıfır vektör için tanımsızdır".to_string(),
                );
            }
            let result = dot / (left_norm * right_norm);
            if result.is_finite() {
                Deger::Sayi(result.clamp(-1.0, 1.0))
            } else {
                Deger::Hata("kosinus_benzerligi: sonlu olmayan sonuç".to_string())
            }
        }),
    );

    globals.insert(
        "vektor_topla".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [left, right] = args.as_slice() else {
                return Deger::Hata(format!(
                    "vektor_topla: tam olarak 2 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let left = match value_to_finite_vector(left, "vektor_topla") {
                Ok(values) => values,
                Err(error) => return Deger::Hata(error),
            };
            let right = match value_to_finite_vector(right, "vektor_topla") {
                Ok(values) => values,
                Err(error) => return Deger::Hata(error),
            };
            if left.len() != right.len() {
                return Deger::Hata("vektor_topla: boyutlar eşit olmalı".to_string());
            }
            let result = left
                .iter()
                .zip(right.iter())
                .map(|(left, right)| left + right)
                .collect::<Vec<_>>();
            if result.iter().any(|value| !value.is_finite()) {
                return Deger::Hata("vektor_topla: sonlu olmayan sonuç".to_string());
            }
            Deger::Vektor(Gc::from_cell(RefCell::new(result)))
        }),
    );

    globals.insert(
        "vektor_carpi".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [left, right] = args.as_slice() else {
                return Deger::Hata(format!(
                    "vektor_carpi: tam olarak 2 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let left = match value_to_finite_vector(left, "vektor_carpi") {
                Ok(values) => values,
                Err(error) => return Deger::Hata(error),
            };
            let right = match value_to_finite_vector(right, "vektor_carpi") {
                Ok(values) => values,
                Err(error) => return Deger::Hata(error),
            };
            if left.len() != right.len() {
                return Deger::Hata("vektor_carpi: boyutlar eşit olmalı".to_string());
            }
            let result = left
                .iter()
                .zip(right.iter())
                .map(|(left, right)| left * right)
                .collect::<Vec<_>>();
            if result.iter().any(|value| !value.is_finite()) {
                return Deger::Hata("vektor_carpi: sonlu olmayan sonuç".to_string());
            }
            Deger::Vektor(Gc::from_cell(RefCell::new(result)))
        }),
    );

    globals.insert(
        "vektor_skalar_carp".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Vektor(vector), Deger::Sayi(scalar)] if scalar.is_finite() => {
                let values = match vector.try_borrow() {
                    Ok(values) => values,
                    Err(_) => {
                        return Deger::Hata("vektor_skalar_carp: vektör kullanımda".to_string())
                    }
                };
                let result = values
                    .iter()
                    .map(|value| value * scalar)
                    .collect::<Vec<_>>();
                if result.iter().any(|value| !value.is_finite()) {
                    Deger::Hata("vektor_skalar_carp: sonlu olmayan sonuç".to_string())
                } else {
                    Deger::Vektor(Gc::from_cell(RefCell::new(result)))
                }
            }
            [Deger::Vektor(_), Deger::Sayi(_)] => {
                Deger::Hata("vektor_skalar_carp: skaler sonlu olmalıdır".to_string())
            }
            [_, _] => Deger::Hata("vektor_skalar_carp: vektör ve sayı gerekir".to_string()),
            _ => Deger::Hata(format!(
                "vektor_skalar_carp: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "listeye_vektor".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [value @ Deger::Liste(_)] = args.as_slice() else {
                return if args.len() == 1 {
                    Deger::Hata(format!(
                        "listeye_vektor: liste bekleniyordu; {} geldi",
                        args[0]
                    ))
                } else {
                    Deger::Hata(format!(
                        "listeye_vektor: tam olarak 1 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            match value_to_finite_vector(value, "listeye_vektor") {
                Ok(values) => Deger::Vektor(Gc::from_cell(RefCell::new(values))),
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "vektore_liste".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Vektor(vector)] => match vector.try_borrow() {
                Ok(values) => Deger::Liste(Gc::from_cell(RefCell::new(
                    values.iter().copied().map(Deger::Sayi).collect(),
                ))),
                Err(_) => Deger::Hata("vektore_liste: vektör kullanımda".to_string()),
            },
            [other] => Deger::Hata(format!(
                "vektore_liste: vektör bekleniyordu; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "vektore_liste: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "vektor_dilim".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Vektor(vector), Deger::Sayi(start), Deger::Sayi(end)]
                if start.is_finite()
                    && end.is_finite()
                    && *start >= 0.0
                    && *end >= 0.0
                    && start.fract() == 0.0
                    && end.fract() == 0.0
                    && *start <= usize::MAX as f64
                    && *end <= usize::MAX as f64 =>
            {
                let values = match vector.try_borrow() {
                    Ok(values) => values,
                    Err(_) => return Deger::Hata("vektor_dilim: vektör kullanımda".to_string()),
                };
                let start = *start as usize;
                let end = *end as usize;
                if start > end || end > values.len() {
                    Deger::Hata(format!(
                        "vektor_dilim: geçerli aralık 0..{} iken {}..{} istendi",
                        values.len(),
                        start,
                        end
                    ))
                } else {
                    Deger::Vektor(Gc::from_cell(RefCell::new(values[start..end].to_vec())))
                }
            }
            [Deger::Vektor(_), Deger::Sayi(_), Deger::Sayi(_)] => Deger::Hata(
                "vektor_dilim: başlangıç ve bitiş negatif olmayan sonlu tamsayılar olmalıdır"
                    .to_string(),
            ),
            [_, _, _] => {
                Deger::Hata("vektor_dilim: vektör, başlangıç ve bitiş sayıları gerekir".to_string())
            }
            _ => Deger::Hata(format!(
                "vektor_dilim: tam olarak 3 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // vektor_ekle — vektöre eleman ekle
    globals.insert(
        "vektor_ekle".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Vektor(vector), Deger::Sayi(value)] if value.is_finite() => {
                match vector.try_borrow_mut() {
                    Ok(mut values) => {
                        if values.len() >= EN_FAZLA_TENSOR_ELEMANI {
                            Deger::Hata(
                                "vektor_ekle: vektör eleman güvenlik sınırına ulaştı".to_string(),
                            )
                        } else {
                            values.push(*value);
                            drop(values);
                            Deger::Vektor(Gc::clone(vector))
                        }
                    }
                    Err(_) => Deger::Hata("vektor_ekle: vektör kullanımda".to_string()),
                }
            }
            [Deger::Vektor(_), Deger::Sayi(_)] => {
                Deger::Hata("vektor_ekle: eklenecek sayı sonlu olmalıdır".to_string())
            }
            [_, _] => Deger::Hata("vektor_ekle: vektör ve sayı gerekir".to_string()),
            _ => Deger::Hata(format!(
                "vektor_ekle: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // vektor_al — vektörden indeks ile eleman oku
    globals.insert(
        "vektor_al".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Vektor(vektor), Deger::Sayi(indeks)] = args.as_slice() else {
                return if args.len() == 2 {
                    Deger::Hata("vektor_al: vektör ve sayı indeks gerekir".to_string())
                } else {
                    Deger::Hata(format!(
                        "vektor_al: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            let vektor = match vektor.try_borrow() {
                Ok(vektor) => vektor,
                Err(_) => return Deger::Hata("vektor_al: vektör kullanımda".to_string()),
            };
            let indeks = match indeks_dogrula(*indeks, vektor.len(), "vektor_al") {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            Deger::Sayi(vektor[indeks])
        }),
    );

    // vektor_ata — vektöre indeks ile değer yaz
    globals.insert(
        "vektor_ata".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Vektor(vektor), Deger::Sayi(indeks), Deger::Sayi(deger)] = args.as_slice()
            else {
                return if args.len() == 3 {
                    Deger::Hata("vektor_ata: vektör, indeks ve sayı gerekir".to_string())
                } else {
                    Deger::Hata(format!(
                        "vektor_ata: tam olarak 3 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            if !deger.is_finite() {
                return Deger::Hata("vektor_ata: atanacak değer sonlu olmalı".to_string());
            }
            let mut vektor = match vektor.try_borrow_mut() {
                Ok(vektor) => vektor,
                Err(_) => return Deger::Hata("vektor_ata: vektör kullanımda".to_string()),
            };
            let indeks = match indeks_dogrula(*indeks, vektor.len(), "vektor_ata") {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            vektor[indeks] = *deger;
            Deger::Sayi(1.0)
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // BLOK D — Matris Operasyonları (Naive GEMM — portable, zero-dep)
    // ═══════════════════════════════════════════════════════════════════════

    globals.insert(
        "matris_olustur".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if !(2..=3).contains(&args.len()) {
                return Deger::Hata(format!(
                    "matris_olustur: 2 veya 3 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            }
            let (Deger::Sayi(satirlar), Deger::Sayi(sutunlar)) = (&args[0], &args[1]) else {
                return Deger::Hata("matris_olustur: satır ve sütun sayıları gerekir".to_string());
            };
            let satirlar = match boyut_dogrula(*satirlar, "matris_olustur", true) {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            let sutunlar = match boyut_dogrula(*sutunlar, "matris_olustur", true) {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            let eleman_sayisi = match eleman_sayisi_dogrula(satirlar, sutunlar, "matris_olustur") {
                Ok(deger) => deger,
                Err(hata) => return Deger::Hata(hata),
            };
            let baslangic = match args.get(2) {
                Some(Deger::Sayi(deger)) if deger.is_finite() => *deger,
                Some(_) => {
                    return Deger::Hata(
                        "matris_olustur: başlangıç değeri sonlu sayı olmalı".to_string(),
                    )
                }
                None => 0.0,
            };
            Deger::Matris {
                satirlar,
                sutunlar,
                veri: Gc::from_cell(RefCell::new(vec![baslangic; eleman_sayisi])),
            }
        }),
    );

    globals.insert(
        "matris_al".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Matris {
                satirlar,
                sutunlar,
                veri,
            }, Deger::Sayi(satir), Deger::Sayi(sutun)] => {
                let expected = match eleman_sayisi_dogrula(*satirlar, *sutunlar, "matris_al") {
                    Ok(value) => value,
                    Err(error) => return Deger::Hata(error),
                };
                let values = match veri.try_borrow() {
                    Ok(values) => values,
                    Err(_) => return Deger::Hata("matris_al: matris kullanımda".to_string()),
                };
                if values.len() != expected {
                    return Deger::Hata("matris_al: bozuk matris veri boyutu".to_string());
                }
                let satir = match indeks_dogrula(*satir, *satirlar, "matris_al satır") {
                    Ok(deger) => deger,
                    Err(hata) => return Deger::Hata(hata),
                };
                let sutun = match indeks_dogrula(*sutun, *sutunlar, "matris_al sütun") {
                    Ok(deger) => deger,
                    Err(hata) => return Deger::Hata(hata),
                };
                let value = values[satir * sutunlar + sutun];
                if value.is_finite() {
                    Deger::Sayi(value)
                } else {
                    Deger::Hata("matris_al: matris sonlu olmayan değer içeriyor".to_string())
                }
            }
            [_, _, _] => Deger::Hata("matris_al: matris, satır ve sütun gerekir".to_string()),
            _ => Deger::Hata(format!(
                "matris_al: tam olarak 3 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "matris_ata".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Matris {
                satirlar,
                sutunlar,
                veri,
            }, Deger::Sayi(satir), Deger::Sayi(sutun), Deger::Sayi(deger)] => {
                if !deger.is_finite() {
                    return Deger::Hata("matris_ata: atanacak değer sonlu olmalı".to_string());
                }
                let expected = match eleman_sayisi_dogrula(*satirlar, *sutunlar, "matris_ata") {
                    Ok(value) => value,
                    Err(error) => return Deger::Hata(error),
                };
                let satir = match indeks_dogrula(*satir, *satirlar, "matris_ata satır") {
                    Ok(deger) => deger,
                    Err(hata) => return Deger::Hata(hata),
                };
                let sutun = match indeks_dogrula(*sutun, *sutunlar, "matris_ata sütun") {
                    Ok(deger) => deger,
                    Err(hata) => return Deger::Hata(hata),
                };
                let mut values = match veri.try_borrow_mut() {
                    Ok(values) => values,
                    Err(_) => return Deger::Hata("matris_ata: matris kullanımda".to_string()),
                };
                if values.len() != expected {
                    return Deger::Hata("matris_ata: bozuk matris veri boyutu".to_string());
                }
                values[satir * sutunlar + sutun] = *deger;
                Deger::Sayi(1.0)
            }
            [_, _, _, _] => {
                Deger::Hata("matris_ata: matris, satır, sütun ve sayı gerekir".to_string())
            }
            _ => Deger::Hata(format!(
                "matris_ata: tam olarak 4 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // matris_carp — Naive O(n³) GEMM
    globals.insert(
        "matris_carp".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Matris {
                satirlar: ra,
                sutunlar: ca,
                veri: va,
            }, Deger::Matris {
                satirlar: rb,
                sutunlar: cb,
                veri: vb,
            }] = args.as_slice()
            else {
                return if args.len() == 2 {
                    Deger::Hata("matris_carp: iki matris argümanı gerekir".to_string())
                } else {
                    Deger::Hata(format!(
                        "matris_carp: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            if ca != rb {
                return Deger::Hata(format!(
                    "matris_carp: boyut uyumsuzluğu {}x{} * {}x{}",
                    ra, ca, rb, cb
                ));
            }
            let (m, n, k) = (*ra, *cb, *ca);
            let element_count = match eleman_sayisi_dogrula(m, n, "matris_carp") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            let _operation_count = match element_count.checked_mul(k) {
                Some(count) if count <= EN_FAZLA_SAYISAL_ISLEM => count,
                _ => {
                    return Deger::Hata(format!(
                        "matris_carp: işlem sayısı {} sınırını aşıyor",
                        EN_FAZLA_SAYISAL_ISLEM
                    ))
                }
            };
            let a = match va.try_borrow() {
                Ok(values) => values,
                Err(_) => return Deger::Hata("matris_carp: sol matris kullanımda".to_string()),
            };
            let b = match vb.try_borrow() {
                Ok(values) => values,
                Err(_) => return Deger::Hata("matris_carp: sağ matris kullanımda".to_string()),
            };
            if a.len() != m.saturating_mul(k) || b.len() != k.saturating_mul(n) {
                return Deger::Hata("matris_carp: bozuk matris veri boyutu".to_string());
            }
            let mut result = vec![0.0f64; element_count];
            for row in 0..m {
                for column in 0..n {
                    let mut sum = 0.0f64;
                    for inner in 0..k {
                        sum += a[row * k + inner] * b[inner * n + column];
                    }
                    if !sum.is_finite() {
                        return Deger::Hata(
                            "matris_carp: işlem sonlu olmayan sonuç üretti".to_string(),
                        );
                    }
                    result[row * n + column] = sum;
                }
            }
            Deger::Matris {
                satirlar: m,
                sutunlar: n,
                veri: Gc::from_cell(RefCell::new(result)),
            }
        }),
    );

    globals.insert(
        "matris_transpoz".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Matris {
                satirlar,
                sutunlar,
                veri,
            }] => {
                let values = match veri.try_borrow() {
                    Ok(values) => values,
                    Err(_) => return Deger::Hata("matris_transpoz: matris kullanımda".to_string()),
                };
                let element_count =
                    match eleman_sayisi_dogrula(*satirlar, *sutunlar, "matris_transpoz") {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                if values.len() != element_count {
                    return Deger::Hata("matris_transpoz: bozuk matris veri boyutu".to_string());
                }
                let mut result = vec![0.0f64; element_count];
                for row in 0..*satirlar {
                    for column in 0..*sutunlar {
                        result[column * satirlar + row] = values[row * sutunlar + column];
                    }
                }
                Deger::Matris {
                    satirlar: *sutunlar,
                    sutunlar: *satirlar,
                    veri: Gc::from_cell(RefCell::new(result)),
                }
            }
            [other] => Deger::Hata(format!(
                "matris_transpoz: matris bekleniyordu; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "matris_transpoz: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // matris_carp_vektor(M, v) → Vektör [M.satirlar]
    // Matris-vektör çarpımı: y = M * v (sinir ağı ileri geçişi için temel)
    globals.insert(
        "matris_carp_vektor".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix, vector] = args.as_slice() else {
                return if args.len() == 2 {
                    Deger::Hata("matris_carp_vektor: matris ve vektör gerekir".to_string())
                } else {
                    Deger::Hata(format!(
                        "matris_carp_vektor: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            let (rows, cols, mat_vals) = match value_to_finite_matrix(matrix, "matris_carp_vektor")
            {
                Ok(m) => m,
                Err(e) => return Deger::Hata(e),
            };
            let vec_vals = match value_to_finite_vector(vector, "matris_carp_vektor") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            if cols != vec_vals.len() {
                return Deger::Hata(format!(
                    "matris_carp_vektor: matris sütun sayısı ({}) vektör boyutu ({}) ile eşleşmeli",
                    cols,
                    vec_vals.len()
                ));
            }
            let _ = match eleman_sayisi_dogrula(rows, cols, "matris_carp_vektor") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let mut result = Vec::with_capacity(rows);
            for r in 0..rows {
                let mut sum = 0.0f64;
                for c in 0..cols {
                    sum += mat_vals[r * cols + c] * vec_vals[c];
                }
                if !sum.is_finite() {
                    return Deger::Hata("matris_carp_vektor: sonlu olmayan sonuç".to_string());
                }
                result.push(sum);
            }
            Deger::Vektor(Gc::from_cell(RefCell::new(result)))
        }),
    );

    // matris_transpoz_carp_vektor(M, v) → Vektör [M.sutunlar]
    // Transpoz matris-vektör çarpımı: y = Mᵀ * v (geri yayılım δ gradyanı için)
    globals.insert(
        "matris_transpoz_carp_vektor".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix, vector] = args.as_slice() else {
                return if args.len() == 2 {
                    Deger::Hata(
                        "matris_transpoz_carp_vektor: matris ve vektör gerekir".to_string(),
                    )
                } else {
                    Deger::Hata(format!(
                        "matris_transpoz_carp_vektor: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            let (rows, cols, mat_vals) =
                match value_to_finite_matrix(matrix, "matris_transpoz_carp_vektor") {
                    Ok(m) => m,
                    Err(e) => return Deger::Hata(e),
                };
            let vec_vals = match value_to_finite_vector(vector, "matris_transpoz_carp_vektor") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            if rows != vec_vals.len() {
                return Deger::Hata(format!(
                    "matris_transpoz_carp_vektor: matris satır sayısı ({}) vektör boyutu ({}) ile eşleşmeli",
                    rows,
                    vec_vals.len()
                ));
            }
            // Mᵀv[c] = Σ_r M[r,c] * v[r]
            let mut result = vec![0.0f64; cols];
            for r in 0..rows {
                for c in 0..cols {
                    result[c] += mat_vals[r * cols + c] * vec_vals[r];
                }
            }
            if result.iter().any(|v| !v.is_finite()) {
                return Deger::Hata(
                    "matris_transpoz_carp_vektor: sonlu olmayan sonuç".to_string(),
                );
            }
            Deger::Vektor(Gc::from_cell(RefCell::new(result)))
        }),
    );

    globals.insert(
        "matris_satir_al".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [matrix @ Deger::Matris { .. }, Deger::Sayi(row)] => {
                let (rows, columns, values) =
                    match value_to_finite_matrix(matrix, "matris_satir_al") {
                        Ok(matrix) => matrix,
                        Err(error) => return Deger::Hata(error),
                    };
                let row = match indeks_dogrula(*row, rows, "matris_satir_al") {
                    Ok(value) => value,
                    Err(error) => return Deger::Hata(error),
                };
                Deger::Vektor(Gc::from_cell(RefCell::new(
                    values[row * columns..(row + 1) * columns].to_vec(),
                )))
            }
            [_, _] => Deger::Hata("matris_satir_al: matris ve satır gerekir".to_string()),
            _ => Deger::Hata(format!(
                "matris_satir_al: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "matris_sutun_al".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [matrix @ Deger::Matris { .. }, Deger::Sayi(column)] => {
                let (rows, columns, values) =
                    match value_to_finite_matrix(matrix, "matris_sutun_al") {
                        Ok(matrix) => matrix,
                        Err(error) => return Deger::Hata(error),
                    };
                let column = match indeks_dogrula(*column, columns, "matris_sutun_al") {
                    Ok(value) => value,
                    Err(error) => return Deger::Hata(error),
                };
                let result = (0..rows)
                    .map(|row| values[row * columns + column])
                    .collect();
                Deger::Vektor(Gc::from_cell(RefCell::new(result)))
            }
            [_, _] => Deger::Hata("matris_sutun_al: matris ve sütun gerekir".to_string()),
            _ => Deger::Hata(format!(
                "matris_sutun_al: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "matris_satir_ata".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Matris {
                satirlar,
                sutunlar,
                veri,
            }, Deger::Sayi(row), new_row @ Deger::Vektor(_)] => {
                let new_row = match value_to_finite_vector(new_row, "matris_satir_ata") {
                    Ok(values) => values,
                    Err(error) => return Deger::Hata(error),
                };
                if new_row.len() != *sutunlar {
                    return Deger::Hata(format!(
                        "matris_satir_ata: vektör uzunluğu {} olmalı",
                        sutunlar
                    ));
                }
                let expected = match eleman_sayisi_dogrula(*satirlar, *sutunlar, "matris_satir_ata")
                {
                    Ok(value) => value,
                    Err(error) => return Deger::Hata(error),
                };
                let row = match indeks_dogrula(*row, *satirlar, "matris_satir_ata") {
                    Ok(value) => value,
                    Err(error) => return Deger::Hata(error),
                };
                let mut values = match veri.try_borrow_mut() {
                    Ok(values) => values,
                    Err(_) => {
                        return Deger::Hata("matris_satir_ata: matris kullanımda".to_string())
                    }
                };
                if values.len() != expected {
                    return Deger::Hata("matris_satir_ata: bozuk matris veri boyutu".to_string());
                }
                let start = row * sutunlar;
                values[start..start + sutunlar].copy_from_slice(&new_row);
                Deger::Sayi(1.0)
            }
            [_, _, _] => {
                Deger::Hata("matris_satir_ata: matris, satır ve vektör gerekir".to_string())
            }
            _ => Deger::Hata(format!(
                "matris_satir_ata: tam olarak 3 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "kimlik_matrisi".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Sayi(n)] => {
                let n = match boyut_dogrula(*n, "kimlik_matrisi", true) {
                    Ok(deger) => deger,
                    Err(hata) => return Deger::Hata(hata),
                };
                let eleman_sayisi = match eleman_sayisi_dogrula(n, n, "kimlik_matrisi") {
                    Ok(deger) => deger,
                    Err(hata) => return Deger::Hata(hata),
                };
                let mut v = vec![0.0f64; eleman_sayisi];
                for i in 0..n {
                    v[i * n + i] = 1.0;
                }
                Deger::Matris {
                    satirlar: n,
                    sutunlar: n,
                    veri: Gc::from_cell(RefCell::new(v)),
                }
            }
            [other] => Deger::Hata(format!(
                "kimlik_matrisi: sayı boyutu bekleniyordu; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "kimlik_matrisi: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "matris_boyutu".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Matris {
                satirlar, sutunlar, ..
            }] => Deger::Liste(Gc::from_cell(RefCell::new(vec![
                Deger::Sayi(*satirlar as f64),
                Deger::Sayi(*sutunlar as f64),
            ]))),
            [other] => Deger::Hata(format!(
                "matris_boyutu: matris bekleniyordu; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "matris_boyutu: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // matris_vektor_carp — M * v (2D matris ile 1D vektör çarpımı)
    globals.insert(
        "matris_vektor_carp".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Matris {
                satirlar,
                sutunlar,
                veri,
            }, Deger::Vektor(vector)] = args.as_slice()
            else {
                return if args.len() == 2 {
                    Deger::Hata("matris_vektor_carp: matris ve vektör argümanı gerekir".to_string())
                } else {
                    Deger::Hata(format!(
                        "matris_vektor_carp: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            let matrix_values = match veri.try_borrow() {
                Ok(values) => values,
                Err(_) => return Deger::Hata("matris_vektor_carp: matris kullanımda".to_string()),
            };
            let vector_values = match vector.try_borrow() {
                Ok(values) => values,
                Err(_) => return Deger::Hata("matris_vektor_carp: vektör kullanımda".to_string()),
            };
            if *sutunlar != vector_values.len() {
                return Deger::Hata(format!(
                    "matris_vektor_carp: matris sütun {} ≠ vektör boyutu {}",
                    sutunlar,
                    vector_values.len()
                ));
            }
            if matrix_values.len() != satirlar.saturating_mul(*sutunlar) {
                return Deger::Hata("matris_vektor_carp: bozuk matris veri boyutu".to_string());
            }
            let mut result = Vec::with_capacity(*satirlar);
            for row in 0..*satirlar {
                let sum = (0..*sutunlar)
                    .map(|column| matrix_values[row * sutunlar + column] * vector_values[column])
                    .sum::<f64>();
                if !sum.is_finite() {
                    return Deger::Hata("matris_vektor_carp: sonlu olmayan sonuç".to_string());
                }
                result.push(sum);
            }
            Deger::Vektor(Gc::from_cell(RefCell::new(result)))
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // BLOK E — Düzenli İfade (Regex) Built-in'leri
    // ═══════════════════════════════════════════════════════════════════════

    globals.insert(
        "regex_eslestir".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(metin), Deger::Metin(desen)] => {
                if let Err(error) = validate_regex_text(metin, "regex_eslestir") {
                    return Deger::Hata(error);
                }
                match compile_regex(desen, "regex_eslestir") {
                    Ok(regex) => Deger::Sayi(if regex.is_match(metin) { 1.0 } else { 0.0 }),
                    Err(error) => Deger::Hata(error),
                }
            }
            [_, _] => Deger::Hata("regex_eslestir: metin ve desen gerekir".to_string()),
            _ => Deger::Hata(format!(
                "regex_eslestir: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "regex_bul".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(metin), Deger::Metin(desen)] => {
                if let Err(error) = validate_regex_text(metin, "regex_bul") {
                    return Deger::Hata(error);
                }
                match compile_regex(desen, "regex_bul") {
                    Ok(regex) => regex
                        .find(metin)
                        .map(|found| Deger::Metin(found.as_str().to_string()))
                        .unwrap_or(Deger::Bos),
                    Err(error) => Deger::Hata(error),
                }
            }
            [_, _] => Deger::Hata("regex_bul: metin ve desen gerekir".to_string()),
            _ => Deger::Hata(format!(
                "regex_bul: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "regex_bul_tum".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(metin), Deger::Metin(desen)] => {
                if let Err(error) = validate_regex_text(metin, "regex_bul_tum") {
                    return Deger::Hata(error);
                }
                let regex = match compile_regex(desen, "regex_bul_tum") {
                    Ok(regex) => regex,
                    Err(error) => return Deger::Hata(error),
                };
                let mut results = Vec::new();
                for found in regex.find_iter(metin) {
                    if results.len() >= EN_FAZLA_BUILTIN_OGE {
                        return Deger::Hata(format!(
                            "regex_bul_tum: eşleşme sayısı {} öğelik güvenlik sınırını aşıyor",
                            EN_FAZLA_BUILTIN_OGE
                        ));
                    }
                    results.push(Deger::Metin(found.as_str().to_string()));
                }
                Deger::Liste(Gc::from_cell(RefCell::new(results)))
            }
            [_, _] => Deger::Hata("regex_bul_tum: metin ve desen gerekir".to_string()),
            _ => Deger::Hata(format!(
                "regex_bul_tum: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "regex_degistir".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(metin), Deger::Metin(desen), Deger::Metin(yeni)] => {
                if let Err(error) = validate_regex_text(metin, "regex_degistir") {
                    return Deger::Hata(error);
                }
                if let Err(error) = validate_regex_text(yeni, "regex_degistir yeni metin") {
                    return Deger::Hata(error);
                }
                let regex = match compile_regex(desen, "regex_degistir") {
                    Ok(regex) => regex,
                    Err(error) => return Deger::Hata(error),
                };
                let mut output = LimitedText::new(EN_FAZLA_DOSYA_BYTES);
                let mut last_end = 0usize;
                for found in regex.find_iter(metin) {
                    if output.write_str(&metin[last_end..found.start()]).is_err()
                        || output.write_str(yeni).is_err()
                    {
                        return Deger::Hata(format!(
                            "regex_degistir: çıktı {} bayt sınırını aşıyor",
                            EN_FAZLA_DOSYA_BYTES
                        ));
                    }
                    last_end = found.end();
                }
                if output.write_str(&metin[last_end..]).is_err() {
                    return Deger::Hata(format!(
                        "regex_degistir: çıktı {} bayt sınırını aşıyor",
                        EN_FAZLA_DOSYA_BYTES
                    ));
                }
                Deger::Metin(output.text)
            }
            [_, _, _] => {
                Deger::Hata("regex_degistir: metin, desen ve yeni metin gerekir".to_string())
            }
            _ => Deger::Hata(format!(
                "regex_degistir: tam olarak 3 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "regex_bol".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(metin), Deger::Metin(desen)] => {
                if let Err(error) = validate_regex_text(metin, "regex_bol") {
                    return Deger::Hata(error);
                }
                let regex = match compile_regex(desen, "regex_bol") {
                    Ok(regex) => regex,
                    Err(error) => return Deger::Hata(error),
                };
                let mut parts = Vec::new();
                for part in regex.split(metin) {
                    if parts.len() >= EN_FAZLA_BUILTIN_OGE {
                        return Deger::Hata(format!(
                            "regex_bol: parça sayısı {} öğelik güvenlik sınırını aşıyor",
                            EN_FAZLA_BUILTIN_OGE
                        ));
                    }
                    parts.push(Deger::Metin(part.to_string()));
                }
                Deger::Liste(Gc::from_cell(RefCell::new(parts)))
            }
            [_, _] => Deger::Hata("regex_bol: metin ve desen gerekir".to_string()),
            _ => Deger::Hata(format!(
                "regex_bol: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // BLOK F — Gelişmiş Rastgele Sayı Üretimi (SmallRng — seed destekli)
    // ═══════════════════════════════════════════════════════════════════════

    // Thread-local SmallRng
    thread_local! {
        static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
    }

    // Box-Muller dönüşümü ile Normal dağılım örneği
    globals.insert(
        "normal_rastgele".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(mean), Deger::Sayi(deviation)] = args.as_slice() else {
                return if args.len() == 2 {
                    Deger::Hata("normal_rastgele: ortalama ve sapma sayı olmalıdır".to_string())
                } else {
                    Deger::Hata(format!(
                        "normal_rastgele: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            if !mean.is_finite() || !deviation.is_finite() || *deviation < 0.0 {
                return Deger::Hata(
                    "normal_rastgele: ortalama sonlu, sapma sonlu ve negatif olmayan sayı olmalıdır"
                        .to_string(),
                );
            }
            let (u1, u2) = match RNG.with(|rng| {
                rng.try_borrow_mut()
                    .map(|mut rng| (rng.gen::<f64>(), rng.gen::<f64>()))
            }) {
                Ok(values) => values,
                Err(_) => {
                    return Deger::Hata(
                        "normal_rastgele: rastgele sayı üreteci kullanımda".to_string(),
                    )
                }
            };
            let u1 = u1.max(1e-10);
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            let result = mean + deviation * z;
            if result.is_finite() {
                Deger::Sayi(result)
            } else {
                Deger::Hata("normal_rastgele: sonlu olmayan sonuç".to_string())
            }
        }),
    );

    globals.insert(
        "uniform_rastgele".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(minimum), Deger::Sayi(maximum)] = args.as_slice() else {
                return if args.len() == 2 {
                    Deger::Hata("uniform_rastgele: alt ve üst sınır sayı olmalıdır".to_string())
                } else {
                    Deger::Hata(format!(
                        "uniform_rastgele: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            if !minimum.is_finite() || !maximum.is_finite() || minimum >= maximum {
                return Deger::Hata(
                    "uniform_rastgele: sonlu alt sınır üst sınırdan küçük olmalıdır".to_string(),
                );
            }
            let unit = match RNG.with(|rng| rng.try_borrow_mut().map(|mut rng| rng.gen::<f64>())) {
                Ok(value) => value,
                Err(_) => {
                    return Deger::Hata(
                        "uniform_rastgele: rastgele sayı üreteci kullanımda".to_string(),
                    )
                }
            };
            let result = minimum + unit * (maximum - minimum);
            if result.is_finite() {
                Deger::Sayi(result)
            } else {
                Deger::Hata("uniform_rastgele: sonlu olmayan sonuç".to_string())
            }
        }),
    );

    globals.insert(
        "rastgele_tohum_ata".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Sayi(seed)]
                if seed.is_finite()
                    && *seed >= 0.0
                    && seed.fract() == 0.0
                    && *seed <= EN_BUYUK_GUVENLI_SAYISAL_KIMLIK as f64 =>
            {
                match RNG.with(|rng| {
                    rng.try_borrow_mut()
                        .map(|mut rng| *rng = SmallRng::seed_from_u64(*seed as u64))
                }) {
                    Ok(()) => Deger::Sayi(1.0),
                    Err(_) => Deger::Hata(
                        "rastgele_tohum_ata: rastgele sayı üreteci kullanımda".to_string(),
                    ),
                }
            }
            [Deger::Sayi(_)] => Deger::Hata(
                "rastgele_tohum_ata: tohum güvenli aralıkta negatif olmayan tamsayı olmalıdır"
                    .to_string(),
            ),
            [other] => Deger::Hata(format!(
                "rastgele_tohum_ata: sayı bekleniyordu; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "rastgele_tohum_ata: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "rastgele_tamsayi".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(minimum), Deger::Sayi(maximum)] = args.as_slice() else {
                return if args.len() == 2 {
                    Deger::Hata("rastgele_tamsayi: alt ve üst sınır sayı olmalıdır".to_string())
                } else {
                    Deger::Hata(format!(
                        "rastgele_tamsayi: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            if !minimum.is_finite()
                || !maximum.is_finite()
                || minimum.fract() != 0.0
                || maximum.fract() != 0.0
                || minimum > maximum
                || minimum.abs() > EN_BUYUK_GUVENLI_SAYISAL_KIMLIK as f64
                || maximum.abs() > EN_BUYUK_GUVENLI_SAYISAL_KIMLIK as f64
            {
                return Deger::Hata(
                    "rastgele_tamsayi: güvenli aralıkta sonlu tamsayı sınırlar ve alt <= üst gerekir"
                        .to_string(),
                );
            }
            match RNG.with(|rng| {
                rng.try_borrow_mut()
                    .map(|mut rng| rng.gen_range((*minimum as i64)..=(*maximum as i64)))
            }) {
                Ok(value) => Deger::Sayi(value as f64),
                Err(_) => Deger::Hata(
                    "rastgele_tamsayi: rastgele sayı üreteci kullanımda".to_string(),
                ),
            }
        }),
    );

    globals.insert(
        "vektor_karistir".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Vektor(vector)] => match vector.try_borrow_mut() {
                Ok(mut b) => match RNG.with(|rng| {
                    rng.try_borrow_mut().map(|mut rng| {
                        let n = b.len();
                        for i in (1..n).rev() {
                            let j = rng.gen_range(0..=i);
                            b.swap(i, j);
                        }
                    })
                }) {
                    Ok(()) => Deger::Sayi(1.0),
                    Err(_) => {
                        Deger::Hata("vektor_karistir: rastgele sayı üreteci kullanımda".to_string())
                    }
                },
                Err(_) => Deger::Hata("vektor_karistir: vektör kullanımda".to_string()),
            },
            [Deger::Liste(list)] => match list.try_borrow_mut() {
                Ok(mut b) => match RNG.with(|rng| {
                    rng.try_borrow_mut().map(|mut rng| {
                        let n = b.len();
                        for i in (1..n).rev() {
                            let j = rng.gen_range(0..=i);
                            b.swap(i, j);
                        }
                    })
                }) {
                    Ok(()) => Deger::Sayi(1.0),
                    Err(_) => {
                        Deger::Hata("vektor_karistir: rastgele sayı üreteci kullanımda".to_string())
                    }
                },
                Err(_) => Deger::Hata("vektor_karistir: liste kullanımda".to_string()),
            },
            [other] => Deger::Hata(format!(
                "vektor_karistir: vektör veya liste bekleniyordu; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "vektor_karistir: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // BLOK G — Gelişmiş Metin & Unicode Built-in'leri
    // ═══════════════════════════════════════════════════════════════════════

    globals.insert(
        "unicode_normalize".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(text)] => {
                let mut output = LimitedText::new(EN_FAZLA_DOSYA_BYTES);
                for character in text.nfc() {
                    if output.write_char(character).is_err() {
                        return Deger::Hata(format!(
                            "unicode_normalize: çıktı {} bayt sınırını aşıyor",
                            EN_FAZLA_DOSYA_BYTES
                        ));
                    }
                }
                Deger::Metin(output.text)
            }
            [other] => Deger::Hata(format!(
                "unicode_normalize: metin bekleniyordu; {} geldi",
                other
            )),
            _ => Deger::Hata(format!(
                "unicode_normalize: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // FNV-1a hash — hızlı, non-kriptografik; lookup table indekslemesi için
    globals.insert(
        "metin_hash".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(text)] => {
                let mut hash: u64 = 0xcbf29ce484222325u64;
                for byte in text.bytes() {
                    hash ^= byte as u64;
                    hash = hash.wrapping_mul(0x100000001b3u64);
                }
                // Hüma'nın tek sayı türünde tam temsil edilebilen 53 bit.
                Deger::Sayi((hash & EN_BUYUK_GUVENLI_SAYISAL_KIMLIK) as f64)
            }
            [other] => Deger::Hata(format!("metin_hash: metin bekleniyordu; {} geldi", other)),
            _ => Deger::Hata(format!(
                "metin_hash: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "bayt_metin".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(text)] if text.len() <= EN_FAZLA_BUILTIN_OGE => {
                Deger::Liste(Gc::from_cell(RefCell::new(
                    text.bytes().map(|byte| Deger::Sayi(byte as f64)).collect(),
                )))
            }
            [Deger::Metin(_)] => Deger::Hata(format!(
                "bayt_metin: çıktı {} öğelik güvenlik sınırını aşıyor",
                EN_FAZLA_BUILTIN_OGE
            )),
            [other] => Deger::Hata(format!("bayt_metin: metin bekleniyordu; {} geldi", other)),
            _ => Deger::Hata(format!(
                "bayt_metin: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "metin_bayt".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Liste(list)] = args.as_slice() else {
                return if args.len() == 1 {
                    Deger::Hata(format!("metin_bayt: liste bekleniyordu; {} geldi", args[0]))
                } else {
                    Deger::Hata(format!(
                        "metin_bayt: tam olarak 1 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            let values = match list.try_borrow() {
                Ok(values) => values,
                Err(_) => return Deger::Hata("metin_bayt: liste kullanımda".to_string()),
            };
            if values.len() > EN_FAZLA_BUILTIN_OGE {
                return Deger::Hata(format!(
                    "metin_bayt: liste {} öğelik güvenlik sınırını aşıyor",
                    EN_FAZLA_BUILTIN_OGE
                ));
            }
            let mut bytes = Vec::with_capacity(values.len());
            for (index, value) in values.iter().enumerate() {
                match value {
                    Deger::Sayi(number)
                        if number.is_finite()
                            && *number >= 0.0
                            && *number <= u8::MAX as f64
                            && number.fract() == 0.0 =>
                    {
                        bytes.push(*number as u8)
                    }
                    _ => {
                        return Deger::Hata(format!(
                            "metin_bayt: {index}. eleman 0..255 aralığında tamsayı olmalıdır"
                        ))
                    }
                }
            }
            match String::from_utf8(bytes) {
                Ok(text) => Deger::Metin(text),
                Err(error) => {
                    Deger::Hata(format!("metin_bayt: geçersiz UTF-8 bayt dizisi: {}", error))
                }
            }
        }),
    );

    // metin_benzerlik — normalized Levenshtein mesafesi (0.0 = farklı, 1.0 = aynı)
    globals.insert(
        "metin_benzerlik".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(left), Deger::Metin(right)] = args.as_slice() else {
                return if args.len() == 2 {
                    Deger::Hata("metin_benzerlik: iki metin argümanı gerekir".to_string())
                } else {
                    Deger::Hata(format!(
                        "metin_benzerlik: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            let left = left.chars().collect::<Vec<_>>();
            let right = right.chars().collect::<Vec<_>>();
            if left.len() > 4096 || right.len() > 4096 {
                return Deger::Hata(
                    "metin_benzerlik: metinler en fazla 4096 karakter olabilir".to_string(),
                );
            }
            if left.is_empty() && right.is_empty() {
                return Deger::Sayi(1.0);
            }
            let mut previous = (0..=right.len()).collect::<Vec<_>>();
            let mut current = vec![0usize; right.len() + 1];
            for (left_index, left_character) in left.iter().enumerate() {
                current[0] = left_index + 1;
                for (right_index, right_character) in right.iter().enumerate() {
                    let replacement_cost = usize::from(left_character != right_character);
                    current[right_index + 1] = (previous[right_index + 1] + 1)
                        .min(current[right_index] + 1)
                        .min(previous[right_index] + replacement_cost);
                }
                std::mem::swap(&mut previous, &mut current);
            }
            let distance = previous[right.len()];
            Deger::Sayi(1.0 - distance as f64 / left.len().max(right.len()) as f64)
        }),
    );

    // metin_sablon — basit {anahtar} şablon dönüşümü (sözlük tabanlı)
    globals.insert(
        "metin_sablon".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(template), values] = args.as_slice() else {
                return if args.len() == 2 {
                    Deger::Hata(
                        "metin_sablon: ilk argüman metin, ikinci argüman sözlük veya nesne olmalıdır"
                            .to_string(),
                    )
                } else {
                    Deger::Hata(format!(
                        "metin_sablon: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            let mut result = template.clone();
            if result.len() > EN_FAZLA_DOSYA_BYTES {
                return Deger::Hata(format!(
                    "metin_sablon: şablon {} bayt sınırını aşıyor",
                    EN_FAZLA_DOSYA_BYTES
                ));
            }
            let mut work = 0usize;
            match values {
                Deger::Sozluk(map) => {
                    let values = match map.try_borrow() {
                        Ok(values) => values,
                        Err(_) => return Deger::Hata("metin_sablon: sözlük kullanımda".to_string()),
                    };
                    if values.len() > EN_FAZLA_BUILTIN_OGE {
                        return Deger::Hata("metin_sablon: sözlük çok büyük".to_string());
                    }
                    for (key, value) in values.iter() {
                        work = match work.checked_add(result.len()) {
                            Some(work) if work <= EN_FAZLA_SAYISAL_ISLEM => work,
                            _ => {
                                return Deger::Hata(format!(
                                    "metin_sablon: iş yükü {} sınırını aşıyor",
                                    EN_FAZLA_SAYISAL_ISLEM
                                ))
                            }
                        };
                        let placeholder = format!("{{{key}}}");
                        let replacement = match display_value_limited(value, "metin_sablon") {
                            Ok(replacement) => replacement,
                            Err(error) => return Deger::Hata(error),
                        };
                        let count = result.matches(&placeholder).count();
                        if let Err(error) = replacement_output_size(
                            &result,
                            &placeholder,
                            &replacement,
                            count,
                            "metin_sablon",
                        ) {
                            return Deger::Hata(error);
                        }
                        result = result.replace(&placeholder, &replacement);
                    }
                }
                Deger::Nesne { alanlar, .. } => {
                    let fields = match alanlar.try_borrow() {
                        Ok(fields) => fields,
                        Err(_) => return Deger::Hata("metin_sablon: nesne kullanımda".to_string()),
                    };
                    if fields.len() > EN_FAZLA_BUILTIN_OGE {
                        return Deger::Hata("metin_sablon: nesne çok büyük".to_string());
                    }
                    for (key, value) in fields.iter() {
                        work = match work.checked_add(result.len()) {
                            Some(work) if work <= EN_FAZLA_SAYISAL_ISLEM => work,
                            _ => {
                                return Deger::Hata(format!(
                                    "metin_sablon: iş yükü {} sınırını aşıyor",
                                    EN_FAZLA_SAYISAL_ISLEM
                                ))
                            }
                        };
                        let placeholder = format!("{{{key}}}");
                        let replacement = match display_value_limited(value, "metin_sablon") {
                            Ok(replacement) => replacement,
                            Err(error) => return Deger::Hata(error),
                        };
                        let count = result.matches(&placeholder).count();
                        if let Err(error) = replacement_output_size(
                            &result,
                            &placeholder,
                            &replacement,
                            count,
                            "metin_sablon",
                        ) {
                            return Deger::Hata(error);
                        }
                        result = result.replace(&placeholder, &replacement);
                    }
                }
                other => {
                    return Deger::Hata(format!(
                        "metin_sablon: sözlük veya nesne bekleniyordu; {} geldi",
                        other
                    ))
                }
            }
            Deger::Metin(result)
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // P1.2 — Tensor Broadcasting & Toplu Matris Operasyonları
    // (Hüma döngüsü olmadan Rust'ta tam vektörize — kritik hız kazanımı)
    // ═══════════════════════════════════════════════════════════════════════

    // matris_satirlara_ekle(M, v) — v vektörünü M'nin her satırına ekle (bias addition)
    globals.insert(
        "matris_satirlara_ekle".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix, vector] = args.as_slice() else {
                return Deger::Hata(format!(
                    "matris_satirlara_ekle: tam olarak 2 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let (rows, columns, mut matrix_values) =
                match value_to_finite_matrix(matrix, "matris_satirlara_ekle") {
                    Ok(matrix) => matrix,
                    Err(error) => return Deger::Hata(error),
                };
            let vector_values = match value_to_finite_vector(vector, "matris_satirlara_ekle") {
                Ok(values) => values,
                Err(error) => return Deger::Hata(error),
            };
            if columns != vector_values.len() {
                return Deger::Hata(format!(
                    "matris_satirlara_ekle: matris sütun {} ≠ vektör boyutu {}",
                    columns,
                    vector_values.len()
                ));
            }
            for row in 0..rows {
                for column in 0..columns {
                    matrix_values[row * columns + column] += vector_values[column];
                }
            }
            if matrix_values.iter().any(|value| !value.is_finite()) {
                return Deger::Hata("matris_satirlara_ekle: sonlu olmayan sonuç".to_string());
            }
            Deger::Matris {
                satirlar: rows,
                sutunlar: columns,
                veri: Gc::from_cell(RefCell::new(matrix_values)),
            }
        }),
    );

    // matris_sutunlara_ekle(M, v) — v vektörünü M'nin her sütununa ekle
    globals.insert(
        "matris_sutunlara_ekle".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix, vector] = args.as_slice() else {
                return Deger::Hata(format!(
                    "matris_sutunlara_ekle: tam olarak 2 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let (rows, columns, mut matrix_values) =
                match value_to_finite_matrix(matrix, "matris_sutunlara_ekle") {
                    Ok(matrix) => matrix,
                    Err(error) => return Deger::Hata(error),
                };
            let vector_values = match value_to_finite_vector(vector, "matris_sutunlara_ekle") {
                Ok(values) => values,
                Err(error) => return Deger::Hata(error),
            };
            if rows != vector_values.len() {
                return Deger::Hata(format!(
                    "matris_sutunlara_ekle: matris satır {} ≠ vektör boyutu {}",
                    rows,
                    vector_values.len()
                ));
            }
            for row in 0..rows {
                for column in 0..columns {
                    matrix_values[row * columns + column] += vector_values[row];
                }
            }
            if matrix_values.iter().any(|value| !value.is_finite()) {
                return Deger::Hata("matris_sutunlara_ekle: sonlu olmayan sonuç".to_string());
            }
            Deger::Matris {
                satirlar: rows,
                sutunlar: columns,
                veri: Gc::from_cell(RefCell::new(matrix_values)),
            }
        }),
    );

    // matris_skalar_carp(M, s) — matrisin tüm elemanlarını skalar ile çarp
    globals.insert(
        "matris_skalar_carp".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix, Deger::Sayi(scalar)] = args.as_slice() else {
                return if args.len() == 2 {
                    Deger::Hata("matris_skalar_carp: matris ve sayı gerekir".to_string())
                } else {
                    Deger::Hata(format!(
                        "matris_skalar_carp: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            if !scalar.is_finite() {
                return Deger::Hata("matris_skalar_carp: skaler sonlu olmalıdır".to_string());
            }
            let (rows, columns, values) = match value_to_finite_matrix(matrix, "matris_skalar_carp")
            {
                Ok(matrix) => matrix,
                Err(error) => return Deger::Hata(error),
            };
            let result = values
                .into_iter()
                .map(|value| value * scalar)
                .collect::<Vec<_>>();
            if result.iter().any(|value| !value.is_finite()) {
                return Deger::Hata("matris_skalar_carp: sonlu olmayan sonuç".to_string());
            }
            Deger::Matris {
                satirlar: rows,
                sutunlar: columns,
                veri: Gc::from_cell(RefCell::new(result)),
            }
        }),
    );

    // matris_elemanlari_topla(M) — tüm elemanların toplamı
    globals.insert(
        "matris_elemanlari_topla".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix] = args.as_slice() else {
                return Deger::Hata(format!(
                    "matris_elemanlari_topla: tam olarak 1 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let (_, _, values) = match value_to_finite_matrix(matrix, "matris_elemanlari_topla") {
                Ok(matrix) => matrix,
                Err(error) => return Deger::Hata(error),
            };
            let sum = values.iter().sum::<f64>();
            if sum.is_finite() {
                Deger::Sayi(sum)
            } else {
                Deger::Hata("matris_elemanlari_topla: sonlu olmayan sonuç".to_string())
            }
        }),
    );

    // matris_satir_toplamları(M) — her satırın toplamı → Vektor [satirlar]
    globals.insert(
        "matris_satir_toplamları".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix] = args.as_slice() else {
                return Deger::Hata(format!(
                    "matris_satir_toplamları: tam olarak 1 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let (rows, columns, values) =
                match value_to_finite_matrix(matrix, "matris_satir_toplamları") {
                    Ok(matrix) => matrix,
                    Err(error) => return Deger::Hata(error),
                };
            let result = (0..rows)
                .map(|row| values[row * columns..(row + 1) * columns].iter().sum())
                .collect::<Vec<f64>>();
            if result.iter().any(|value| !value.is_finite()) {
                return Deger::Hata("matris_satir_toplamları: sonlu olmayan sonuç".to_string());
            }
            Deger::Vektor(Gc::from_cell(RefCell::new(result)))
        }),
    );

    // matris_sutun_toplamları(M) — her sütunun toplamı → Vektor [sutunlar]
    globals.insert(
        "matris_sutun_toplamları".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix] = args.as_slice() else {
                return Deger::Hata(format!(
                    "matris_sutun_toplamları: tam olarak 1 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let (rows, columns, values) =
                match value_to_finite_matrix(matrix, "matris_sutun_toplamları") {
                    Ok(matrix) => matrix,
                    Err(error) => return Deger::Hata(error),
                };
            let mut result = vec![0.0f64; columns];
            for row in 0..rows {
                for column in 0..columns {
                    result[column] += values[row * columns + column];
                }
            }
            if result.iter().any(|value| !value.is_finite()) {
                return Deger::Hata("matris_sutun_toplamları: sonlu olmayan sonuç".to_string());
            }
            Deger::Vektor(Gc::from_cell(RefCell::new(result)))
        }),
    );

    // batch_softmax(M) — her satıra softmax uygula, Matris döndür
    globals.insert(
        "batch_softmax".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix] = args.as_slice() else {
                return Deger::Hata(format!(
                    "batch_softmax: tam olarak 1 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let (rows, columns, values) = match value_to_finite_matrix(matrix, "batch_softmax") {
                Ok(matrix) => matrix,
                Err(error) => return Deger::Hata(error),
            };
            if columns == 0 && rows > 0 {
                return Deger::Hata(
                    "batch_softmax: satırlar en az bir sütun içermelidir".to_string(),
                );
            }
            let mut result = vec![0.0f64; values.len()];
            for row in 0..rows {
                let row_values = &values[row * columns..(row + 1) * columns];
                let Some(maximum) = row_values.iter().copied().reduce(f64::max) else {
                    continue;
                };
                let exponentials = row_values
                    .iter()
                    .map(|value| (value - maximum).exp())
                    .collect::<Vec<_>>();
                let sum = exponentials.iter().sum::<f64>();
                if !sum.is_finite() || sum <= 0.0 {
                    return Deger::Hata(
                        "batch_softmax: geçersiz normalizasyon toplamı".to_string(),
                    );
                }
                for (column, exponential) in exponentials.iter().enumerate() {
                    result[row * columns + column] = exponential / sum;
                }
            }
            Deger::Matris {
                satirlar: rows,
                sutunlar: columns,
                veri: Gc::from_cell(RefCell::new(result)),
            }
        }),
    );

    // matris_satir_normlari(M) — her satırın L2 normunu hesapla → Vektor
    globals.insert(
        "matris_satir_normlari".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix] = args.as_slice() else {
                return Deger::Hata(format!(
                    "matris_satir_normlari: tam olarak 1 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let (rows, columns, values) =
                match value_to_finite_matrix(matrix, "matris_satir_normlari") {
                    Ok(matrix) => matrix,
                    Err(error) => return Deger::Hata(error),
                };
            let result = (0..rows)
                .map(|row| {
                    values[row * columns..(row + 1) * columns]
                        .iter()
                        .map(|value| value * value)
                        .sum::<f64>()
                        .sqrt()
                })
                .collect::<Vec<_>>();
            if result.iter().any(|value| !value.is_finite()) {
                return Deger::Hata("matris_satir_normlari: sonlu olmayan sonuç".to_string());
            }
            Deger::Vektor(Gc::from_cell(RefCell::new(result)))
        }),
    );

    // vektor_dis_carpim(v1, v2) — dış çarpım → Matris [n1×n2]
    globals.insert(
        "vektor_dis_carpim".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [left, right] = args.as_slice() else {
                return Deger::Hata(format!(
                    "vektor_dis_carpim: tam olarak 2 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let left = match value_to_finite_vector(left, "vektor_dis_carpim") {
                Ok(values) => values,
                Err(error) => return Deger::Hata(error),
            };
            let right = match value_to_finite_vector(right, "vektor_dis_carpim") {
                Ok(values) => values,
                Err(error) => return Deger::Hata(error),
            };
            let element_count =
                match eleman_sayisi_dogrula(left.len(), right.len(), "vektor_dis_carpim") {
                    Ok(value) => value,
                    Err(error) => return Deger::Hata(error),
                };
            let mut result = Vec::with_capacity(element_count);
            for left_value in &left {
                for right_value in &right {
                    let value = left_value * right_value;
                    if !value.is_finite() {
                        return Deger::Hata("vektor_dis_carpim: sonlu olmayan sonuç".to_string());
                    }
                    result.push(value);
                }
            }
            Deger::Matris {
                satirlar: left.len(),
                sutunlar: right.len(),
                veri: Gc::from_cell(RefCell::new(result)),
            }
        }),
    );

    // Element-wise aktivasyon fonksiyonları (tüm matris üzerinde — döngüsüz)
    globals.insert(
        "matris_relu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            unary_matrix_builtin(args, "matris_relu", |value| value.max(0.0))
        }),
    );

    globals.insert(
        "matris_sigmoid".to_string(),
        Deger::DahiliFonksiyon(|args| {
            unary_matrix_builtin(args, "matris_sigmoid", |value| {
                if value >= 0.0 {
                    1.0 / (1.0 + (-value).exp())
                } else {
                    let exponential = value.exp();
                    exponential / (1.0 + exponential)
                }
            })
        }),
    );

    globals.insert(
        "matris_tanh_akt".to_string(),
        Deger::DahiliFonksiyon(|args| unary_matrix_builtin(args, "matris_tanh_akt", f64::tanh)),
    );

    globals.insert(
        "matris_gelu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            unary_matrix_builtin(args, "matris_gelu", |value| {
                0.5 * value
                    * (1.0
                        + (0.7978845608028654 * (value + 0.044715 * value * value * value)).tanh())
            })
        }),
    );

    // matris_klamp(M, min, max) — tüm elemanları sınırla
    globals.insert(
        "matris_klamp".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix, Deger::Sayi(minimum), Deger::Sayi(maximum)] = args.as_slice() else {
                return if args.len() == 3 {
                    Deger::Hata("matris_klamp: matris, alt ve üst sınır gerekir".to_string())
                } else {
                    Deger::Hata(format!(
                        "matris_klamp: tam olarak 3 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            if !minimum.is_finite() || !maximum.is_finite() || minimum > maximum {
                return Deger::Hata(
                    "matris_klamp: sonlu alt sınır üst sınırdan küçük/eşit olmalıdır".to_string(),
                );
            }
            let (rows, columns, values) = match value_to_finite_matrix(matrix, "matris_klamp") {
                Ok(matrix) => matrix,
                Err(error) => return Deger::Hata(error),
            };
            let result = values
                .into_iter()
                .map(|value| value.clamp(*minimum, *maximum))
                .collect();
            Deger::Matris {
                satirlar: rows,
                sutunlar: columns,
                veri: Gc::from_cell(RefCell::new(result)),
            }
        }),
    );

    // matris_topla(M1, M2) — element-wise matris toplama
    globals.insert(
        "matris_topla".to_string(),
        Deger::DahiliFonksiyon(|args| {
            binary_matrix_builtin(args, "matris_topla", |left, right| left + right)
        }),
    );

    // matris_cikart(M1, M2) — element-wise matris çıkarma
    globals.insert(
        "matris_cikart".to_string(),
        Deger::DahiliFonksiyon(|args| {
            binary_matrix_builtin(args, "matris_cikart", |left, right| left - right)
        }),
    );

    // matris_carpi_elemanlari(M1, M2) — element-wise (Hadamard) çarpım
    globals.insert(
        "matris_carpi_elemanlari".to_string(),
        Deger::DahiliFonksiyon(|args| {
            binary_matrix_builtin(args, "matris_carpi_elemanlari", |left, right| left * right)
        }),
    );

    // gradyan_kirp(vektor, maks_norm) — gradient clipping (exploding gradients için)
    globals.insert(
        "gradyan_kirp".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [value, Deger::Sayi(maximum_norm)] = args.as_slice() else {
                return if args.len() == 2 {
                    Deger::Hata(
                        "gradyan_kirp: vektör/matris ve azami norm sayısı gerekir".to_string(),
                    )
                } else {
                    Deger::Hata(format!(
                        "gradyan_kirp: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            if !maximum_norm.is_finite() || *maximum_norm <= 0.0 {
                return Deger::Hata(
                    "gradyan_kirp: azami norm pozitif ve sonlu olmalıdır".to_string(),
                );
            }
            match value {
                Deger::Vektor(_) | Deger::Liste(_) => {
                    let values = match value_to_finite_vector(value, "gradyan_kirp") {
                        Ok(values) => values,
                        Err(error) => return Deger::Hata(error),
                    };
                    let norm = values.iter().map(|value| value * value).sum::<f64>().sqrt();
                    if !norm.is_finite() {
                        return Deger::Hata("gradyan_kirp: sonlu olmayan norm".to_string());
                    }
                    if norm > *maximum_norm {
                        let scale = maximum_norm / norm;
                        Deger::Vektor(Gc::from_cell(RefCell::new(
                            values.into_iter().map(|value| value * scale).collect(),
                        )))
                    } else {
                        Deger::Vektor(Gc::from_cell(RefCell::new(values)))
                    }
                }
                Deger::Matris { .. } => {
                    let (rows, columns, values) =
                        match value_to_finite_matrix(value, "gradyan_kirp") {
                            Ok(matrix) => matrix,
                            Err(error) => return Deger::Hata(error),
                        };
                    let norm = values.iter().map(|value| value * value).sum::<f64>().sqrt();
                    if !norm.is_finite() {
                        return Deger::Hata("gradyan_kirp: sonlu olmayan norm".to_string());
                    }
                    let result = if norm > *maximum_norm {
                        let scale = maximum_norm / norm;
                        values.into_iter().map(|value| value * scale).collect()
                    } else {
                        values
                    };
                    Deger::Matris {
                        satirlar: rows,
                        sutunlar: columns,
                        veri: Gc::from_cell(RefCell::new(result)),
                    }
                }
                other => Deger::Hata(format!(
                    "gradyan_kirp: vektör, liste veya matris bekleniyordu; {} geldi",
                    other
                )),
            }
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // P1.4 — Boyut Doğrulama (Erken Hata Tespiti)
    // ═══════════════════════════════════════════════════════════════════════

    // matris_dogrula(M, beklenen_satir, beklenen_sutun) → 1 ya da Hata
    globals.insert(
        "matris_dogrula".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [matrix @ Deger::Matris { .. }, Deger::Sayi(expected_rows), Deger::Sayi(expected_columns)] => {
                let expected_rows =
                    match boyut_dogrula(*expected_rows, "matris_dogrula", true) {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                let expected_columns =
                    match boyut_dogrula(*expected_columns, "matris_dogrula", true) {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                let (actual_rows, actual_columns, _) =
                    match value_to_finite_matrix(matrix, "matris_dogrula") {
                        Ok(matrix) => matrix,
                        Err(error) => return Deger::Hata(error),
                    };
                if actual_rows != expected_rows || actual_columns != expected_columns {
                    return Deger::Hata(format!(
                        "matris_dogrula: beklenen {}×{}, bulunan {}×{}",
                        expected_rows, expected_columns, actual_rows, actual_columns
                    ));
                }
                Deger::Sayi(1.0)
            }
            [_, _, _] => Deger::Hata(
                "matris_dogrula: matris ile iki negatif olmayan tamsayı boyut gerekir".to_string(),
            ),
            _ => Deger::Hata(format!(
                "matris_dogrula: tam olarak 3 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // vektor_dogrula(v, beklenen_boyut) → 1 ya da Hata
    globals.insert(
        "vektor_dogrula".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [vector @ Deger::Vektor(_), Deger::Sayi(expected)] => {
                let expected = match boyut_dogrula(*expected, "vektor_dogrula", true) {
                    Ok(value) => value,
                    Err(error) => return Deger::Hata(error),
                };
                let actual = match value_to_finite_vector(vector, "vektor_dogrula") {
                    Ok(values) => values.len(),
                    Err(error) => return Deger::Hata(error),
                };
                if actual != expected {
                    return Deger::Hata(format!(
                        "vektor_dogrula: beklenen {}, bulunan {}",
                        expected, actual
                    ));
                }
                Deger::Sayi(1.0)
            }
            [_, _] => Deger::Hata(
                "vektor_dogrula: vektör ve negatif olmayan tamsayı boyut gerekir".to_string(),
            ),
            _ => Deger::Hata(format!(
                "vektor_dogrula: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // boyut_esit_mi(a, b) → 1 ya da 0 — iki vektör/matrisin boyutu eşit mi?
    globals.insert(
        "boyut_esit_mi".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [left @ Deger::Vektor(_), right @ Deger::Vektor(_)] => {
                let left_len = match value_to_finite_vector(left, "boyut_esit_mi") {
                    Ok(values) => values.len(),
                    Err(error) => return Deger::Hata(error),
                };
                let right_len = match value_to_finite_vector(right, "boyut_esit_mi") {
                    Ok(values) => values.len(),
                    Err(error) => return Deger::Hata(error),
                };
                Deger::Sayi(if left_len == right_len { 1.0 } else { 0.0 })
            }
            [left @ Deger::Matris { .. }, right @ Deger::Matris { .. }] => {
                let (left_rows, left_columns, _) =
                    match value_to_finite_matrix(left, "boyut_esit_mi") {
                        Ok(matrix) => matrix,
                        Err(error) => return Deger::Hata(error),
                    };
                let (right_rows, right_columns, _) =
                    match value_to_finite_matrix(right, "boyut_esit_mi") {
                        Ok(matrix) => matrix,
                        Err(error) => return Deger::Hata(error),
                    };
                Deger::Sayi(
                    if left_rows == right_rows && left_columns == right_columns {
                        1.0
                    } else {
                        0.0
                    },
                )
            }
            [_, _] => Deger::Hata(
                "boyut_esit_mi: iki vektör veya iki matris karşılaştırılabilir".to_string(),
            ),
            _ => Deger::Hata(format!(
                "boyut_esit_mi: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // P2.3 — Profiling & İlerleme Takibi
    // ═══════════════════════════════════════════════════════════════════════

    // zamanlayici_baslat() → anlık zaman (f64 saniye, yüksek çözünürlük)
    globals.insert(
        "zamanlayici_baslat".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if !args.is_empty() {
                return Deger::Hata(format!(
                    "zamanlayici_baslat: argüman beklenmiyordu; {} geldi",
                    args.len()
                ));
            }
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(duration) => Deger::Sayi(duration.as_secs_f64()),
                Err(error) => Deger::Hata(format!(
                    "zamanlayici_baslat: sistem saati Unix başlangıcından önce: {error}"
                )),
            }
        }),
    );

    // zamanlayici_bitir(baslangic) → geçen süre ms cinsinden
    globals.insert(
        "zamanlayici_bitir".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(baslangic)] = args.as_slice() else {
                return Deger::Hata(
                    "zamanlayici_bitir: tam olarak bir sayısal başlangıç değeri gerekir"
                        .to_string(),
                );
            };
            if !baslangic.is_finite() || *baslangic < 0.0 {
                return Deger::Hata(
                    "zamanlayici_bitir: başlangıç sonlu ve negatif olmayan sayı olmalı".to_string(),
                );
            }
            let simdi = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(duration) => duration.as_secs_f64(),
                Err(error) => {
                    return Deger::Hata(format!(
                        "zamanlayici_bitir: sistem saati Unix başlangıcından önce: {error}"
                    ))
                }
            };
            let elapsed = (simdi - baslangic) * 1000.0;
            if !elapsed.is_finite() || elapsed < 0.0 {
                Deger::Hata(
                    "zamanlayici_bitir: başlangıç sistem saatinden sonra olamaz".to_string(),
                )
            } else {
                Deger::Sayi(elapsed)
            }
        }),
    );

    // ilerleme_cubugu(simdi, toplam, mesaj?) → kotaya tabi yazdırma için metin üret
    globals.insert(
        "ilerleme_cubugu".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Sayi(current), Deger::Sayi(total)]
            | [Deger::Sayi(current), Deger::Sayi(total), Deger::Metin(_)] => {
                if !current.is_finite()
                    || !total.is_finite()
                    || *total <= 0.0
                    || *current < 0.0
                    || *current > *total
                {
                    return Deger::Hata(
                        "ilerleme_cubugu: ilerleme 0 ile pozitif toplam arasında olmalıdır"
                            .to_string(),
                    );
                }
                let message = match args.get(2) {
                    Some(Deger::Metin(message)) => message,
                    None => "",
                    Some(_) => {
                        return Deger::Hata(
                            "ilerleme_cubugu: isteğe bağlı mesaj metin olmalıdır".to_string(),
                        )
                    }
                };
                if message.len() > 4096 {
                    return Deger::Hata("ilerleme_cubugu: mesaj 4096 baytı aşamaz".to_string());
                }
                let percentage = ((*current / *total) * 100.0).floor() as usize;
                let filled = percentage / 5;
                let empty = 20usize.saturating_sub(filled);
                Deger::Metin(format!(
                    "[{}{}] {}% {}",
                    "█".repeat(filled),
                    "░".repeat(empty),
                    percentage,
                    message
                ))
            }
            [_, _] | [_, _, _] => Deger::Hata(
                "ilerleme_cubugu: iki sayı ve isteğe bağlı metin mesajı gerekir".to_string(),
            ),
            _ => Deger::Hata(format!(
                "ilerleme_cubugu: 2 veya 3 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // P2.4 — Model Değerlendirme Metrikleri
    // ═══════════════════════════════════════════════════════════════════════

    // f1_skoru(tahmin_listesi, gercek_listesi) → {f1, precision, recall} sözlüğü
    globals.insert(
        "f1_skoru".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [predictions, actuals] => {
                let predictions = match value_to_finite_vector(predictions, "f1_skoru") {
                    Ok(values) => values,
                    Err(error) => return Deger::Hata(error),
                };
                let actuals = match value_to_finite_vector(actuals, "f1_skoru") {
                    Ok(values) => values,
                    Err(error) => return Deger::Hata(error),
                };
                if predictions.len() != actuals.len() {
                    return Deger::Hata("f1_skoru: liste uzunlukları eşit olmalıdır".to_string());
                }
                if predictions.is_empty() {
                    return Deger::Hata("f1_skoru: boş veri kümesi tanımsızdır".to_string());
                }
                if predictions
                    .iter()
                    .chain(actuals.iter())
                    .any(|value| !(0.0..=1.0).contains(value))
                {
                    return Deger::Hata(
                        "f1_skoru: bütün değerler 0 ile 1 arasında olmalıdır".to_string(),
                    );
                }
                let mut tp = 0.0f64;
                let mut fp = 0.0f64;
                let mut fn_ = 0.0f64;
                for (prediction, actual) in predictions.iter().zip(actuals.iter()) {
                    match (*prediction >= 0.5, *actual >= 0.5) {
                        (true, true) => tp += 1.0,
                        (true, false) => fp += 1.0,
                        (false, true) => fn_ += 1.0,
                        _ => {}
                    }
                }
                let precision = if tp + fp > 0.0 { tp / (tp + fp) } else { 0.0 };
                let recall = if tp + fn_ > 0.0 { tp / (tp + fn_) } else { 0.0 };
                let f1 = if precision + recall > 0.0 {
                    2.0 * precision * recall / (precision + recall)
                } else {
                    0.0
                };
                let mut m = std::collections::HashMap::new();
                m.insert("f1".to_string(), Deger::Sayi(f1));
                m.insert("precision".to_string(), Deger::Sayi(precision));
                m.insert("recall".to_string(), Deger::Sayi(recall));
                m.insert("tp".to_string(), Deger::Sayi(tp));
                m.insert("fp".to_string(), Deger::Sayi(fp));
                m.insert("fn".to_string(), Deger::Sayi(fn_));
                Deger::Sozluk(Gc::from_cell(RefCell::new(m)))
            }
            _ => Deger::Hata(format!(
                "f1_skoru: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // karisiklik_matrisi(tahmin, gercek, sinif_sayisi) → Matris [n×n]
    globals.insert(
        "karisiklik_matrisi".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [predictions, actuals, Deger::Sayi(class_count)] => {
                let class_count = match boyut_dogrula(*class_count, "karisiklik_matrisi", false) {
                    Ok(value) => value,
                    Err(error) => return Deger::Hata(error),
                };
                let cell_count =
                    match eleman_sayisi_dogrula(class_count, class_count, "karisiklik_matrisi") {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                let predictions = match value_to_finite_vector(predictions, "karisiklik_matrisi") {
                    Ok(values) => values,
                    Err(error) => return Deger::Hata(error),
                };
                let actuals = match value_to_finite_vector(actuals, "karisiklik_matrisi") {
                    Ok(values) => values,
                    Err(error) => return Deger::Hata(error),
                };
                if predictions.len() != actuals.len() {
                    return Deger::Hata(
                        "karisiklik_matrisi: liste uzunlukları eşit olmalıdır".to_string(),
                    );
                }
                if predictions.is_empty() {
                    return Deger::Hata(
                        "karisiklik_matrisi: boş veri kümesi tanımsızdır".to_string(),
                    );
                }
                let mut matrix = vec![0.0f64; cell_count];
                for (index, (prediction, actual)) in
                    predictions.iter().zip(actuals.iter()).enumerate()
                {
                    if prediction.fract() != 0.0
                        || actual.fract() != 0.0
                        || *prediction < 0.0
                        || *actual < 0.0
                        || *prediction >= class_count as f64
                        || *actual >= class_count as f64
                    {
                        return Deger::Hata(format!(
                            "karisiklik_matrisi: {}. etiket 0 ile {} arasında tamsayı olmalıdır",
                            index + 1,
                            class_count - 1
                        ));
                    }
                    let prediction = *prediction as usize;
                    let actual = *actual as usize;
                    matrix[actual * class_count + prediction] += 1.0;
                }
                Deger::Matris {
                    satirlar: class_count,
                    sutunlar: class_count,
                    veri: Gc::from_cell(RefCell::new(matrix)),
                }
            }
            [_, _, _] => Deger::Hata(
                "karisiklik_matrisi: iki sayı listesi ve pozitif sınıf sayısı gerekir".to_string(),
            ),
            _ => Deger::Hata(format!(
                "karisiklik_matrisi: tam olarak 3 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // perplexity(log_olasiliklar_listesi) → e^(-ortalama_log_olasilik)
    globals.insert(
        "perplexity".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [values] => {
                let values = match value_to_finite_vector(values, "perplexity") {
                    Ok(values) => values,
                    Err(error) => return Deger::Hata(error),
                };
                if values.is_empty() {
                    return Deger::Hata("perplexity: boş veri kümesi tanımsızdır".to_string());
                }
                if values.iter().any(|value| *value > 0.0) {
                    return Deger::Hata(
                        "perplexity: log olasılıkları sıfırdan büyük olamaz".to_string(),
                    );
                }
                let count = values.len() as f64;
                let mean = values.iter().map(|value| value / count).sum::<f64>();
                let result = (-mean).exp();
                if result.is_finite() {
                    Deger::Sayi(result)
                } else {
                    Deger::Hata("perplexity: sonuç sonlu sayı aralığını aşıyor".to_string())
                }
            }
            _ => Deger::Hata(format!(
                "perplexity: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // ML HIZI — Rust-native matris ilklendirme ve rastgele matris üretme
    // Bu built-in'ler Hüma döngüsü OLMADAN çalışır; büyük ağ katmanlarında
    // (512×256 vb.) He/Xavier init'i 10-100× hızlandırır.
    // ═══════════════════════════════════════════════════════════════════════

    // matris_he_ilklendir_builtin(satirlar, sutunlar)
    // He başlangıcı: N(0, sqrt(2/fan_in)), ReLU katmanları için önerilen
    globals.insert(
        "matris_he_ilklendir_builtin".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(rows_f), Deger::Sayi(cols_f)] = args.as_slice() else {
                return if args.len() == 2 {
                    Deger::Hata(
                        "matris_he_ilklendir_builtin: iki sayı (satır, sütun) gerekir".to_string(),
                    )
                } else {
                    Deger::Hata(format!(
                        "matris_he_ilklendir_builtin: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            let rows = match boyut_dogrula(*rows_f, "matris_he_ilklendir_builtin", false) {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let cols = match boyut_dogrula(*cols_f, "matris_he_ilklendir_builtin", false) {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let total = match eleman_sayisi_dogrula(rows, cols, "matris_he_ilklendir_builtin") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let std_dev = (2.0f64 / rows as f64).sqrt();
            // Box-Muller Normal dağılım — RNG thread_local ile
            thread_local! {
                static HE_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
            }
            let mut result = Vec::with_capacity(total);
            let ok = HE_RNG.with(|rng| match rng.try_borrow_mut() {
                Ok(mut rng) => {
                    let mut i = 0usize;
                    while i < total {
                        let u1 = (rng.gen::<f64>()).max(1e-10);
                        let u2 = rng.gen::<f64>();
                        let r = (-2.0 * u1.ln()).sqrt();
                        let theta = 2.0 * std::f64::consts::PI * u2;
                        let z0 = r * theta.cos() * std_dev;
                        let z1 = r * theta.sin() * std_dev;
                        result.push(z0);
                        if i + 1 < total {
                            result.push(z1);
                        }
                        i += 2;
                    }
                    true
                }
                Err(_) => false,
            });
            if !ok {
                return Deger::Hata(
                    "matris_he_ilklendir_builtin: rastgele sayı üreteci kullanımda".to_string(),
                );
            }
            if result.iter().any(|v| !v.is_finite()) {
                return Deger::Hata(
                    "matris_he_ilklendir_builtin: sonlu olmayan değer üretildi".to_string(),
                );
            }
            Deger::Matris {
                satirlar: rows,
                sutunlar: cols,
                veri: Gc::from_cell(RefCell::new(result)),
            }
        }),
    );

    // matris_xavier_ilklendir_builtin(satirlar, sutunlar)
    // Xavier başlangıcı: U(-sqrt(6/(fan_in+fan_out)), +sqrt(6/(fan_in+fan_out)))
    // Sigmoid/Tanh katmanları için önerilen
    globals.insert(
        "matris_xavier_ilklendir_builtin".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(rows_f), Deger::Sayi(cols_f)] = args.as_slice() else {
                return if args.len() == 2 {
                    Deger::Hata(
                        "matris_xavier_ilklendir_builtin: iki sayı (satır, sütun) gerekir"
                            .to_string(),
                    )
                } else {
                    Deger::Hata(format!(
                        "matris_xavier_ilklendir_builtin: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            let rows = match boyut_dogrula(*rows_f, "matris_xavier_ilklendir_builtin", false) {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let cols = match boyut_dogrula(*cols_f, "matris_xavier_ilklendir_builtin", false) {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let total = match eleman_sayisi_dogrula(rows, cols, "matris_xavier_ilklendir_builtin") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let limit = (6.0f64 / (rows + cols) as f64).sqrt();
            thread_local! {
                static XAV_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
            }
            let mut result = Vec::with_capacity(total);
            let ok = XAV_RNG.with(|rng| match rng.try_borrow_mut() {
                Ok(mut rng) => {
                    for _ in 0..total {
                        let v = -limit + rng.gen::<f64>() * 2.0 * limit;
                        result.push(v);
                    }
                    true
                }
                Err(_) => false,
            });
            if !ok {
                return Deger::Hata(
                    "matris_xavier_ilklendir_builtin: rastgele sayı üreteci kullanımda".to_string(),
                );
            }
            if result.iter().any(|v| !v.is_finite()) {
                return Deger::Hata(
                    "matris_xavier_ilklendir_builtin: sonlu olmayan değer üretildi".to_string(),
                );
            }
            Deger::Matris {
                satirlar: rows,
                sutunlar: cols,
                veri: Gc::from_cell(RefCell::new(result)),
            }
        }),
    );

    // matris_normal_rastgele(satirlar, sutunlar, ortalama, std_sapma)
    // Belirtilen normal dağılımdan matris üret (Gaussian noise, weight init vb.)
    globals.insert(
        "matris_normal_rastgele".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(rows_f), Deger::Sayi(cols_f), Deger::Sayi(mean), Deger::Sayi(std_dev)] =
                args.as_slice()
            else {
                return if args.len() == 4 {
                    Deger::Hata(
                        "matris_normal_rastgele: satır, sütun, ortalama, std_sapma sayı olmalı"
                            .to_string(),
                    )
                } else {
                    Deger::Hata(format!(
                        "matris_normal_rastgele: tam olarak 4 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            if !mean.is_finite() || !std_dev.is_finite() || *std_dev < 0.0 {
                return Deger::Hata(
                    "matris_normal_rastgele: ortalama sonlu, std_sapma ≥ 0 olmalı".to_string(),
                );
            }
            let rows = match boyut_dogrula(*rows_f, "matris_normal_rastgele", false) {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let cols = match boyut_dogrula(*cols_f, "matris_normal_rastgele", false) {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let total = match eleman_sayisi_dogrula(rows, cols, "matris_normal_rastgele") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            thread_local! {
                static NRM_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
            }
            let mut result = Vec::with_capacity(total);
            let ok = NRM_RNG.with(|rng| match rng.try_borrow_mut() {
                Ok(mut rng) => {
                    let mut i = 0usize;
                    while i < total {
                        let u1 = (rng.gen::<f64>()).max(1e-10);
                        let u2 = rng.gen::<f64>();
                        let r = (-2.0 * u1.ln()).sqrt();
                        let theta = 2.0 * std::f64::consts::PI * u2;
                        let z0 = mean + std_dev * r * theta.cos();
                        let z1 = mean + std_dev * r * theta.sin();
                        result.push(z0);
                        if i + 1 < total {
                            result.push(z1);
                        }
                        i += 2;
                    }
                    true
                }
                Err(_) => false,
            });
            if !ok {
                return Deger::Hata(
                    "matris_normal_rastgele: rastgele sayı üreteci kullanımda".to_string(),
                );
            }
            if result.iter().any(|v| !v.is_finite()) {
                return Deger::Hata(
                    "matris_normal_rastgele: sonlu olmayan değer üretildi".to_string(),
                );
            }
            Deger::Matris {
                satirlar: rows,
                sutunlar: cols,
                veri: Gc::from_cell(RefCell::new(result)),
            }
        }),
    );

    // matris_uniform_rastgele(satirlar, sutunlar, alt, ust)
    // Uniform dağılımdan matris üret
    globals.insert(
        "matris_uniform_rastgele".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(rows_f), Deger::Sayi(cols_f), Deger::Sayi(low), Deger::Sayi(high)] =
                args.as_slice()
            else {
                return if args.len() == 4 {
                    Deger::Hata(
                        "matris_uniform_rastgele: satır, sütun, alt, üst sayı olmalı".to_string(),
                    )
                } else {
                    Deger::Hata(format!(
                        "matris_uniform_rastgele: tam olarak 4 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            if !low.is_finite() || !high.is_finite() || low >= high {
                return Deger::Hata(
                    "matris_uniform_rastgele: sonlu alt sınır üst sınırdan küçük olmalıdır"
                        .to_string(),
                );
            }
            let rows = match boyut_dogrula(*rows_f, "matris_uniform_rastgele", false) {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let cols = match boyut_dogrula(*cols_f, "matris_uniform_rastgele", false) {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let total = match eleman_sayisi_dogrula(rows, cols, "matris_uniform_rastgele") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            thread_local! {
                static UNI_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
            }
            let mut result = Vec::with_capacity(total);
            let ok = UNI_RNG.with(|rng| match rng.try_borrow_mut() {
                Ok(mut rng) => {
                    for _ in 0..total {
                        let v = low + rng.gen::<f64>() * (high - low);
                        result.push(v);
                    }
                    true
                }
                Err(_) => false,
            });
            if !ok {
                return Deger::Hata(
                    "matris_uniform_rastgele: rastgele sayı üreteci kullanımda".to_string(),
                );
            }
            if result.iter().any(|v| !v.is_finite()) {
                return Deger::Hata(
                    "matris_uniform_rastgele: sonlu olmayan değer üretildi".to_string(),
                );
            }
            Deger::Matris {
                satirlar: rows,
                sutunlar: cols,
                veri: Gc::from_cell(RefCell::new(result)),
            }
        }),
    );

    // matris_satir_maksimum(M) → Vektor: her satırın maksimum değeri
    // Sayısal kararlı softmax ve log-sum-exp için kritik
    globals.insert(
        "matris_satir_maksimum".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix] = args.as_slice() else {
                return Deger::Hata(format!(
                    "matris_satir_maksimum: tam olarak 1 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let (rows, columns, values) =
                match value_to_finite_matrix(matrix, "matris_satir_maksimum") {
                    Ok(m) => m,
                    Err(e) => return Deger::Hata(e),
                };
            if columns == 0 {
                return Deger::Hata(
                    "matris_satir_maksimum: matrisin en az 1 sütunu olmalıdır".to_string(),
                );
            }
            let result: Vec<f64> = (0..rows)
                .map(|r| {
                    values[r * columns..(r + 1) * columns]
                        .iter()
                        .copied()
                        .fold(f64::NEG_INFINITY, f64::max)
                })
                .collect();
            if result.iter().any(|v| !v.is_finite()) {
                return Deger::Hata("matris_satir_maksimum: sonlu olmayan sonuç".to_string());
            }
            Deger::Vektor(Gc::from_cell(RefCell::new(result)))
        }),
    );

    // matris_satir_ortalamalar(M) → Vektor: her satırın ortalaması (batch norm vb.)
    globals.insert(
        "matris_satir_ortalamalar".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix] = args.as_slice() else {
                return Deger::Hata(format!(
                    "matris_satir_ortalamalar: tam olarak 1 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let (rows, columns, values) =
                match value_to_finite_matrix(matrix, "matris_satir_ortalamalar") {
                    Ok(m) => m,
                    Err(e) => return Deger::Hata(e),
                };
            if columns == 0 {
                return Deger::Hata(
                    "matris_satir_ortalamalar: matrisin en az 1 sütunu olmalıdır".to_string(),
                );
            }
            let result: Vec<f64> = (0..rows)
                .map(|r| {
                    let sum: f64 = values[r * columns..(r + 1) * columns].iter().sum();
                    sum / columns as f64
                })
                .collect();
            if result.iter().any(|v| !v.is_finite()) {
                return Deger::Hata("matris_satir_ortalamalar: sonlu olmayan sonuç".to_string());
            }
            Deger::Vektor(Gc::from_cell(RefCell::new(result)))
        }),
    );

    // matris_satir_varyanslar(M) → Vektor: her satırın varyansı (batch norm için)
    globals.insert(
        "matris_satir_varyanslar".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix] = args.as_slice() else {
                return Deger::Hata(format!(
                    "matris_satir_varyanslar: tam olarak 1 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let (rows, columns, values) =
                match value_to_finite_matrix(matrix, "matris_satir_varyanslar") {
                    Ok(m) => m,
                    Err(e) => return Deger::Hata(e),
                };
            if columns == 0 {
                return Deger::Hata(
                    "matris_satir_varyanslar: matrisin en az 1 sütunu olmalıdır".to_string(),
                );
            }
            let result: Vec<f64> = (0..rows)
                .map(|r| {
                    let row = &values[r * columns..(r + 1) * columns];
                    let mean = row.iter().sum::<f64>() / columns as f64;
                    let var =
                        row.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / columns as f64;
                    var
                })
                .collect();
            if result.iter().any(|v| !v.is_finite()) {
                return Deger::Hata("matris_satir_varyanslar: sonlu olmayan sonuç".to_string());
            }
            Deger::Vektor(Gc::from_cell(RefCell::new(result)))
        }),
    );

    // vektor_dropout(v, oran, egitim_modu) → yeni Vektör
    // Dropout: eğitim modunda rastgele elemanları sıfırla ve inverted scaling uygula
    globals.insert(
        "vektor_dropout".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [vector, Deger::Sayi(rate), Deger::Sayi(training)] = args.as_slice() else {
                return if args.len() == 3 {
                    Deger::Hata(
                        "vektor_dropout: vektör, oran (0-1), eğitim_modu (0/1) gerekir".to_string(),
                    )
                } else {
                    Deger::Hata(format!(
                        "vektor_dropout: tam olarak 3 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            if !rate.is_finite() || *rate < 0.0 || *rate >= 1.0 {
                return Deger::Hata("vektor_dropout: oran [0, 1) aralığında olmalıdır".to_string());
            }
            let values = match value_to_finite_vector(vector, "vektor_dropout") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            // inference modda identity
            if *training == 0.0 {
                return Deger::Vektor(Gc::from_cell(RefCell::new(values)));
            }
            thread_local! {
                static DROP_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
            }
            let keep_prob = 1.0 - rate;
            let scale = if keep_prob > 0.0 {
                1.0 / keep_prob
            } else {
                0.0
            };
            let mut result = Vec::with_capacity(values.len());
            let ok = DROP_RNG.with(|rng| match rng.try_borrow_mut() {
                Ok(mut rng) => {
                    for v in &values {
                        if rng.gen::<f64>() < *rate {
                            result.push(0.0);
                        } else {
                            result.push(v * scale);
                        }
                    }
                    true
                }
                Err(_) => false,
            });
            if !ok {
                return Deger::Hata("vektor_dropout: rastgele sayı üreteci kullanımda".to_string());
            }
            Deger::Vektor(Gc::from_cell(RefCell::new(result)))
        }),
    );

    // matris_dropout(M, oran, egitim_modu) → yeni Matris
    globals.insert(
        "matris_dropout".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix, Deger::Sayi(rate), Deger::Sayi(training)] = args.as_slice() else {
                return if args.len() == 3 {
                    Deger::Hata(
                        "matris_dropout: matris, oran (0-1), eğitim_modu (0/1) gerekir".to_string(),
                    )
                } else {
                    Deger::Hata(format!(
                        "matris_dropout: tam olarak 3 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            if !rate.is_finite() || *rate < 0.0 || *rate >= 1.0 {
                return Deger::Hata("matris_dropout: oran [0, 1) aralığında olmalıdır".to_string());
            }
            let (rows, cols, values) = match value_to_finite_matrix(matrix, "matris_dropout") {
                Ok(m) => m,
                Err(e) => return Deger::Hata(e),
            };
            if *training == 0.0 {
                return Deger::Matris {
                    satirlar: rows,
                    sutunlar: cols,
                    veri: Gc::from_cell(RefCell::new(values)),
                };
            }
            thread_local! {
                static MDRP_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
            }
            let keep_prob = 1.0 - rate;
            let scale = if keep_prob > 0.0 {
                1.0 / keep_prob
            } else {
                0.0
            };
            let mut result = Vec::with_capacity(values.len());
            let ok = MDRP_RNG.with(|rng| match rng.try_borrow_mut() {
                Ok(mut rng) => {
                    for v in &values {
                        if rng.gen::<f64>() < *rate {
                            result.push(0.0);
                        } else {
                            result.push(v * scale);
                        }
                    }
                    true
                }
                Err(_) => false,
            });
            if !ok {
                return Deger::Hata("matris_dropout: rastgele sayı üreteci kullanımda".to_string());
            }
            Deger::Matris {
                satirlar: rows,
                sutunlar: cols,
                veri: Gc::from_cell(RefCell::new(result)),
            }
        }),
    );

    // matris_batch_norm(M, gamma_v, beta_v, epsilon) → normalize edilmiş Matris
    // Batch Normalization: her sütun bazında normalize et, gamma ile ölçekle, beta ekle
    globals.insert(
        "matris_batch_norm".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix, gamma, beta, Deger::Sayi(epsilon)] = args.as_slice() else {
                return if args.len() == 4 {
                    Deger::Hata(
                        "matris_batch_norm: matris, gamma_vektör, beta_vektör, epsilon gerekir"
                            .to_string(),
                    )
                } else {
                    Deger::Hata(format!(
                        "matris_batch_norm: tam olarak 4 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            if !epsilon.is_finite() || *epsilon <= 0.0 {
                return Deger::Hata(
                    "matris_batch_norm: epsilon pozitif ve sonlu olmalıdır".to_string(),
                );
            }
            let (rows, cols, values) = match value_to_finite_matrix(matrix, "matris_batch_norm") {
                Ok(m) => m,
                Err(e) => return Deger::Hata(e),
            };
            let gamma_vals = match value_to_finite_vector(gamma, "matris_batch_norm") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let beta_vals = match value_to_finite_vector(beta, "matris_batch_norm") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            if gamma_vals.len() != cols || beta_vals.len() != cols {
                return Deger::Hata(format!(
                    "matris_batch_norm: gamma ve beta sütun sayısı ({}) ile eşleşmeli",
                    cols
                ));
            }
            // Sütun bazında mean ve variance
            let mut col_means = vec![0.0f64; cols];
            let mut col_vars = vec![0.0f64; cols];
            for c in 0..cols {
                let sum: f64 = (0..rows).map(|r| values[r * cols + c]).sum();
                col_means[c] = sum / rows as f64;
            }
            for c in 0..cols {
                let var: f64 = (0..rows)
                    .map(|r| {
                        let d = values[r * cols + c] - col_means[c];
                        d * d
                    })
                    .sum::<f64>()
                    / rows as f64;
                col_vars[c] = var;
            }
            let mut result = Vec::with_capacity(rows * cols);
            for r in 0..rows {
                for c in 0..cols {
                    let x_hat =
                        (values[r * cols + c] - col_means[c]) / (col_vars[c] + epsilon).sqrt();
                    let y = gamma_vals[c] * x_hat + beta_vals[c];
                    if !y.is_finite() {
                        return Deger::Hata("matris_batch_norm: sonlu olmayan sonuç".to_string());
                    }
                    result.push(y);
                }
            }
            Deger::Matris {
                satirlar: rows,
                sutunlar: cols,
                veri: Gc::from_cell(RefCell::new(result)),
            }
        }),
    );

    // matris_duzenle(M, yeni_satirlar, yeni_sutunlar) → yeniden şekillendir
    // Toplam eleman sayısı korunur. Flatten (1, N) dahil.
    globals.insert(
        "matris_duzenle".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix, Deger::Sayi(new_rows_f), Deger::Sayi(new_cols_f)] = args.as_slice() else {
                return if args.len() == 3 {
                    Deger::Hata(
                        "matris_duzenle: matris, yeni_satır_sayısı, yeni_sütun_sayısı gerekir"
                            .to_string(),
                    )
                } else {
                    Deger::Hata(format!(
                        "matris_duzenle: tam olarak 3 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            let (_, _, values) = match value_to_finite_matrix(matrix, "matris_duzenle") {
                Ok(m) => m,
                Err(e) => return Deger::Hata(e),
            };
            let new_rows = match boyut_dogrula(*new_rows_f, "matris_duzenle", false) {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let new_cols = match boyut_dogrula(*new_cols_f, "matris_duzenle", false) {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let new_total = match eleman_sayisi_dogrula(new_rows, new_cols, "matris_duzenle") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            if new_total != values.len() {
                return Deger::Hata(format!(
                    "matris_duzenle: toplam eleman sayısı değişemez ({} → {})",
                    values.len(),
                    new_total
                ));
            }
            Deger::Matris {
                satirlar: new_rows,
                sutunlar: new_cols,
                veri: Gc::from_cell(RefCell::new(values)),
            }
        }),
    );

    // vektor_tekrarla(v, n) → v vektörünü n kez yan yana koyarak Matris [n × uzunluk]
    // Mini-batch bias yayımı için kullanışlı
    globals.insert(
        "vektor_tekrarla".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [vector, Deger::Sayi(n_f)] = args.as_slice() else {
                return if args.len() == 2 {
                    Deger::Hata("vektor_tekrarla: vektör ve tekrar sayısı gerekir".to_string())
                } else {
                    Deger::Hata(format!(
                        "vektor_tekrarla: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            let n = match boyut_dogrula(*n_f, "vektor_tekrarla", false) {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let values = match value_to_finite_vector(vector, "vektor_tekrarla") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let cols = values.len();
            let _ = match eleman_sayisi_dogrula(n, cols, "vektor_tekrarla") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let result: Vec<f64> = values.iter().copied().cycle().take(n * cols).collect();
            Deger::Matris {
                satirlar: n,
                sutunlar: cols,
                veri: Gc::from_cell(RefCell::new(result)),
            }
        }),
    );

    // ─── Öklit Mesafesi ────────────────────────────────────────────────────
    // oklid_mesafe(v1, v2) → iki vektör arasındaki L2 mesafesi (k-NN için)
    globals.insert(
        "oklid_mesafe".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [v1, v2] = args.as_slice() else {
                return Deger::Hata(format!(
                    "oklid_mesafe: tam olarak 2 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let a = match value_to_finite_vector(v1, "oklid_mesafe") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            let b = match value_to_finite_vector(v2, "oklid_mesafe") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            if a.len() != b.len() {
                return Deger::Hata(format!(
                    "oklid_mesafe: vektör boyutları eşit olmalı; {} ve {} geldi",
                    a.len(),
                    b.len()
                ));
            }
            let dist = a
                .iter()
                .zip(b.iter())
                .map(|(x, y)| (x - y) * (x - y))
                .sum::<f64>()
                .sqrt();
            if dist.is_finite() {
                Deger::Sayi(dist)
            } else {
                Deger::Hata("oklid_mesafe: sonlu olmayan sonuç".to_string())
            }
        }),
    );

    // ─── Argmax / Argmin ───────────────────────────────────────────────────
    // vektor_argmax(v) → en büyük elemanın indeksi (tahmin sınıfı için)
    globals.insert(
        "vektor_argmax".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [vector] = args.as_slice() else {
                return Deger::Hata(format!(
                    "vektor_argmax: tam olarak 1 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let values = match value_to_finite_vector(vector, "vektor_argmax") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            if values.is_empty() {
                return Deger::Hata("vektor_argmax: boş vektör".to_string());
            }
            let idx = values
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
                .unwrap_or(0);
            Deger::Sayi(idx as f64)
        }),
    );

    // vektor_argmin(v) → en küçük elemanın indeksi
    globals.insert(
        "vektor_argmin".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [vector] = args.as_slice() else {
                return Deger::Hata(format!(
                    "vektor_argmin: tam olarak 1 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let values = match value_to_finite_vector(vector, "vektor_argmin") {
                Ok(v) => v,
                Err(e) => return Deger::Hata(e),
            };
            if values.is_empty() {
                return Deger::Hata("vektor_argmin: boş vektör".to_string());
            }
            let idx = values
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
                .unwrap_or(0);
            Deger::Sayi(idx as f64)
        }),
    );

    // matris_satir_argmax(M) → Vektor: her satırın argmax'ı (çok-sınıflı tahmin)
    globals.insert(
        "matris_satir_argmax".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [matrix] = args.as_slice() else {
                return Deger::Hata(format!(
                    "matris_satir_argmax: tam olarak 1 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            };
            let (rows, cols, values) = match value_to_finite_matrix(matrix, "matris_satir_argmax") {
                Ok(m) => m,
                Err(e) => return Deger::Hata(e),
            };
            if cols == 0 {
                return Deger::Hata(
                    "matris_satir_argmax: matrisin en az 1 sütunu olmalıdır".to_string(),
                );
            }
            let result: Vec<f64> = (0..rows)
                .map(|r| {
                    let row = &values[r * cols..(r + 1) * cols];
                    row.iter()
                        .enumerate()
                        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                        .map(|(i, _)| i as f64)
                        .unwrap_or(0.0)
                })
                .collect();
            Deger::Vektor(Gc::from_cell(RefCell::new(result)))
        }),
    );

    globals
}

impl Default for Yorumlayici {
    fn default() -> Self {
        Self::new()
    }
}

impl BuiltinRuntime for Yorumlayici {
    fn call_value(&mut self, function: Deger, args: Vec<Deger>) -> Deger {
        self.fonksiyon_cagrisi(function, args)
    }
}

impl Yorumlayici {
    pub fn new() -> Self {
        Self {
            global_degiskenler: varsayilan_global_degiskenler(),
            yerel_scopes: Vec::new(),
            donus_degeri: None,
            yuklenen_dosyalar: HashSet::new(),
            yuklenmekte_olan_dosyalar: HashSet::new(),
            module_namespaces: HashMap::new(),
            module_environments: HashMap::new(),
            active_exports: Vec::new(),
            active_module_bindings: Vec::new(),
            active_module_calls: Vec::new(),
            arama_yolları: vec![
                ".".to_string(),
                "./lib".to_string(),
                "./huma_modulleri".to_string(),
            ],
            output_buffer: None,
            call_depth: 0,
            runtime_errors: Vec::new(),
            current_location: None,
            _heap_sweep: HeapSweepGuard,
            call_stack: Vec::new(),
            dongu_kontrolu: None,
            dongu_derinligi: 0,
            limits: crate::limits::ExecutionLimits::default(),
            executed_steps: 0,
            output_bytes: 0,
            task_awaiter: None,
        }
    }

    pub fn with_limits(mut self, limits: crate::limits::ExecutionLimits) -> Result<Self, String> {
        self.limits = limits.validate()?;
        if self.limits.max_call_depth > INTERPRETER_MAX_CALL_DEPTH {
            return Err(format!(
                "Yorumlayıcı çağrı derinliği güvenli üst sınırı olan {}'ü aşamaz; \
                 daha derin çağrılar için VM kullanın",
                INTERPRETER_MAX_CALL_DEPTH
            ));
        }
        Ok(self)
    }

    pub fn fonksiyon_cagrisi(&mut self, f: Deger, args: Vec<Deger>) -> Deger {
        self.fonksiyon_cagrisi_detayli(f, args, None, None)
    }

    /// Uzun ömürlü ev sahibi döngülerinden (GUI, oyun döngüsü vb.) gelen tek
    /// bir geri çağrıyı bağımsız yürütme bütçesiyle çalıştırır.
    ///
    /// Normal program yürütmesinden kalan hata ve sayaçların sonraki kareyi
    /// zehirlemesini engeller; geri çağrı hatasını da sessiz bir `Boş` değer
    /// yerine yapılandırılmış hata olarak ev sahibine iletir.
    pub fn geri_cagri_kontrollu(&mut self, f: Deger, args: Vec<Deger>) -> HumaResult<Deger> {
        self.runtime_errors.clear();
        self.executed_steps = 0;
        self.output_bytes = 0;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.fonksiyon_cagrisi(f, args)
        }));
        let value = match result {
            Ok(value) => value,
            Err(payload) => {
                self.runtime_error_ekle(format!(
                    "Geri çağrı paniği güvenli biçimde yakalandı: {}",
                    crate::error::panik_mesaji(payload)
                ));
                Deger::Bos
            }
        };

        match self.runtime_errors.first() {
            Some(diagnostic) => Err(HumaError::RuntimeError(diagnostic.clone())),
            None => Ok(value),
        }
    }

    pub fn fonksiyon_cagrisi_detayli(
        &mut self,
        f: Deger,
        args: Vec<Deger>,
        nesne: Option<Deger>,
        call_site: Option<StackFrame>,
    ) -> Deger {
        if let Some(frame) = &call_site {
            self.call_stack.push(frame.clone());
        }
        let effective_call_limit = self.limits.max_call_depth.min(INTERPRETER_MAX_CALL_DEPTH);
        if self.call_depth >= effective_call_limit {
            let message = "Azami özyineleme derinliği aşıldı".to_string();
            self.runtime_error_ekle(message.clone());
            if call_site.is_some() {
                self.call_stack.pop();
            }
            return Deger::Hata(message);
        }
        let module_context = match &f {
            Deger::Fonksiyon { module_kimligi, .. } | Deger::Sinif { module_kimligi, .. } => {
                module_kimligi.clone()
            }
            _ => None,
        };
        self.call_depth += 1;
        if let Some(module_id) = &module_context {
            self.active_module_calls.push(module_id.clone());
        }

        let res = match f {
            Deger::Sinif {
                ad,
                alan_baslangic,
                module_kimligi,
                ..
            } => {
                if !args.is_empty() {
                    let message = format!(
                        "{} sınıfı kurucu argümanı kabul etmiyor; {} argüman geldi",
                        ad,
                        args.len()
                    );
                    self.runtime_error_ekle(message.clone());
                    if module_context.is_some() {
                        self.active_module_calls.pop();
                    }
                    if call_site.is_some() {
                        self.call_stack.pop();
                    }
                    self.call_depth -= 1;
                    return Deger::Hata(message);
                }
                let mut fields = HashMap::new();
                for (alan_ad, alan_ifade) in alan_baslangic {
                    let val = self.ifade_hesapla(alan_ifade);
                    if !self.runtime_errors.is_empty() {
                        break;
                    }
                    fields.insert(alan_ad, val);
                }
                Deger::Nesne {
                    sinif_adi: ad,
                    alanlar: Gc::from_cell(RefCell::new(fields)),
                    module_kimligi,
                }
            }
            Deger::Fonksiyon {
                parametreler,
                govde,
                yakalanan_kapsamlar,
                ..
            } => {
                if args.len() != parametreler.len() {
                    let message = format!(
                        "Fonksiyon {} argüman bekliyor; {} argüman geldi",
                        parametreler.len(),
                        args.len()
                    );
                    self.runtime_error_ekle(message.clone());
                    if module_context.is_some() {
                        self.active_module_calls.pop();
                    }
                    if call_site.is_some() {
                        self.call_stack.pop();
                    }
                    self.call_depth -= 1;
                    return Deger::Hata(message);
                }
                let mut yerel = HashMap::new();
                if let Some(ins) = nesne {
                    yerel.insert("kendisi".to_string(), ins);
                }
                for (i, p) in parametreler.iter().enumerate() {
                    yerel.insert(p.clone(), args[i].clone());
                }
                let onceki_kapsamlar =
                    std::mem::replace(&mut self.yerel_scopes, yakalanan_kapsamlar);
                self.yerel_scopes.push(yerel);
                let eski = self.donus_degeri.take();
                let eski_dongu_kontrolu = self.dongu_kontrolu.take();
                let eski_dongu_derinligi = self.dongu_derinligi;
                self.dongu_derinligi = 0;
                for k in govde {
                    self.komut_calistir(k);
                    if self.donus_degeri.is_some()
                        || self.dongu_kontrolu.is_some()
                        || !self.runtime_errors.is_empty()
                    {
                        break;
                    }
                }
                let res = self.donus_degeri.take().unwrap_or(Deger::Bos);
                if self.dongu_kontrolu.take().is_some() {
                    self.runtime_error_ekle(
                        "devam/kır komutu bir fonksiyonun döngüsü dışında kullanılamaz".to_string(),
                    );
                }
                self.dongu_derinligi = eski_dongu_derinligi;
                self.dongu_kontrolu = eski_dongu_kontrolu;
                self.yerel_scopes = onceki_kapsamlar;
                self.donus_degeri = eski;
                res
            }
            Deger::DahiliFonksiyon(df) => {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| df(args))) {
                    Ok(value) => value,
                    Err(payload) => {
                        let message = format!(
                            "Yerleşik fonksiyon paniği güvenli biçimde yakalandı: {}",
                            crate::error::panik_mesaji(payload)
                        );
                        self.runtime_error_ekle(message.clone());
                        Deger::Hata(message)
                    }
                }
            }
            Deger::BaglamliDahiliFonksiyon(df) => {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| df(self, args))) {
                    Ok(value) => value,
                    Err(payload) => {
                        let message = format!(
                            "Bağlamlı yerleşik fonksiyon paniği güvenli biçimde yakalandı: {}",
                            crate::error::panik_mesaji(payload)
                        );
                        self.runtime_error_ekle(message.clone());
                        Deger::Hata(message)
                    }
                }
            }
            _ => {
                let message = format!("Çağrılamayan değer: {}", f);
                self.runtime_error_ekle(message.clone());
                Deger::Hata(message)
            }
        };

        if module_context.is_some() {
            self.active_module_calls.pop();
        }
        if call_site.is_some() {
            self.call_stack.pop();
        }
        self.call_depth -= 1;
        res
    }

    pub fn with_output_buffer(mut self, buffer: Rc<RefCell<String>>) -> Self {
        self.output_buffer = Some(buffer);
        self
    }

    pub fn with_task_awaiter(mut self, awaiter: fn(u64) -> Deger) -> Self {
        self.task_awaiter = Some(awaiter);
        self
    }

    pub fn task_awaiter_ayarla(&mut self, awaiter: fn(u64) -> Deger) {
        self.task_awaiter = Some(awaiter);
    }

    fn satir_yazdir(&mut self, content: &str) {
        let byte_count = match content.len().checked_add(1) {
            Some(byte_count) => byte_count,
            None => {
                self.runtime_error_ekle("Çıktı boyutu hesaplanırken taştı".to_string());
                return;
            }
        };
        let next_output = match self.output_bytes.checked_add(byte_count) {
            Some(next_output) if next_output <= self.limits.max_output_bytes => next_output,
            _ => {
                self.runtime_error_ekle(format!(
                    "Çıktı sınırı aşıldı: en fazla {} bayt",
                    self.limits.max_output_bytes
                ));
                return;
            }
        };
        self.output_bytes = next_output;
        if let Some(buf) = self.output_buffer.clone() {
            match buf.try_borrow_mut() {
                Ok(mut output) => {
                    output.push_str(content);
                    output.push('\n');
                }
                Err(_) => self.runtime_error_ekle("Çıktı tamponu kullanımda".to_string()),
            }
        } else {
            println!("{}", content);
        }
    }

    pub fn yorumla(&mut self, komutlar: Vec<Komut>) {
        self.executed_steps = 0;
        self.output_bytes = 0;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.yorumla_ic(komutlar);
        }));
        if let Err(payload) = result {
            self.runtime_error_ekle(format!(
                "Çalışma zamanı paniği güvenli biçimde yakalandı: {}",
                crate::error::panik_mesaji(payload)
            ));
        }
    }

    fn yorumla_ic(&mut self, komutlar: Vec<Komut>) {
        for komut in komutlar {
            self.komut_calistir(komut);
            if self.donus_degeri.is_some() || !self.runtime_errors.is_empty() {
                break;
            }
        }
    }

    pub fn yorumla_kontrollu(&mut self, komutlar: Vec<Komut>) -> HumaResult<()> {
        self.runtime_errors.clear();
        self.yorumla(komutlar);
        match self.runtime_errors.first() {
            Some(diagnostic) => Err(HumaError::RuntimeError(diagnostic.clone())),
            None => Ok(()),
        }
    }

    pub fn runtime_hatalari(&self) -> &[RuntimeDiagnostic] {
        &self.runtime_errors
    }

    fn runtime_error_ekle(&mut self, message: String) {
        let diagnostic = RuntimeDiagnostic {
            message,
            location: self.current_location,
            stack: self.call_stack.iter().rev().cloned().collect(),
        };
        if !self.runtime_errors.contains(&diagnostic) {
            self.runtime_errors.push(diagnostic);
        }
    }

    fn adim_tuket(&mut self) -> bool {
        self.executed_steps = self.executed_steps.saturating_add(1);
        if self.executed_steps > self.limits.max_steps {
            self.runtime_error_ekle(format!(
                "Çalıştırma adım sınırı aşıldı: {}",
                self.limits.max_steps
            ));
            false
        } else {
            true
        }
    }

    fn get_degisken(&mut self, ad: &str) -> Deger {
        for scope in self.yerel_scopes.iter().rev() {
            if let Some(val) = scope.get(ad) {
                return val.clone();
            }
        }
        for module_id in self.active_module_calls.iter().rev() {
            if let Some(value) = self
                .module_environments
                .get(module_id)
                .and_then(|environment| environment.get(ad))
            {
                return value.clone();
            }
        }
        match self.global_degiskenler.get(ad).cloned() {
            Some(value) => value,
            None => {
                let message = format!("Tanımsız değişken: {}", ad);
                self.runtime_error_ekle(message.clone());
                Deger::Hata(message)
            }
        }
    }

    fn degisken_ata(&mut self, ad: String, deger: Deger) {
        if ad.contains('.') {
            let parts: Vec<&str> = ad.splitn(2, '.').collect();
            let nesne_adi = parts[0];
            let alan_adi = parts[1];
            let obj = self.get_degisken(nesne_adi);
            if let Deger::Nesne { alanlar, .. } = obj {
                let mut fields = match alanlar.try_borrow_mut() {
                    Ok(fields) => fields,
                    Err(_) => {
                        self.runtime_error_ekle("Nesne alanları kullanımda".to_string());
                        return;
                    }
                };
                if !fields.contains_key(alan_adi)
                    && fields.len() >= self.limits.max_collection_items
                {
                    self.runtime_error_ekle(format!(
                        "Nesne alan sınırı aşıldı: {}",
                        self.limits.max_collection_items
                    ));
                    return;
                }
                fields.insert(alan_adi.to_string(), deger);
                return;
            }
        }
        for scope in self.yerel_scopes.iter_mut().rev() {
            if let Some(current) = scope.get_mut(&ad) {
                *current = deger;
                return;
            }
        }
        for module_id in self.active_module_calls.iter().rev() {
            if let Some(current) = self
                .module_environments
                .get_mut(module_id)
                .and_then(|environment| environment.get_mut(&ad))
            {
                *current = deger;
                return;
            }
        }
        self.global_degiskenler.insert(ad, deger);
    }

    fn degisken_tanimla(&mut self, ad: String, deger: Deger) {
        if ad.contains('.') {
            let parts: Vec<&str> = ad.splitn(2, '.').collect();
            let nesne_adi = parts[0];
            let alan_adi = parts[1];
            let obj = self.get_degisken(nesne_adi);
            if let Deger::Nesne { alanlar, .. } = obj {
                let mut fields = match alanlar.try_borrow_mut() {
                    Ok(fields) => fields,
                    Err(_) => {
                        self.runtime_error_ekle("Nesne alanları kullanımda".to_string());
                        return;
                    }
                };
                if !fields.contains_key(alan_adi)
                    && fields.len() >= self.limits.max_collection_items
                {
                    self.runtime_error_ekle(format!(
                        "Nesne alan sınırı aşıldı: {}",
                        self.limits.max_collection_items
                    ));
                    return;
                }
                fields.insert(alan_adi.to_string(), deger);
                return;
            }
        }
        if let Some(scope) = self.yerel_scopes.last_mut() {
            scope.insert(ad, deger);
        } else {
            if let Some(bindings) = self.active_module_bindings.last_mut() {
                bindings.insert(ad.clone());
            }
            self.global_degiskenler.insert(ad, deger);
        }
    }

    fn komut_calistir(&mut self, komut: Komut) {
        if let Komut::Konumlu {
            komut,
            satir,
            sutun,
        } = komut
        {
            let previous = self.current_location.replace(SourceSpan {
                line: satir,
                column: sutun,
            });
            self.komut_calistir(*komut);
            self.current_location = previous;
            return;
        }
        if self.donus_degeri.is_some()
            || self.dongu_kontrolu.is_some()
            || !self.runtime_errors.is_empty()
        {
            return;
        }
        if !self.adim_tuket() {
            return;
        }
        match komut {
            Komut::Konumlu {
                komut,
                satir,
                sutun,
            } => {
                let previous = self.current_location.replace(SourceSpan {
                    line: satir,
                    column: sutun,
                });
                self.komut_calistir(*komut);
                self.current_location = previous;
            }
            Komut::YazdirKomutu(ifade) => {
                let d = self.ifade_hesapla(ifade);
                if self.runtime_errors.is_empty() {
                    match d.to_string_limited(self.limits.max_output_bytes) {
                        Ok(text) => self.satir_yazdir(&text),
                        Err(error) => self.runtime_error_ekle(error),
                    }
                }
            }
            Komut::DegiskenTanimla { ad, deger } => {
                let res = self.ifade_hesapla(deger);
                if self.runtime_errors.is_empty() {
                    self.degisken_tanimla(ad, res);
                }
            }
            Komut::Atama { ad, deger } => {
                let res = self.ifade_hesapla(deger);
                if self.runtime_errors.is_empty() {
                    self.degisken_ata(ad, res);
                }
            }
            Komut::EgerKomutu {
                kosul,
                govde,
                degilse_govde,
            } => {
                let r = self.ifade_hesapla(kosul);
                let condition = match self.dogruluk_kontrolu(r) {
                    Ok(condition) => condition,
                    Err(error) => {
                        self.runtime_error_ekle(error);
                        return;
                    }
                };
                if condition {
                    for k in govde {
                        self.komut_calistir(k);
                        if self.donus_degeri.is_some() {
                            break;
                        }
                    }
                } else if let Some(d) = degilse_govde {
                    for k in d {
                        self.komut_calistir(k);
                        if self.donus_degeri.is_some() {
                            break;
                        }
                    }
                }
            }
            Komut::DonguKomutu { kosul, govde } => {
                self.dongu_derinligi += 1;
                loop {
                    if !self.adim_tuket() {
                        break;
                    }
                    let r = self.ifade_hesapla(kosul.clone());
                    let condition = match self.dogruluk_kontrolu(r) {
                        Ok(condition) => condition,
                        Err(error) => {
                            self.runtime_error_ekle(error);
                            break;
                        }
                    };
                    if !condition || self.donus_degeri.is_some() {
                        break;
                    }
                    for k in &govde {
                        self.komut_calistir(k.clone());
                        if self.donus_degeri.is_some()
                            || self.dongu_kontrolu.is_some()
                            || !self.runtime_errors.is_empty()
                        {
                            break;
                        }
                    }
                    match self.dongu_kontrolu.take() {
                        Some(DonguKontrolu::Kir) => break,
                        Some(DonguKontrolu::Devam) | None => {}
                    }
                    if !self.runtime_errors.is_empty() {
                        break;
                    }
                }
                self.dongu_derinligi = self.dongu_derinligi.saturating_sub(1);
            }
            Komut::FonksiyonTanimla {
                ad,
                parametreler,
                govde,
            } => {
                self.degisken_tanimla(
                    ad,
                    Deger::Fonksiyon {
                        parametreler,
                        govde,
                        yakalanan_kapsamlar: self.yerel_scopes.clone(),
                        module_kimligi: None,
                    },
                );
            }
            Komut::SinifTanimla { ad, metotlar } => {
                let mut ms = HashMap::new();
                // Sınıf içindeki değişken tanımlarını da işle
                let mut init_fields: Vec<(String, Ifade)> = Vec::new();
                for m in metotlar {
                    if let Komut::FonksiyonTanimla {
                        ad: m_ad,
                        parametreler,
                        govde,
                    } = m
                    {
                        ms.insert(m_ad, (parametreler, govde));
                    } else if let Komut::DegiskenTanimla { ad: f_ad, deger } = m {
                        init_fields.push((f_ad, deger));
                    }
                }
                self.degisken_tanimla(
                    ad.clone(),
                    Deger::Sinif {
                        ad,
                        metotlar: ms,
                        alan_baslangic: init_fields,
                        module_kimligi: None,
                    },
                );
            }
            Komut::DondurKomutu(ifade) => {
                let v = self.ifade_hesapla(ifade);
                self.donus_degeri = Some(v);
            }
            Komut::YukleKomutu { yol, takma_ad } => self.modül_yükle(&yol, takma_ad.as_deref()),
            Komut::DisaAktar(ad) => {
                if self.active_exports.is_empty() {
                    self.runtime_error_ekle(
                        "dışa aktar yalnızca yüklenen bir modül içinde kullanılabilir".to_string(),
                    );
                } else if !self.yerel_scopes.is_empty() {
                    self.runtime_error_ekle(
                        "dışa aktar yalnızca modülün üst düzeyinde kullanılabilir".to_string(),
                    );
                } else if !self.global_degiskenler.contains_key(&ad)
                    && !self
                        .yerel_scopes
                        .iter()
                        .rev()
                        .any(|scope| scope.contains_key(&ad))
                {
                    self.runtime_error_ekle(format!("Dışa aktarılacak değer tanımlı değil: {ad}"));
                } else if let Some(exports) = self.active_exports.last_mut() {
                    exports.insert(ad);
                }
            }
            Komut::ListeOlustur { ad } => {
                self.degisken_tanimla(ad, Deger::Liste(Gc::from_cell(RefCell::new(Vec::new()))));
            }
            Komut::ListeEkle { liste, deger } => {
                let deger_val = self.ifade_hesapla(deger);
                let liste_val = self.ifade_hesapla(liste);
                let Deger::Liste(list) = liste_val else {
                    if self.runtime_errors.is_empty() {
                        self.runtime_error_ekle("Liste ekleme hedefi liste olmalıdır".to_string());
                    }
                    return;
                };
                let additions = if let Deger::Liste(values) = &deger_val {
                    match values.try_borrow() {
                        Ok(values) => values.clone(),
                        Err(_) => {
                            self.runtime_error_ekle("Eklenecek liste kullanımda".to_string());
                            return;
                        }
                    }
                } else {
                    vec![deger_val]
                };
                let mut list = match list.try_borrow_mut() {
                    Ok(list) => list,
                    Err(_) => {
                        self.runtime_error_ekle("Hedef liste kullanımda".to_string());
                        return;
                    }
                };
                let Some(next_length) = list.len().checked_add(additions.len()) else {
                    self.runtime_error_ekle("Liste uzunluğu taştı".to_string());
                    return;
                };
                if next_length > self.limits.max_collection_items {
                    self.runtime_error_ekle(format!(
                        "Liste eleman sınırı aşıldı: {} > {}",
                        next_length, self.limits.max_collection_items
                    ));
                    return;
                }
                list.extend(additions);
            }
            Komut::ListeCikar { liste, indeks } => {
                let idx_val = self.ifade_hesapla(indeks);
                let liste_val = self.ifade_hesapla(liste);

                // Eğer indeks bir listeyse (özellikle [i] syntax'ında), ilk elemanı al
                let final_idx = if let Deger::Liste(index_list) = &idx_val {
                    let index_list = match index_list.try_borrow() {
                        Ok(index_list) => index_list,
                        Err(_) => {
                            self.runtime_error_ekle("İndeks listesi kullanımda".to_string());
                            return;
                        }
                    };
                    if index_list.len() != 1 {
                        self.runtime_error_ekle(
                            "Liste çıkarma indeks sarmalayıcısı tek elemanlı olmalıdır".to_string(),
                        );
                        return;
                    }
                    index_list[0].clone()
                } else {
                    idx_val
                };
                let (Deger::Liste(list), Deger::Sayi(index)) = (liste_val, final_idx) else {
                    self.runtime_error_ekle(
                        "Liste çıkarma hedefi liste, indeks ise sayı olmalıdır".to_string(),
                    );
                    return;
                };
                let mut list = match list.try_borrow_mut() {
                    Ok(list) => list,
                    Err(_) => {
                        self.runtime_error_ekle("Hedef liste kullanımda".to_string());
                        return;
                    }
                };
                let index = match indeks_dogrula(index, list.len(), "Liste çıkarma") {
                    Ok(index) => index,
                    Err(error) => {
                        self.runtime_error_ekle(error);
                        return;
                    }
                };
                list.remove(index);
            }
            Komut::DeneKomutu {
                dene_govde,
                hata_degisken,
                hata_govde,
            } => {
                let onceki_hatalar = std::mem::take(&mut self.runtime_errors);
                let onceki_donus = self.donus_degeri.take();
                let onceki_dongu_kontrolu = self.dongu_kontrolu.take();
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    for k in dene_govde {
                        self.komut_calistir(k);
                        if !self.runtime_errors.is_empty()
                            || self.donus_degeri.is_some()
                            || self.dongu_kontrolu.is_some()
                        {
                            break;
                        }
                    }
                }));
                let yakalanan_hata = match result {
                    Ok(()) => self.runtime_errors.first().cloned(),
                    Err(error) => Some(RuntimeDiagnostic {
                        message: crate::error::panik_mesaji(error),
                        location: self.current_location,
                        stack: self.call_stack.iter().rev().cloned().collect(),
                    }),
                };

                if let Some(diagnostic) = yakalanan_hata {
                    self.runtime_errors = onceki_hatalar;
                    self.donus_degeri = onceki_donus;
                    self.dongu_kontrolu = onceki_dongu_kontrolu;
                    if let Some(var) = hata_degisken {
                        self.degisken_tanimla(var, Deger::Metin(diagnostic.to_string()));
                    }
                    for k in hata_govde {
                        self.komut_calistir(k);
                        if self.donus_degeri.is_some()
                            || self.dongu_kontrolu.is_some()
                            || !self.runtime_errors.is_empty()
                        {
                            break;
                        }
                    }
                } else {
                    self.runtime_errors = onceki_hatalar;
                    if self.donus_degeri.is_none() {
                        self.donus_degeri = onceki_donus;
                    }
                    if self.dongu_kontrolu.is_none() {
                        self.dongu_kontrolu = onceki_dongu_kontrolu;
                    }
                }
            }
            Komut::AralikDongusu {
                degisken,
                baslangic,
                bitis,
                govde,
            } => {
                let start_val = self.ifade_hesapla(baslangic);
                let end_val = self.ifade_hesapla(bitis);
                if let (Deger::Sayi(s), Deger::Sayi(e)) = (start_val, end_val) {
                    if !s.is_finite() || !e.is_finite() {
                        self.runtime_error_ekle(
                            "Aralık sınırları sonlu sayılar olmalıdır".to_string(),
                        );
                        return;
                    }
                    self.dongu_derinligi += 1;
                    let mut i = s;
                    while i <= e {
                        if !self.adim_tuket() {
                            break;
                        }
                        self.degisken_tanimla(degisken.clone(), Deger::Sayi(i));
                        for k in &govde {
                            self.komut_calistir(k.clone());
                            if self.donus_degeri.is_some()
                                || self.dongu_kontrolu.is_some()
                                || !self.runtime_errors.is_empty()
                            {
                                break;
                            }
                        }
                        if self.donus_degeri.is_some() {
                            break;
                        }
                        match self.dongu_kontrolu.take() {
                            Some(DonguKontrolu::Kir) => break,
                            Some(DonguKontrolu::Devam) | None => {}
                        }
                        if !self.runtime_errors.is_empty() {
                            break;
                        }
                        i += 1.0;
                    }
                    self.dongu_derinligi = self.dongu_derinligi.saturating_sub(1);
                } else if self.runtime_errors.is_empty() {
                    self.runtime_error_ekle(
                        "Aralık başlangıcı ve bitişi sayı olmalıdır".to_string(),
                    );
                }
            }
            Komut::Devam => {
                if self.dongu_derinligi == 0 {
                    self.runtime_error_ekle(
                        "devam komutu yalnızca döngü içinde kullanılabilir".to_string(),
                    );
                } else {
                    self.dongu_kontrolu = Some(DonguKontrolu::Devam);
                }
            }
            Komut::Kir => {
                if self.dongu_derinligi == 0 {
                    self.runtime_error_ekle(
                        "kır komutu yalnızca döngü içinde kullanılabilir".to_string(),
                    );
                } else {
                    self.dongu_kontrolu = Some(DonguKontrolu::Kir);
                }
            }
            Komut::NesneAlaniAtama {
                nesne,
                ozellik,
                deger,
            } => {
                let deger_val = self.ifade_hesapla(deger);
                let nesne_val = self.ifade_hesapla(nesne);
                if let Deger::Nesne { alanlar, .. } = nesne_val {
                    if let Err(error) = insert_keyed_value(
                        &alanlar,
                        ozellik,
                        deger_val,
                        self.limits.max_collection_items,
                        "Nesne alan ataması",
                    ) {
                        self.runtime_error_ekle(error);
                    }
                } else if self.runtime_errors.is_empty() {
                    self.runtime_error_ekle(
                        "Alan atamasının hedefi bir nesne olmalıdır".to_string(),
                    );
                }
            }
            Komut::IfadeKomutu(ifade) => {
                if let Ifade::IkiliIslem {
                    sol,
                    operator: Token::Esittir,
                    sag,
                } = ifade
                {
                    let d = self.ifade_hesapla(*sag);
                    match *sol {
                        Ifade::Degisken(ad) => self.degisken_ata(ad, d),
                        Ifade::NesneErisim { nesne, ozellik } => {
                            if let Deger::Nesne { alanlar, .. } = self.ifade_hesapla(*nesne) {
                                if let Err(error) = insert_keyed_value(
                                    &alanlar,
                                    ozellik,
                                    d,
                                    self.limits.max_collection_items,
                                    "Nesne alan ataması",
                                ) {
                                    self.runtime_error_ekle(error);
                                }
                            } else if self.runtime_errors.is_empty() {
                                self.runtime_error_ekle(
                                    "Alan atamasının hedefi nesne olmalıdır".to_string(),
                                );
                            }
                        }
                        Ifade::KendisiErisim { ozellik } => {
                            let kendisi = self.get_degisken("kendisi");
                            if let Deger::Nesne { alanlar, .. } = kendisi {
                                if let Err(error) = insert_keyed_value(
                                    &alanlar,
                                    ozellik,
                                    d,
                                    self.limits.max_collection_items,
                                    "kendisi alan ataması",
                                ) {
                                    self.runtime_error_ekle(error);
                                }
                            } else if self.runtime_errors.is_empty() {
                                self.runtime_error_ekle(
                                    "kendisi yalnızca sınıf metotlarında kullanılabilir"
                                        .to_string(),
                                );
                            }
                        }
                        Ifade::ListeErisim { liste, indeks } => {
                            let l_val = self.ifade_hesapla((*liste).clone());
                            let i_val = self.ifade_hesapla(*indeks);
                            match (l_val, i_val) {
                                (Deger::Liste(l), Deger::Sayi(i))
                                    if i >= 0.0 && i.fract() == 0.0 =>
                                {
                                    let mut values = match l.try_borrow_mut() {
                                        Ok(values) => values,
                                        Err(_) => {
                                            self.runtime_error_ekle(
                                                "Liste ataması hedefi kullanımda".to_string(),
                                            );
                                            return;
                                        }
                                    };
                                    let index =
                                        match indeks_dogrula(i, values.len(), "Liste ataması") {
                                            Ok(index) => index,
                                            Err(error) => {
                                                self.runtime_error_ekle(error);
                                                return;
                                            }
                                        };
                                    values[index] = d.clone();
                                }
                                (Deger::Sozluk(m), Deger::Metin(key)) => {
                                    if let Err(error) = insert_keyed_value(
                                        &m,
                                        key,
                                        d.clone(),
                                        self.limits.max_collection_items,
                                        "Sözlük ataması",
                                    ) {
                                        self.runtime_error_ekle(error);
                                    }
                                }
                                (Deger::Nesne { alanlar, .. }, Deger::Metin(key)) => {
                                    if let Err(error) = insert_keyed_value(
                                        &alanlar,
                                        key,
                                        d.clone(),
                                        self.limits.max_collection_items,
                                        "Nesne ataması",
                                    ) {
                                        self.runtime_error_ekle(error);
                                    }
                                }
                                (container, index) if self.runtime_errors.is_empty() => {
                                    self.runtime_error_ekle(format!(
                                        "{} değerine {} indeksiyle atama yapılamaz",
                                        container, index
                                    ));
                                }
                                _ => {}
                            }
                        }
                        _ => self.runtime_error_ekle("Geçersiz atama hedefi".to_string()),
                    }
                } else {
                    self.ifade_hesapla(ifade);
                }
            }
        }
    }

    fn modül_programi_calistir(
        &mut self,
        kimlik: String,
        gorunen_ad: &str,
        program: Vec<Komut>,
        parent: Option<&Path>,
        takma_ad: Option<&str>,
    ) {
        if self.yuklenen_dosyalar.contains(&kimlik) {
            if let Some(alias) = takma_ad {
                let Some(namespace) = self.module_namespaces.get(&kimlik).cloned() else {
                    self.runtime_error_ekle(format!(
                        "{gorunen_ad} modülünün önbelleğe alınmış ad alanı bulunamadı"
                    ));
                    return;
                };
                self.degisken_tanimla(
                    alias.to_string(),
                    Deger::Sozluk(Gc::from_cell(RefCell::new(namespace))),
                );
            }
            return;
        }
        if let Some(alias) = takma_ad {
            if self.global_degiskenler.contains_key(alias)
                || self
                    .yerel_scopes
                    .iter()
                    .rev()
                    .any(|scope| scope.contains_key(alias))
            {
                self.runtime_error_ekle(format!("Modül takma adı zaten tanımlı: {alias}"));
                return;
            }
        }
        if !self.yuklenmekte_olan_dosyalar.insert(kimlik.clone()) {
            self.runtime_error_ekle(format!("Döngüsel modül yükleme algılandı: {}", gorunen_ad));
            return;
        }

        let onceki_globaller = self.global_degiskenler.clone();
        let onceki_yuklenenler = self.yuklenen_dosyalar.clone();
        let onceki_ad_alanlari = self.module_namespaces.clone();
        let onceki_modul_ortamlari = self.module_environments.clone();
        let onceki_donus = self.donus_degeri.take();
        let onceki_hata_sayisi = self.runtime_errors.len();
        let mut arama_yolu_eklendi = false;
        if takma_ad.is_some() {
            self.global_degiskenler = varsayilan_global_degiskenler();
        }
        self.active_exports.push(HashSet::new());
        self.active_module_bindings.push(HashSet::new());

        if let Some(parent) = parent {
            let parent = parent.to_string_lossy().to_string();
            if !parent.is_empty() && !self.arama_yolları.contains(&parent) {
                self.arama_yolları.insert(0, parent);
                arama_yolu_eklendi = true;
            }
        }

        self.yorumla_ic(program);

        if arama_yolu_eklendi {
            self.arama_yolları.remove(0);
        }
        let export_names = self.active_exports.pop();
        let binding_names = self.active_module_bindings.pop();
        let (Some(export_names), Some(mut binding_names)) = (export_names, binding_names) else {
            self.runtime_error_ekle(
                "İç hata: modül bağlamı beklenmedik biçimde kayboldu".to_string(),
            );
            self.global_degiskenler = onceki_globaller;
            self.yuklenen_dosyalar = onceki_yuklenenler;
            self.module_namespaces = onceki_ad_alanlari;
            self.module_environments = onceki_modul_ortamlari;
            self.donus_degeri = onceki_donus;
            self.yuklenmekte_olan_dosyalar.remove(&kimlik);
            return;
        };
        binding_names.extend(export_names.iter().cloned());
        self.donus_degeri = onceki_donus;
        self.yuklenmekte_olan_dosyalar.remove(&kimlik);

        if self.runtime_errors.len() == onceki_hata_sayisi {
            let mut module_environment = HashMap::with_capacity(binding_names.len());
            for name in binding_names {
                let Some(value) = self.global_degiskenler.get(&name).cloned() else {
                    self.runtime_error_ekle(format!(
                        "{gorunen_ad} modülünde tanımlanan '{name}' değeri kayboldu"
                    ));
                    break;
                };
                module_environment.insert(name, Self::module_degerini_bagla(value, &kimlik));
            }
            let mut namespace = HashMap::with_capacity(export_names.len());
            if self.runtime_errors.len() == onceki_hata_sayisi {
                for name in export_names {
                    let Some(value) = module_environment.get(&name).cloned() else {
                        self.runtime_error_ekle(format!(
                            "{gorunen_ad} modülü dışa aktarılan '{name}' değerini kaybetti"
                        ));
                        break;
                    };
                    namespace.insert(name, value);
                }
            }
            if self.runtime_errors.len() == onceki_hata_sayisi {
                self.module_environments
                    .insert(kimlik.clone(), module_environment.clone());
                self.module_namespaces
                    .insert(kimlik.clone(), namespace.clone());
                self.yuklenen_dosyalar.insert(kimlik);
                if let Some(alias) = takma_ad {
                    self.global_degiskenler = onceki_globaller;
                    self.degisken_tanimla(
                        alias.to_string(),
                        Deger::Sozluk(Gc::from_cell(RefCell::new(namespace))),
                    );
                } else {
                    for (name, value) in module_environment {
                        self.global_degiskenler.insert(name, value);
                    }
                }
                return;
            }
        }

        self.global_degiskenler = onceki_globaller;
        self.yuklenen_dosyalar = onceki_yuklenenler;
        self.module_namespaces = onceki_ad_alanlari;
        self.module_environments = onceki_modul_ortamlari;
    }

    fn module_degerini_bagla(value: Deger, module_id: &str) -> Deger {
        match value {
            Deger::Fonksiyon {
                parametreler,
                govde,
                yakalanan_kapsamlar,
                ..
            } => Deger::Fonksiyon {
                parametreler,
                govde,
                yakalanan_kapsamlar,
                module_kimligi: Some(module_id.to_string()),
            },
            Deger::Sinif {
                ad,
                metotlar,
                alan_baslangic,
                ..
            } => Deger::Sinif {
                ad,
                metotlar,
                alan_baslangic,
                module_kimligi: Some(module_id.to_string()),
            },
            other => other,
        }
    }

    fn modül_yükle(&mut self, dosya_adı: &str, takma_ad: Option<&str>) {
        if dosya_adı.trim().is_empty() {
            self.runtime_error_ekle("Modül yolu boş olamaz".to_string());
            return;
        }
        if Path::new(dosya_adı).is_absolute() {
            self.runtime_error_ekle(
                "Mutlak modül yolları desteklenmez; arama yollarından göreli yol kullanın"
                    .to_string(),
            );
            return;
        }
        // Önce gömülü kütüphaneleri kontrol et
        for (ad, icerik) in builtin_files::get_lib_files() {
            if ad == dosya_adı {
                let mut parser = crate::parser::Parser::new(crate::lexer::Lexer::new(icerik));
                let (program, diagnostics) = parser.parse_program_with_diagnostics();
                if let Some(first) = diagnostics.into_iter().next() {
                    self.runtime_error_ekle(format!("{} modülü: {}", dosya_adı, first));
                    return;
                }
                self.modül_programi_calistir(
                    format!("gömülü:{ad}"),
                    dosya_adı,
                    program,
                    None,
                    takma_ad,
                );
                return;
            }
        }

        let mut bulunan_yol: Option<PathBuf> = None;
        for temel in &self.arama_yolları {
            let temel = Path::new(temel);
            let tam_yol = temel.join(dosya_adı);
            if tam_yol.is_file() {
                bulunan_yol = Some(tam_yol);
                break;
            }

            // Paket yöneticisi için destek: modul/modul.hb pattern'ini kontrol et
            let paket_yol = temel.join(dosya_adı).join(format!("{dosya_adı}.hb"));
            if paket_yol.is_file() {
                bulunan_yol = Some(paket_yol);
                break;
            }

            // Uzantı ekleyerek kontrol et
            if !dosya_adı.ends_with(".hb") {
                let hb_yol = temel.join(format!("{dosya_adı}.hb"));
                if hb_yol.is_file() {
                    bulunan_yol = Some(hb_yol);
                    break;
                }
            } else if let Some(temel_ad) = dosya_adı.strip_suffix(".hb") {
                // "yükle "paket.hb";" biçimi de modul/modul.hb paket düzenini bulabilsin
                let paket_yol_uzantili = temel.join(temel_ad).join(dosya_adı);
                if paket_yol_uzantili.is_file() {
                    bulunan_yol = Some(paket_yol_uzantili);
                    break;
                }
            }
        }

        let Some(yol) = bulunan_yol else {
            self.runtime_error_ekle(format!("Modül bulunamadı: {}", dosya_adı));
            return;
        };
        let kanonik_yol = match yol.canonicalize() {
            Ok(yol) => yol,
            Err(error) => {
                self.runtime_error_ekle(format!(
                    "Modül yolu çözülemedi ({}): {}",
                    dosya_adı, error
                ));
                return;
            }
        };
        let izinli = self.arama_yolları.iter().any(|base| {
            Path::new(base)
                .canonicalize()
                .is_ok_and(|canonical_base| kanonik_yol.starts_with(canonical_base))
        });
        if !izinli {
            self.runtime_error_ekle(format!(
                "Modül yolu izin verilen arama köklerinin dışında: {}",
                kanonik_yol.display()
            ));
            return;
        }
        let kimlik = format!("dosya:{}", kanonik_yol.to_string_lossy());
        if self.yuklenen_dosyalar.contains(&kimlik) {
            if let Some(alias) = takma_ad {
                let Some(namespace) = self.module_namespaces.get(&kimlik).cloned() else {
                    self.runtime_error_ekle(format!(
                        "{} modülünün önbelleğe alınmış ad alanı bulunamadı",
                        dosya_adı
                    ));
                    return;
                };
                self.degisken_tanimla(
                    alias.to_string(),
                    Deger::Sozluk(Gc::from_cell(RefCell::new(namespace))),
                );
            }
            return;
        }
        let icerik = match read_utf8_file_limited(&kanonik_yol, "Modül okuma") {
            Ok(icerik) => icerik,
            Err(error) => {
                self.runtime_error_ekle(format!("Modül okunamadı ({}): {}", dosya_adı, error));
                return;
            }
        };
        let mut parser = crate::parser::Parser::new(crate::lexer::Lexer::new(&icerik));
        let (program, diagnostics) = parser.parse_program_with_diagnostics();
        if let Some(first) = diagnostics.into_iter().next() {
            self.runtime_error_ekle(format!("{} modülü: {}", dosya_adı, first));
            return;
        }
        self.modül_programi_calistir(kimlik, dosya_adı, program, kanonik_yol.parent(), takma_ad);
    }

    fn ifade_hesapla(&mut self, ifade: Ifade) -> Deger {
        let value = self.ifade_hesapla_ic(ifade);
        if let Deger::Hata(message) = &value {
            self.runtime_error_ekle(message.clone());
        }
        value
    }

    fn ifade_hesapla_ic(&mut self, ifade: Ifade) -> Deger {
        match ifade {
            Ifade::Bekle(inner) => {
                let v = self.ifade_hesapla(*inner);
                match v {
                    Deger::GorevId(id) => match self.task_awaiter {
                        Some(awaiter) => awaiter(id),
                        None => {
                            Deger::Hata("Asenkron görev ana makinesi yapılandırılmadı".to_string())
                        }
                    },
                    other => Deger::Hata(format!("bekle: await edilemez değer: {}", other)),
                }
            }
            Ifade::Sayi(n) => Deger::Sayi(n),
            Ifade::Metin(s) => Deger::Metin(s),
            Ifade::Bos => Deger::Bos,
            Ifade::Dogru => Deger::Sayi(1.0),
            Ifade::Yanlis => Deger::Sayi(0.0),
            Ifade::Degisken(ad) => self.get_degisken(&ad),
            Ifade::Liste(el) => {
                if el.len() > self.limits.max_collection_items {
                    return Deger::Hata(format!(
                        "Liste eleman sınırı aşıldı: {} > {}",
                        el.len(),
                        self.limits.max_collection_items
                    ));
                }
                Deger::Liste(Gc::from_cell(RefCell::new(
                    el.into_iter().map(|e| self.ifade_hesapla(e)).collect(),
                )))
            }
            Ifade::Sozluk(el) => {
                if el.len() > self.limits.max_collection_items {
                    return Deger::Hata(format!(
                        "Sözlük eleman sınırı aşıldı: {} > {}",
                        el.len(),
                        self.limits.max_collection_items
                    ));
                }
                let mut map = HashMap::new();
                for (k, v) in el {
                    let key = match self.ifade_hesapla(k) {
                        Deger::Metin(key) => key,
                        other => {
                            return Deger::Hata(format!(
                                "Sözlük anahtarı metin olmalıdır; {} geldi",
                                other
                            ))
                        }
                    };
                    let val = self.ifade_hesapla(v);
                    map.insert(key, val);
                }
                Deger::Sozluk(Gc::from_cell(RefCell::new(map)))
            }
            Ifade::ListeErisim { liste, indeks } => {
                let l_val = self.ifade_hesapla(*liste);
                let i_val = self.ifade_hesapla(*indeks);
                match (l_val, i_val) {
                    (Deger::Liste(l), Deger::Sayi(i)) if i >= 0.0 && i.fract() == 0.0 => {
                        let values = match l.try_borrow() {
                            Ok(values) => values,
                            Err(_) => return Deger::Hata("Liste kullanımda".to_string()),
                        };
                        let index = match indeks_dogrula(i, values.len(), "Liste erişimi") {
                            Ok(index) => index,
                            Err(error) => return Deger::Hata(error),
                        };
                        values[index].clone()
                    }
                    (Deger::Metin(s), Deger::Sayi(i)) if i >= 0.0 && i.fract() == 0.0 => {
                        let length = s.chars().count();
                        let index = match indeks_dogrula(i, length, "Metin erişimi") {
                            Ok(index) => index,
                            Err(error) => return Deger::Hata(error),
                        };
                        match s.chars().nth(index) {
                            Some(character) => Deger::Metin(character.to_string()),
                            None => Deger::Hata("Metin indeksi çözülemedi".to_string()),
                        }
                    }
                    (Deger::Sozluk(m), Deger::Metin(key)) => {
                        match get_keyed_value(&m, &key, "Sözlük erişimi") {
                            Ok(value) => value.unwrap_or(Deger::Bos),
                            Err(error) => Deger::Hata(error),
                        }
                    }
                    (Deger::Nesne { alanlar, .. }, Deger::Metin(key)) => {
                        match get_keyed_value(&alanlar, &key, "Nesne erişimi") {
                            Ok(value) => value.unwrap_or(Deger::Bos),
                            Err(error) => Deger::Hata(error),
                        }
                    }
                    (container, index) => Deger::Hata(format!(
                        "{} değeri {} indeksiyle erişilemez",
                        container, index
                    )),
                }
            }
            Ifade::NesneErisim { nesne, ozellik } => {
                let inst = self.ifade_hesapla(*nesne);
                if let Deger::Nesne { alanlar, .. } = inst {
                    match get_keyed_value(&alanlar, &ozellik, "Nesne özelliği erişimi") {
                        Ok(Some(value)) => value,
                        Ok(None) => Deger::Hata(format!("Nesne özelliği bulunamadı: {}", ozellik)),
                        Err(error) => Deger::Hata(error),
                    }
                } else if let Deger::Sozluk(m) = inst {
                    match get_keyed_value(&m, &ozellik, "Sözlük özelliği erişimi") {
                        Ok(value) => value.unwrap_or(Deger::Bos),
                        Err(error) => Deger::Hata(error),
                    }
                } else {
                    Deger::Hata(format!("Nesne özelliğine erişilemez: {}", ozellik))
                }
            }
            Ifade::KendisiErisim { ozellik } => {
                let kendisi = self.get_degisken("kendisi");
                if let Deger::Nesne { alanlar, .. } = kendisi {
                    match get_keyed_value(&alanlar, &ozellik, "kendisi özelliği erişimi") {
                        Ok(Some(value)) => value,
                        Ok(None) => Deger::Hata(format!("Nesne özelliği bulunamadı: {}", ozellik)),
                        Err(error) => Deger::Hata(error),
                    }
                } else {
                    Deger::Hata("kendisi yalnızca sınıf metotlarında kullanılabilir".to_string())
                }
            }
            Ifade::Uzunluk(ifade) => {
                let val = self.ifade_hesapla(*ifade);
                match val {
                    Deger::Liste(l) => match l.try_borrow() {
                        Ok(values) => Deger::Sayi(values.len() as f64),
                        Err(_) => Deger::Hata("Uzunluk alınırken liste kullanımda".to_string()),
                    },
                    Deger::Metin(s) => Deger::Sayi(s.chars().count() as f64),
                    Deger::Bayt(b) => Deger::Sayi(b.len() as f64),
                    Deger::Sozluk(m) => match m.try_borrow() {
                        Ok(values) => Deger::Sayi(values.len() as f64),
                        Err(_) => Deger::Hata("Uzunluk alınırken sözlük kullanımda".to_string()),
                    },
                    Deger::Vektor(v) => match v.try_borrow() {
                        Ok(values) => Deger::Sayi(values.len() as f64),
                        Err(_) => Deger::Hata("Uzunluk alınırken vektör kullanımda".to_string()),
                    },
                    other => Deger::Hata(format!("{} değerinin uzunluğu alınamaz", other)),
                }
            }
            Ifade::FonksiyonIfadesi {
                parametreler,
                govde,
            } => Deger::Fonksiyon {
                parametreler,
                govde,
                yakalanan_kapsamlar: self.yerel_scopes.clone(),
                module_kimligi: self.active_module_calls.last().cloned(),
            },
            Ifade::NesneOlustur { sinif_adi, .. } => Deger::Hata(format!(
                "Eski nesne oluşturma AST biçimi desteklenmiyor: {}",
                sinif_adi
            )),
            Ifade::Cagri {
                fonksiyon,
                argumanlar,
                pos,
            } => {
                let call_name = match fonksiyon.as_ref() {
                    Ifade::Degisken(name) => name.clone(),
                    Ifade::NesneErisim { ozellik, .. } => ozellik.clone(),
                    _ => "<anonim>".to_string(),
                };
                let mut method_instance = None;
                let f = if let Ifade::NesneErisim { nesne, ozellik } = *fonksiyon.clone() {
                    let instance = self.ifade_hesapla(*nesne);
                    if let Deger::Nesne {
                        ref sinif_adi,
                        ref alanlar,
                        ref module_kimligi,
                    } = instance
                    {
                        // 1. Önce sınıf metotlarını kontrol et
                        let class_value = module_kimligi
                            .as_ref()
                            .and_then(|module_id| self.module_environments.get(module_id))
                            .and_then(|environment| environment.get(sinif_adi))
                            .or_else(|| self.global_degiskenler.get(sinif_adi))
                            .cloned();
                        if let Some(Deger::Sinif {
                            metotlar,
                            module_kimligi,
                            ..
                        }) = class_value
                        {
                            if let Some((ps, bd)) = metotlar.get(&ozellik) {
                                method_instance = Some(instance.clone());
                                Deger::Fonksiyon {
                                    parametreler: ps.clone(),
                                    govde: bd.clone(),
                                    yakalanan_kapsamlar: Vec::new(),
                                    module_kimligi,
                                }
                            } else {
                                // 2. Sınıf metodu yoksa alanlara bak
                                let field =
                                    match get_keyed_value(alanlar, &ozellik, "Metot/alan erişimi")
                                    {
                                        Ok(Some(field)) => field,
                                        Ok(None) => {
                                            return Deger::Hata(format!(
                                                "Nesne özelliği bulunamadı: {}",
                                                ozellik
                                            ))
                                        }
                                        Err(error) => return Deger::Hata(error),
                                    };
                                if matches!(
                                    &field,
                                    Deger::Fonksiyon { .. }
                                        | Deger::DahiliFonksiyon(_)
                                        | Deger::BaglamliDahiliFonksiyon(_)
                                ) {
                                    method_instance = Some(instance.clone());
                                }
                                field
                            }
                        } else {
                            // 3. Sınıf yoksa (düz nesne) alanlara bak
                            let field =
                                match get_keyed_value(alanlar, &ozellik, "Metot/alan erişimi") {
                                    Ok(Some(field)) => field,
                                    Ok(None) => {
                                        return Deger::Hata(format!(
                                            "Nesne özelliği bulunamadı: {}",
                                            ozellik
                                        ))
                                    }
                                    Err(error) => return Deger::Hata(error),
                                };
                            if matches!(
                                &field,
                                Deger::Fonksiyon { .. }
                                    | Deger::DahiliFonksiyon(_)
                                    | Deger::BaglamliDahiliFonksiyon(_)
                            ) {
                                method_instance = Some(instance.clone());
                            }
                            field
                        }
                    } else if let Deger::Sozluk(ref m) = instance {
                        if ozellik == "getir" {
                            let args = argumanlar
                                .into_iter()
                                .map(|a| self.ifade_hesapla(a))
                                .collect::<Vec<_>>();
                            let [Deger::Metin(key)] = args.as_slice() else {
                                return Deger::Hata(
                                    "sözlük.getir: tam olarak 1 metin anahtarı gerekir".to_string(),
                                );
                            };
                            return match get_keyed_value(m, key, "sözlük.getir") {
                                Ok(value) => value.unwrap_or(Deger::Bos),
                                Err(error) => Deger::Hata(error),
                            };
                        } else if ozellik == "ayarla" {
                            let args = argumanlar
                                .into_iter()
                                .map(|a| self.ifade_hesapla(a))
                                .collect::<Vec<_>>();
                            let [Deger::Metin(key), value] = args.as_slice() else {
                                return Deger::Hata(
                                    "sözlük.ayarla: tam olarak metin anahtar ve değer gerekir"
                                        .to_string(),
                                );
                            };
                            return match insert_keyed_value(
                                m,
                                key.clone(),
                                value.clone(),
                                self.limits.max_collection_items,
                                "sözlük.ayarla",
                            ) {
                                Ok(()) => Deger::Sayi(1.0),
                                Err(error) => Deger::Hata(error),
                            };
                        } else {
                            match get_keyed_value(m, &ozellik, "Sözlük özelliği erişimi") {
                                Ok(Some(value)) => value,
                                Ok(None) => {
                                    Deger::Hata(format!("Sözlük özelliği bulunamadı: {}", ozellik))
                                }
                                Err(error) => Deger::Hata(error),
                            }
                        }
                    } else {
                        self.ifade_hesapla(*fonksiyon)
                    }
                } else {
                    self.ifade_hesapla(*fonksiyon)
                };

                let args = argumanlar
                    .into_iter()
                    .map(|a| self.ifade_hesapla(a))
                    .collect();
                if !matches!(
                    f,
                    Deger::Fonksiyon { .. }
                        | Deger::DahiliFonksiyon(_)
                        | Deger::BaglamliDahiliFonksiyon(_)
                        | Deger::Sinif { .. }
                ) {
                    return Deger::Hata(format!(
                        "Satır {}, Sütun {}: Çağrılamayan değer: {}",
                        pos.0, pos.1, f
                    ));
                }
                let previous_location = self.current_location.replace(SourceSpan {
                    line: pos.0,
                    column: pos.1,
                });
                let result = self.fonksiyon_cagrisi_detayli(
                    f,
                    args,
                    method_instance,
                    Some(StackFrame {
                        function: call_name,
                        location: Some(SourceSpan {
                            line: pos.0,
                            column: pos.1,
                        }),
                    }),
                );
                self.current_location = previous_location;
                result
            }
            Ifade::IkiliIslem { sol, operator, sag } => {
                let l = self.ifade_hesapla(*sol);
                if operator == Token::Ve {
                    let left = match self.dogruluk_kontrolu(l) {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    if !left {
                        return Deger::Sayi(0.0);
                    }
                    let r = self.ifade_hesapla(*sag);
                    return match self.dogruluk_kontrolu(r) {
                        Ok(value) => Deger::Sayi(if value { 1.0 } else { 0.0 }),
                        Err(error) => Deger::Hata(error),
                    };
                }
                if operator == Token::Veya {
                    let left = match self.dogruluk_kontrolu(l) {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    if left {
                        return Deger::Sayi(1.0);
                    }
                    let r = self.ifade_hesapla(*sag);
                    return match self.dogruluk_kontrolu(r) {
                        Ok(value) => Deger::Sayi(if value { 1.0 } else { 0.0 }),
                        Err(error) => Deger::Hata(error),
                    };
                }
                let r = self.ifade_hesapla(*sag);
                crate::semantics::ikili_islem(&operator, l, r).unwrap_or_else(Deger::Hata)
            }
            Ifade::MantıksalDegil(i) => {
                let v = self.ifade_hesapla(*i);
                match self.dogruluk_kontrolu(v) {
                    Ok(value) => Deger::Sayi(if value { 0.0 } else { 1.0 }),
                    Err(error) => Deger::Hata(error),
                }
            }
        }
    }

    fn dogruluk_kontrolu(&self, deger: Deger) -> Result<bool, String> {
        crate::semantics::dogru_mu(&deger)
    }
}
