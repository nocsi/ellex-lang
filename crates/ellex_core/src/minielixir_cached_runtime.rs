use crate::cached_ast::{CachedType, CacheEntry, CacheStats};
use crate::minielixir::{MiniElixirExpr, MiniElixirInterpreter, EvaluationContext, InterpreterStats};
use crate::minielixir_validator::MiniElixirValidator;
use crate::values::EllexValue;
use crate::{EllexError, EllexResult};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Cached MiniElixir expression with inline caching metadata
#[derive(Debug, Clone)]
pub struct CachedMiniElixirExpr {
    pub expr: MiniElixirExpr,
    pub cache_entry: Option<CacheEntry>,
    pub execution_count: u64,
    pub total_execution_time: Duration,
    pub is_deterministic: bool,
    pub complexity_score: usize,
}

impl CachedMiniElixirExpr {
    pub fn new(expr: MiniElixirExpr, validator: &MiniElixirValidator) -> Self {
        Self {
            is_deterministic: validator.is_deterministic(&expr),
            complexity_score: validator.complexity_score(&expr),
            expr,
            cache_entry: None,
            execution_count: 0,
            total_execution_time: Duration::new(0, 0),
        }
    }
    
    pub fn cache_hit_rate(&self) -> f64 {
        if self.execution_count == 0 {
            0.0
        } else if let Some(cache) = &self.cache_entry {
            cache.hit_count as f64 / self.execution_count as f64
        } else {
            0.0
        }
    }
    
    pub fn average_execution_time(&self) -> Duration {
        if self.execution_count == 0 {
            Duration::new(0, 0)
        } else {
            self.total_execution_time / self.execution_count as u32
        }
    }
}

/// High-performance cached MiniElixir runtime
pub struct CachedMiniElixirRuntime {
    interpreter: MiniElixirInterpreter,
    validator: MiniElixirValidator,
    expression_cache: HashMap<String, CachedMiniElixirExpr>,
    result_cache: HashMap<String, (EllexValue, CachedType)>,
    global_stats: CacheStats,
    total_evaluations: u64,
    cache_enabled: bool,
}

impl CachedMiniElixirRuntime {
    pub fn new() -> Self {
        Self {
            interpreter: MiniElixirInterpreter::new(),
            validator: MiniElixirValidator::new(),
            expression_cache: HashMap::new(),
            result_cache: HashMap::new(),
            global_stats: CacheStats::default(),
            total_evaluations: 0,
            cache_enabled: true,
        }
    }
    
    pub fn with_custom_functions(functions: Vec<String>) -> Self {
        Self {
            interpreter: MiniElixirInterpreter::new(),
            validator: MiniElixirValidator::new().with_custom_functions(functions),
            expression_cache: HashMap::new(),
            result_cache: HashMap::new(),
            global_stats: CacheStats::default(),
            total_evaluations: 0,
            cache_enabled: true,
        }
    }
    
    pub fn disable_caching(&mut self) {
        self.cache_enabled = false;
    }
    
    pub fn enable_caching(&mut self) {
        self.cache_enabled = true;
    }
    
    /// Parse and evaluate MiniElixir code with caching
    pub fn eval_code(&mut self, code: &str, ctx: &mut EvaluationContext) -> EllexResult<EllexValue> {
        // Parse the code
        let expr = self.interpreter.parse(code)?;
        
        // Validate for safety
        self.validator.validate(&expr)?;
        
        // Evaluate with caching
        self.eval_expr(&expr, ctx)
    }
    
    /// Evaluate expression with inline caching
    pub fn eval_expr(&mut self, expr: &MiniElixirExpr, ctx: &mut EvaluationContext) -> EllexResult<EllexValue> {
        let start_time = Instant::now();
        self.total_evaluations += 1;
        
        // Generate cache key
        let cache_key = self.generate_cache_key(expr, ctx);
        
        // Try cache first if enabled
        if self.cache_enabled {
            if let Some(cached_result) = self.try_cache_lookup(&cache_key, expr) {
                self.update_cache_hit_stats();
                return Ok(cached_result);
            }
        }
        
        // Cache miss - evaluate expression
        let result = self.interpreter.eval(expr, ctx)?;
        let execution_time = start_time.elapsed();
        
        // Update caching metadata
        if self.cache_enabled {
            self.update_expression_cache(&cache_key, expr, execution_time);
            self.maybe_cache_result(&cache_key, &result);
        }
        
        self.update_cache_miss_stats();
        Ok(result)
    }
    
    /// Evaluate multiple expressions in a block with optimizations
    pub fn eval_block(&mut self, exprs: &[MiniElixirExpr], ctx: &mut EvaluationContext) -> EllexResult<EllexValue> {
        if exprs.is_empty() {
            return Ok(EllexValue::Nil);
        }
        
        let mut result = EllexValue::Nil;
        
        // Pre-analyze block for optimization opportunities
        let analysis = self.analyze_block(exprs);
        
        for (i, expr) in exprs.iter().enumerate() {
            // Apply block-level optimizations
            if analysis.can_skip_evaluation(i) {
                continue;
            }
            
            result = self.eval_expr(expr, ctx)?;
            
            // Early exit optimization for certain patterns
            if analysis.should_early_exit(i, &result) {
                break;
            }
        }
        
        Ok(result)
    }
    
    fn generate_cache_key(&self, expr: &MiniElixirExpr, ctx: &EvaluationContext) -> String {
        // For deterministic expressions, include relevant context
        if self.validator.is_deterministic(expr) {
            format!("{:?}|{:?}", expr, self.extract_relevant_bindings(expr, ctx))
        } else {
            // Non-deterministic expressions get unique keys
            format!("{:?}|{}", expr, self.total_evaluations)
        }
    }
    
    fn extract_relevant_bindings(&self, expr: &MiniElixirExpr, ctx: &EvaluationContext) -> HashMap<String, String> {
        let mut relevant = HashMap::new();
        self.collect_variable_references(expr, &mut relevant, ctx);
        relevant
    }
    
    fn collect_variable_references(&self, expr: &MiniElixirExpr, relevant: &mut HashMap<String, String>, ctx: &EvaluationContext) {
        match expr {
            MiniElixirExpr::Variable(name) => {
                if let Some(value) = ctx.get(name) {
                    relevant.insert(name.clone(), format!("{:?}", value));
                }
            }
            MiniElixirExpr::BinaryOp { left, right, .. } => {
                self.collect_variable_references(left, relevant, ctx);
                self.collect_variable_references(right, relevant, ctx);
            }
            MiniElixirExpr::UnaryOp { operand, .. } => {
                self.collect_variable_references(operand, relevant, ctx);
            }
            MiniElixirExpr::Call { args, .. } => {
                for arg in args {
                    self.collect_variable_references(arg, relevant, ctx);
                }
            }
            MiniElixirExpr::List(elements) | MiniElixirExpr::Tuple(elements) => {
                for elem in elements {
                    self.collect_variable_references(elem, relevant, ctx);
                }
            }
            MiniElixirExpr::If { condition, then_branch, else_branch } => {
                self.collect_variable_references(condition, relevant, ctx);
                self.collect_variable_references(then_branch, relevant, ctx);
                if let Some(else_expr) = else_branch {
                    self.collect_variable_references(else_expr, relevant, ctx);
                }
            }
            MiniElixirExpr::Block(exprs) => {
                for expr in exprs {
                    self.collect_variable_references(expr, relevant, ctx);
                }
            }
            MiniElixirExpr::Access { object, key } => {
                self.collect_variable_references(object, relevant, ctx);
                self.collect_variable_references(key, relevant, ctx);
            }
            // Add other expression types as needed
            _ => {}
        }
    }
    
    fn try_cache_lookup(&self, cache_key: &str, expr: &MiniElixirExpr) -> Option<EllexValue> {
        // Only cache deterministic expressions with reasonable complexity
        if !self.validator.is_deterministic(expr) {
            return None;
        }
        
        let complexity = self.validator.complexity_score(expr);
        if complexity < 3 {
            return None; // Too simple to benefit from caching
        }
        
        if let Some((cached_result, _cached_type)) = self.result_cache.get(cache_key) {
            Some(cached_result.clone())
        } else {
            None
        }
    }
    
    fn update_expression_cache(&mut self, cache_key: &str, expr: &MiniElixirExpr, execution_time: Duration) {
        let cached_expr = self.expression_cache
            .entry(cache_key.to_string())
            .or_insert_with(|| CachedMiniElixirExpr::new(expr.clone(), &self.validator));
        
        cached_expr.execution_count += 1;
        cached_expr.total_execution_time += execution_time;
    }
    
    fn maybe_cache_result(&mut self, cache_key: &str, result: &EllexValue) {
        // Only cache if the result cache isn't too large
        if self.result_cache.len() < 1000 {
            let cached_type = CachedType::from(result);
            self.result_cache.insert(cache_key.to_string(), (result.clone(), cached_type));
        }
    }
    
    fn update_cache_hit_stats(&mut self) {
        self.global_stats.total_hits += 1;
    }
    
    fn update_cache_miss_stats(&mut self) {
        self.global_stats.total_misses += 1;
    }
    
    fn analyze_block(&self, exprs: &[MiniElixirExpr]) -> BlockAnalysis {
        BlockAnalysis::new(exprs, &self.validator)
    }
    
    /// Clear all caches
    pub fn clear_caches(&mut self) {
        self.expression_cache.clear();
        self.result_cache.clear();
        self.global_stats = CacheStats::default();
    }
    
    /// Get comprehensive performance statistics
    pub fn performance_stats(&self) -> CachedRuntimeStats {
        let interpreter_stats = self.interpreter.stats();
        
        let total_cached_expressions = self.expression_cache.len();
        let total_cached_results = self.result_cache.len();
        
        let avg_execution_time = if total_cached_expressions > 0 {
            let total_time: Duration = self.expression_cache.values()
                .map(|cached| cached.total_execution_time)
                .sum();
            total_time / total_cached_expressions as u32
        } else {
            Duration::new(0, 0)
        };
        
        let cache_hit_rate = if self.global_stats.total_hits + self.global_stats.total_misses > 0 {
            self.global_stats.total_hits as f64 / 
            (self.global_stats.total_hits + self.global_stats.total_misses) as f64
        } else {
            0.0
        };
        
        CachedRuntimeStats {
            total_evaluations: self.total_evaluations,
            cached_expressions: total_cached_expressions,
            cached_results: total_cached_results,
            cache_hit_rate,
            average_execution_time: avg_execution_time,
            interpreter_stats,
            global_cache_stats: self.global_stats.clone(),
        }
    }
    
    /// Get detailed cache analysis
    pub fn cache_analysis(&self) -> CacheAnalysis {
        let mut hot_expressions = Vec::new();
        let mut cold_expressions = Vec::new();
        
        for (key, cached_expr) in &self.expression_cache {
            let analysis_entry = CacheAnalysisEntry {
                cache_key: key.clone(),
                execution_count: cached_expr.execution_count,
                cache_hit_rate: cached_expr.cache_hit_rate(),
                average_execution_time: cached_expr.average_execution_time(),
                complexity_score: cached_expr.complexity_score,
                is_deterministic: cached_expr.is_deterministic,
            };
            
            if cached_expr.execution_count > 10 {
                hot_expressions.push(analysis_entry);
            } else {
                cold_expressions.push(analysis_entry);
            }
        }
        
        // Sort by execution count
        hot_expressions.sort_by(|a, b| b.execution_count.cmp(&a.execution_count));
        cold_expressions.sort_by(|a, b| b.execution_count.cmp(&a.execution_count));
        
        CacheAnalysis {
            hot_expressions,
            cold_expressions,
            total_memory_usage: self.estimate_memory_usage(),
        }
    }
    
    fn estimate_memory_usage(&self) -> usize {
        // Rough estimate of cache memory usage
        let expr_cache_size = self.expression_cache.len() * 200; // Rough estimate per entry
        let result_cache_size = self.result_cache.len() * 100;   // Rough estimate per result
        expr_cache_size + result_cache_size
    }
}

impl Default for CachedMiniElixirRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Block analysis for optimization
struct BlockAnalysis {
    early_exit_opportunities: Vec<usize>,
    skippable_expressions: Vec<usize>,
}

impl BlockAnalysis {
    fn new(exprs: &[MiniElixirExpr], validator: &MiniElixirValidator) -> Self {
        let mut analysis = Self {
            early_exit_opportunities: Vec::new(),
            skippable_expressions: Vec::new(),
        };
        
        // Analyze expressions for optimization opportunities
        for (i, expr) in exprs.iter().enumerate() {
            // Skip very simple expressions that don't need caching
            if validator.complexity_score(expr) < 2 {
                analysis.skippable_expressions.push(i);
            }
            
            // Look for early exit patterns (like if statements that always return)
            if matches!(expr, MiniElixirExpr::If { .. }) {
                analysis.early_exit_opportunities.push(i);
            }
        }
        
        analysis
    }
    
    fn can_skip_evaluation(&self, index: usize) -> bool {
        self.skippable_expressions.contains(&index)
    }
    
    fn should_early_exit(&self, index: usize, _result: &EllexValue) -> bool {
        // Simplified early exit logic
        self.early_exit_opportunities.contains(&index)
    }
}

/// Performance statistics for the cached runtime
#[derive(Debug, Clone)]
pub struct CachedRuntimeStats {
    pub total_evaluations: u64,
    pub cached_expressions: usize,
    pub cached_results: usize,
    pub cache_hit_rate: f64,
    pub average_execution_time: Duration,
    pub interpreter_stats: InterpreterStats,
    pub global_cache_stats: CacheStats,
}

/// Detailed cache analysis
#[derive(Debug, Clone)]
pub struct CacheAnalysis {
    pub hot_expressions: Vec<CacheAnalysisEntry>,
    pub cold_expressions: Vec<CacheAnalysisEntry>,
    pub total_memory_usage: usize,
}

#[derive(Debug, Clone)]
pub struct CacheAnalysisEntry {
    pub cache_key: String,
    pub execution_count: u64,
    pub cache_hit_rate: f64,
    pub average_execution_time: Duration,
    pub complexity_score: usize,
    pub is_deterministic: bool,
}

impl CacheAnalysis {
    pub fn print_summary(&self) {
        println!("=== CACHE ANALYSIS SUMMARY ===");
        println!("Hot expressions (>10 executions): {}", self.hot_expressions.len());
        println!("Cold expressions (≤10 executions): {}", self.cold_expressions.len());
        println!("Estimated memory usage: {} bytes", self.total_memory_usage);
        
        if !self.hot_expressions.is_empty() {
            println!("\nTop 5 Hot Expressions:");
            for (i, entry) in self.hot_expressions.iter().take(5).enumerate() {
                println!("  {}. {} executions, {:.1}% hit rate, {:?} avg time", 
                        i + 1, entry.execution_count, entry.cache_hit_rate * 100.0, 
                        entry.average_execution_time);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cached_runtime_basic() {
        let mut runtime = CachedMiniElixirRuntime::new();
        let mut ctx = EvaluationContext::new();
        
        let result = runtime.eval_code("42", &mut ctx).unwrap();
        assert_eq!(result, EllexValue::Number(42.0));
    }
    
    #[test]
    fn test_caching_deterministic_expressions() {
        let mut runtime = CachedMiniElixirRuntime::new();
        let mut ctx = EvaluationContext::new();
        
        // First evaluation
        let result1 = runtime.eval_code("1 + 2", &mut ctx).unwrap();
        
        // Second evaluation (should hit cache)
        let result2 = runtime.eval_code("1 + 2", &mut ctx).unwrap();
        
        assert_eq!(result1, result2);
        assert_eq!(result1, EllexValue::Number(3.0));
        
        let stats = runtime.performance_stats();
        assert!(stats.cache_hit_rate > 0.0);
    }
    
    #[test]
    fn test_variable_context_in_caching() {
        let mut runtime = CachedMiniElixirRuntime::new();
        let mut ctx = EvaluationContext::new();
        
        ctx.bind("x".to_string(), EllexValue::Number(5.0));
        let result1 = runtime.eval_code("x + 1", &mut ctx).unwrap();
        
        ctx.bind("x".to_string(), EllexValue::Number(10.0));
        let result2 = runtime.eval_code("x + 1", &mut ctx).unwrap();
        
        assert_eq!(result1, EllexValue::Number(6.0));
        assert_eq!(result2, EllexValue::Number(11.0));
    }
    
    #[test]
    fn test_block_evaluation() {
        let mut runtime = CachedMiniElixirRuntime::new();
        let mut ctx = EvaluationContext::new();
        
        let exprs = vec![
            runtime.interpreter.parse("1 + 2").unwrap(),
            runtime.interpreter.parse("3 * 4").unwrap(),
            runtime.interpreter.parse("10").unwrap(),
        ];
        
        let result = runtime.eval_block(&exprs, &mut ctx).unwrap();
        assert_eq!(result, EllexValue::Number(10.0)); // Last expression
    }
    
    #[test]
    fn test_cache_analysis() {
        let mut runtime = CachedMiniElixirRuntime::new();
        let mut ctx = EvaluationContext::new();
        
        // Execute some expressions multiple times
        for _ in 0..15 {
            runtime.eval_code("1 + 2", &mut ctx).unwrap();
        }
        
        for _ in 0..5 {
            runtime.eval_code("3 * 4", &mut ctx).unwrap();
        }
        
        let analysis = runtime.cache_analysis();
        assert!(analysis.hot_expressions.len() >= 1);
        assert!(analysis.cold_expressions.len() >= 1);
    }
}