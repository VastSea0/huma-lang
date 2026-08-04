//! Yetenek denetimli dosya, CSV ve JSONL adaptörü.

use huma_runtime::capability::{self, Capability};
use huma_runtime::gc::Gc;
use huma_runtime::value::Deger;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_FILE_BYTES: usize = 64 * 1024 * 1024;
const MAX_ITEMS: usize = 1_000_000;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(1);

fn capability_error(required: Capability, operation: &str) -> Option<Deger> {
    capability::require(required, operation)
        .err()
        .map(Deger::Hata)
}

fn read_limited(path: &str, operation: &str) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|error| format!("{operation}: '{path}': {error}"))?;
    if file
        .metadata()
        .is_ok_and(|metadata| metadata.len() > MAX_FILE_BYTES as u64)
    {
        return Err(format!(
            "{operation}: dosya {MAX_FILE_BYTES} bayt sınırını aşıyor"
        ));
    }
    let mut bytes = Vec::new();
    file.take((MAX_FILE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("{operation}: '{path}': {error}"))?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err(format!(
            "{operation}: dosya {MAX_FILE_BYTES} bayt sınırını aşıyor"
        ));
    }
    Ok(bytes)
}

fn read_text(path: &str, operation: &str) -> Result<String, String> {
    String::from_utf8(read_limited(path, operation)?)
        .map_err(|_| format!("{operation}: dosya geçerli UTF-8 değil"))
}

fn unique_sibling(path: &Path, suffix: &str) -> Result<PathBuf, String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Dosya adı geçerli UTF-8 değil".to_string())?;
    for _ in 0..1_024 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            ".{name}.{suffix}-{}-{sequence}",
            std::process::id()
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("Benzersiz geçici dosya yolu üretilemedi".to_string())
}

fn atomic_write(path: &str, bytes: &[u8], operation: &str) -> Result<(), String> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err(format!("{operation}: çıktı boyut sınırını aşıyor"));
    }
    let path = Path::new(path);
    let temporary = unique_sibling(path, "tmp")?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| format!("{operation}: geçici dosya açılamadı: {error}"))?;
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = fs::remove_file(&temporary);
        return Err(format!("{operation}: çıktı yazılamadı: {error}"));
    }
    drop(file);
    if fs::rename(&temporary, path).is_ok() {
        return Ok(());
    }
    if !path.exists() {
        let _ = fs::remove_file(&temporary);
        return Err(format!("{operation}: çıktı etkinleştirilemedi"));
    }
    let backup = unique_sibling(path, "backup")?;
    fs::rename(path, &backup).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("{operation}: eski çıktı yedeklenemedi: {error}")
    })?;
    if let Err(error) = fs::rename(&temporary, path) {
        let restore = fs::rename(&backup, path);
        let _ = fs::remove_file(&temporary);
        return Err(match restore {
            Ok(()) => format!("{operation}: yeni çıktı etkinleştirilemedi: {error}"),
            Err(restore_error) => format!(
                "{operation}: çıktı etkinleştirilemedi ({error}); yedek geri yüklenemedi ({restore_error}): {}",
                backup.display()
            ),
        });
    }
    let _ = fs::remove_file(backup);
    Ok(())
}

fn append_line(path: &str, line: &str, operation: &str) -> Result<(), String> {
    let existing = match fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
        Err(error) => return Err(format!("{operation}: '{path}': {error}")),
    };
    let added = line
        .len()
        .checked_add(1)
        .ok_or_else(|| format!("{operation}: satır boyutu taştı"))?;
    if existing.saturating_add(added as u64) > MAX_FILE_BYTES as u64 {
        return Err(format!("{operation}: dosya boyut sınırını aşar"));
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("{operation}: '{path}': {error}"))?;
    writeln!(file, "{line}").map_err(|error| format!("{operation}: '{path}': {error}"))
}

fn delimiter(value: Option<&Deger>, operation: &str) -> Result<u8, String> {
    let Some(value) = value else {
        return Ok(b',');
    };
    let Deger::Metin(value) = value else {
        return Err(format!("{operation}: ayraç tek baytlık ASCII metin olmalı"));
    };
    let bytes = value.as_bytes();
    if bytes.len() != 1 || !bytes[0].is_ascii() {
        return Err(format!("{operation}: ayraç tek baytlık ASCII metin olmalı"));
    }
    if matches!(bytes[0], 0 | b'\r' | b'\n' | b'"') {
        return Err(format!("{operation}: geçersiz CSV ayracı"));
    }
    Ok(bytes[0])
}

fn csv_field(value: &Deger, row: usize, column: usize) -> Result<String, String> {
    match value {
        Deger::Metin(value) => Ok(value.clone()),
        Deger::Sayi(value) if value.is_finite() => Ok(value.to_string()),
        Deger::Bos => Ok(String::new()),
        _ => Err(format!(
            "csv_yaz: {}. satır {}. sütunda metin, sayı veya boş değer gerekir (sayı sonlu olmalı)",
            row + 1,
            column + 1
        )),
    }
}

pub fn kayit_et(globals: &mut HashMap<String, Deger>) {
    globals.insert(
        "dosya_oku".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(path)] = args.as_slice() else {
                return Deger::Hata("dosya_oku: tam olarak bir dosya yolu gerekir".to_string());
            };
            if let Some(error) = capability_error(Capability::FileRead, "dosya_oku") {
                return error;
            }
            read_text(path, "dosya_oku")
                .map(Deger::Metin)
                .unwrap_or_else(Deger::Hata)
        }),
    );
    globals.insert(
        "dosya_oku_bayt".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(path)] = args.as_slice() else {
                return Deger::Hata(
                    "dosya_oku_bayt: tam olarak bir dosya yolu gerekir".to_string(),
                );
            };
            if let Some(error) = capability_error(Capability::FileRead, "dosya_oku_bayt") {
                return error;
            }
            read_limited(path, "dosya_oku_bayt")
                .map(Deger::Bayt)
                .unwrap_or_else(Deger::Hata)
        }),
    );
    globals.insert(
        "dosya_yaz".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(path), Deger::Metin(content)] = args.as_slice() else {
                return Deger::Hata(
                    "dosya_yaz: tam olarak dosya yolu ve metin gerekir".to_string(),
                );
            };
            if let Some(error) = capability_error(Capability::FileWrite, "dosya_yaz") {
                return error;
            }
            atomic_write(path, content.as_bytes(), "dosya_yaz")
                .map(|()| Deger::Sayi(1.0))
                .unwrap_or_else(Deger::Hata)
        }),
    );
    globals.insert(
        "dosya_var_mı".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(path)] = args.as_slice() else {
                return Deger::Hata("dosya_var_mı: tam olarak bir dosya yolu gerekir".to_string());
            };
            if let Some(error) = capability_error(Capability::FileRead, "dosya_var_mı") {
                return error;
            }
            Deger::Sayi(if Path::new(path).exists() { 1.0 } else { 0.0 })
        }),
    );
    globals.insert(
        "dosya_satir_oku".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(path)] = args.as_slice() else {
                return Deger::Hata("dosya_satir_oku: bir dosya yolu gerekir".to_string());
            };
            if let Some(error) = capability_error(Capability::FileRead, "dosya_satir_oku") {
                return error;
            }
            read_text(path, "dosya_satir_oku")
                .and_then(|content| {
                    let lines = content.lines().collect::<Vec<_>>();
                    if lines.len() > MAX_ITEMS {
                        return Err("dosya_satir_oku: satır sınırı aşıldı".to_string());
                    }
                    Ok(lines
                        .into_iter()
                        .map(|line| Deger::Metin(line.to_string()))
                        .collect())
                })
                .map(Gc::new)
                .map(Deger::Liste)
                .unwrap_or_else(Deger::Hata)
        }),
    );
    globals.insert(
        "dosya_satir_ekle".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(path), Deger::Metin(line)] = args.as_slice() else {
                return Deger::Hata("dosya_satir_ekle: dosya yolu ve satır gerekir".to_string());
            };
            if let Some(error) = capability_error(Capability::FileWrite, "dosya_satir_ekle") {
                return error;
            }
            append_line(path, line, "dosya_satir_ekle")
                .map(|()| Deger::Sayi(1.0))
                .unwrap_or_else(Deger::Hata)
        }),
    );
    register_csv(globals);
    register_jsonl(globals);
}

fn register_csv(globals: &mut HashMap<String, Deger>) {
    globals.insert(
        "csv_oku".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (Some(Deger::Metin(path)), delimiter_value) = (args.first(), args.get(1)) else {
                return Deger::Hata("csv_oku: dosya yolu gerekir".to_string());
            };
            if args.len() > 2 {
                return Deger::Hata("csv_oku: en fazla 2 argüman gerekir".to_string());
            }
            let delimiter = match delimiter(delimiter_value, "csv_oku") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            if let Some(error) = capability_error(Capability::FileRead, "csv_oku") {
                return error;
            }
            let content = match read_limited(path, "csv_oku") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .flexible(true)
                .delimiter(delimiter)
                .from_reader(content.as_slice());
            let mut rows = Vec::new();
            let mut items = 0usize;
            for record in reader.records() {
                let record = match record {
                    Ok(record) => record,
                    Err(error) => return Deger::Hata(format!("csv_oku: {error}")),
                };
                items = items.saturating_add(record.len() + 1);
                if items > MAX_ITEMS {
                    return Deger::Hata("csv_oku: öğe sınırı aşıldı".to_string());
                }
                rows.push(Deger::Liste(Gc::new(
                    record
                        .iter()
                        .map(|field| Deger::Metin(field.to_string()))
                        .collect(),
                )));
            }
            Deger::Liste(Gc::new(rows))
        }),
    );
    globals.insert(
        "csv_yaz".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if !(2..=3).contains(&args.len()) {
                return Deger::Hata("csv_yaz: 2 veya 3 argüman gerekir".to_string());
            }
            let (Deger::Metin(path), Deger::Liste(rows)) = (&args[0], &args[1]) else {
                return Deger::Hata("csv_yaz: dosya yolu ve satır listesi gerekir".to_string());
            };
            let delimiter = match delimiter(args.get(2), "csv_yaz") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            if let Some(error) = capability_error(Capability::FileWrite, "csv_yaz") {
                return error;
            }
            let rows = match rows.try_borrow() {
                Ok(rows) => rows,
                Err(_) => return Deger::Hata("csv_yaz: satır listesi kullanımda".to_string()),
            };
            let mut writer = csv::WriterBuilder::new()
                .has_headers(false)
                .delimiter(delimiter)
                .from_writer(Vec::new());
            let mut items = 0usize;
            for (row_index, row) in rows.iter().enumerate() {
                let Deger::Liste(fields) = row else {
                    return Deger::Hata("csv_yaz: her satır liste olmalı".to_string());
                };
                let fields = match fields.try_borrow() {
                    Ok(fields) => fields,
                    Err(_) => return Deger::Hata("csv_yaz: satır kullanımda".to_string()),
                };
                items = items.saturating_add(fields.len() + 1);
                if items > MAX_ITEMS {
                    return Deger::Hata("csv_yaz: öğe sınırı aşıldı".to_string());
                }
                let record = match fields
                    .iter()
                    .enumerate()
                    .map(|(column, value)| csv_field(value, row_index, column))
                    .collect::<Result<Vec<_>, _>>()
                {
                    Ok(record) => record,
                    Err(error) => return Deger::Hata(error),
                };
                if let Err(error) = writer.write_record(record) {
                    return Deger::Hata(format!("csv_yaz: {error}"));
                }
                if writer.get_ref().len() > MAX_FILE_BYTES {
                    return Deger::Hata("csv_yaz: çıktı boyut sınırı aşıldı".to_string());
                }
            }
            let output = match writer.into_inner() {
                Ok(output) => output,
                Err(error) => return Deger::Hata(format!("csv_yaz: {error}")),
            };
            atomic_write(path, &output, "csv_yaz")
                .map(|()| Deger::Sayi(1.0))
                .unwrap_or_else(Deger::Hata)
        }),
    );
}

fn register_jsonl(globals: &mut HashMap<String, Deger>) {
    globals.insert(
        "jsonl_oku".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(path)] = args.as_slice() else {
                return Deger::Hata("jsonl_oku: bir dosya yolu gerekir".to_string());
            };
            if let Some(error) = capability_error(Capability::FileRead, "jsonl_oku") {
                return error;
            }
            let content = match read_text(path, "jsonl_oku") {
                Ok(content) => content,
                Err(error) => return Deger::Hata(error),
            };
            let mut values = Vec::new();
            for (index, line) in content.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                if values.len() >= MAX_ITEMS {
                    return Deger::Hata("jsonl_oku: kayıt sınırı aşıldı".to_string());
                }
                let json = match serde_json::from_str(line) {
                    Ok(json) => json,
                    Err(error) => {
                        return Deger::Hata(format!(
                            "jsonl_oku: {}. satır geçersiz: {error}",
                            index + 1
                        ))
                    }
                };
                match Deger::from_json_checked(&json) {
                    Ok(value) => values.push(value),
                    Err(error) => return Deger::Hata(format!("jsonl_oku: {error}")),
                }
            }
            Deger::Liste(Gc::new(values))
        }),
    );
    globals.insert(
        "jsonl_yaz".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(path), value] = args.as_slice() else {
                return Deger::Hata("jsonl_yaz: dosya yolu ve değer gerekir".to_string());
            };
            if let Some(error) = capability_error(Capability::FileWrite, "jsonl_yaz") {
                return error;
            }
            let line = match value
                .to_json_checked()
                .and_then(|value| serde_json::to_string(&value).map_err(|error| error.to_string()))
            {
                Ok(line) if line.len() <= MAX_FILE_BYTES => line,
                Ok(_) => return Deger::Hata("jsonl_yaz: satır boyut sınırı aşıldı".to_string()),
                Err(error) => return Deger::Hata(format!("jsonl_yaz: {error}")),
            };
            append_line(path, &line, "jsonl_yaz")
                .map(|()| Deger::Sayi(1.0))
                .unwrap_or_else(Deger::Hata)
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varsayilan_yetenek_dosya_okumayi_reddeder() {
        let mut globals = HashMap::new();
        kayit_et(&mut globals);
        let Deger::DahiliFonksiyon(read) = globals["dosya_oku"] else {
            panic!("dosya_oku kayıtlı olmalı");
        };
        assert!(matches!(
            read(vec![Deger::Metin("yok".to_string())]),
            Deger::Hata(error) if error.contains("yeteneği verilmedi")
        ));
    }
}
