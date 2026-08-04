use huma_compiler::Derleyici;
use huma_runtime::gc::{collect_cycles, collect_young, Gc};
use huma_runtime::interpreter::Yorumlayici;
use huma_runtime::value::Deger;
use huma_syntax::ast::Komut;
use huma_syntax::lexer::Lexer;
use huma_syntax::parser::Parser;
use huma_vm::VM;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

const LOOP_SOURCE: &str = r#"
toplam = 0 olsun
i = 1'den 10000'e kadar {
    toplam = toplam + i olsun
}
"#;
const EMPTY_SOURCE: &str = "";
const FUNCTION_SOURCE: &str = r#"
topla fonksiyon olsun a, b alsın {
    a + b'yi döndür
}
toplam = 0 olsun
i = 1'den 1000'e kadar {
    toplam = topla(toplam, i) olsun
}
"#;
const COLLECTION_SOURCE: &str = r#"
değerler = [] olsun
i = 1'den 500'e kadar {
    değerler = listeye_ekle(değerler, i) olsun
}
"#;
const MAP_SOURCE: &str = r#"
harita = {} olsun
i = 1'den 500'e kadar {
    değer_ata(harita, "anahtar-" + i, i) olsun
}
"#;
const CLOSURE_SOURCE: &str = r#"
toplayıcı_üret fonksiyon olsun taban alsın {
    toplayıcı = fonksiyon olsun değer alsın {
        taban + değer'i döndür
    } olsun
    toplayıcı'yı döndür
}
artırıcı = toplayıcı_üret(3) olsun
toplam = 0 olsun
i = 1'den 1000'e kadar {
    toplam = artırıcı(i) olsun
}
"#;
const UNICODE_SOURCE: &str = r#"
metin = "" olsun
i = 1'den 500'e kadar {
    metin = metin + "İıŞşĞğÜüÖöÇç" olsun
}
"#;
const MODULE_SOURCE: &str = r#"
gizli = 40 olsun
topla fonksiyon olsun değer alsın {
    gizli + değer'i döndür
}
topla'yı dışa aktar
"#;
const MODULE_LOAD_SOURCE: &str = r#"yükle "perf_modul.hb" olarak perf"#;
const MEDIAN_LIMIT: f64 = 1.05;
const P95_LIMIT: f64 = 1.10;
const RSS_LIMIT: f64 = 1.10;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    schema_version: u16,
    revision: String,
    suites: BTreeMap<String, Measurement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Measurement {
    iterations: usize,
    median_ns: u64,
    p95_ns: u64,
    rss_kib: u64,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    match arguments.as_slice() {
        [command, output] if command == "measure" => measure(Path::new(output)),
        [command, baseline, candidate] if command == "check" => {
            check(Path::new(baseline), Path::new(candidate))
        }
        _ => Err(
            "Kullanım: huma-perf measure <rapor.json> | huma-perf check <taban.json> <aday.json>"
                .to_string(),
        ),
    }
}

fn parse(source: &str) -> Result<Vec<Komut>, String> {
    let mut parser = Parser::new(Lexer::new(source));
    let (program, diagnostics) = parser.parse_program_with_diagnostics();
    if let Some(error) = diagnostics.first() {
        Err(error.to_string())
    } else {
        Ok(program)
    }
}

fn measure(output: &Path) -> Result<(), String> {
    let ast = parse(LOOP_SOURCE)?;
    let empty_ast = parse(EMPTY_SOURCE)?;
    let function_ast = parse(FUNCTION_SOURCE)?;
    let collection_ast = parse(COLLECTION_SOURCE)?;
    let map_ast = parse(MAP_SOURCE)?;
    let closure_ast = parse(CLOSURE_SOURCE)?;
    let unicode_ast = parse(UNICODE_SOURCE)?;
    let module_load_ast = parse(MODULE_LOAD_SOURCE)?;
    let module_directory = tempfile::tempdir()
        .map_err(|error| format!("Geçici benchmark modül dizini oluşturulamadı: {error}"))?;
    std::fs::write(module_directory.path().join("perf_modul.hb"), MODULE_SOURCE)
        .map_err(|error| format!("Benchmark modülü yazılamadı: {error}"))?;
    let module_search_path = module_directory.path().to_string_lossy().into_owned();
    let mut compiler = Derleyici::new();
    let bytecode = compiler
        .derle_kontrollu(ast.clone())
        .map_err(|error| format!("Benchmark bytecode derlemesi başarısız: {error}"))?;

    // Isınma: ayırıcı, yorumlayıcı ve VM'nin tembel başlatma maliyetini
    // aday ölçümlerinden çıkarır.
    for _ in 0..10 {
        black_box(parse(black_box(LOOP_SOURCE))?);
        run_interpreter(&ast)?;
        run_interpreter(&function_ast)?;
        run_interpreter(&collection_ast)?;
        run_interpreter(&map_ast)?;
        run_interpreter(&closure_ast)?;
        run_interpreter(&unicode_ast)?;
        let mut vm = VM::new(bytecode.clone());
        vm.run_checked().map_err(|error| error.to_string())?;
    }

    let mut suites = BTreeMap::new();
    suites.insert(
        "parser/10k-loop-source".to_string(),
        sample(200, || {
            black_box(parse(black_box(LOOP_SOURCE)).expect("Isınmış kaynak ayrıştırılmalı"));
        }),
    );
    suites.insert(
        "interpreter/startup-empty".to_string(),
        sample(200, || {
            run_interpreter(&empty_ast).expect("Boş program yorumlayıcıda çalışmalı");
        }),
    );
    suites.insert(
        "interpreter/10k-loop".to_string(),
        sample(40, || {
            let mut interpreter = Yorumlayici::new();
            interpreter
                .yorumla_kontrollu(black_box(ast.clone()))
                .expect("Benchmark yorumlayıcıda çalışmalı");
            black_box(interpreter);
        }),
    );
    suites.insert(
        "interpreter/1k-function-calls".to_string(),
        sample(40, || {
            run_interpreter(&function_ast).expect("Fonksiyon benchmark'ı çalışmalı");
        }),
    );
    suites.insert(
        "interpreter/500-list-appends".to_string(),
        sample(40, || {
            run_interpreter(&collection_ast).expect("Koleksiyon benchmark'ı çalışmalı");
        }),
    );
    suites.insert(
        "interpreter/500-map-writes".to_string(),
        sample(40, || {
            run_interpreter(&map_ast).expect("Sözlük benchmark'ı çalışmalı");
        }),
    );
    suites.insert(
        "interpreter/1k-closure-calls".to_string(),
        sample(40, || {
            run_interpreter(&closure_ast).expect("Closure benchmark'ı çalışmalı");
        }),
    );
    suites.insert(
        "interpreter/500-unicode-concats".to_string(),
        sample(40, || {
            run_interpreter(&unicode_ast).expect("Unicode benchmark'ı çalışmalı");
        }),
    );
    suites.insert(
        "module/load-small-file".to_string(),
        sample(40, || {
            let mut interpreter = Yorumlayici::new();
            interpreter.arama_yolları = vec![module_search_path.clone()];
            interpreter
                .yorumla_kontrollu(module_load_ast.clone())
                .expect("Dosya modülü benchmark'ta yüklenmeli");
            black_box(interpreter);
        }),
    );
    suites.insert(
        "gc/minor-1k-live".to_string(),
        sample(40, || {
            let roots = (0..1_000)
                .map(|_| Gc::new(Vec::<Deger>::new()))
                .collect::<Vec<_>>();
            black_box(collect_young());
            black_box(roots);
        }),
    );
    suites.insert(
        "gc/major-1k-cycles".to_string(),
        sample(40, || {
            let roots = (0..1_000)
                .map(|_| {
                    let cycle = Gc::new(Vec::new());
                    cycle.borrow_mut().push(Deger::Liste(cycle.clone()));
                    cycle
                })
                .collect::<Vec<_>>();
            drop(roots);
            black_box(collect_cycles());
        }),
    );
    suites.insert(
        "vm/10k-loop".to_string(),
        sample(40, || {
            let mut vm = VM::new(black_box(bytecode.clone()));
            vm.run_checked().expect("Benchmark VM'de çalışmalı");
            black_box(vm);
        }),
    );

    let report = Report {
        schema_version: 1,
        revision: revision(),
        suites,
    };
    let bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("Performans raporu serileştirilemedi: {error}"))?;
    std::fs::write(output, bytes)
        .map_err(|error| format!("Performans raporu yazılamadı: {error}"))?;
    Ok(())
}

fn run_interpreter(program: &[Komut]) -> Result<(), String> {
    let mut interpreter = Yorumlayici::new();
    interpreter
        .yorumla_kontrollu(program.to_vec())
        .map_err(|error| error.to_string())?;
    black_box(interpreter);
    Ok(())
}

fn sample(iterations: usize, mut operation: impl FnMut()) -> Measurement {
    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        operation();
        samples.push(start.elapsed().as_nanos().min(u64::MAX as u128) as u64);
    }
    samples.sort_unstable();
    let median = percentile(&samples, 0.50);
    let p95 = percentile(&samples, 0.95);
    Measurement {
        iterations,
        median_ns: median,
        p95_ns: p95,
        rss_kib: resident_set_kib().unwrap_or(0),
    }
}

fn percentile(samples: &[u64], quantile: f64) -> u64 {
    let index = ((samples.len() - 1) as f64 * quantile).ceil() as usize;
    samples[index]
}

fn resident_set_kib() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        let line = status.lines().find(|line| line.starts_with("VmRSS:"))?;
        return line.split_whitespace().nth(1)?.parse().ok();
    }
    #[cfg(not(target_os = "linux"))]
    {
        let output = std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .ok()?;
        String::from_utf8(output.stdout).ok()?.trim().parse().ok()
    }
}

fn revision() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|revision| revision.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn check(baseline_path: &Path, candidate_path: &Path) -> Result<(), String> {
    let baseline = read_report(baseline_path)?;
    let candidate = read_report(candidate_path)?;
    let mut regressions = Vec::new();
    for (name, baseline_measurement) in &baseline.suites {
        let candidate_measurement = candidate
            .suites
            .get(name)
            .ok_or_else(|| format!("Aday raporda benchmark eksik: {name}"))?;
        compare_metric(
            &mut regressions,
            name,
            "median",
            baseline_measurement.median_ns,
            candidate_measurement.median_ns,
            MEDIAN_LIMIT,
        );
        compare_metric(
            &mut regressions,
            name,
            "p95",
            baseline_measurement.p95_ns,
            candidate_measurement.p95_ns,
            P95_LIMIT,
        );
        if baseline_measurement.rss_kib > 0 && candidate_measurement.rss_kib > 0 {
            compare_metric(
                &mut regressions,
                name,
                "rss_kib",
                baseline_measurement.rss_kib,
                candidate_measurement.rss_kib,
                RSS_LIMIT,
            );
        }
    }
    if regressions.is_empty() {
        println!("Performans kapısı geçti: median ≤ %5, p95/RSS ≤ %10 gerileme.");
        Ok(())
    } else {
        Err(format!(
            "Performans gerilemesi onay gerektiriyor:\n{}",
            regressions.join("\n")
        ))
    }
}

fn read_report(path: &Path) -> Result<Report, String> {
    let bytes = std::fs::read(path)
        .map_err(|error| format!("Performans raporu okunamadı ({}): {error}", path.display()))?;
    let report: Report = serde_json::from_slice(&bytes)
        .map_err(|error| format!("Performans raporu geçersiz ({}): {error}", path.display()))?;
    if report.schema_version != 1 {
        return Err(format!(
            "Desteklenmeyen performans raporu şeması: {}",
            report.schema_version
        ));
    }
    Ok(report)
}

fn compare_metric(
    regressions: &mut Vec<String>,
    suite: &str,
    metric: &str,
    baseline: u64,
    candidate: u64,
    limit: f64,
) {
    if baseline > 0 && candidate as f64 > baseline as f64 * limit {
        let change = (candidate as f64 / baseline as f64 - 1.0) * 100.0;
        regressions.push(format!(
            "- {suite} {metric}: {baseline} -> {candidate} ({change:+.2}%)"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yuzdelik_sirali_ornekten_hesaplanir() {
        let samples = (1..=100).collect::<Vec<_>>();
        assert_eq!(percentile(&samples, 0.50), 51);
        assert_eq!(percentile(&samples, 0.95), 96);
    }

    #[test]
    fn esik_uzerindeki_metric_gerileme_sayilir() {
        let mut regressions = Vec::new();
        compare_metric(&mut regressions, "test", "median", 100, 105, MEDIAN_LIMIT);
        assert!(regressions.is_empty(), "tam sınır geçmeli");
        compare_metric(&mut regressions, "test", "median", 100, 106, MEDIAN_LIMIT);
        assert_eq!(regressions.len(), 1);
    }
}
