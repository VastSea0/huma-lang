//! # Hüma CLI — Unified Entry Point
//!
//! ```text
//! hüma run   <dosya.hb>              # Interpreter modunda çalıştır
//! hüma build <dosya.hb> [çıktı.hbc]  # Bytecode'a derle
//! hüma exec  <dosya.hbc>             # Bytecode çalıştır
//! hüma repl                          # Etkileşimli REPL
//! hüma test  [yol]                   # Testleri çalıştır (tests/ veya *_test.hb)
//! hüma version                       # Sürüm bilgisi
//! ```

mod commands;
mod package_manager;

use anyhow::anyhow;
use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use tracing::error;

/// Standardised exit codes for CI/CD compatibility.
#[allow(dead_code)]
pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const FILE_NOT_FOUND: i32 = 2;
    pub const COMPILATION_ERROR: i32 = 3;
    pub const RUNTIME_ERROR: i32 = 4;
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

    /// Dış dünya yeteneği ver (tekrarlanabilir; ör. --izin dosya-okuma)
    #[arg(long = "izin", global = true, value_enum)]
    capabilities: Vec<CapabilityArg>,

    /// Tüm dış dünya yeteneklerini ver (yalnızca güvenilen kod için)
    #[arg(long = "tüm-izinler", alias = "allow-all", global = true)]
    allow_all: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CapabilityArg {
    #[value(name = "dosya-okuma", alias = "file-read")]
    FileRead,
    #[value(name = "dosya-yazma", alias = "file-write")]
    FileWrite,
    #[value(name = "ağ-istemci", alias = "network-client")]
    NetworkClient,
    #[value(name = "ağ-sunucu", alias = "network-server")]
    NetworkServer,
    #[value(name = "süreç", alias = "process")]
    Process,
    #[value(name = "ffi")]
    Ffi,
    #[value(name = "veritabanı", alias = "database")]
    Database,
    #[value(name = "gui")]
    Gui,
}

impl From<CapabilityArg> for huma_core::capability::Capability {
    fn from(value: CapabilityArg) -> Self {
        match value {
            CapabilityArg::FileRead => Self::FileRead,
            CapabilityArg::FileWrite => Self::FileWrite,
            CapabilityArg::NetworkClient => Self::NetworkClient,
            CapabilityArg::NetworkServer => Self::NetworkServer,
            CapabilityArg::Process => Self::Process,
            CapabilityArg::Ffi => Self::Ffi,
            CapabilityArg::Database => Self::Database,
            CapabilityArg::Gui => Self::Gui,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Bir Hüma kaynak dosyasını veya projedeki bir betiği çalıştır
    #[command(alias = "çalıştır")]
    Run {
        /// .hb dosyası veya huma.json içindeki betik adı
        target: Option<String>,

        /// Bytecode Sanal Makinesi (VM) kullanarak çalıştır
        #[arg(long = "vm")]
        vm: bool,
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

    /// Cranelift AOT kullanarak doğrudan native makine koduna (binary) derle
    #[command(alias = "native", alias = "makine")]
    Aot {
        /// Girdi .hb dosyası
        file: String,

        /// Çıktı binary adı (varsayılan: program)
        #[arg(short, long, default_value = "program")]
        output: String,

        /// Optimizasyon seviyesi (0=yok, 1=hız, 2=hız ve boyut)
        #[arg(short = 'O', long, default_value = "2")]
        opt_level: u8,
    },

    /// Etkileşimli REPL (Okuma-Değerlendirme-Yazdırma Döngüsü)
    #[command(alias = "kabuk")]
    Repl,

    /// Projedeki test dosyalarını çalıştırır (birim_test.hb çıktısını raporlar)
    #[command(alias = "sına")]
    Test {
        /// İsteğe bağlı: tek bir .hb dosyası veya bir klasör yolu
        target: Option<String>,
    },

    /// Paket yöneticisi (yerel kütüphane kurma, silme ve doğrulama)
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

        /// Ayrılmış uyumluluk bayrağı; doğrulanmamış native ABI'yi etkinleştirmez
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
    #[command(name = "ilkle", alias = "init")]
    İlkle,
}

#[derive(Subcommand)]
pub enum PackageAction {
    /// Proje bağımlılıklarını kurar veya yeni bir paket ekler
    #[command(alias = "install", alias = "add", alias = "ekle")]
    Kur {
        /// Paketin adı (boş bırakılırsa tüm bağımlılıklar kurulur)
        name: Option<String>,

        /// Ayrılmış uyumluluk bayrağı; doğrulanmamış native ABI'yi etkinleştirmez
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
    #[command(name = "ilkle", alias = "init")]
    İlkle,
    /// Projenin yayınlanmaya hazır olup olmadığını kontrol eder
    #[command(alias = "verify")]
    Doğrula,
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
    let capability_set = if cli.allow_all {
        huma_core::capability::CapabilitySet::allow_all()
    } else {
        cli.capabilities.iter().copied().fold(
            huma_core::capability::CapabilitySet::deny_all(),
            |set, item| set.allow(item.into()),
        )
    };
    let _capability_guard = match huma_core::capability::install(capability_set) {
        Ok(guard) => guard,
        Err(message) => {
            error!("{}", message.bright_red());
            return exit_codes::GENERAL_ERROR;
        }
    };
    let result = match cli.command {
        Some(Commands::Run { target, vm }) => {
            let runner = if vm {
                commands::run_vm_file
            } else {
                commands::run_file
            };
            if let Some(t) = target {
                if std::path::Path::new(&t).is_file() {
                    runner(&t)
                } else {
                    match package_manager::has_local_script(&t) {
                        Ok(true) => package_manager::run_script(&t),
                        Ok(false) => {
                            Err(anyhow!("'{}' adlı dosya veya proje betiği bulunamadı.", t))
                        }
                        Err(error) if std::path::Path::new("huma.json").exists() => Err(error),
                        Err(_) => Err(anyhow!("'{}' adlı dosya veya proje betiği bulunamadı.", t)),
                    }
                }
            } else {
                match package_manager::get_local_metadata() {
                    Ok(meta) => {
                        if meta
                            .betikler
                            .as_ref()
                            .is_some_and(|scripts| scripts.contains_key("baslat"))
                        {
                            package_manager::run_script("baslat")
                        } else if meta
                            .betikler
                            .as_ref()
                            .is_some_and(|scripts| scripts.contains_key("start"))
                        {
                            package_manager::run_script("start")
                        } else {
                            runner(&meta.giris)
                        }
                    }
                    Err(error) => Err(error.context(
                        "Ne bir .hb dosyası belirtildi ne de geçerli bir huma.json projesi bulundu",
                    )),
                }
            }
        }
        Some(Commands::Build { file, output, json }) => commands::build_file(&file, &output, json),
        Some(Commands::Exec { file }) => commands::exec_bytecode(&file),
        Some(Commands::Aot {
            file,
            output,
            opt_level,
        }) => commands::compile_aot(&file, &output, opt_level),
        Some(Commands::Repl) => commands::start_repl(),

        Some(Commands::Test { target }) => commands::run_tests(target.as_deref()),
        Some(Commands::Paket { action }) => match action {
            PackageAction::Kur { name, trusted } => {
                package_manager::install_package(name.as_deref(), trusted)
            }
            PackageAction::Sil { name } => package_manager::remove_package(&name),
            PackageAction::Yeni { name } => package_manager::create_package(&name),
            PackageAction::İlkle => package_manager::init_project(),
            PackageAction::Liste => package_manager::list_packages(),
            PackageAction::Doğrula => package_manager::verify_package(),
            PackageAction::Run { name } => package_manager::run_script(&name),
        },

        Some(Commands::Kur { name, trusted }) => {
            package_manager::install_package(name.as_deref(), trusted)
        }
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
                if std::path::Path::new(&file).is_file() {
                    commands::run_file(&file)
                } else {
                    match package_manager::has_local_script(&file) {
                        Ok(true) => package_manager::run_script(&file),
                        Ok(false) => Err(anyhow!(
                            "'{}' adlı dosya veya proje betiği bulunamadı.",
                            file
                        )),
                        Err(error) if std::path::Path::new("huma.json").exists() => Err(error),
                        Err(_) => Err(anyhow!(
                            "'{}' adlı dosya veya proje betiği bulunamadı.",
                            file
                        )),
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
