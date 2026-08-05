# Hüma Programming Language

Hüma is being engineered as a modern general-purpose language foundation that
preserves Turkish grammar. Its current priority is not AI or any other domain
library; it is the correct, fast, secure, and versioned language/runtime base
needed to support a broad library ecosystem later.

The AI, NLP, GUI, networking, and data libraries currently in this repository
are experimental verification material. They are not stable public APIs and do
not constrain the core architecture. The canonical direction and release gates
are defined in the [Engineering Constitution](docs/MUHENDISLIK_ANAYASASI.md)
(Turkish).

[Türkçe README](README.md)

## Verified status

| Component | Status | Verified scope |
|---|---|---|
| Interpreter | Canonical execution path | Functions, recursion, classes, lists/maps, modules, loops, `dene/yakala`, bundled libraries |
| Bytecode VM | Verified subset | Independent frames/closures, functions, collections, and control flow; unsupported AST is rejected |
| Cranelift AOT | Experimental numeric subset | Numeric expressions and supported control flow; strings, modules, and classes fail explicitly |
| LSP | Basic tooling | Parser diagnostics, completion, hover, and go-to-definition |
| HMI | Versioned out-of-process boundary | Signature/effect/error catalog, API compatibility checks, framed limits, and timeout termination |
| Heap/isolate | Generational cycle collector | Stable `Gc` handles, a young generation with a write barrier, major cycle collection, and unshared isolate heaps |
| Domain libraries | Experimental / unstable | Not part of the stable core contract; may be rewritten or removed |

Here, “verified” means that covered behavior is regression-tested and errors
are not silently converted into plausible output. Hüma does not yet provide a
static type system, a complete native backend, an operating-system sandbox, or
production-scale performance guarantees.

## Build and verify

The Rust workspace is pinned to Rust 1.94.1 through `rust-toolchain.toml`.

```bash
git clone https://github.com/VastSea0/huma-lang.git
cd huma-lang
cargo build --release --locked
./target/release/huma --version

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo audit
cargo run --locked -p huma-cli -- test tests
```

## Example

Every Hüma program is created with the package manager and executed through a
script in `huma.json`. Direct execution of loose `.hb` files is disabled:

```bash
huma package new fibonacci_app
cd fibonacci_app
```

Edit the generated entry file with the following program:

```huma
fibonacci fonksiyon olsun n alsın {
    n <= 1 ise { n'i döndür }
    (fibonacci(n - 1) + fibonacci(n - 2))'yi döndür
}

i = 0'dan 9'a kadar {
    "fib(" + i + ") = " + fibonacci(i)'yi yazdır
}
```

```bash
huma package run start
```

The interpreter is the default full-language backend. VM and AOT compilation reject unsupported constructs instead of producing placeholder values.

## Turkish-inspired grammar

Hüma uses Turkish control words and accepts a defined set of case-suffix forms
after an apostrophe. Unknown suffixes are syntax errors. For names whose
pronunciation can be derived from spelling, the lexer also checks vowel harmony,
buffer consonants, and voiceless-consonant assimilation. See the Turkish
[Language Specification](docs/DIL_TANIMI.md) and canonical
[EBNF](docs/DIL_GRAMERI.ebnf).

## Security boundary

External capabilities are denied by default and granted individually, for
example `--izin dosya-okuma` or `--izin ağ-istemci`. File writing, network
servers, processes, FFI, databases, and GUI access are separate capabilities.
`--tüm-izinler` is only for trusted code and is not an operating-system sandbox.
Native libraries use the out-of-process [HMI](docs/HMI.md) by default. The
narrow in-process FFI is available only with the additional
`--güvenilir-süreç-içi-ffi` opt-in.

## Library policy

Domain libraries, including AI, will become stable only after the core/runtime
contracts are complete. GUI, networking, SQL, tensors, and other domains must
live in separate capability-aware adapters and packages, not in the language
core. The `huma-stdlib-gui` adapter provides a Dear ImGui native window managed
by the CLI when the explicit `gui` capability is enabled. The previous website
has been removed ahead of a ground-up redesign.

## Documentation

- [Language Specification](docs/DIL_TANIMI.md)
- [Canonical EBNF](docs/DIL_GRAMERI.ebnf)
- [Bytecode Container Specification](docs/BYTECODE_BICIMI.md)
- [Libraries](KUTUPHANELER.md)
- [Package Security](docs/PAKET_GUVENLIGI.md)
- [HMI v1](docs/HMI.md)
- [API and Compatibility Policy](docs/API_STABILITE_POLITIKASI.md) (Turkish)
- [Performance and Memory Measurement](docs/PERFORMANS.md)
- [Cross-Language Benchmark](docs/KARSILASTIRMALI_BENCHMARK.md)
- [Status and Roadmap](docs/DURUM_VE_YOL_HARITASI.md)
- [Engineering Constitution](docs/MUHENDISLIK_ANAYASASI.md) (Turkish)
- [Changelog](CHANGELOG.md)

## License

MIT
