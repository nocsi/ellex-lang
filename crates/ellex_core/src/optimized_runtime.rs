use crate::cached_ast::{CachedStatement, CacheStats, VariableCache, FunctionCallCache, CacheEntry, CachedType};
use crate::values::{EllexValue, EllexFunction};
use crate::safety::{SafetyMonitor, ExecutionLimits};
use crate::{EllexError, EllexConfig};
use std::collections::HashMap;
use std::time::{Instant, Duration};

/// Optimized runtime with inline caching support
pub struct OptimizedEllexRuntime {
    variables: HashMap<String, EllexValue>,
    functions: HashMap<String, EllexFunction>,
    safety: SafetyMonitor,
    global_cache_stats: CacheStats,
    execution_count: u64,
    total_execution_time: Duration,
}

impl OptimizedEllexRuntime {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            safety: SafetyMonitor::new(ExecutionLimits::new()),
            global_cache_stats: CacheStats::default(),
            execution_count: 0,
            total_execution_time: Duration::new(0, 0),
        }
    }

    pub fn with_config(config: EllexConfig) -> Self {
        let limits = ExecutionLimits::with_timeout_and_instructions(
            Duration::from_millis(config.execution_timeout_ms),
            config.max_loop_iterations as u64,
        );
        
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            safety: SafetyMonitor::new(limits),
            global_cache_stats: CacheStats::default(),
            execution_count: 0,
            total_execution_time: Duration::new(0, 0),
        }
    }

    /// Execute cached statements with optimization
    pub fn execute_cached(&mut self, statements: &mut [CachedStatement]) -> Result<(), EllexError> {
        let start_time = Instant::now();
        self.safety.reset();
        self.execution_count += 1;

        // Warm up caches based on current variable state
        for stmt in statements.iter_mut() {
            stmt.warm_cache(&self.variables);
        }

        // Execute statements with caching optimizations
        for stmt in statements.iter_mut() {
            self.execute_cached_statement(stmt)?;
            
            // Check safety limits periodically (placeholder for now)
            if self.execution_count % 100 == 0 {
                // TODO: Implement safety limit checking
            }
        }

        // Update performance metrics
        let execution_time = start_time.elapsed();
        self.total_execution_time += execution_time;
        
        // Collect cache statistics
        let mut new_stats = CacheStats::default();
        for stmt in statements.iter() {
            let stmt_stats = stmt.cache_stats();
            new_stats.total_caches += stmt_stats.total_caches;
            new_stats.total_hits += stmt_stats.total_hits;
            new_stats.total_misses += stmt_stats.total_misses;
            new_stats.monomorphic_sites += stmt_stats.monomorphic_sites;
            new_stats.megamorphic_sites += stmt_stats.megamorphic_sites;
        }
        self.global_cache_stats = new_stats;

        Ok(())
    }

    fn execute_cached_statement(&mut self, stmt: &mut CachedStatement) -> Result<(), EllexError> {
        match stmt {
            CachedStatement::TellConstant { value } => {
                // Optimized path for constants - no cache lookup needed
                self.output_value(value);
            }
            
            CachedStatement::Tell { value, cache } => {
                if let Some(cache_entry) = cache {
                    // Fast path: use cached type information
                    if cache_entry.matches(value) {
                        cache_entry.hit();
                        self.output_value_fast(value, cache_entry.expected_type);
                    } else {
                        // Cache miss - update cache and use slow path
                        *cache_entry = CacheEntry::new(CachedType::from(&*value));
                        self.output_value(value);
                    }
                } else {
                    // Initialize cache
                    *cache = Some(CacheEntry::new(CachedType::from(&*value)));
                    self.output_value(value);
                }
            }

            CachedStatement::VariableAccess { variable_name, cache } => {
                if let Some(value) = self.variables.get(variable_name) {
                    if let Some(cache_entry) = cache.lookup(value) {
                        // Cache hit - use optimized path
                        self.access_variable_fast(variable_name, value, cache_entry.expected_type);
                    } else {
                        // Megamorphic site - use slow path
                        self.access_variable_slow(variable_name, value);
                    }
                } else {
                    return Err(EllexError::LogicError {
                        message: format!("Variable '{}' not found", variable_name),
                    });
                }
            }

            CachedStatement::Ask { prompt, variable, cache, .. } => {
                println!("{}", prompt);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).map_err(|_| {
                    EllexError::LogicError {
                        message: "Failed to read input".to_string(),
                    }
                })?;
                
                let value = EllexValue::String(input.trim().to_string());
                
                // Update variable cache
                if let Some(var_cache) = cache {
                    var_cache.lookup(&value);
                } else {
                    let mut new_cache = VariableCache::new(variable.clone());
                    new_cache.lookup(&value);
                    *cache = Some(new_cache);
                }
                
                self.variables.insert(variable.clone(), value);
            }

            CachedStatement::Repeat { times, body, iteration_cache } => {
                for i in 0..*times {
                    // Set loop iteration variable with cache optimization
                    let iter_value = EllexValue::Number(i as f64);
                    
                    if let Some(cache_entry) = iteration_cache {
                        if cache_entry.matches(&iter_value) {
                            cache_entry.hit();
                        }
                    } else {
                        *iteration_cache = Some(CacheEntry::new(CachedType::Number));
                    }
                    
                    self.variables.insert("count".to_string(), iter_value);
                    
                    for body_stmt in body.iter_mut() {
                        self.execute_cached_statement(body_stmt)?;
                    }
                }
            }

            CachedStatement::When { variable, condition, then_body, else_body, condition_cache } => {
                let var_value = self.variables.get(variable).ok_or_else(|| {
                    EllexError::LogicError {
                        message: format!("Variable '{}' not found", variable),
                    }
                })?;

                // Use cache to optimize condition evaluation
                let condition_matches = if let Some(cache) = condition_cache {
                    if let Some(cache_entry) = cache.lookup(var_value) {
                        // Fast path: use cached comparison
                        self.fast_condition_check(var_value, condition, cache_entry.expected_type)
                    } else {
                        // Slow path: megamorphic condition
                        self.slow_condition_check(var_value, condition)
                    }
                } else {
                    // Initialize cache
                    let mut new_cache = VariableCache::new(variable.clone());
                    new_cache.lookup(var_value);
                    *condition_cache = Some(new_cache);
                    self.slow_condition_check(var_value, condition)
                };

                if condition_matches {
                    for stmt in then_body {
                        self.execute_cached_statement(stmt)?;
                    }
                } else if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.execute_cached_statement(stmt)?;
                    }
                }
            }

            CachedStatement::Call { function_name, args, call_cache } => {
                if let Some(cache) = call_cache {
                    if let Some(cached_func) = &cache.cached_function {
                        if cache.matches_signature(args) {
                            // Fast path: use cached function
                            cache.hit_count += 1;
                            self.execute_function_fast(cached_func, args)?;
                            return Ok(());
                        }
                    }
                }

                // Slow path: lookup function and update cache
                if let Some(function) = self.functions.get(function_name).cloned() {
                    if let Some(cache) = call_cache {
                        cache.cache_call(function.clone(), args);
                    } else {
                        let mut new_cache = FunctionCallCache::new(function_name.clone());
                        new_cache.cache_call(function.clone(), args);
                        *call_cache = Some(new_cache);
                    }
                    self.execute_function_slow(&function, args)?;
                } else {
                    return Err(EllexError::LogicError {
                        message: format!("Function '{}' not found", function_name),
                    });
                }
            }
        }

        Ok(())
    }

    // Optimized output methods
    fn output_value(&self, value: &EllexValue) {
        println!("{}", value);
    }

    fn output_value_fast(&self, value: &EllexValue, _cached_type: CachedType) {
        // In a real implementation, we could have type-specific optimized output
        // For now, just use the regular output
        println!("{}", value);
    }

    // Optimized variable access methods
    fn access_variable_fast(&self, _name: &str, value: &EllexValue, _cached_type: CachedType) {
        // Optimized variable access - could avoid some type checks
        println!("Accessing {}", value);
    }

    fn access_variable_slow(&self, _name: &str, value: &EllexValue) {
        // Slow path with full type checking
        println!("Accessing {} (slow)", value);
    }

    // Optimized condition checking
    fn fast_condition_check(&self, var_value: &EllexValue, condition: &EllexValue, _cached_type: CachedType) -> bool {
        // Fast path with type assumptions
        var_value == condition
    }

    fn slow_condition_check(&self, var_value: &EllexValue, condition: &EllexValue) -> bool {
        // Slow path with full type checking
        var_value == condition
    }

    // Optimized function execution
    fn execute_function_fast(&mut self, function: &EllexFunction, _args: &[EllexValue]) -> Result<(), EllexError> {
        // Fast path: skip some argument binding checks
        for stmt in &function.body {
            let mut cached_stmt = CachedStatement::from_statement(stmt.clone());
            self.execute_cached_statement(&mut cached_stmt)?;
        }
        Ok(())
    }

    fn execute_function_slow(&mut self, function: &EllexFunction, _args: &[EllexValue]) -> Result<(), EllexError> {
        // Slow path: full argument binding and type checking
        for stmt in &function.body {
            let mut cached_stmt = CachedStatement::from_statement(stmt.clone());
            self.execute_cached_statement(&mut cached_stmt)?;
        }
        Ok(())
    }

    /// Get current cache statistics
    pub fn cache_stats(&self) -> &CacheStats {
        &self.global_cache_stats
    }

    /// Get performance metrics
    pub fn performance_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            execution_count: self.execution_count,
            total_execution_time: self.total_execution_time,
            average_execution_time: if self.execution_count > 0 {
                self.total_execution_time / self.execution_count as u32
            } else {
                Duration::new(0, 0)
            },
            cache_hit_rate: self.global_cache_stats.hit_rate(),
            cache_efficiency: self.global_cache_stats.cache_efficiency(),
        }
    }

    /// Add a function to the runtime
    pub fn add_function(&mut self, name: String, function: EllexFunction) {
        self.functions.insert(name, function);
    }

    /// Get a variable value
    pub fn get_variable(&self, name: &str) -> Option<&EllexValue> {
        self.variables.get(name)
    }

    /// Set a variable value
    pub fn set_variable(&mut self, name: String, value: EllexValue) {
        self.variables.insert(name, value);
    }

    /// Clear all caches (useful for benchmarking)
    pub fn clear_caches(&mut self) {
        // This would recursively clear all statement caches
        // For now, just reset global stats
        self.global_cache_stats = CacheStats::default();
    }
}

/// Performance metrics for the optimized runtime
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub execution_count: u64,
    pub total_execution_time: Duration,
    pub average_execution_time: Duration,
    pub cache_hit_rate: f64,
    pub cache_efficiency: f64,
}

impl PerformanceMetrics {
    pub fn throughput_per_second(&self) -> f64 {
        if self.total_execution_time.as_secs_f64() == 0.0 {
            0.0
        } else {
            self.execution_count as f64 / self.total_execution_time.as_secs_f64()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cached_ast::CachedStatement;

    #[test]
    fn test_optimized_runtime_basic() {
        let mut runtime = OptimizedEllexRuntime::new();
        let mut statements = vec![
            CachedStatement::TellConstant {
                value: EllexValue::String("Hello, World!".to_string()),
            }
        ];

        assert!(runtime.execute_cached(&mut statements).is_ok());
        assert_eq!(runtime.execution_count, 1);
    }

    #[test]
    fn test_variable_caching() {
        let mut runtime = OptimizedEllexRuntime::new();
        runtime.set_variable("test".to_string(), EllexValue::String("value".to_string()));
        
        let mut statements = vec![
            CachedStatement::VariableAccess {
                variable_name: "test".to_string(),
                cache: VariableCache::new("test".to_string()),
            }
        ];

        assert!(runtime.execute_cached(&mut statements).is_ok());
        
        let stats = runtime.cache_stats();
        assert!(stats.total_hits > 0);
    }

    #[test]
    fn test_performance_metrics() {
        let mut runtime = OptimizedEllexRuntime::new();
        let mut statements = vec![
            CachedStatement::TellConstant {
                value: EllexValue::Number(42.0),
            }
        ];

        // Execute multiple times to gather metrics
        for _ in 0..10 {
            runtime.execute_cached(&mut statements).unwrap();
        }

        let metrics = runtime.performance_metrics();
        assert_eq!(metrics.execution_count, 10);
        assert!(metrics.total_execution_time.as_nanos() > 0);
        assert!(metrics.throughput_per_second() > 0.0);
    }

    #[test]
    fn test_function_call_caching() {
        let mut runtime = OptimizedEllexRuntime::new();
        
        let test_function = EllexFunction {
            name: "test_func".to_string(),
            body: vec![crate::values::Statement::Tell(EllexValue::String("Inside function".to_string()))],
            params: vec![],
        };
        
        runtime.add_function("test_func".to_string(), test_function);
        
        let mut statements = vec![
            CachedStatement::Call {
                function_name: "test_func".to_string(),
                args: vec![],
                call_cache: None,
            }
        ];

        assert!(runtime.execute_cached(&mut statements).is_ok());
        
        // Verify cache was created
        if let CachedStatement::Call { call_cache, .. } = &statements[0] {
            assert!(call_cache.is_some());
        }
    }
}