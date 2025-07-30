use ellex_core::{
    EllexMiniElixirBridge, 
    MiniElixirInterpreter, 
    EvaluationContext,
    MiniElixirValidator,
    CachedMiniElixirRuntime,
    EllexValue,
    values::Statement,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Ellex + MiniElixir Integration Demonstration");
    println!("================================================");
    println!();

    // Demo 1: Basic MiniElixir interpreter
    basic_minielixir_demo()?;
    
    // Demo 2: Ellex to MiniElixir bridge
    ellex_bridge_demo()?;
    
    // Demo 3: Cached runtime performance
    cached_runtime_demo()?;
    
    // Demo 4: Safety validation
    safety_validation_demo()?;
    
    // Demo 5: JSON AST representation
    json_ast_demo()?;

    println!("\n🎉 All demos completed successfully!");
    println!("The Ellex + MiniElixir integration provides:");
    println!("  • Safe execution of Elixir-like code");
    println!("  • Inline caching for performance optimization");
    println!("  • Comprehensive validation and safety checks");
    println!("  • Seamless bridge between Ellex and MiniElixir syntax");

    Ok(())
}

fn basic_minielixir_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demo 1: Basic MiniElixir Interpreter");
    println!("----------------------------------------");

    let mut interpreter = MiniElixirInterpreter::new();
    let mut ctx = EvaluationContext::new();
    
    // Test basic expressions
    let expressions = vec![
        ("42", "integer"),
        ("3.14", "float"),
        ("\"hello world\"", "string"),
        ("true", "boolean"),
        ("nil", "nil"),
        ("[1, 2, 3]", "list"),
    ];
    
    for (code, desc) in expressions {
        let expr = interpreter.parse(code)?;
        let result = interpreter.eval(&expr, &mut ctx)?;
        println!("  {} ({}): {:?}", code, desc, result);
    }
    
    // Test arithmetic operations
    println!("\n  Arithmetic operations:");
    let arithmetic_exprs = vec![
        "1 + 2",
        "10 - 5",
        "3 * 4", 
        "15 / 3",
    ];
    
    for code in arithmetic_exprs {
        let expr = interpreter.parse(code).unwrap_or_else(|_| {
            // Fallback for unsupported operations in simple parser
            ellex_core::minielixir::MiniElixirExpr::String(format!("Unsupported: {}", code))
        });
        let result = interpreter.eval(&expr, &mut ctx)?;
        println!("    {} = {:?}", code, result);
    }
    
    // Test variables
    println!("\n  Variable binding:");
    ctx.bind("x".to_string(), EllexValue::Number(42.0));
    ctx.bind("name".to_string(), EllexValue::String("Ellex".to_string()));
    
    let var_expr = interpreter.parse("x")?;
    let result = interpreter.eval(&var_expr, &mut ctx)?;
    println!("    x = {:?}", result);
    
    let name_expr = interpreter.parse("name")?;
    let result = interpreter.eval(&name_expr, &mut ctx)?;
    println!("    name = {:?}", result);
    
    // Test built-in functions
    println!("\n  Built-in functions:");
    let func_exprs = vec![
        ("length([1, 2, 3, 4])", "list length"),
        ("to_string(42)", "number to string"),
    ];
    
    for (code, desc) in func_exprs {
        let expr = interpreter.parse(code).unwrap_or_else(|_| {
            ellex_core::minielixir::MiniElixirExpr::String(format!("Unsupported: {}", code))
        });
        let result = interpreter.eval(&expr, &mut ctx)?;
        println!("    {} ({}): {:?}", code, desc, result);
    }
    
    println!();
    Ok(())
}

fn ellex_bridge_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌉 Demo 2: Ellex to MiniElixir Bridge");
    println!("--------------------------------------");

    let mut bridge = EllexMiniElixirBridge::new();
    
    // Test simple Ellex statements
    println!("  Ellex tell statements:");
    let ellex_codes = vec![
        "tell \"Hello from Ellex!\"",
        "tell 42",
        "tell \"Testing the bridge\"",
    ];
    
    for code in ellex_codes {
        match bridge.execute_ellex_code(code) {
            Ok(result) => println!("    {} -> {:?}", code, result),
            Err(e) => println!("    {} -> Error: {}", code, e),
        }
    }
    
    // Test statement conversion
    println!("\n  Statement conversion:");
    let statements = vec![
        Statement::Tell(EllexValue::String("Direct statement".to_string())),
        Statement::Call("test_function".to_string()),
    ];
    
    for stmt in statements {
        match bridge.statement_to_minielixir(&stmt) {
            Ok(minielixir_expr) => println!("    {:?} -> {}", stmt, minielixir_expr),
            Err(e) => println!("    {:?} -> Error: {}", stmt, e),
        }
    }
    
    // Test JSON AST generation
    println!("\n  JSON AST representation:");
    let test_statements = vec![
        Statement::Tell(EllexValue::String("JSON test".to_string())),
    ];
    
    match bridge.ellex_to_json(&test_statements) {
        Ok(json_ast) => {
            for (i, json) in json_ast.iter().enumerate() {
                println!("    Statement {}: {}", i + 1, serde_json::to_string_pretty(json)?);
            }
        }
        Err(e) => println!("    JSON conversion error: {}", e),
    }
    
    println!();
    Ok(())
}

fn cached_runtime_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Demo 3: Cached Runtime Performance");
    println!("--------------------------------------");

    let mut runtime = CachedMiniElixirRuntime::new();
    let mut ctx = EvaluationContext::new();
    
    // Set up some variables
    ctx.bind("x".to_string(), EllexValue::Number(10.0));
    ctx.bind("y".to_string(), EllexValue::Number(5.0));
    
    // Test expressions multiple times to show caching
    let test_expressions = vec![
        "42",              // Simple literal
        "x",               // Variable access  
        "\"constant\"",    // String literal
    ];
    
    println!("  Executing expressions multiple times to test caching:");
    
    for expr_code in test_expressions {
        println!("\n    Testing: {}", expr_code);
        
        // Execute multiple times
        for i in 1..=5 {
            match runtime.eval_code(expr_code, &mut ctx) {
                Ok(result) => println!("      Run {}: {:?}", i, result),
                Err(e) => println!("      Run {}: Error: {}", i, e),
            }
        }
    }
    
    // Show performance statistics
    let stats = runtime.performance_stats();
    println!("\n  Performance Statistics:");
    println!("    Total evaluations: {}", stats.total_evaluations);
    println!("    Cached expressions: {}", stats.cached_expressions);
    println!("    Cached results: {}", stats.cached_results);
    println!("    Cache hit rate: {:.1}%", stats.cache_hit_rate * 100.0);
    println!("    Average execution time: {:?}", stats.average_execution_time);
    
    // Show cache analysis
    let analysis = runtime.cache_analysis();
    println!("\n  Cache Analysis:");
    println!("    Hot expressions: {}", analysis.hot_expressions.len());
    println!("    Cold expressions: {}", analysis.cold_expressions.len());
    println!("    Memory usage: {} bytes", analysis.total_memory_usage);
    
    println!();
    Ok(())
}

fn safety_validation_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️  Demo 4: Safety Validation");
    println!("------------------------------");

    let validator = MiniElixirValidator::new();
    let mut interpreter = MiniElixirInterpreter::new();
    
    // Test safe expressions
    println!("  Safe expressions:");
    let safe_exprs = vec![
        "42",
        "\"hello\"",
        "[1, 2, 3]",
        ":atom",
    ];
    
    for code in safe_exprs {
        let expr = interpreter.parse(code)?;
        match validator.validate(&expr) {
            Ok(()) => {
                let is_deterministic = validator.is_deterministic(&expr);
                let complexity = validator.complexity_score(&expr);
                println!("    ✓ {} (deterministic: {}, complexity: {})", 
                        code, is_deterministic, complexity);
            }
            Err(e) => println!("    ✗ {}: {}", code, e),
        }
    }
    
    // Test potentially unsafe expressions
    println!("\n  Testing safety boundaries:");
    let test_cases = vec![
        ("length([1, 2, 3])", "allowed function"),
        ("\"System.cmd('ls')\"", "forbidden pattern in string"),
    ];
    
    for (code, description) in test_cases {
        let expr = interpreter.parse(code).unwrap_or_else(|_| {
            ellex_core::minielixir::MiniElixirExpr::String(code.to_string())
        });
        
        match validator.validate(&expr) {
            Ok(()) => println!("    ✓ {} ({})", code, description),
            Err(e) => println!("    ✗ {} ({}): {}", code, description, e),
        }
    }
    
    println!();
    Ok(())
}

fn json_ast_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Demo 5: JSON AST Representation");
    println!("-----------------------------------");

    let bridge = EllexMiniElixirBridge::new();
    
    // Create sample Ellex statements
    let statements = vec![
        Statement::Tell(EllexValue::String("Hello JSON!".to_string())),
        Statement::Ask("user_name".to_string(), Some("string".to_string())),
        Statement::Repeat(3, vec![
            Statement::Tell(EllexValue::String("Repeated message".to_string()))
        ]),
        Statement::When(
            "response".to_string(),
            EllexValue::String("yes".to_string()),
            vec![Statement::Tell(EllexValue::String("Affirmative!".to_string()))],
            Some(vec![Statement::Tell(EllexValue::String("Negative".to_string()))])
        ),
    ];
    
    println!("  Converting Ellex statements to JSON AST:");
    
    for (i, stmt) in statements.iter().enumerate() {
        println!("\n    Statement {}: {:?}", i + 1, stmt);
        
        match bridge.ellex_to_json(&[stmt.clone()]) {
            Ok(json_ast) => {
                if let Some(json) = json_ast.get(0) {
                    println!("    JSON AST:");
                    let formatted = serde_json::to_string_pretty(json)?;
                    for line in formatted.lines() {
                        println!("      {}", line);
                    }
                }
            }
            Err(e) => println!("    Error: {}", e),
        }
    }
    
    println!();
    Ok(())
}