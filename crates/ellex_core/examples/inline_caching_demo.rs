use ellex_core::{
    cached_ast::CachedStatement,
    optimized_runtime::OptimizedEllexRuntime,
    benchmarks::{BenchmarkSuite, MemoryBenchmark},
    values::{EllexValue, Statement},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Ellex AST Inline Caching Demonstration");
    println!("==========================================");
    println!();

    // Demo 1: Basic optimized execution
    basic_optimization_demo()?;
    
    // Demo 2: Cache behavior analysis
    cache_behavior_demo()?;
    
    // Demo 3: Performance benchmarks
    performance_benchmark_demo()?;
    
    // Demo 4: Memory usage analysis
    memory_analysis_demo()?;

    Ok(())
}

fn basic_optimization_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demo 1: Basic Optimized Execution");
    println!("-------------------------------------");

    let mut runtime = OptimizedEllexRuntime::new();
    
    // Create some test statements
    let original_statements = vec![
        Statement::Tell(EllexValue::String("Hello, Ellex!".to_string())),
        Statement::Tell(EllexValue::Number(42.0)),
        Statement::Repeat(3, vec![
            Statement::Tell(EllexValue::String("Repeated message".to_string())),
        ]),
    ];

    // Convert to cached statements
    let mut cached_statements: Vec<CachedStatement> = original_statements
        .into_iter()
        .map(CachedStatement::from_statement)
        .collect();

    println!("Executing statements with optimization...");
    
    // Execute multiple times to warm up caches
    for i in 1..=5 {
        println!("\nExecution #{}", i);
        runtime.execute_cached(&mut cached_statements)?;
        
        let stats = runtime.cache_stats();
        println!("Cache hits: {}, Hit rate: {:.1}%", 
                stats.total_hits, 
                stats.hit_rate() * 100.0);
    }

    let metrics = runtime.performance_metrics();
    println!("\n📈 Performance Metrics:");
    println!("  Total executions: {}", metrics.execution_count);
    println!("  Average execution time: {:?}", metrics.average_execution_time);
    println!("  Cache efficiency: {:.1}%", metrics.cache_efficiency * 100.0);
    
    Ok(())
}

fn cache_behavior_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧠 Demo 2: Cache Behavior Analysis");
    println!("-----------------------------------");

    let mut runtime = OptimizedEllexRuntime::new();
    
    // Set up variables with different types to test polymorphic caching
    runtime.set_variable("test_var".to_string(), EllexValue::String("initial".to_string()));
    
    let mut statements = vec![
        CachedStatement::VariableAccess {
            variable_name: "test_var".to_string(),
            cache: ellex_core::cached_ast::VariableCache::new("test_var".to_string()),
        }
    ];

    println!("Testing monomorphic cache behavior:");
    
    // Execute with same type multiple times
    for i in 1..=3 {
        runtime.set_variable("test_var".to_string(), 
                           EllexValue::String(format!("value_{}", i)));
        runtime.execute_cached(&mut statements)?;
        
        let stats = statements[0].cache_stats();
        println!("  Execution {}: {} cache entries, {} hits", 
                i, stats.total_caches, stats.total_hits);
    }

    println!("\nTesting polymorphic cache behavior:");
    
    // Now change types to test polymorphic behavior
    let type_values = vec![
        EllexValue::String("string".to_string()),
        EllexValue::Number(42.0),
        EllexValue::Nil,
        EllexValue::String("another_string".to_string()), // Same type as first
    ];

    for (i, value) in type_values.into_iter().enumerate() {
        runtime.set_variable("test_var".to_string(), value);
        runtime.execute_cached(&mut statements)?;
        
        let stats = statements[0].cache_stats();
        println!("  Type change {}: {} monomorphic sites, {} megamorphic sites", 
                i + 1, stats.monomorphic_sites, stats.megamorphic_sites);
    }

    Ok(())
}

fn performance_benchmark_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚡ Demo 3: Performance Benchmarks");
    println!("----------------------------------");

    let benchmark_suite = BenchmarkSuite::new()
        .with_iterations(1000)  // Reduced for demo
        .with_warmup(100);

    println!("Running performance comparison benchmarks...");
    println!("(This may take a moment)");
    
    let results = benchmark_suite.run_all();
    
    println!("\n🏆 Key Takeaways:");
    println!("  • Average speedup: {:.2}x", results.get_average_speedup());
    println!("  • Best case speedup: {:.2}x", results.get_best_speedup());
    println!("  • Cache overhead is offset by execution speed gains");
    println!("  • Monomorphic sites show best performance improvements");

    Ok(())
}

fn memory_analysis_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💾 Demo 4: Memory Usage Analysis");
    println!("----------------------------------");

    let memory_usage = MemoryBenchmark::measure_cache_overhead();
    memory_usage.print_analysis();

    println!("\n🔍 Implementation Details:");
    println!("  • Inline caches store type information per AST node");
    println!("  • Variable caches track up to 3 type entries (monomorphic → polymorphic → megamorphic)");
    println!("  • Function call caches store argument type signatures");
    println!("  • Global cache invalidation available for dynamic environments");
    println!("  • Cache statistics help identify optimization opportunities");

    Ok(())
}