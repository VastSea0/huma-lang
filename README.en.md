# Hüma Programming Language

Hüma is an experimental general-purpose programming language implemented in
Rust and built around Turkish keywords. Version 0.6.0 establishes its first
verified core boundary: interpreter/VM correctness, structured errors, a
defined Turkish surface grammar, least-privilege external access, and a base
for libraries in different domains.

[Türkçe README](README.md)

## Verified status

| Component | Status | Verified scope |
|---|---|---|
| Interpreter | Canonical execution path | Functions, recursion, classes, lists/maps, modules, loops, `dene/yakala`, bundled libraries |
| Bytecode VM | Verified subset | Independent frames/closures, functions, collections, and control flow; unsupported AST is rejected |
| Cranelift AOT | Experimental numeric subset | Numeric expressions and supported control flow; strings, modules, and classes fail explicitly |
| LSP | Basic tooling | Parser diagnostics, completion, hover, and go-to-definition |
| AI/NLP | Working CPU prototype | Dense layers, backpropagation, Adam, gradient clipping, TF-IDF, and embeddings |

Here, “verified” means that covered behavior is regression-tested and errors
are not silently converted into plausible output. Hüma does not yet provide a
static type system, a complete native backend, an operating-system sandbox, or
production-scale performance guarantees.

## Build and verify

The Rust workspace requires stable Rust 1.92 or newer. The website acceptance
gate uses Node.js 22.

```bash
git clone https://github.com/VastSea0/huma-lang.git
cd huma-lang
cargo build --release
./target/release/huma --version

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo audit
cargo run -p huma-cli -- test tests
cargo run -p huma-cli -- test examples
cd www/site && npm ci && npm audit && npm run lint && npm run build
```

## Example

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
huma run examples/fibonacci.hb
huma run examples/fibonacci.hb --vm
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

## AI example

`examples/nlp_siniflandirma.hb` trains a dense network on TF-IDF features with real backpropagation and Adam updates:

```bash
huma run examples/nlp_siniflandirma.hb
```

The current runtime targets learning experiments and small CPU workloads. It does not yet include GPU devices, mixed precision, distributed execution, or an industrial data pipeline.

## Documentation

- [Language Specification](docs/DIL_TANIMI.md)
- [Canonical EBNF](docs/DIL_GRAMERI.ebnf)
- [Bytecode Container Specification](docs/BYTECODE_BICIMI.md)
- [Libraries](KUTUPHANELER.md)
- [Package Security](docs/PAKET_GUVENLIGI.md)
- [Performance and Memory Measurement](docs/PERFORMANS.md)
- [Cross-Language Benchmark](docs/KARSILASTIRMALI_BENCHMARK.md)
- [Status and Roadmap](docs/DURUM_VE_YOL_HARITASI.md)
- [Changelog](CHANGELOG.md)

## License

MIT
