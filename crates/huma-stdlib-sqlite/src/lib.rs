//! Hüma'nın isteğe bağlı SQLite adaptörü.

use huma_runtime::capability::{self, Capability};
use huma_runtime::gc::Gc;
use huma_runtime::value::Deger;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

const MAX_SAFE_NUMERIC_ID: u64 = (1_u64 << 53) - 1;
const MAX_CONNECTIONS: usize = 1_024;
const MAX_SQL_BYTES: usize = 1_024 * 1_024;
const MAX_COLUMNS: usize = 1_024;
const MAX_ITEMS: usize = 1_000_000;
const MAX_PAYLOAD_BYTES: usize = 64 * 1_024 * 1_024;

static CONNECTIONS: Lazy<Mutex<HashMap<u64, rusqlite::Connection>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> Result<u64, String> {
    NEXT_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            (current < MAX_SAFE_NUMERIC_ID).then_some(current + 1)
        })
        .map_err(|_| "SQLite bağlantı kimliği alanı tükendi".to_string())
}

fn numeric_id(value: f64, operation: &str) -> Result<u64, String> {
    if !value.is_finite()
        || value < 0.0
        || value.fract() != 0.0
        || value > MAX_SAFE_NUMERIC_ID as f64
    {
        return Err(format!(
            "{operation}: kimlik güvenli aralıkta negatif olmayan tamsayı olmalıdır"
        ));
    }
    Ok(value as u64)
}

fn capability_error(operation: &str) -> Option<Deger> {
    capability::require(Capability::Database, operation)
        .err()
        .map(Deger::Hata)
}

/// SQLite yerleşiklerini verilen ana makine küresel tablosuna ekler.
pub fn kayit_et(globals: &mut HashMap<String, Deger>) {
    globals.insert(
        "dahili_sql_bağlan".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(path)] = args.as_slice() else {
                return if args.len() == 1 {
                    Deger::Hata("dahili_sql_bağlan: dosya yolu metin olmalıdır".to_string())
                } else {
                    Deger::Hata(format!(
                        "dahili_sql_bağlan: tam olarak 1 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            if path.len() > 4_096 {
                return Deger::Hata("dahili_sql_bağlan: dosya yolu çok uzun".to_string());
            }
            if let Some(error) = capability_error("dahili_sql_bağlan") {
                return error;
            }
            let connection = match rusqlite::Connection::open(path) {
                Ok(connection) => connection,
                Err(error) => {
                    return Deger::Hata(format!("dahili_sql_bağlan: veritabanı açılamadı: {error}"))
                }
            };
            let mut connections = match CONNECTIONS.lock() {
                Ok(connections) => connections,
                Err(_) => {
                    return Deger::Hata(
                        "dahili_sql_bağlan: bağlantı tablosu kilitlenemedi".to_string(),
                    )
                }
            };
            if connections.len() >= MAX_CONNECTIONS {
                return Deger::Hata(format!(
                    "dahili_sql_bağlan: en fazla {MAX_CONNECTIONS} bağlantı açık olabilir"
                ));
            }
            let id = match next_id() {
                Ok(id) => id,
                Err(error) => return Deger::Hata(format!("dahili_sql_bağlan: {error}")),
            };
            connections.insert(id, connection);
            Deger::Sayi(id as f64)
        }),
    );

    globals.insert(
        "dahili_sql_kapat".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(id)] = args.as_slice() else {
                return if args.len() == 1 {
                    Deger::Hata("dahili_sql_kapat: bağlantı kimliği sayı olmalıdır".to_string())
                } else {
                    Deger::Hata(format!(
                        "dahili_sql_kapat: tam olarak 1 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            let id = match numeric_id(*id, "dahili_sql_kapat") {
                Ok(id) => id,
                Err(error) => return Deger::Hata(error),
            };
            if let Some(error) = capability_error("dahili_sql_kapat") {
                return error;
            }
            let mut connections = match CONNECTIONS.lock() {
                Ok(connections) => connections,
                Err(_) => {
                    return Deger::Hata(
                        "dahili_sql_kapat: bağlantı tablosu kilitlenemedi".to_string(),
                    )
                }
            };
            match connections.remove(&id) {
                Some(connection) => match connection.close() {
                    Ok(()) => Deger::Sayi(1.0),
                    Err((connection, error)) => {
                        connections.insert(id, connection);
                        Deger::Hata(format!("dahili_sql_kapat: {error}"))
                    }
                },
                None => Deger::Hata(format!(
                    "dahili_sql_kapat: {id} kimlikli bağlantı bulunamadı"
                )),
            }
        }),
    );

    globals.insert(
        "dahili_sql_yürüt".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(id), Deger::Metin(sql)] = args.as_slice() else {
                return if args.len() == 2 {
                    Deger::Hata(
                        "dahili_sql_yürüt: bağlantı kimliği ve SQL metni gerekir".to_string(),
                    )
                } else {
                    Deger::Hata(format!(
                        "dahili_sql_yürüt: tam olarak 2 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            let id = match numeric_id(*id, "dahili_sql_yürüt") {
                Ok(id) => id,
                Err(error) => return Deger::Hata(error),
            };
            if sql.len() > MAX_SQL_BYTES {
                return Deger::Hata(format!(
                    "dahili_sql_yürüt: SQL metni {MAX_SQL_BYTES} bayt sınırını aşıyor"
                ));
            }
            if let Some(error) = capability_error("dahili_sql_yürüt") {
                return error;
            }
            let connections = match CONNECTIONS.lock() {
                Ok(connections) => connections,
                Err(_) => {
                    return Deger::Hata(
                        "dahili_sql_yürüt: bağlantı tablosu kilitlenemedi".to_string(),
                    )
                }
            };
            let Some(connection) = connections.get(&id) else {
                return Deger::Hata(format!(
                    "dahili_sql_yürüt: {id} kimlikli bağlantı bulunamadı"
                ));
            };
            match connection.execute(sql, []) {
                Ok(affected) => Deger::Sayi(affected as f64),
                Err(error) => Deger::Hata(format!("dahili_sql_yürüt: {error}")),
            }
        }),
    );

    globals.insert(
        "dahili_sql_sorgula".to_string(),
        Deger::DahiliFonksiyon(sql_query),
    );
}

fn sql_query(args: Vec<Deger>) -> Deger {
    let [Deger::Sayi(id), Deger::Metin(sql)] = args.as_slice() else {
        return if args.len() == 2 {
            Deger::Hata("dahili_sql_sorgula: bağlantı kimliği ve SQL metni gerekir".to_string())
        } else {
            Deger::Hata(format!(
                "dahili_sql_sorgula: tam olarak 2 argüman bekleniyordu; {} geldi",
                args.len()
            ))
        };
    };
    let id = match numeric_id(*id, "dahili_sql_sorgula") {
        Ok(id) => id,
        Err(error) => return Deger::Hata(error),
    };
    if sql.len() > MAX_SQL_BYTES {
        return Deger::Hata(format!(
            "dahili_sql_sorgula: SQL metni {MAX_SQL_BYTES} bayt sınırını aşıyor"
        ));
    }
    if let Some(error) = capability_error("dahili_sql_sorgula") {
        return error;
    }
    let connections = match CONNECTIONS.lock() {
        Ok(connections) => connections,
        Err(_) => {
            return Deger::Hata("dahili_sql_sorgula: bağlantı tablosu kilitlenemedi".to_string())
        }
    };
    let Some(connection) = connections.get(&id) else {
        return Deger::Hata(format!(
            "dahili_sql_sorgula: {id} kimlikli bağlantı bulunamadı"
        ));
    };
    let mut statement = match connection.prepare(sql) {
        Ok(statement) => statement,
        Err(error) => return Deger::Hata(format!("dahili_sql_sorgula: {error}")),
    };
    let columns = statement
        .column_names()
        .iter()
        .map(|name| name.trim().to_string())
        .collect::<Vec<_>>();
    if columns.len() > MAX_COLUMNS {
        return Deger::Hata(format!(
            "dahili_sql_sorgula: en fazla {MAX_COLUMNS} sütun desteklenir"
        ));
    }
    if columns.iter().any(String::is_empty) {
        return Deger::Hata("dahili_sql_sorgula: sonuç sütunlarının adı boş olamaz".to_string());
    }
    if columns.iter().collect::<HashSet<_>>().len() != columns.len() {
        return Deger::Hata(
            "dahili_sql_sorgula: sonuç sütun adları benzersiz olmalıdır; AS kullanın".to_string(),
        );
    }
    let mut rows = match statement.query([]) {
        Ok(rows) => rows,
        Err(error) => return Deger::Hata(format!("dahili_sql_sorgula: {error}")),
    };
    let mut result = Vec::new();
    let mut cell_count = 0_usize;
    let mut payload_bytes = 0_usize;
    loop {
        let row = match rows.next() {
            Ok(Some(row)) => row,
            Ok(None) => break,
            Err(error) => {
                return Deger::Hata(format!("dahili_sql_sorgula: satır okunamadı: {error}"))
            }
        };
        if result.len() >= MAX_ITEMS {
            return Deger::Hata(format!(
                "dahili_sql_sorgula: satır sayısı {MAX_ITEMS} öğelik güvenlik sınırını aşıyor"
            ));
        }
        cell_count = match cell_count.checked_add(columns.len()) {
            Some(count) if count <= MAX_ITEMS => count,
            _ => {
                return Deger::Hata(format!(
                    "dahili_sql_sorgula: hücre sayısı {MAX_ITEMS} öğelik güvenlik sınırını aşıyor"
                ))
            }
        };
        let mut fields = HashMap::new();
        for (index, column) in columns.iter().enumerate() {
            let value: rusqlite::types::Value = match row.get(index) {
                Ok(value) => value,
                Err(error) => {
                    return Deger::Hata(format!(
                        "dahili_sql_sorgula: {}. sütun okunamadı: {error}",
                        index + 1
                    ))
                }
            };
            let value = match value {
                rusqlite::types::Value::Null => Deger::Bos,
                rusqlite::types::Value::Integer(value)
                    if value.unsigned_abs() <= MAX_SAFE_NUMERIC_ID =>
                {
                    Deger::Sayi(value as f64)
                }
                rusqlite::types::Value::Integer(value) => {
                    return Deger::Hata(format!(
                    "dahili_sql_sorgula: {value} tamsayısı Hüma sayı türünde tam temsil edilemez"
                ))
                }
                rusqlite::types::Value::Real(value) if value.is_finite() => Deger::Sayi(value),
                rusqlite::types::Value::Real(_) => {
                    return Deger::Hata(
                        "dahili_sql_sorgula: sonlu olmayan gerçek sayı döndü".to_string(),
                    )
                }
                rusqlite::types::Value::Text(value) => {
                    if !add_payload(&mut payload_bytes, value.len()) {
                        return payload_limit_error();
                    }
                    Deger::Metin(value)
                }
                rusqlite::types::Value::Blob(value) => {
                    if !add_payload(&mut payload_bytes, value.len()) {
                        return payload_limit_error();
                    }
                    Deger::Bayt(value)
                }
            };
            fields.insert(column.clone(), value);
        }
        result.push(Deger::Nesne {
            sinif_adi: "Satır".to_string(),
            alanlar: Gc::new(fields),
            module_kimligi: None,
        });
    }
    Deger::Liste(Gc::new(result))
}

fn add_payload(current: &mut usize, amount: usize) -> bool {
    match current.checked_add(amount) {
        Some(next) if next <= MAX_PAYLOAD_BYTES => {
            *current = next;
            true
        }
        _ => false,
    }
}

fn payload_limit_error() -> Deger {
    Deger::Hata(format!(
        "dahili_sql_sorgula: metin/bayt çıktısı {MAX_PAYLOAD_BYTES} bayt sınırını aşıyor"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kimlik_dogrulamasi_sonlu_tamsayi_ister() {
        assert!(numeric_id(f64::NAN, "test").is_err());
        assert!(numeric_id(1.5, "test").is_err());
        assert_eq!(numeric_id(1.0, "test").unwrap(), 1);
    }
}
