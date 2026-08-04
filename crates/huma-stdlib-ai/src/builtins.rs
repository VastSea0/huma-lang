use crate::autograd::{TensorData, AUTOGRAD_GRAF};
use crate::tokenizer::BPE_TOKENIZER;
use huma_runtime::gc::Gc;
use huma_runtime::value::{Deger, HostObject};
use std::collections::HashMap;

/// Deneysel AI yerleşiklerini açıkça verilen global tabloya ekler.
pub fn kayit_et(globals: &mut HashMap<String, Deger>) {
    globals.insert(
        "tensor_olustur".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if !(3..=4).contains(&args.len()) {
                return Deger::Hata(format!(
                    "tensor_olustur: 3 veya 4 argüman bekleniyordu; {} geldi",
                    args.len()
                ));
            }
            let (Deger::Sayi(rows), Deger::Sayi(cols), Deger::Liste(values)) =
                (&args[0], &args[1], &args[2])
            else {
                return Deger::Hata(
                    "tensor_olustur: satır, sütun ve sayı listesi gerekir".to_string(),
                );
            };
            let rows = match dimension(*rows, "tensor_olustur") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            let cols = match dimension(*cols, "tensor_olustur") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            let expected = match rows.checked_mul(cols) {
                Some(value) if value <= 10_000_000 => value,
                _ => return Deger::Hata("tensor_olustur: eleman sınırı aşıldı".to_string()),
            };
            let requires_grad = match args.get(3) {
                None => true,
                Some(Deger::Sayi(0.0)) => false,
                Some(Deger::Sayi(1.0)) => true,
                Some(_) => {
                    return Deger::Hata(
                        "tensor_olustur: gradyan bayrağı 0 veya 1 olmalı".to_string(),
                    )
                }
            };
            let values = match values.try_borrow() {
                Ok(values) => values,
                Err(_) => return Deger::Hata("tensor_olustur: liste kullanımda".to_string()),
            };
            if values.len() != expected {
                return Deger::Hata(format!(
                    "tensor_olustur: {rows}x{cols} boyut için {expected} eleman gerekir; {} geldi",
                    values.len()
                ));
            }
            let mut data = Vec::with_capacity(expected);
            for (index, value) in values.iter().enumerate() {
                match value {
                    Deger::Sayi(number) if number.is_finite() => data.push(*number),
                    _ => {
                        return Deger::Hata(format!(
                            "tensor_olustur: {index}. eleman sonlu sayı olmalı"
                        ))
                    }
                }
            }
            let mut graph = match AUTOGRAD_GRAF.lock() {
                Ok(graph) => graph,
                Err(_) => return Deger::Hata("tensor_olustur: autograd kilidi bozuk".to_string()),
            };
            graph
                .tensor_olustur(rows, cols, data, requires_grad)
                .map(host_tensor)
                .unwrap_or_else(Deger::Hata)
        }),
    );
    globals.insert(
        "tensor_topla".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [left, right] = args.as_slice() else {
                return Deger::Hata(
                    "tensor_topla: tam olarak iki tensor argümanı gerekir".to_string(),
                );
            };
            let (Some(left), Some(right)) = (tensor(left), tensor(right)) else {
                return Deger::Hata(
                    "tensor_topla: tam olarak iki tensor argümanı gerekir".to_string(),
                );
            };
            AUTOGRAD_GRAF
                .lock()
                .map_err(|_| "tensor_topla: autograd kilidi bozuk".to_string())
                .and_then(|mut graph| graph.topla(left, right))
                .map(host_tensor)
                .unwrap_or_else(Deger::Hata)
        }),
    );
    globals.insert(
        "tensor_matris_carp".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [left, right] = args.as_slice() else {
                return Deger::Hata(
                    "tensor_matris_carp: tam olarak iki tensor argümanı gerekir".to_string(),
                );
            };
            let (Some(left), Some(right)) = (tensor(left), tensor(right)) else {
                return Deger::Hata(
                    "tensor_matris_carp: tam olarak iki tensor argümanı gerekir".to_string(),
                );
            };
            AUTOGRAD_GRAF
                .lock()
                .map_err(|_| "tensor_matris_carp: autograd kilidi bozuk".to_string())
                .and_then(|mut graph| graph.matris_carp(left, right))
                .map(host_tensor)
                .unwrap_or_else(Deger::Hata)
        }),
    );
    globals.insert(
        "tensor_relu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [value] = args.as_slice() else {
                return Deger::Hata(
                    "tensor_relu: tam olarak bir tensor argümanı gerekir".to_string(),
                );
            };
            let Some(value) = tensor(value) else {
                return Deger::Hata(
                    "tensor_relu: tam olarak bir tensor argümanı gerekir".to_string(),
                );
            };
            AUTOGRAD_GRAF
                .lock()
                .map_err(|_| "tensor_relu: autograd kilidi bozuk".to_string())
                .and_then(|mut graph| graph.relu(value))
                .map(host_tensor)
                .unwrap_or_else(Deger::Hata)
        }),
    );
    globals.insert(
        "tensor_geri_yayilim".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [value] = args.as_slice() else {
                return Deger::Hata(
                    "tensor_geri_yayilim: tam olarak bir tensor argümanı gerekir".to_string(),
                );
            };
            let Some(value) = tensor(value) else {
                return Deger::Hata(
                    "tensor_geri_yayilim: tam olarak bir tensor argümanı gerekir".to_string(),
                );
            };
            AUTOGRAD_GRAF
                .lock()
                .map_err(|_| "tensor_geri_yayilim: autograd kilidi bozuk".to_string())
                .and_then(|mut graph| graph.backward(value.id))
                .map(|()| Deger::Sayi(1.0))
                .unwrap_or_else(Deger::Hata)
        }),
    );
    globals.insert(
        "tensor_gradyan".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [value] = args.as_slice() else {
                return Deger::Hata(
                    "tensor_gradyan: tam olarak bir tensor argümanı gerekir".to_string(),
                );
            };
            let Some(value) = tensor(value) else {
                return Deger::Hata(
                    "tensor_gradyan: tam olarak bir tensor argümanı gerekir".to_string(),
                );
            };
            value
                .gradyan
                .lock()
                .map(|gradient| {
                    Deger::Liste(Gc::new(gradient.iter().copied().map(Deger::Sayi).collect()))
                })
                .unwrap_or_else(|_| Deger::Hata("tensor_gradyan: gradyan kilidi bozuk".to_string()))
        }),
    );

    globals.insert(
        "bpe_eğit".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(text), Deger::Sayi(vocabulary)] = args.as_slice() else {
                return Deger::Hata("bpe_eğit: metin ve sözlük boyutu gerekir".to_string());
            };
            if !vocabulary.is_finite()
                || vocabulary.fract() != 0.0
                || !(256.0..=65_536.0).contains(vocabulary)
            {
                return Deger::Hata(
                    "bpe_eğit: sözlük boyutu 256 ile 65536 arasında bir tamsayı olmalı".to_string(),
                );
            }
            BPE_TOKENIZER
                .lock()
                .map_err(|_| "bpe_eğit: tokenizer kilidi bozuk".to_string())
                .and_then(|mut tokenizer| tokenizer.egit(text, *vocabulary as usize))
                .map(|()| Deger::Sayi(1.0))
                .unwrap_or_else(Deger::Hata)
        }),
    );
    globals.insert(
        "bpe_kodla".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(text)] = args.as_slice() else {
                return Deger::Hata("bpe_kodla: tam olarak bir metin gerekir".to_string());
            };
            BPE_TOKENIZER
                .lock()
                .map_err(|_| "bpe_kodla: tokenizer kilidi bozuk".to_string())
                .and_then(|tokenizer| tokenizer.kodla(text))
                .map(|ids| {
                    Deger::Liste(Gc::new(
                        ids.into_iter().map(|id| Deger::Sayi(id as f64)).collect(),
                    ))
                })
                .unwrap_or_else(Deger::Hata)
        }),
    );
    globals.insert(
        "bpe_çöz".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Liste(values)] = args.as_slice() else {
                return Deger::Hata("bpe_çöz: token kimliği listesi gerekir".to_string());
            };
            let values = match values.try_borrow() {
                Ok(values) => values,
                Err(_) => return Deger::Hata("bpe_çöz: liste kullanımda".to_string()),
            };
            let ids = match values
                .iter()
                .map(|value| match value {
                    Deger::Sayi(id)
                        if id.is_finite()
                            && *id >= 0.0
                            && id.fract() == 0.0
                            && *id <= usize::MAX as f64 =>
                    {
                        Ok(*id as usize)
                    }
                    _ => Err(
                        "bpe_çöz: token kimlikleri negatif olmayan tamsayılar olmalı".to_string(),
                    ),
                })
                .collect::<Result<Vec<_>, _>>()
            {
                Ok(ids) => ids,
                Err(error) => return Deger::Hata(error),
            };
            BPE_TOKENIZER
                .lock()
                .map_err(|_| "bpe_çöz: tokenizer kilidi bozuk".to_string())
                .and_then(|tokenizer| tokenizer.coz(&ids))
                .map(Deger::Metin)
                .unwrap_or_else(Deger::Hata)
        }),
    );
}

fn dimension(value: f64, operation: &str) -> Result<usize, String> {
    if !value.is_finite() || value < 1.0 || value.fract() != 0.0 || value > usize::MAX as f64 {
        return Err(format!("{operation}: boyut pozitif sonlu tamsayı olmalı"));
    }
    Ok(value as usize)
}

fn tensor(value: &Deger) -> Option<&TensorData> {
    match value {
        Deger::Harici(value) => value.downcast_ref(),
        _ => None,
    }
}

fn host_tensor(value: TensorData) -> Deger {
    Deger::Harici(HostObject::new(value))
}
