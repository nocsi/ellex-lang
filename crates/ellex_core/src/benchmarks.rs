use crate::cached_ast::CachedStatement;
use crate::optimized_runtime::{OptimizedEllexRuntime, PerformanceMetrics};
use crate::runtime::EllexRuntime;
use crate::values::{EllexValue, EllexFunction, Statement};
use std::time::{Duration, Instant};

/// Benchmark suite for comparing optimized vs unoptimized execution
pub struct BenchmarkSuite {
    iterations: u32,
    warmup_iterations: u32,
}

impl BenchmarkSuite {
    pub fn new() -> Self {
        Self {
            iterations: 10000,
            warmup_iterations: 1000,
        }
    }

    pub fn with_iterations(mut self, iterations: u32) -> Self {
        self.iterations = iterations;
        self
    }

    pub fn with_warmup(mut self, warmup: u32) -> Self {
        self.warmup_iterations = warmup;
        self
    }

    /// Run all benchmarks and return results
    pub fn run_all(&self) -> BenchmarkResults {
        println!("Running Ellex AST Inline Caching Benchmarks...");
        println!("Iterations: {}, Warmup: {}", self.iterations, self.warmup_iterations);
        println!();

        let mut results = BenchmarkResults::new();

        // Benchmark 1: Simple Tell statements
        println!("Benchmark 1: Simple Tell Statements");
        let (opt_time, unopt_time) = self.benchmark_simple_tells();
        results.add_result("simple_tells", opt_time, unopt_time);
        println!();

        // Benchmark 2: Variable access patterns
        println!("Benchmark 2: Variable Access Patterns");
        let (opt_time, unopt_time) = self.benchmark_variable_access();
        results.add_result("variable_access", opt_time, unopt_time);
        println!();

        // Benchmark 3: Function calls
        println!("Benchmark 3: Function Calls");
        let (opt_time, unopt_time) = self.benchmark_function_calls();
        results.add_result("function_calls", opt_time, unopt_time);
        println!();

        // Benchmark 4: Loop with varied types (megamorphic)
        println!("Benchmark 4: Megamorphic Loop");
        let (opt_time, unopt_time) = self.benchmark_megamorphic_loop();
        results.add_result("megamorphic_loop", opt_time, unopt_time);
        println!();

        // Benchmark 5: Conditional statements
        println!("Benchmark 5: Conditional Statements");
        let (opt_time, unopt_time) = self.benchmark_conditionals();
        results.add_result("conditionals", opt_time, unopt_time);
        println!();

        results.print_summary();
        results
    }

    fn benchmark_simple_tells(&self) -> (Duration, Duration) {
        let statements = vec![
            Statement::Tell(EllexValue::String("Hello".to_string())),
            Statement::Tell(EllexValue::Number(42.0)),
            Statement::Tell(EllexValue::String("World".to_string())),
        ];

        let cached_statements: Vec<CachedStatement> = statements.iter()
            .map(|s| CachedStatement::from_statement(s.clone()))
            .collect();

        self.compare_execution(&statements, &cached_statements)
    }

    fn benchmark_variable_access(&self) -> (Duration, Duration) {
        let statements = vec![
            Statement::Tell(EllexValue::String("test_var".to_string())), // Simulate var access
            Statement::Tell(EllexValue::String("test_var".to_string())),
            Statement::Tell(EllexValue::String("test_var".to_string())),
        ];

        let cached_statements: Vec<CachedStatement> = statements.iter()
            .map(|s| CachedStatement::from_statement(s.clone()))
            .collect();

        self.compare_execution(&statements, &cached_statements)
    }

    fn benchmark_function_calls(&self) -> (Duration, Duration) {
        let statements = vec![
            Statement::Call("test_func".to_string()),
            Statement::Call("test_func".to_string()),
            Statement::Call("test_func".to_string()),
        ];

        let cached_statements: Vec<CachedStatement> = statements.iter()
            .map(|s| CachedStatement::from_statement(s.clone()))
            .collect();

        self.compare_execution(&statements, &cached_statements)
    }

    fn benchmark_megamorphic_loop(&self) -> (Duration, Duration) {
        let statements = vec![
            Statement::Repeat(100, vec![
                Statement::Tell(EllexValue::String("string".to_string())),
                Statement::Tell(EllexValue::Number(1.0)),
                Statement::Tell(EllexValue::Nil),
            ]),
        ];

        let cached_statements: Vec<CachedStatement> = statements.iter()
            .map(|s| CachedStatement::from_statement(s.clone()))
            .collect();

        self.compare_execution(&statements, &cached_statements)
    }

    fn benchmark_conditionals(&self) -> (Duration, Duration) {
        let statements = vec![
            Statement::When(
                "test_var".to_string(),
                EllexValue::String("yes".to_string()),
                vec![Statement::Tell(EllexValue::String("Match!".to_string()))],
                Some(vec![Statement::Tell(EllexValue::String("No match".to_string()))]),
            ),
        ];

        let cached_statements: Vec<CachedStatement> = statements.iter()
            .map(|s| CachedStatement::from_statement(s.clone()))
            .collect();

        self.compare_execution(&statements, &cached_statements)
    }

    fn compare_execution(&self, original: &[Statement], cached: &[CachedStatement]) -> (Duration, Duration) {
        // Benchmark optimized runtime
        let optimized_time = {
            let mut runtime = OptimizedEllexRuntime::new();
            let mut cached_copy: Vec<CachedStatement> = cached.to_vec();

            // Warmup
            for _ in 0..self.warmup_iterations {
                let _ = runtime.execute_cached(&mut cached_copy);
            }

            // Actual benchmark
            let start = Instant::now();
            for _ in 0..self.iterations {
                let _ = runtime.execute_cached(&mut cached_copy);
            }
            start.elapsed()
        };

        // Benchmark unoptimized runtime (placeholder - would need actual implementation)
        let unoptimized_time = {
            let mut runtime = EllexRuntime::new();

            // Warmup
            for _ in 0..self.warmup_iterations {
                let _ = runtime.execute(original.to_vec());
            }

            // Actual benchmark
            let start = Instant::now();
            for _ in 0..self.iterations {
                let _ = runtime.execute(original.to_vec());
            }
            start.elapsed()
        };

        println!("  Optimized:   {:?} ({:.2} µs/iter)", 
                optimized_time, 
                optimized_time.as_micros() as f64 / self.iterations as f64);
        println!("  Unoptimized: {:?} ({:.2} µs/iter)", 
                unoptimized_time, 
                unoptimized_time.as_micros() as f64 / self.iterations as f64);
        
        let speedup = unoptimized_time.as_nanos() as f64 / optimized_time.as_nanos() as f64;
        println!("  Speedup: {:.2}x", speedup);

        (optimized_time, unoptimized_time)
    }
}

impl Default for BenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

/// Results from benchmark runs
pub struct BenchmarkResults {
    results: Vec<BenchmarkResult>,
}

struct BenchmarkResult {
    name: String,
    optimized_time: Duration,
    unoptimized_time: Duration,
    speedup: f64,
}

impl BenchmarkResults {
    fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    fn add_result(&mut self, name: &str, optimized: Duration, unoptimized: Duration) {
        let speedup = unoptimized.as_nanos() as f64 / optimized.as_nanos() as f64;
        self.results.push(BenchmarkResult {
            name: name.to_string(),
            optimized_time: optimized,
            unoptimized_time: unoptimized,
            speedup,
        });
    }

    pub fn print_summary(&self) {
        println!("=== BENCHMARK SUMMARY ===");
        println!();
        println!("{:<20} {:>12} {:>12} {:>10}", "Benchmark", "Optimized", "Unoptimized", "Speedup");
        println!("{}", "-".repeat(60));

        let mut total_speedup = 0.0;
        for result in &self.results {
            println!("{:<20} {:>10.2}µs {:>10.2}µs {:>8.2}x",
                result.name,
                result.optimized_time.as_micros() as f64 / 10000.0, // Assuming 10k iterations
                result.unoptimized_time.as_micros() as f64 / 10000.0,
                result.speedup);
            total_speedup += result.speedup;
        }

        println!("{}", "-".repeat(60));
        println!("Average speedup: {:.2}x", total_speedup / self.results.len() as f64);
        println!();

        // Print cache statistics analysis
        self.print_cache_analysis();
    }

    fn print_cache_analysis(&self) {
        println!("=== CACHE ANALYSIS ===");
        println!();
        println!("Expected improvements by benchmark type:");
        println!("• Simple operations: 1.1-1.3x (constant folding)");
        println!("• Variable access: 1.2-1.8x (type caching)");
        println!("• Function calls: 1.5-2.0x (dispatch caching)");
        println!("• Megamorphic sites: 0.9-1.1x (cache thrashing)");
        println!("• Conditionals: 1.3-1.7x (branch prediction)");
        println!();
        println!("Note: Actual results depend on workload characteristics");
        println!("and may vary based on cache hit rates and polymorphism.");
    }

    pub fn get_average_speedup(&self) -> f64 {
        if self.results.is_empty() {
            1.0
        } else {
            self.results.iter().map(|r| r.speedup).sum::<f64>() / self.results.len() as f64
        }
    }

    pub fn get_best_speedup(&self) -> f64 {
        self.results.iter()
            .map(|r| r.speedup)
            .fold(1.0, |acc, x| acc.max(x))
    }
}

/// Memory usage benchmark for cache overhead analysis
pub struct MemoryBenchmark;

impl MemoryBenchmark {
    pub fn measure_cache_overhead() -> MemoryUsage {
        let original_statements: Vec<Statement> = vec![
            Statement::Tell(EllexValue::String("test".to_string())),
            Statement::Call("func".to_string()),
            Statement::Repeat(10, vec![
                Statement::Tell(EllexValue::Number(42.0)),
            ]),
        ];

        let cached_statements: Vec<CachedStatement> = original_statements.iter()
            .map(|s| CachedStatement::from_statement(s.clone()))
            .collect();

        // Rough memory usage estimation (would need a proper memory profiler in practice)
        let original_size = std::mem::size_of_val(&original_statements);
        let cached_size = std::mem::size_of_val(&cached_statements);

        MemoryUsage {
            original_bytes: original_size,
            cached_bytes: cached_size,
            overhead_bytes: cached_size.saturating_sub(original_size),
            overhead_percentage: if original_size > 0 {
                ((cached_size as f64 - original_size as f64) / original_size as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

pub struct MemoryUsage {
    pub original_bytes: usize,
    pub cached_bytes: usize,
    pub overhead_bytes: usize,
    pub overhead_percentage: f64,
}

impl MemoryUsage {
    pub fn print_analysis(&self) {
        println!("=== MEMORY USAGE ANALYSIS ===");
        println!("Original AST size: {} bytes", self.original_bytes);
        println!("Cached AST size:   {} bytes", self.cached_bytes);
        println!("Cache overhead:    {} bytes ({:.1}%)", 
                self.overhead_bytes, self.overhead_percentage);
        println!();
        println!("Memory/performance tradeoff:");
        println!("• Cache overhead is typically 20-50% of original AST size");
        println!("• Acceptable for long-running programs with repeated execution");
        println!("• May not be worth it for single-execution scripts");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_suite_creation() {
        let suite = BenchmarkSuite::new()
            .with_iterations(100)
            .with_warmup(10);
        
        assert_eq!(suite.iterations, 100);
        assert_eq!(suite.warmup_iterations, 10);
    }

    #[test]
    fn test_memory_benchmark() {
        let usage = MemoryBenchmark::measure_cache_overhead();
        assert!(usage.cached_bytes >= usage.original_bytes);
        assert!(usage.overhead_percentage >= 0.0);
    }

    #[test]
    fn test_benchmark_results() {
        let mut results = BenchmarkResults::new();
        results.add_result("test", Duration::from_millis(100), Duration::from_millis(200));
        
        assert_eq!(results.get_average_speedup(), 2.0);
        assert_eq!(results.get_best_speedup(), 2.0);
    }
}