
# Değişim Günlüğü (Changelog)

Tüm önemli değişiklikler bu dosyada takip edilecektir.

## [v0.6.0-alfa.1] - 2026-04-09

### Dil Çekirdeği ve Sağlamlık (Faz 1)
- **Yapısal Hata Yönetimi**: `dene...yakala` blokları eklendi. Artık sıfıra bölme gibi çalışma zamanı hataları programı çökertmeden yakalanabiliyor.
- **Yerleşik Sözlük (Sözlük) Desteği**: Süslü parantez `{}` ile tanımlanabilen yeni bir veri tipi eklendi.
- **Sözlük Metotları**: `getir` ve `ayarla` metotları ile sözlük elemanı yönetimi ve ek sistemi (`sozluk'ten "anahtar"'ı getir`) entegrasyonu sağlandı.
- **VM Hata Katmanı**: Bytecode seviyesinde `TryBlockStart` ve `TryBlockEnd` opcodes eklendi.
- **Gelişmiş JSON Desteği**: Sözlük tipi doğrudan `to_json` ve `from_json` işlemlerine dahil edildi.

## [v0.5.2] - 2026-04-08

### Güvenlik, Stabilite ve Paket Yönetimi
- **Paket Yöneticisi Alt Dizin Desteği**: GitHub reposu içindeki alt klasörlerden paket yükleme özelliği eklendi (Mono-repo desteği).
- **Dahili Registry Düzeltildi**: Gömülü modüllerin indirme yolları ve repository isimleri güncellenerek erişilebilirlik sağlandı.
- **Güvenlik Sertleştirmesi**: Path traversal, atomic-write ve yerleşik Rust kodu güvenlik uyarıları eklendi.
- **Web Sitesi Derleme Hataları Giderildi**: Next.js 16.2.1 geçişindeki proxy ve sözlük (i18n) uyumsuzlukları düzeltildi.
- **Kütüphane Standardizasyonu**: Standart kütüphane dosyaları v1.1.0 sürümüne ve yeni header formatına güncellendi.
- **Kod Kalitesi**: huma-core üzerindeki derleyici uyarıları temizlendi ve performans iyileştirmeleri yapıldı.
- **SHA-256 Bütünlük Doğrulaması**: huma.lock dosyasında paket hash kontrolü eklendi.

## [v0.5.1] - 2026-04-03

### Doğal Mimari ve SQLite
- **Native Paket Mimarisi**: Hüma projelerine doğrudan Rust Crates (Cargo) dahil etme desteği.
- **Resmi SQLite Desteği**: rusqlite tabanlı, yüksek performanslı yerel veritabanı modülü (huma_sqlite).
- **Tam Bağımsız Derleme**: 'huma gen' komutu artık tüm yerel bağımlılıkları içeren tam Cargo projeleri üretiyor.
- **Veritabanı Nesne Eşleme**: SQL sorgu sonuçlarını otomatik olarak Hüma nesnelerine dönüştürme desteği.
- **Hüma CLI İyileştirmeleri**: Bağımsız proje üretimi sonrası otomatik Cargo rehberliği.

## [v0.5.0] - 2026-04-02

### Eklendi
- **Modern Ağ Desteği**: `ag_istekleri` kütüphanesi ile Rust tabanlı `ureq` motoru kullanılarak HTTP istekleri yapabilme.
- **Hüma Sunucu Framework**: `huma_sunucu` modülü ile dinamik rota (routing) yönetimi ve yanıt başlıkları desteği.
- **İkili Veri Desteği**: `Bayt` (Byte) veri tipi ile görsel ve dosya gibi ikili verilerin işlenebilmesi.
- **Çift Dilli CLI**: Komut satırı arayüzünde hem Türkçe hem İngilizce komut ve takma ad (alias) desteği.
- **Gelişmiş Paket Yöneticisi**:
  - `huma.json` üzerinden özel betik (script) çalıştırma desteği.
  - Paket bütünlüğü doğrulaması (integrity hashing).
  - Proje başlangıcında otomatik `.gitignore` ve Git başlatma.
  - Bağımlılık takibi ve toplu kurulum özellikleri.

## [0.4.0] - 2026-03-31

### Eklendi
- **Yerleşik GUI Desteği**: `eframe` entegrasyonu ile pencere, düğme, giriş alanları ve gelişmiş düzen (layout) yönetimi.
- **Paket Yöneticisi (Sürüm 1)**: GitHub üzerinden paket yükleme, proje iskeleti oluşturma ve merkezi kayıt listesi.
- **Anonim Fonksiyonlar**: Lambda ifadeleri ve isimsiz fonksiyon tanımlama desteği.
- **Kilit Dosyası Sistemi**: `huma.lock` ile sürüm uyumluluğu ve tutarlılık kontrolü.

## [0.3.1] - 2026-03-24

### Eklendi
- **Çalışma Alanı (Workspace) Yapısı**: Proje `huma-core`, `huma-compiler` ve `huma-cli` olarak modüler sandıklara ayrıldı.
- **VS Code Eklentisi**: Hüma dili için resmi sözdizimi vurgulama ve dil desteği eklentisi başlatıldı.
- **Yenilenen Web Arayüzü**: Vite ve React tabanlı modern dökümantasyon sitesi ve IDE.
- **Uluslararasılaştırma (i18n)**: Dökümantasyon ve site için çoklu dil desteği.
- **Birleşik CLI**: `--ide` parametresi ile tüm araçların tek bir binary altında toplanması.

## [0.2.0] - 2026-03-20

### Eklendi
- **Tauri IDE**: Yerleşik dosya erişimi ve kaydetme özelliklerine sahip taşınabilir masaüstü IDE.
- **NLP Kütüphanesi**: Metin işleme, analiz ve doğal dil işleme araçları (`nlp.hb`).
- **Sözdizimi İyileştirmeleri**: Değişken atama, kontrol akışı ve fonksiyon tanımlamalarında daha esnek yapı.

## [0.1.0] - 2026-03-16

### Eklendi
- **Yeni Rust Temelli Mimari**: Dil tamamen Rust ile yeniden yazıldı.
- **Bytecode Sanal Makine (VM)**: Yüksek performanslı bytecode yürütme motoru.
- **İkili Dosya Oluşturma**: Hüma kodlarını bağımsız çalıştırılabilir dosyalara derleyebilme özelliği.
- **REPL**: Etkileşimli kod deneme ortamı.
- **Modül Sistemi**: `yükle` komutu ile kütüphane desteği.

---

# Changelog (English)

All notable changes to this project will be documented in this file.

## [v0.6.0-alfa.1] - 2026-04-09

### Language Core & Robustness (Phase 1)
- **Structural Error Handling**: Added `dene...yakala` (try...catch) blocks. Runtime errors like division by zero can now be caught without crashing.
- **Native Dictionary (Sözlük) Support**: Introduced a new data type defined via curly braces `{}`.
- **Dictionary Methods**: Added `getir` (get) and `ayarla` (set) methods with full suffix system integration.
- **VM Error Layer**: Implemented `TryBlockStart` and `TryBlockEnd` opcodes at the bytecode level.
- **Enhanced JSON Support**: Native dictionary type is now fully supported in `to_json` and `from_json` operations.

## [v0.5.2] - 2026-04-08

### Security, Stability and Package Management
- **Package Manager Subdirectory Support**: Ability to install packages from subfolders within GitHub repositories (Mono-repo support).
- **Internal Registry Fixed**: Download paths and repository names for embedded modules were updated to ensure accessibility.
- **Security Hardening**: Path traversal, atomic-write, and built-in Rust code security warnings added.
- **Website Build Errors Resolved**: Proxy and dictionary (i18n) incompatibilities during the Next.js 16.2.1 migration were fixed.
- **Library Standardization**: Standard library files updated to v1.1.0 with a new header format.
- **Code Quality**: Compiler warnings on huma-core were cleaned, and performance improvements were made.
- **SHA-256 Integrity Verification**: Package hash control added to huma.lock.

## [v0.5.1] - 2026-04-03

### Native Architecture and SQLite
- **Native Package Architecture**: Direct integration of Rust Crates (Cargo) into Hüma projects.
- **Official SQLite Support**: High-performance local database module based on rusqlite (huma_sqlite).
- **Full Standalone Compilation**: 'huma gen' now produces full Cargo projects containing all native dependencies.
- **Database Object Mapping**: Automatic conversion of SQL query results to Hüma objects.
- **Hüma CLI Improvements**: Automatic Cargo guidance after standalone project generation.

## [v0.5.0] - 2026-04-02

### Added
- **Modern Networking**: HTTP request capabilities using the Rust-based `ureq` engine via the `ag_istekleri` library.
- **Huma Server Framework**: Dynamic routing and response header support via the `huma_sunucu` module.
- **Binary Data Support**: `Bayt` (Byte) data type for handling images and binary files.
- **Bilingual CLI**: Full support for both Turkish and English commands and aliases in the CLI.
- **Advanced Package Manager**:
  - Custom script execution support via `huma.json`.
  - Package integrity verification (hashing).
  - Automated `.gitignore` and Git initialization during project creation.
  - Dependency tracking and bulk installation features.

## [0.4.0] - 2026-03-31

### Added
- **Native GUI Support**: Windowing, buttons, input fields, and advanced layout management via `eframe` integration.
- **Package Manager (v1)**: Package installation from GitHub, project scaffolding, and registry listing.
- **Anonymous Functions**: Support for lambda expressions and anonymous function definitions.
- **Lock File System**: Version compatibility and consistency checks via `huma.lock`.

## [0.3.1] - 2026-03-24

### Added
- **Workspace Architecture**: Project split into modular crates: `huma-core`, `huma-compiler`, and `huma-cli`.
- **VS Code Extension**: Official syntax highlighting and language support extension for VS Code.
- **Modern Web Interface**: Documentation site and IDE rebuilt with Vite and React.
- **Internationalization (i18n)**: Multi-language support for documentation and the website.
- **Unified CLI**: consolidation of all tools under a single binary with `--ide` support.

## [0.2.0] - 2026-03-20

### Added
- **Tauri IDE**: Portable desktop IDE with built-in file access and saving capabilities.
- **NLP Library**: Text processing, analysis, and natural language tools (`nlp.hb`).
- **Syntax Improvements**: More flexible structures for variable assignment, control flow, and function definitions.

## [0.1.0] - 2026-03-16

### Added
- **New Rust-Based Architecture**: Complete rewrite to Rust.
- **Bytecode Virtual Machine (VM)**: High-performance execution engine.
- **Standalone Binary Compilation**: Ability to compile scripts into native executables.
- **REPL**: Interactive shell environment.
- **Module System**: Library support via `yükle` (load) command.
