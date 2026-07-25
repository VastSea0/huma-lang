use criterion::{black_box, criterion_group, criterion_main, Criterion};
use huma_core::compiler::Derleyici;
use huma_core::interpreter::Yorumlayici;
use huma_core::lexer::Lexer;
use huma_core::parser::Parser;
use huma_core::vm::VM;
use std::time::Duration;

const DONGU_KAYNAGI: &str = r#"
toplam = 0 olsun
i = 1'den 10000'e kadar {
    toplam = toplam + i olsun
}
"#;

fn parse(source: &str) -> Vec<huma_core::ast::Komut> {
    let mut parser = Parser::new(Lexer::new(source));
    let (program, diagnostics) = parser.parse_program_with_diagnostics();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    program
}

fn runtime_benchmarks(criterion: &mut Criterion) {
    let ast = parse(DONGU_KAYNAGI);
    let mut compiler = Derleyici::new();
    let bytecode = compiler
        .derle_kontrollu(ast.clone())
        .expect("Benchmark kaynağı bytecode'a derlenmeli");

    criterion.bench_function("parser/10k-loop-source", |bencher| {
        bencher.iter(|| black_box(parse(black_box(DONGU_KAYNAGI))))
    });
    criterion.bench_function("interpreter/10k-loop", |bencher| {
        bencher.iter(|| {
            let mut interpreter = Yorumlayici::new();
            interpreter
                .yorumla_kontrollu(black_box(ast.clone()))
                .expect("Benchmark yorumlayıcıda çalışmalı");
            black_box(interpreter)
        })
    });
    criterion.bench_function("vm/10k-loop", |bencher| {
        bencher.iter(|| {
            let mut vm = VM::new(black_box(bytecode.clone()));
            vm.run_checked().expect("Benchmark VM'de çalışmalı");
            black_box(vm)
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3))
        .sample_size(30);
    targets = runtime_benchmarks
}
criterion_main!(benches);
