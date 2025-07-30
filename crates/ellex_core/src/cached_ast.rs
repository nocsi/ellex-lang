use crate::values::{EllexValue, EllexFunction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Type information cached for optimized execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CachedType {
    String,
    Number,
    List,
    Function,
    Nil,
}

impl From<&EllexValue> for CachedType {
    fn from(value: &EllexValue) -> Self {
        match value {
            EllexValue::String(_) => CachedType::String,
            EllexValue::Number(_) => CachedType::Number,
            EllexValue::List(_) => CachedType::List,
            EllexValue::Function(_) => CachedType::Function,
            EllexValue::Nil => CachedType::Nil,
        }
    }
}

impl From<&mut EllexValue> for CachedType {
    fn from(value: &mut EllexValue) -> Self {
        CachedType::from(&*value)
    }
}

/// Cache entry for inline caching optimization
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub expected_type: CachedType,
    pub hit_count: u64,
    pub last_value: Option<EllexValue>, // For constant folding
}

impl CacheEntry {
    pub fn new(expected_type: CachedType) -> Self {
        Self {
            expected_type,
            hit_count: 0,
            last_value: None,
        }
    }

    pub fn hit(&mut self) {
        self.hit_count += 1;
    }

    pub fn matches(&self, value: &EllexValue) -> bool {
        CachedType::from(value) == self.expected_type
    }
}

/// Inline cache for variable access optimization
#[derive(Debug, Clone)]
pub struct VariableCache {
    pub variable_name: String,
    pub cache_entries: Vec<CacheEntry>,
    pub miss_count: u64,
}

impl VariableCache {
    pub fn new(variable_name: String) -> Self {
        Self {
            variable_name,
            cache_entries: Vec::new(),
            miss_count: 0,
        }
    }

    pub fn lookup(&mut self, value: &EllexValue) -> Option<&mut CacheEntry> {
        let value_type = CachedType::from(value);
        
        // Try to find existing cache entry
        for (i, entry) in self.cache_entries.iter_mut().enumerate() {
            if entry.expected_type == value_type {
                entry.hit();
                // Return a reference that doesn't conflict with the loop
                return self.cache_entries.get_mut(i);
            }
        }

        // Add new cache entry if we have space (limit to 3 entries for performance)
        if self.cache_entries.len() < 3 {
            let mut entry = CacheEntry::new(value_type);
            entry.hit();
            self.cache_entries.push(entry);
            self.cache_entries.last_mut()
        } else {
            // Cache miss - too many types (megamorphic)
            self.miss_count += 1;
            None
        }
    }

    pub fn is_monomorphic(&self) -> bool {
        self.cache_entries.len() == 1
    }

    pub fn is_megamorphic(&self) -> bool {
        self.miss_count > 10 || self.cache_entries.len() >= 3
    }
}

/// Function call cache for method dispatch optimization
#[derive(Debug, Clone)]
pub struct FunctionCallCache {
    pub function_name: String,
    pub cached_function: Option<EllexFunction>,
    pub argument_type_cache: Vec<CachedType>,
    pub hit_count: u64,
    pub version: u64, // For cache invalidation
}

impl FunctionCallCache {
    pub fn new(function_name: String) -> Self {
        Self {
            function_name,
            cached_function: None,
            argument_type_cache: Vec::new(),
            hit_count: 0,
            version: 0,
        }
    }

    pub fn matches_signature(&self, args: &[EllexValue]) -> bool {
        if args.len() != self.argument_type_cache.len() {
            return false;
        }

        args.iter()
            .zip(&self.argument_type_cache)
            .all(|(arg, &cached_type)| CachedType::from(arg) == cached_type)
    }

    pub fn cache_call(&mut self, function: EllexFunction, args: &[EllexValue]) {
        self.cached_function = Some(function);
        self.argument_type_cache = args.iter().map(CachedType::from).collect();
        self.hit_count += 1;
        self.version += 1;
    }
}

/// Optimized AST node with inline caching
#[derive(Debug, Clone)]
pub enum CachedStatement {
    Tell {
        value: EllexValue,
        cache: Option<CacheEntry>,
    },
    Ask {
        prompt: String,
        variable: String,
        type_hint: Option<String>,
        cache: Option<VariableCache>,
    },
    Repeat {
        times: u32,
        body: Vec<CachedStatement>,
        iteration_cache: Option<CacheEntry>, // Cache for loop variable types
    },
    When {
        variable: String,
        condition: EllexValue,
        then_body: Vec<CachedStatement>,
        else_body: Option<Vec<CachedStatement>>,
        condition_cache: Option<VariableCache>,
    },
    Call {
        function_name: String,
        args: Vec<EllexValue>,
        call_cache: Option<FunctionCallCache>,
    },
    // Optimized variants for common patterns
    TellConstant {
        value: EllexValue,
        // No cache needed - this is already optimized
    },
    VariableAccess {
        variable_name: String,
        cache: VariableCache,
    },
}

impl CachedStatement {
    /// Convert a regular Statement to a CachedStatement
    pub fn from_statement(stmt: crate::values::Statement) -> Self {
        match stmt {
            crate::values::Statement::Tell(value) => {
                // Check if this is a constant that can be pre-cached
                match value {
                    EllexValue::String(_) | EllexValue::Number(_) | EllexValue::Nil => {
                        CachedStatement::TellConstant { value }
                    }
                    _ => CachedStatement::Tell {
                        value,
                        cache: None,
                    }
                }
            }
            crate::values::Statement::Ask(variable, type_hint) => {
                CachedStatement::Ask {
                    prompt: format!("Enter value for {}", variable),
                    variable,
                    type_hint,
                    cache: None,
                }
            }
            crate::values::Statement::Repeat(times, body) => {
                let cached_body = body.into_iter()
                    .map(CachedStatement::from_statement)
                    .collect();
                CachedStatement::Repeat {
                    times,
                    body: cached_body,
                    iteration_cache: None,
                }
            }
            crate::values::Statement::When(variable, condition, then_body, else_body) => {
                let cached_then = then_body.into_iter()
                    .map(CachedStatement::from_statement)
                    .collect();
                let cached_else = else_body.map(|body| {
                    body.into_iter()
                        .map(CachedStatement::from_statement)
                        .collect()
                });
                CachedStatement::When {
                    variable,
                    condition,
                    then_body: cached_then,
                    else_body: cached_else,
                    condition_cache: None,
                }
            }
            crate::values::Statement::Call(function_name) => {
                CachedStatement::Call {
                    function_name,
                    args: Vec::new(),
                    call_cache: None,
                }
            }
        }
    }

    /// Warm up caches based on execution patterns
    pub fn warm_cache(&mut self, variables: &HashMap<String, EllexValue>) {
        match self {
            CachedStatement::Tell { value, cache } => {
                if cache.is_none() {
                    *cache = Some(CacheEntry::new(CachedType::from(&*value)));
                }
            }
            CachedStatement::VariableAccess { variable_name, cache } => {
                if let Some(var_value) = variables.get(variable_name) {
                    cache.lookup(var_value);
                }
            }
            CachedStatement::When { variable, condition_cache, .. } => {
                if condition_cache.is_none() {
                    *condition_cache = Some(VariableCache::new(variable.clone()));
                }
                if let (Some(cache), Some(var_value)) = (condition_cache.as_mut(), variables.get(variable)) {
                    cache.lookup(var_value);
                }
            }
            CachedStatement::Repeat { body, .. } => {
                for stmt in body {
                    stmt.warm_cache(variables);
                }
            }
            _ => {}
        }
    }

    /// Get cache statistics for performance monitoring
    pub fn cache_stats(&self) -> CacheStats {
        let mut stats = CacheStats::default();
        self.collect_cache_stats(&mut stats);
        stats
    }

    fn collect_cache_stats(&self, stats: &mut CacheStats) {
        match self {
            CachedStatement::Tell { cache, .. } => {
                if let Some(cache_entry) = cache {
                    stats.total_caches += 1;
                    stats.total_hits += cache_entry.hit_count;
                }
            }
            CachedStatement::VariableAccess { cache, .. } => {
                stats.total_caches += 1;
                stats.total_hits += cache.cache_entries.iter().map(|e| e.hit_count).sum::<u64>();
                stats.total_misses += cache.miss_count;
                if cache.is_monomorphic() {
                    stats.monomorphic_sites += 1;
                } else if cache.is_megamorphic() {
                    stats.megamorphic_sites += 1;
                }
            }
            CachedStatement::Call { call_cache, .. } => {
                if let Some(cache) = call_cache {
                    stats.total_caches += 1;
                    stats.total_hits += cache.hit_count;
                }
            }
            CachedStatement::Repeat { body, .. } => {
                for stmt in body {
                    stmt.collect_cache_stats(stats);
                }
            }
            CachedStatement::When { then_body, else_body, condition_cache, .. } => {
                if let Some(cache) = condition_cache {
                    stats.total_caches += 1;
                    stats.total_hits += cache.cache_entries.iter().map(|e| e.hit_count).sum::<u64>();
                    stats.total_misses += cache.miss_count;
                }
                for stmt in then_body {
                    stmt.collect_cache_stats(stats);
                }
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        stmt.collect_cache_stats(stats);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Statistics for cache performance monitoring
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub total_caches: u64,
    pub total_hits: u64,
    pub total_misses: u64,
    pub monomorphic_sites: u64,
    pub megamorphic_sites: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_hits + self.total_misses == 0 {
            0.0
        } else {
            self.total_hits as f64 / (self.total_hits + self.total_misses) as f64
        }
    }

    pub fn cache_efficiency(&self) -> f64 {
        if self.total_caches == 0 {
            0.0
        } else {
            self.monomorphic_sites as f64 / self.total_caches as f64
        }
    }
}

/// Global cache version counter for invalidation
static GLOBAL_CACHE_VERSION: AtomicU64 = AtomicU64::new(0);

/// Invalidate all caches (useful when global state changes)
pub fn invalidate_all_caches() {
    GLOBAL_CACHE_VERSION.fetch_add(1, Ordering::Relaxed);
}

pub fn current_cache_version() -> u64 {
    GLOBAL_CACHE_VERSION.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_basics() {
        let mut cache = CacheEntry::new(CachedType::String);
        let value = EllexValue::String("test".to_string());
        
        assert!(cache.matches(&value));
        assert_eq!(cache.hit_count, 0);
        
        cache.hit();
        assert_eq!(cache.hit_count, 1);
    }

    #[test]
    fn test_variable_cache() {
        let mut cache = VariableCache::new("test_var".to_string());
        let string_value = EllexValue::String("hello".to_string());
        let number_value = EllexValue::Number(42.0);
        
        // First access - should create cache entry
        assert!(cache.lookup(&string_value).is_some());
        assert!(cache.is_monomorphic());
        
        // Second access with same type - should hit cache
        let entry = cache.lookup(&string_value).unwrap();
        assert_eq!(entry.hit_count, 2);
        
        // Different type - should create second entry
        assert!(cache.lookup(&number_value).is_some());
        assert!(!cache.is_monomorphic());
        assert!(!cache.is_megamorphic());
    }

    #[test]
    fn test_function_call_cache() {
        let mut cache = FunctionCallCache::new("test_func".to_string());
        let args = vec![EllexValue::String("arg1".to_string())];
        let function = EllexFunction {
            name: "test_func".to_string(),
            body: vec![],
            params: vec!["param1".to_string()],
        };
        
        assert!(!cache.matches_signature(&args));
        
        cache.cache_call(function.clone(), &args);
        assert!(cache.matches_signature(&args));
        assert_eq!(cache.hit_count, 1);
    }

    #[test]
    fn test_cached_statement_conversion() {
        let original = crate::values::Statement::Tell(EllexValue::String("test".to_string()));
        let cached = CachedStatement::from_statement(original);
        
        match cached {
            CachedStatement::TellConstant { value } => {
                assert_eq!(value, EllexValue::String("test".to_string()));
            }
            _ => panic!("Expected TellConstant variant"),
        }
    }

    #[test]
    fn test_cache_stats() {
        let mut stmt = CachedStatement::VariableAccess {
            variable_name: "test".to_string(),
            cache: VariableCache::new("test".to_string()),
        };
        
        let mut vars = HashMap::new();
        vars.insert("test".to_string(), EllexValue::String("value".to_string()));
        
        stmt.warm_cache(&vars);
        
        let stats = stmt.cache_stats();
        assert!(stats.total_hits > 0);
        assert_eq!(stats.monomorphic_sites, 1);
    }
}