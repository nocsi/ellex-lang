//! Benchmarks for the Ellex transpiler

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ellex_core::values::{EllexValue, Statement};
use ellex_transpiler::{EllexTranspiler, TranspilerOptions, Target, EsVersion};

fn bench_simple_transpilation(c: &mut Criterion) {
    let ast = vec![
        Statement::Tell(EllexValue::String("Hello, world!".to_string())),
        Statement::Tell(EllexValue::Number(42.0)),
        Statement::Ask("name".to_string(), None),
        Statement::Repeat(5, vec![
            Statement::Tell(EllexValue::String("Loop iteration".to_string())),
        ]),
    ];
    
    let options = TranspilerOptions::default();
    let transpiler = EllexTranspiler::with_options(options);
    
    c.bench_function("simple_transpilation", |b| {
        b.iter(|| {
            let result = transpiler.transpile(black_box(&ast));
            black_box(result)
        })
    });
}

fn bench_optimization_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_levels");
    
    let ast = vec![
        Statement::Repeat(3, vec![
            Statement::Tell(EllexValue::String("Optimizable".to_string())),
        ]),
        Statement::When(
            "flag".to_string(),
            EllexValue::String("true".to_string()),
            vec![Statement::Tell(EllexValue::Number(1.0))],
            Some(vec![Statement::Tell(EllexValue::Number(0.0))]),
        ),
    ];
    
    for optimize in [false, true] {
        let options = TranspilerOptions {
            target: Target::JavaScript {
                async_support: true,
                es_version: EsVersion::Es2020,
                optimize,
            },
            optimize,
            ..Default::default()
        };
        
        let transpiler = EllexTranspiler::with_options(options);
        
        group.bench_with_input(
            BenchmarkId::new("js_optimization", optimize),
            &optimize,
            |b, _| {
                b.iter(|| {
                    let result = transpiler.transpile(black_box(&ast));
                    black_box(result)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_different_targets(c: &mut Criterion) {
    let mut group = c.benchmark_group("target_languages");
    
    let ast = vec![
        Statement::Tell(EllexValue::String("Multi-target test".to_string())),
        Statement::Repeat(2, vec![
            Statement::Tell(EllexValue::Number(123.0)),
        ]),
    ];
    
    let targets = vec![
        ("javascript", Target::JavaScript {
            async_support: true,
            es_version: EsVersion::Es2020,
            optimize: false,
        }),
        ("typescript", Target::TypeScript {
            declarations: true,
            strict: true,
        }),
        ("webassembly", Target::WebAssembly {
            simd: false,
            threads: false,
            opt_level: 1,
        }),
    ];
    
    for (name, target) in targets {
        let options = TranspilerOptions {
            target,
            ..Default::default()
        };
        
        let transpiler = EllexTranspiler::with_options(options);
        
        group.bench_function(name, |b| {
            b.iter(|| {
                let result = transpiler.transpile(black_box(&ast));
                black_box(result)
            })
        });
    }
    
    group.finish();
}

fn bench_js_parsing(c: &mut Criterion) {
    let js_code = r#"
        console.log("Hello, world!");
        
        for (let i = 0; i < 10; i++) {
            console.log("Iteration " + i);
        }
        
        if (flag === true) {
            console.log("Flag is set");
        } else {
            console.log("Flag is not set");
        }
        
        function greet(name) {
            console.log("Hello " + name);
        }
        
        greet("World");
    "#;
    
    c.bench_function("js_parsing", |b| {
        b.iter(|| {
            let result = EllexTranspiler::from_js(black_box(js_code));
            black_box(result)
        })
    });
}

fn bench_large_ast(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_ast");
    
    let sizes = vec![100, 500, 1000, 5000];
    
    for size in sizes {
        let mut ast = Vec::new();
        for i in 0..size {
            ast.push(Statement::Tell(EllexValue::String(format!("Statement {}", i))));
        }
        
        let options = TranspilerOptions::default();
        let transpiler = EllexTranspiler::with_options(options);
        
        group.bench_with_input(
            BenchmarkId::new("statements", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let result = transpiler.transpile(black_box(&ast));
                    black_box(result)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_complex_nesting(c: &mut Criterion) {
    // Create deeply nested structure
    let mut ast = vec![
        Statement::When(
            "level1".to_string(),
            EllexValue::String("true".to_string()),
            vec![
                Statement::Repeat(3, vec![
                    Statement::When(
                        "level2".to_string(),
                        EllexValue::String("active".to_string()),
                        vec![
                            Statement::Repeat(2, vec![
                                Statement::Tell(EllexValue::String("Deep nesting".to_string())),
                                Statement::Ask("input".to_string(), None),
                            ]),
                        ],
                        Some(vec![
                            Statement::Tell(EllexValue::String("Level2 inactive".to_string())),
                        ]),
                    ),
                ]),
            ],
            Some(vec![
                Statement::Tell(EllexValue::String("Level1 false".to_string())),
            ]),
        ),
    ];
    
    let options = TranspilerOptions::default();
    let transpiler = EllexTranspiler::with_options(options);
    
    c.bench_function("complex_nesting", |b| {
        b.iter(|| {
            let result = transpiler.transpile(black_box(&ast));
            black_box(result)
        })
    });
}

fn bench_string_interpolation(c: &mut Criterion) {
    let ast = vec![
        Statement::Tell(EllexValue::String("Hello {name}, you are {age} years old and live in {city}!".to_string())),
        Statement::Tell(EllexValue::String("Your score is {score} out of {max_score}".to_string())),
        Statement::Tell(EllexValue::String("Welcome to {app_name} version {version}".to_string())),
    ];
    
    let options = TranspilerOptions::default();
    let transpiler = EllexTranspiler::with_options(options);
    
    c.bench_function("string_interpolation", |b| {
        b.iter(|| {
            let result = transpiler.transpile(black_box(&ast));
            black_box(result)
        })
    });
}

fn bench_wasm_compilation(c: &mut Criterion) {
    let ast = vec![
        Statement::Tell(EllexValue::String("WASM test".to_string())),
        Statement::Tell(EllexValue::Number(42.0)),
        Statement::Repeat(5, vec![
            Statement::Tell(EllexValue::String("WASM loop".to_string())),
        ]),
    ];
    
    let options = TranspilerOptions {
        target: Target::WebAssembly {
            simd: false,
            threads: false,
            opt_level: 2,
        },
        ..Default::default()
    };
    
    let transpiler = EllexTranspiler::with_options(options);
    
    c.bench_function("wasm_compilation", |b| {
        b.iter(|| {
            let result = transpiler.transpile(black_box(&ast));
            black_box(result)
        })
    });
}

criterion_group!(
    benches,
    bench_simple_transpilation,
    bench_optimization_levels,
    bench_different_targets,
    bench_js_parsing,
    bench_large_ast,
    bench_complex_nesting,
    bench_string_interpolation,
    bench_wasm_compilation
);

criterion_main!(benches);