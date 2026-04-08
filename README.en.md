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

### 1. Interactive Mode (REPL)

Try code directly in the terminal:

```bash
huma repl
```

### 2. Run a Script (Interpreter)

Run a `.hb` file using the tree-walking interpreter (`run` or `çalıştır`):

```bash
huma run script.hb
# or in Turkish
huma çalıştır script.hb
```

### 3. Bytecode Mode (Performance)

Compile to bytecode (`build`/`derle`) and execute via the VM (`exec`/`yürüt`):

```bash
# Compile to bytecode (.hbc)
huma build script.hb -o output.hbc
# Execute bytecode
huma exec output.hbc
```

### 4. Package Manager & Scripts

Hüma features a powerful package manager (`package` or `paket`):

```bash
# Initialize a project
huma package init
# Install all dependencies
huma package install
# Add a specific dependency
huma package add ag_istekleri
# Run a project script (like npm run)
huma run baslat
```

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
