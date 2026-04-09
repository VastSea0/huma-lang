//! # Hüma CLI — Unified Entry Point
//!
//! ```text
//! hüma run   <dosya.hb>              # Interpreter modunda çalıştır
//! hüma build <dosya.hb> [çıktı.hbc]  # Bytecode'a derle
//! hüma exec  <dosya.hbc>             # Bytecode çalıştır
//! hüma gen   <dosya.hb> [isim]       # Standalone Rust dosyası üret
//! hüma repl                          # Etkileşimli REPL
//! hüma test  [yol]                   # Testleri çalıştır (tests/ veya *_test.hb)
//! hüma update                        # GitHub'dan son sürüme güncelle
//! hüma version                       # Sürüm bilgisi
//! ```

mod commands;
mod updater;
mod package_manager;

use clap::{Parser, Subcommand};
use colored::Colorize;
use tracing::error;

/// Standardised exit codes for CI/CD compatibility.
mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const FILE_NOT_FOUND: i32 = 2;
    pub const COMPILATION_ERROR: i32 = 3;
    pub const RUNTIME_ERROR: i32 = 4;
    pub const UPDATE_ERROR: i32 = 5;
}

// ─── CLI Definition ────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "huma",
    version,
    about = "Hüma Programlama Dili — Birleşik Araç Takımı (Bilingual CLI)",
    long_about = "Hüma diline ait tüm araçları tek bir komut altında birleştirir.\n\
                  Tüm komutlar hem Türkçe hem İngilizce olarak kullanılabilir.\n\
                  Örn: 'huma run' veya 'huma çalıştır', 'huma build' veya 'huma derle'."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Directly run a .hb source file (shortcut for `huma run <dosya>`)
    #[arg(value_name = "DOSYA")]
    file: Option<String>,

    /// Output diagnostics in JSON (machine-readable) format
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Bir Hüma kaynak dosyasını veya projedeki bir betiği çalıştır
    #[command(alias = "çalıştır")]
    Run {
        /// .hb dosyası veya huma.json içindeki betik adı
        target: Option<String>,
    },

    /// Bir Hüma kaynak dosyasını bytecode'a derle
    #[command(alias = "derle")]
    Build {
        /// Derlenecek .hb dosyası
        file: String,

        /// Çıktı bytecode dosyası (varsayılan: cikti.hbc)
        #[arg(short, long, default_value = "cikti.hbc")]
        output: String,

        /// Derleme sonucunu JSON olarak yazdır
        #[arg(long)]
        json: bool,
    },

    /// Derlenmiş bytecode (.hbc) dosyasını VM'de çalıştır
    #[command(alias = "yürüt")]
    Exec {
        /// Çalıştırılacak .hbc dosyası
        file: String,
    },

    /// Kaynak dosyadan bağımsız Rust kaynak kodu üret
    #[command(alias = "üret")]
    Gen {
        /// Girdi .hb dosyası
        file: String,

        /// Çıktı adı (varsayılan: program)
        #[arg(short, long, default_value = "program")]
        output: String,
    },

    /// Etkileşimli REPL (Okuma-Değerlendirme-Yazdırma Döngüsü)
    #[command(alias = "kabuk")]
    Repl,

    /// Projedeki test dosyalarını çalıştırır (birim_test.hb çıktısını raporlar)
    #[command(alias = "test")]
    Test {
        /// İsteğe bağlı: tek bir .hb dosyası veya bir klasör yolu
        target: Option<String>,
    },

    /// Masaüstü IDE (Geliştirme Ortamı) uygulamasını başlat
    #[command(alias = "arayüz")]
    Ide,

    /// Hüma araç takımını GitHub'dan en son sürüme güncelle
    #[command(alias = "güncelle")]
    Update {
        /// Mevcut sürümü kontrol et ancak güncelleme yapma
        #[arg(long)]
        check: bool,
    },

    /// Paket yöneticisi (kütüphane kurma, silme, güncelleme)
    #[command(alias = "package")]
    Paket {
        #[command(subcommand)]
        action: PackageAction,
    },

    /// Sürüm bilgisini göster
    #[command(alias = "sürüm")]
    Version,

    /// Proje bağımlılıklarını kurar (Kısa yol)
    #[command(alias = "install", alias = "add")]
    Kur {
        /// Paketin adı
        name: Option<String>,

        /// Native kod içeren paketleri onay sormadan kur
        #[arg(long = "güvenilir", alias = "trusted")]
        trusted: bool,
    },

    /// Yeni bir paket projesi şablonu oluşturur (Kısa yol)
    #[command(alias = "new")]
    Yeni {
        /// Paketin adı
        name: String,
    },

    /// Kurulu paketleri listeler (Kısa yol)
    #[command(alias = "liste", alias = "list")]
    Listele,

    /// Mevcut dizini ilklendirir (Kısa yol)
    #[command(alias = "init")]
    İlkle,
}

#[derive(Subcommand)]
pub enum PackageAction {
    /// Proje bağımlılıklarını kurar veya yeni bir paket ekler
    #[command(alias = "install", alias = "add")]
    Kur {
        /// Paketin adı (boş bırakılırsa tüm bağımlılıklar kurulur)
        name: Option<String>,

        /// Native kod içeren paketleri onay sormadan kur
        #[arg(long = "güvenilir", alias = "trusted")]
        trusted: bool,
    },
    /// Kurulu bir paketi siler
    #[command(alias = "remove", alias = "uninstall")]
    Sil {
        /// Paketin adı
        name: String,
    },
    /// Mevcut tüm paketleri listeler
    #[command(alias = "list")]
    Liste,
    /// Yeni bir paket projesi şablonu oluşturur (yeni klasörde)
    #[command(alias = "new", alias = "create")]
    Yeni {
        /// Paketin adı
        name: String,
    },
    /// Mevcut dizini bir Hüma projesi olarak ilklendirir
    #[command(alias = "init")]
    İlkle,
    /// Projenin yayınlanmaya hazır olup olmadığını kontrol eder
    #[command(alias = "verify")]
    Doğrula,
    /// Tüm paketleri günceller
    #[command(alias = "update")]
    Güncelle,
    /// Projedeki bir betiği çalıştırır (npm run gibi)
    #[command(alias = "çalıştır", alias = "betik")]
    Run {
        /// Betiğin adı
        name: String,
    },
}


// ─── Main ──────────────────────────────────────────────────────────────────

fn main() {
    // Initialise structured tracing.
    // Interactive usage → coloured stderr.  JSON flag → machine-readable stdout.
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let exit_code = run(cli);
    std::process::exit(exit_code);
}

fn run(cli: Cli) -> i32 {
    let result = match cli.command {
        Some(Commands::Run { target }) => {
            if let Some(t) = target {
                // Eğer bir .hb dosyası ise direkt çalıştır
                if t.ends_with(".hb") && std::path::Path::new(&t).exists() {
                    commands::run_file(&t)
                } else {
                    // Script olup olmadığını kontrol et
                    match package_manager::run_script(&t) {
                        Ok(_) => Ok(()),
                        Err(_e) => {
                            // Eğer script değilse ve dosya olarak da yoksa hata ver
                            if !std::path::Path::new(&t).exists() {
                                return 1; // Hata kodu
                            }
                            commands::run_file(&t)
                        }
                    }
                }
            } else {
                // Hiçbir şey verilmemişse:
                // 1. "start" betiği var mı bak
                // 2. huma.json'daki "giris" dosyasına bak
                if let Ok(meta) = package_manager::get_local_metadata() {
                    if let Some(ref betikler) = meta.betikler {
                        if betikler.contains_key("baslat") {
                            return match package_manager::run_script("baslat") {
                                Ok(_) => 0,
                                Err(_) => 1,
                            };
                        } else if betikler.contains_key("start") {
                            return match package_manager::run_script("start") {
                                Ok(_) => 0,
                                Err(_) => 1,
                            };
                        }
                    }
                    // Giriş dosyasını dene
                    commands::run_file(&meta.giris)
                } else {
                    println!("{} Ne bir .hb dosyası belirtildi ne de bir huma.json projesi bulundu.", "Hata:".bright_red());
                    Ok(())
                }
            }
        },
        Some(Commands::Build { file, output, json }) => commands::build_file(&file, &output, json),
        Some(Commands::Exec { file }) => commands::exec_bytecode(&file),
        Some(Commands::Gen { file, output }) => commands::generate_standalone(&file, &output),
        Some(Commands::Repl) => commands::start_repl(),
        Some(Commands::Test { target }) => commands::run_tests(target.as_deref()),
        Some(Commands::Ide) => commands::start_ide(),
        Some(Commands::Update { check }) => {
            if check {
                updater::check_for_update()
            } else {
                updater::run_self_update()
            }
        }
        Some(Commands::Paket { action }) => match action {
            PackageAction::Kur { name, trusted } => package_manager::install_package(name.as_deref(), trusted),
            PackageAction::Sil { name } => package_manager::remove_package(&name),
            PackageAction::Güncelle => package_manager::update_packages(),
            PackageAction::Yeni { name } => package_manager::create_package(&name),
            PackageAction::İlkle => package_manager::init_project(),
            PackageAction::Liste => package_manager::list_packages(),
            PackageAction::Doğrula => package_manager::verify_package(),
            PackageAction::Run { name } => package_manager::run_script(&name),
        },

        Some(Commands::Kur { name, trusted }) => package_manager::install_package(name.as_deref(), trusted),
        Some(Commands::Yeni { name }) => package_manager::create_package(&name),
        Some(Commands::Listele) => package_manager::list_packages(),
        Some(Commands::İlkle) => package_manager::init_project(),

        Some(Commands::Version) => {
            println!(
                "{} {} ({} {})",
                "Hüma".bright_cyan().bold(),
                env!("CARGO_PKG_VERSION").bright_white().bold(),
                std::env::consts::OS,
                std::env::consts::ARCH,
            );
            Ok(())
        }
        // No subcommand — check if a bare file was passed
        None => {
            if let Some(file) = cli.file {
                // Bare file passed
                if file.ends_with(".hb") && std::path::Path::new(&file).exists() {
                    commands::run_file(&file)
                } else {
                    // Try script first if it's a bare word
                    if package_manager::run_script(&file).is_ok() {
                        Ok(())
                    } else {
                        commands::run_file(&file)
                    }
                }
            } else {
                // Default: start the REPL
                commands::start_repl()
            }
        }
    };

    match result {
        Ok(()) => exit_codes::SUCCESS,
        Err(e) => {
            // Try to determine the right exit code from the error chain.
            if let Some(he) = e.downcast_ref::<huma_core::HumaError>() {
                match he {
                    huma_core::HumaError::IoError(_) => {
                        error!("{}", format!("{:#}", e).bright_red());
                        exit_codes::FILE_NOT_FOUND
                    }
                    huma_core::HumaError::CompileError(_)
                    | huma_core::HumaError::SyntaxError { .. } => {
                        error!("{}", format!("{:#}", e).bright_red());
                        exit_codes::COMPILATION_ERROR
                    }
                    huma_core::HumaError::RuntimeError(_) => {
                        error!("{}", format!("{:#}", e).bright_red());
                        exit_codes::RUNTIME_ERROR
                    }
                    _ => {
                        error!("{}", format!("{:#}", e).bright_red());
                        exit_codes::GENERAL_ERROR
                    }
                }
            } else {
                error!("{}", format!("{:#}", e).bright_red());
                exit_codes::GENERAL_ERROR
            }
        }
    }
}
