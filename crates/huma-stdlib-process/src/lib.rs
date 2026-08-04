//! Hüma'nın isteğe bağlı ve sınırlı alt-süreç adaptörü.

use huma_runtime::capability::{self, Capability};
use huma_runtime::value::Deger;
use std::collections::HashMap;
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use wait_timeout::ChildExt;

const MAX_COMMAND_BYTES: usize = 64 * 1_024;
const MAX_OUTPUT_BYTES: usize = 16 * 1_024 * 1_024;
const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

pub fn kayit_et(globals: &mut HashMap<String, Deger>) {
    globals.insert(
        "sistem".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(command)] = args.as_slice() else {
                return Deger::Hata("sistem: tam olarak 1 metin komutu gerekir".to_string());
            };
            if let Err(error) = capability::require(Capability::Process, "sistem") {
                return Deger::Hata(error);
            }
            match run_command(command) {
                Ok(output) => Deger::Metin(output),
                Err(error) => Deger::Hata(error),
            }
        }),
    );
}

fn read_pipe_limited<R: Read>(
    pipe: R,
    limit: usize,
    stream_name: &'static str,
) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut limited = pipe.take((limit as u64) + 1);
    limited
        .read_to_end(&mut bytes)
        .map_err(|error| format!("sistem: {stream_name} okunamadı: {error}"))?;
    if bytes.len() > limit {
        return Err(format!(
            "sistem: {stream_name} {limit} baytlık çıktı sınırını aşıyor"
        ));
    }
    Ok(bytes)
}

fn run_command(command: &str) -> Result<String, String> {
    if command.is_empty() {
        return Err("sistem: komut boş olamaz".to_string());
    }
    if command.len() > MAX_COMMAND_BYTES {
        return Err(format!(
            "sistem: komut {MAX_COMMAND_BYTES} bayt sınırını aşıyor"
        ));
    }
    let mut process = if cfg!(target_os = "windows") {
        let mut process = Command::new("cmd");
        process.args(["/C", command]);
        process
    } else {
        let mut process = Command::new("sh");
        process.args(["-c", command]);
        process
    };
    let mut child = process
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("sistem: komut başlatılamadı: {error}"))?;
    let stdout = child.stdout.take().ok_or_else(|| {
        let _ = child.kill();
        let _ = child.wait();
        "sistem: standart çıktı yakalanamadı".to_string()
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        let _ = child.kill();
        let _ = child.wait();
        "sistem: standart hata yakalanamadı".to_string()
    })?;

    let stdout_reader = thread::Builder::new()
        .name("huma-sistem-stdout".to_string())
        .spawn(move || read_pipe_limited(stdout, MAX_OUTPUT_BYTES, "standart çıktı"))
        .map_err(|error| {
            let _ = child.kill();
            let _ = child.wait();
            format!("sistem: çıktı okuyucusu başlatılamadı: {error}")
        })?;
    let stderr_reader = match thread::Builder::new()
        .name("huma-sistem-stderr".to_string())
        .spawn(move || read_pipe_limited(stderr, MAX_OUTPUT_BYTES, "standart hata"))
    {
        Ok(reader) => reader,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            return Err(format!(
                "sistem: hata çıktısı okuyucusu başlatılamadı: {error}"
            ));
        }
    };

    let status = match child.wait_timeout(COMMAND_TIMEOUT) {
        Ok(Some(status)) => status,
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err(format!(
                "sistem: komut {} saniyede tamamlanmadı",
                COMMAND_TIMEOUT.as_secs()
            ));
        }
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err(format!("sistem: komut beklenemedi: {error}"));
        }
    };
    let stdout = stdout_reader.join().map_err(|_| {
        "sistem: standart çıktı okuyucusu beklenmedik biçimde sonlandı".to_string()
    })??;
    let stderr = stderr_reader.join().map_err(|_| {
        "sistem: standart hata okuyucusu beklenmedik biçimde sonlandı".to_string()
    })??;
    if stdout
        .len()
        .checked_add(stderr.len())
        .is_none_or(|length| length > MAX_OUTPUT_BYTES)
    {
        return Err(format!(
            "sistem: toplam çıktı {MAX_OUTPUT_BYTES} bayt sınırını aşıyor"
        ));
    }
    if status.success() {
        String::from_utf8(stdout)
            .map(|output| output.trim().to_string())
            .map_err(|_| "sistem: standart çıktı geçerli UTF-8 değil".to_string())
    } else {
        let code = status
            .code()
            .map_or_else(|| "sinyal".to_string(), |value| value.to_string());
        let stderr = String::from_utf8(stderr)
            .map_err(|_| "sistem: standart hata geçerli UTF-8 değil".to_string())?;
        let stderr = stderr.trim();
        if stderr.is_empty() {
            Err(format!("sistem: komut başarısız oldu (çıkış: {code})"))
        } else {
            Err(format!(
                "sistem: komut başarısız oldu (çıkış: {code}): {stderr}"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bos_ve_asiri_uzun_komut_calistirilmadan_reddedilir() {
        assert!(run_command("").unwrap_err().contains("boş"));
        assert!(run_command(&"x".repeat(MAX_COMMAND_BYTES + 1)).is_err());
    }
}
