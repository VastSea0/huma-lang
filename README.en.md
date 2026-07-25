# Hüma Programming Language

Hüma is an experimental programming language implemented in Rust and built around Turkish keywords. Version 0.6.0 establishes its first stable core boundary: interpreter correctness, structured errors, consistent syntax, and working CPU-based AI/NLP examples.

[Türkçe README](README.md)

## Verified status

| Component | Status | Verified scope |
|---|---|---|
| Interpreter | Canonical execution path | Functions, recursion, classes, lists/maps, modules, loops, `dene/yakala`, bundled libraries |
| Bytecode VM | Stable subset | Numbers, strings, lists, control flow, loops, and functions; unsupported AST is rejected |
| Cranelift AOT | Experimental numeric subset | Numeric expressions and supported control flow; strings, modules, and classes fail explicitly |
| LSP | Basic tooling | Parser diagnostics, completion, hover, and go-to-definition |
| AI/NLP | Working CPU prototype | Dense layers, backpropagation, Adam, gradient clipping, TF-IDF, and embeddings |

Here, “stable” means that supported behavior is regression-tested and errors are not silently converted into plausible output. Hüma does not yet provide a proven static type system, a complete native backend, a GPU runtime, or production-scale performance guarantees.

## Build and verify

```bash
git clone https://github.com/VastSea0/huma-lang.git
cd huma-lang
cargo build --release
./target/release/huma --version

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p huma-cli -- test tests
cargo run -p huma-cli -- test examples
cd www/site && npm ci && npm run lint && npm run build
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

Hüma uses Turkish control words and accepts a defined set of case-suffix forms after an apostrophe. Unknown suffixes are syntax errors. This is a deterministic programming-language grammar inspired by Turkish; it is not a general natural-language parser or a complete Turkish morphology engine. See the Turkish [Language Specification](docs/DIL_TANIMI.md) for the normative rules.

## AI example

`examples/nlp_siniflandirma.hb` trains a dense network on TF-IDF features with real backpropagation and Adam updates:

```bash
huma run examples/nlp_siniflandirma.hb
```

The current runtime targets learning experiments and small CPU workloads. It does not yet include GPU devices, mixed precision, distributed execution, or an industrial data pipeline.

## Documentation

- [Language Specification](docs/DIL_TANIMI.md)
- [Bytecode Container Specification](docs/BYTECODE_BICIMI.md)
- [Libraries](KUTUPHANELER.md)
- [Status and Roadmap](docs/DURUM_VE_YOL_HARITASI.md)
- [Changelog](CHANGELOG.md)

## License

MIT
