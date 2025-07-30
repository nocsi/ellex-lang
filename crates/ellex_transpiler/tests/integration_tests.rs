//! Integration tests for the Ellex transpiler

use ellex_core::values::{EllexValue, Statement};
use ellex_transpiler::{EllexTranspiler, TranspilerOptions, Target, EsVersion};

#[test]
fn test_simple_ellex_to_js() {
    let ast = vec![
        Statement::Tell(EllexValue::String("Hello, world!".to_string())),
        Statement::Tell(EllexValue::Number(42.0)),
    ];
    
    let options = TranspilerOptions {
        target: Target::JavaScript {
            async_support: true,
            es_version: EsVersion::Es2020,
            optimize: false,
        },
        ..Default::default()
    };
    
    let transpiler = EllexTranspiler::with_options(options);
    let result = transpiler.transpile(&ast).unwrap();
    
    assert!(result.contains("console.log(\"Hello, world!\")"));
    assert!(result.contains("console.log(42)"));
    assert!(result.contains("async function main"));
    assert!(result.contains("EllexRuntime"));
}

#[test]
fn test_ellex_to_js_with_optimization() {
    let ast = vec![
        Statement::Repeat(2, vec![
            Statement::Tell(EllexValue::String("Optimized".to_string())),
        ]),
    ];
    
    let options = TranspilerOptions {
        target: Target::JavaScript {
            async_support: true,
            es_version: EsVersion::Es2020,
            optimize: true,
        },
        ..Default::default()
    };
    
    let transpiler = EllexTranspiler::with_options(options);
    let result = transpiler.transpile(&ast).unwrap();
    
    // Small loops should be unrolled when optimizing
    assert!(!result.contains("for (let i = 0;"));
}

#[test]
fn test_js_to_ellex_simple() {
    let js_code = r#"
        console.log("Hello from JS");
        console.log(123);
    "#;
    
    let result = EllexTranspiler::from_js(js_code).unwrap();
    
    assert_eq!(result.len(), 2);
    assert!(matches!(result[0], Statement::Tell(EllexValue::String(_))));
    assert!(matches!(result[1], Statement::Tell(EllexValue::Number(_))));
}

#[test]
fn test_js_to_ellex_for_loop() {
    let js_code = r#"
        for (let i = 0; i < 5; i++) {
            console.log("Loop iteration");
        }
    "#;
    
    let result = EllexTranspiler::from_js(js_code).unwrap();
    
    assert_eq!(result.len(), 1);
    if let Statement::Repeat(count, body) = &result[0] {
        assert_eq!(*count, 5);
        assert_eq!(body.len(), 1);
        assert!(matches!(body[0], Statement::Tell(_)));
    } else {
        panic!("Expected Repeat statement");
    }
}

#[test]
fn test_js_to_ellex_if_statement() {
    let js_code = r#"
        if (answer === "yes") {
            console.log("Great!");
        } else {
            console.log("Okay");
        }
    "#;
    
    let result = EllexTranspiler::from_js(js_code).unwrap();
    
    assert_eq!(result.len(), 1);
    if let Statement::When(var, condition, then_body, else_body) = &result[0] {
        assert_eq!(var, "answer");
        assert!(matches!(condition, EllexValue::String(_)));
        assert_eq!(then_body.len(), 1);
        assert!(else_body.is_some());
        assert_eq!(else_body.as_ref().unwrap().len(), 1);
    } else {
        panic!("Expected When statement");
    }
}

#[test]
fn test_wasm_compilation() {
    let ast = vec![
        Statement::Tell(EllexValue::String("Hello WASM!".to_string())),
    ];
    
    let options = TranspilerOptions {
        target: Target::WebAssembly {
            simd: false,
            threads: false,
            opt_level: 1,
        },
        ..Default::default()
    };
    
    let transpiler = EllexTranspiler::with_options(options);
    let result = transpiler.transpile(&ast);
    
    assert!(result.is_ok());
    let wat = result.unwrap();
    assert!(wat.contains("module"));
    assert!(wat.contains("memory"));
}

#[test]
fn test_typescript_generation() {
    let ast = vec![
        Statement::Ask("name".to_string(), Some("string".to_string())),
        Statement::Tell(EllexValue::String("Hello {name}!".to_string())),
    ];
    
    let options = TranspilerOptions {
        target: Target::TypeScript {
            declarations: true,
            strict: true,
        },
        ..Default::default()
    };
    
    let transpiler = EllexTranspiler::with_options(options);
    let result = transpiler.transpile(&ast).unwrap();
    
    assert!(result.contains("Generated TypeScript"));
    assert!(result.contains("ask("));
}

#[test]
fn test_minification() {
    let ast = vec![
        Statement::Tell(EllexValue::String("Test minification".to_string())),
    ];
    
    let options = TranspilerOptions {
        target: Target::JavaScript {
            async_support: true,
            es_version: EsVersion::Es2020,
            optimize: false,
        },
        minify: true,
        ..Default::default()
    };
    
    let transpiler = EllexTranspiler::with_options(options);
    let result = transpiler.transpile(&ast).unwrap();
    
    // Minified code should not contain comments or extra whitespace
    assert!(!result.contains("//"));
    assert!(!result.contains("\n\n"));
}

#[test]
fn test_string_interpolation() {
    let ast = vec![
        Statement::Tell(EllexValue::String("Hello {name}, you are {age} years old!".to_string())),
    ];
    
    let options = TranspilerOptions::default();
    let transpiler = EllexTranspiler::with_options(options);
    let result = transpiler.transpile(&ast).unwrap();
    
    // Should generate template literal
    assert!(result.contains("`"));
    assert!(result.contains("${name}"));
    assert!(result.contains("${age}"));
}

#[test]
fn test_complex_nested_structures() {
    let ast = vec![
        Statement::When(
            "mode".to_string(),
            EllexValue::String("interactive".to_string()),
            vec![
                Statement::Repeat(3, vec![
                    Statement::Ask("input".to_string(), None),
                    Statement::Tell(EllexValue::String("Got: {input}".to_string())),
                ]),
            ],
            Some(vec![
                Statement::Tell(EllexValue::String("Non-interactive mode".to_string())),
            ]),
        ),
    ];
    
    let options = TranspilerOptions::default();
    let transpiler = EllexTranspiler::with_options(options);
    let result = transpiler.transpile(&ast).unwrap();
    
    assert!(result.contains("if (mode === \"interactive\")"));
    assert!(result.contains("for (let i = 0; i < 3; i++)"));
    assert!(result.contains("} else {"));
    assert!(result.contains("Non-interactive mode"));
}

#[test]
fn test_error_handling() {
    let ast = vec![
        Statement::Tell(EllexValue::Function(ellex_core::values::EllexFunction {
            name: "test".to_string(),
            body: vec![],
            params: vec![],
        })),
    ];
    
    let options = TranspilerOptions::default();
    let transpiler = EllexTranspiler::with_options(options);
    let result = transpiler.transpile(&ast);
    
    // Should return error for unsupported function values
    assert!(result.is_err());
}

#[test]
fn test_array_literals() {
    let js_code = r#"console.log([1, 2, "three", [4, 5]]);"#;
    let result = EllexTranspiler::from_js(js_code).unwrap();
    
    assert_eq!(result.len(), 1);
    if let Statement::Tell(EllexValue::List(items)) = &result[0] {
        assert_eq!(items.len(), 4);
        assert!(matches!(items[0], EllexValue::Number(_)));
        assert!(matches!(items[1], EllexValue::Number(_)));
        assert!(matches!(items[2], EllexValue::String(_)));
        assert!(matches!(items[3], EllexValue::List(_)));
    } else {
        panic!("Expected Tell with list");
    }
}

#[test]
fn test_template_literals() {
    let js_code = r#"console.log(`Hello ${name}, welcome to ${place}!`);"#;
    let result = EllexTranspiler::from_js(js_code).unwrap();
    
    assert_eq!(result.len(), 1);
    if let Statement::Tell(EllexValue::String(s)) = &result[0] {
        assert!(s.contains("{name}"));
        assert!(s.contains("{place}"));
    } else {
        panic!("Expected Tell with interpolated string");
    }
}

#[test]
fn test_function_declarations() {
    let js_code = r#"
        function greet(name) {
            console.log("Hello " + name);
        }
        
        greet("World");
    "#;
    
    let result = EllexTranspiler::from_js(js_code).unwrap();
    
    // Should have function declaration and call
    assert!(result.len() >= 2);
    assert!(result.iter().any(|stmt| matches!(stmt, Statement::Call(_))));
}

#[test]
fn test_performance_with_large_ast() {
    use std::time::Instant;
    
    // Create a large AST with many statements
    let mut ast = Vec::new();
    for i in 0..1000 {
        ast.push(Statement::Tell(EllexValue::String(format!("Statement {}", i))));
    }
    
    let options = TranspilerOptions {
        target: Target::JavaScript {
            async_support: true,
            es_version: EsVersion::Es2020,
            optimize: true,
        },
        ..Default::default()
    };
    
    let transpiler = EllexTranspiler::with_options(options);
    
    let start = Instant::now();
    let result = transpiler.transpile(&ast);
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration.as_millis() < 5000); // Should complete within 5 seconds
    
    let code = result.unwrap();
    assert!(code.contains("console.log(\"Statement 0\")"));
    assert!(code.contains("console.log(\"Statement 999\")"));
}

#[test]
fn test_round_trip_conversion() {
    // Test: Ellex -> JS -> Ellex
    let original_ast = vec![
        Statement::Tell(EllexValue::String("Hello".to_string())),
        Statement::Tell(EllexValue::Number(42.0)),
    ];
    
    // Convert to JavaScript
    let js_code = EllexTranspiler::to_js(&original_ast).unwrap();
    
    // Convert back to Ellex
    let converted_ast = EllexTranspiler::from_js(&js_code).unwrap();
    
    // Should have same number of statements
    assert_eq!(original_ast.len(), converted_ast.len());
    
    // Content should be similar (not exact due to conversion limitations)
    assert!(matches!(converted_ast[0], Statement::Tell(EllexValue::String(_))));
    assert!(matches!(converted_ast[1], Statement::Tell(EllexValue::Number(_))));
}

#[test]
fn test_source_maps() {
    let ast = vec![
        Statement::Tell(EllexValue::String("Test source maps".to_string())),
    ];
    
    let options = TranspilerOptions {
        source_maps: true,
        ..Default::default()
    };
    
    let transpiler = EllexTranspiler::with_options(options);
    let result = transpiler.transpile(&ast).unwrap();
    
    assert!(result.contains("//# sourceMappingURL="));
}