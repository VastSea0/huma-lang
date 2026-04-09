# Hüma Programming Language

Hüma is a high-performance, safe, and intuitive programming language that combines modern software principles with a natural Turkish syntax. Developed in Rust, it features a hybrid architecture supporting both tree-walking interpretation and a bytecode-based Virtual Machine (VM).

[**Türkçe README için tıklayın.**](README.md)

---

## 🚀 Key Features

- **Natural Language Syntax:** Write code as you think in your native language (using terms like `olsun` for definition, `ise / yoksa` for conditionals, and `olduğu sürece` for loops).
- **Flexible Suffix System:** Turkish grammatical suffixes like `x'i yazdır` or `liste'ye ekle` are automatically handled for a most natural writing experience.
- **Hybrid Execution:** Choose between direct interpretation for development and bytecode execution for higher performance.
- **Standalone Compilation:** Compile your code into native binary executables with zero external dependencies.
- **Rich Standard Library:** Built-in support for mathematics, NLP, terminal coloring, time management, advanced list manipulation, and unit testing.
- **Secure Package Management:** SHA-256 integrity verification, path traversal protection, and mono-repo (sub-directory) support (v0.5.2).
- **Bilingual CLI (Turkish & English):** All commands work in both languages (e.g., `run` or `çalıştır`, `build` or `derle`, `package` or `paket`).
- **Modern IDE Support:** Full-featured IDEs available in both Native (GTK) and Web/Tauri flavors.

---

## 🛠️ Installation & Build

You need [Rust](https://www.rust-lang.org/) installed on your system.

```bash
git clone https://github.com/VastSea0/huma-lang.git
cd huma-lang
cargo build --release
```

The compiled binary will be located at `target/release/huma`.

---

## 💻 Usage Guide

## 💻 CLI Command Reference

Hüma features a **Bilingual CLI**, meaning you can use the English commands (e.g., `run`, `build`, `update`) or their Turkish equivalents (e.g., `çalıştır`, `derle`, `güncelle`) interchangeably.

### 1. Execution & Interactive Modes

- `huma run <target>` (Alias: `çalıştır`)
    - **Description:** Run a `.hb` source file or a script defined in `huma.json`. 
    - **Smart Logic:** If no target is provided, it looks for a script named `start` or `baslat`. If neither is found, it runs the entry file specified in `huma.json`.
- `huma repl` (Alias: `kabuk`)
    - **Description:** Starts the interactive Read-Eval-Print Loop for quick prototyping and testing.
- `huma ide` (Alias: `arayüz`)
    - **Description:** Launches the desktop Integrated Development Environment.

### 2. Compilation & Bytecode

- `huma build <file>` (Alias: `derle`)
    - **Description:** Compiles a source file into `.hbc` bytecode. Use `-o` to specify the output filename.
- `huma exec <file>` (Alias: `yürüt`)
    - **Description:** Executes a pre-compiled `.hbc` bytecode file using the Hüma VM.
- `huma gen <file>` (Alias: `üret`)
    - **Description:** Generates standalone Rust source code from a Hüma file for native performance.

### 3. Package Management (`package` or `paket`)

- `huma init` (Alias: `paket ilkle`)
    - **Description:** Initializes a new Hüma project in the current directory.
- `huma new <name>` (Alias: `paket yeni`)
    - **Description:** Scaffolds a new Hüma project in a new directory named `<name>`.
- `huma install [name]` (Alias: `paket kur`, `add`)
    - **Description:** Installs dependencies from `huma.json`. If `[name]` is provided, it adds that specific package. Use `--trusted` to bypass security warnings for native modules.
- `huma remove <name>` (Alias: `paket sil`)
    - **Description:** Uninstalls a package and updates `huma.lock`.
- `huma list` (Alias: `paket liste`)
    - **Description:** Lists all installed dependencies and their versions.
- `huma package update` (Alias: `paket güncelle`)
    - **Description:** Checks and updates project dependencies to their latest versions.
- `huma package verify` (Alias: `paket doğrula`)
    - **Description:** Performs a pre-distribution check on the package structure and metadata.

### 4. Maintenance & Information

- `huma update` (Alias: `güncelle`)
    - **Description:** Updates the Hüma CLI binary to the latest version from GitHub. Use `--check` to check for updates without installing.
- `huma version` (Alias: `sürüm`)
    - **Description:** Displays the current version of the Hüma binary.

---

## 📖 Language Reference

### Basic Syntax

```huma
// Variable Definition and Assignment
x = 10 olsun
name = "Hüma" olsun

// Arithmetic
total = x + 5 * 2 olsun

// Conditionals (ise / yoksa)
x > 5 ise {
    "Greater than 5"'ı yazdır;
} yoksa {
    "Less than or equal to 5"'ı yazdır;
}

// Loops (olduğu sürece)
i = 0 olsun
i < 5 olduğu sürece {
    "Index: " + i'yi yazdır;
    i = i + 1 olsun
}
```

### Functions & Classes

```huma
yükle "matematik.hb";

greet fonksiyon olsun user alsın {
    "Hello, " + user + "!"'ı döndür
}

msg = greet("World") olsun
msg'yı yazdır;

calculator sınıf olsun {
    add fonksiyon olsun a, b alsın {
        a + b'yi döndür
    }
}
calc = calculator() olsun
calc.add(10, 20)'yi yazdır;
```

### Lists

```huma
fruits = ["Apple", "Pear"] olsun
fruits'e ["Banana"]'yı ekle;
fruits'ten [0]'ı çıkar; // delete by index

i = 0 olsun
i < fruits'ın uzunluğu olduğu sürece {
    fruits[i]'yi yazdır;
    i = i + 1 olsun
}
```

### 🇹🇷 Natural Language & Suffix System

The most distinctive feature of Hüma is its built-in support for Turkish grammar and agglutination. Grammatical suffixes appended via an apostrophe (`'`) are cleanly and automatically stripped by the lexer at compile time. This drastically improves code readability and natural flow while ensuring absolutely zero runtime cost.

**Supported Suffix Patterns:**
- **Accusative Case (Direct Object / -i hali):** `'i`, `'ı`, `'u`, `'ü`, `'yi`, `'yı`, `'yu`, `'yü`
  *Usage:* Outputting or returning values. Example: `hata'yı yazdır` (print error), `isim'i döndür` (return name)
- **Dative Case (Direction / -e hali):** `'e`, `'a`, `'ye`, `'ya`
  *Usage:* Adding to collections or assignments. Example: `liste'ye ekle` (append to list)
- **Ablative Case (Origin / -den hali):** `'den`, `'dan`, `'ten`, `'tan`
  *Usage:* Removing from collections or reading from sources. Example: `liste'den çıkar` (remove from list)
- **Genitive Case (Possession / -nin hali):** `'nin`, `'nın`, `'nun`, `'nün`, `'ın`, `'in`, `'un`, `'ün`
  *Usage:* Property access or lengths. Example: `liste'nin uzunluğu` (length of list), `ayarlar'ın sürümü` (version of settings)

```huma
// Writing fluid and highly readable code with suffixes:
ayarlar = { "tema": "koyu" } olsun

// Traditional: yazdır ayarlar.tema
// Hüma style:
yazdır ayarlar'ın tema'sı;
```

---

## 📚 Standard Libraries (`lib/`)

- **`matematik.hb`**: `karesi(n)`, `kuvvet(a, b)`, `faktöriyel(n)`.
- **`renkler.hb`**: `başarı_yaz(m)`, `hata_yaz(m)`, terminal colors.
- **`dizgi.hb`**: String tools like `kırp` (trim), `içeriyor_mu` (contains).
- **`liste.hb`**: `eşle(d, f)`, `filtrele(d, f)`, `indirge(d, f, b)`.
- **`birim_test.hb`**: Native unit testing framework.

---

## 📜 License

This project is licensed under the MIT License.
