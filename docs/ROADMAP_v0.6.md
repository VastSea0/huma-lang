# 🕊️ Hüma Roadmap: Path to v0.6.0

## Vision: "The Professionalization Release"
While v0.5.2 solidified the core runtime and package management, **v0.6.0** aims to transform Hüma into a production-ready ecosystem. The focus shifts from "adding features" to "stabilizing the developer journey."

---

## ✅ Phase 1: Language Core & Robustness (Completed: 2026-04-09)
The foundation is now stabilized.

### 1.1 Structural Error Handling (Try/Catch)
Currently, Hüma lacks a native way to recover from runtime errors.
- **Goal:** Introduce `dene...yakala` (try...catch) blocks.
- **Detail:** Implement a stack-aware error propagation system in the VM.
- **Syntax Example:**
  ```huma
  dene {
      dosya = "veri.txt"'i aç;
  } yakala hata {
      "Hata oluştu: " + hata'yı yazdır;
  }
  ```

### 1.2 Native Map/Dictionary Support
Move away from simulating dictionaries via Objects.
- **Goal:** Dedicated `Sözlük` (Map) type with hashing.
- **Detail:** Add a new `Deger::Sozluk(HashMap<String, Deger>)` variant to the core.
- **Syntax:** `ogrenci = { "ad": "Hüma", "not": 100 } olsun`.

---

## 🏗 Phase 2: Modern Developer Experience (DX)
A language is only as good as its tools.

### 2.1 Hüma LSP (Language Server Protocol)
The most critical missing piece for professional adoption.
- **Goal:** Real-time feedback in VS Code and the Hüma IDE.
- **Features:** 
    - **Go-to-Definition:** Track function declarations across multiple `.hb` files.
    - **Auto-completion:** Predict standard library functions and local variables.
    - **Linting:** Highlight syntax errors *as you type* instead of at runtime.

### 2.2 Built-in Test Runner (`huma test`)
Solidify the testing culture.
- **Goal:** Elevate `birim_test.hb` to a first-class CLI command.
- **Action:** Add a `huma test` command that scans for `*_test.hb` files and generates a formatted report.

---

## 🌍 Phase 3: Ecosystem & Distribution
Scaling how code is shared and documented.

### 3.1 Central Registry (Huma Hub)
Transition from pure GitHub dependency to a structured registry.
- **Goal:** `huma.registry.io` (concept).
- **Detail:** A central index mapping package names to specific Git tags/hashes with versioning logic (SemVer).

### 3.2 Automated Doc Generation (`huma doc`)
- **Goal:** Generate beautiful static HTML documentation from source code comments.
- **Logic:** Parse `///` comments to extract parameters, return types, and examples.

---

## ⚡ Phase 4: High Performance & Scaling
Hüma for the web and high-load systems.

### 4.1 Asynchronous Programming (`async/await`)
- **Goal:** Non-blocking I/O for `huma_sunucu`.
- **Detail:** Implement a "Yaprak" (Leaf/Task) executor in the VM to handle concurrent operations without blocking the main thread.
- **Syntax:** `veri = bekle ag_istekleri.getir(url)`.

### 4.2 JIT (Just-In-Time) Research
- **Goal:** Identify "Hot Loops" and compile them into machine code on the fly.
- **Focus:** Performance parity with high-level languages like Python or JS (V8).

---

## 🎨 Phase 5: Public Presence & Training
Lowering the barrier to entry.

### 5.1 Interactive Playground
- **Goal:** A "Try Hüma" button on the official website.
- **Tech:** Compile the Hüma VM to **WebAssembly (WASM)** to run code entirely in the browser.

### 5.2 Official API Reference
- **Goal:** A comprehensive, searchable database of every standard library function with interactive code examples.

---

## 📅 Target Timeline
- **v0.5.5 (Internal Beta):** Native Maps and Hata Yönetimi (Error Handling).
- **v0.5.8 (Tooling Alpha):** First version of Hüma LSP.
- **v0.6.0 (Stable):** Unified CLI with `test`, `doc`, and Async support.
