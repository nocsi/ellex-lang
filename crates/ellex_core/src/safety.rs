use serde::{Deserialize, Serialize};
use std::time::Instant;
use thiserror::Error;

/// Safety-related errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum SafetyError {
    #[error("Execution timeout: exceeded {limit_ms}ms")]
    ExecutionTimeout { limit_ms: u64 },
    
    #[error("Memory limit exceeded: {current_mb}MB > {limit_mb}MB")]
    MemoryLimitExceeded { current_mb: usize, limit_mb: usize },
    
    #[error("Recursion depth exceeded: {current} > {limit}")]
    RecursionDepthExceeded { current: usize, limit: usize },
    
    #[error("Loop iteration limit exceeded: {current} > {limit}")]
    LoopLimitExceeded { current: usize, limit: usize },
    
    #[error("Instruction count exceeded: {count}")]
    InstructionLimitExceeded { count: u64 },
}

/// Execution limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLimits {
    pub timeout_ms: u64,
    pub memory_limit_mb: usize,
    pub max_recursion_depth: usize,
    pub max_loop_iterations: usize,
    pub max_instructions: u64,
}

impl ExecutionLimits {
    pub fn new() -> Self {
        ExecutionLimits {
            timeout_ms: 5000,          // 5 seconds
            memory_limit_mb: 64,       // 64 MB
            max_recursion_depth: 100,  // 100 levels
            max_loop_iterations: 10000, // 10k iterations
            max_instructions: 100000,   // 100k instructions
        }
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn with_memory_limit(mut self, memory_limit_mb: usize) -> Self {
        self.memory_limit_mb = memory_limit_mb;
        self
    }

    pub fn with_recursion_limit(mut self, max_recursion_depth: usize) -> Self {
        self.max_recursion_depth = max_recursion_depth;
        self
    }

    pub fn with_loop_limit(mut self, max_loop_iterations: usize) -> Self {
        self.max_loop_iterations = max_loop_iterations;
        self
    }

    pub fn for_beginners() -> Self {
        ExecutionLimits {
            timeout_ms: 3000,          // 3 seconds
            memory_limit_mb: 32,       // 32 MB
            max_recursion_depth: 50,   // 50 levels
            max_loop_iterations: 1000, // 1k iterations
            max_instructions: 10000,   // 10k instructions
        }
    }

    pub fn for_advanced() -> Self {
        ExecutionLimits {
            timeout_ms: 30000,         // 30 seconds
            memory_limit_mb: 256,      // 256 MB
            max_recursion_depth: 500,  // 500 levels
            max_loop_iterations: 100000, // 100k iterations
            max_instructions: 1000000, // 1M instructions
        }
    }
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracks execution safety and enforces limits
#[derive(Debug)]
pub struct SafetyMonitor {
    limits: ExecutionLimits,
    start_time: Instant,
    instruction_count: u64,
    recursion_depth: usize,
    memory_usage_mb: usize,
}

impl SafetyMonitor {
    pub fn new(limits: ExecutionLimits) -> Self {
        SafetyMonitor {
            limits,
            start_time: Instant::now(),
            instruction_count: 0,
            recursion_depth: 0,
            memory_usage_mb: 0,
        }
    }

    /// Reset the monitor for a new execution
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.instruction_count = 0;
        self.recursion_depth = 0;
        self.memory_usage_mb = 0;
    }

    /// Check if execution can start
    pub fn check_execution_start(&mut self) -> Result<(), SafetyError> {
        self.start_time = Instant::now();
        self.instruction_count = 0;
        Ok(())
    }

    /// Check if execution can continue (called before each statement)
    pub fn check_execution_continue(&mut self) -> Result<(), SafetyError> {
        self.instruction_count += 1;
        
        // Check instruction limit
        if self.instruction_count > self.limits.max_instructions {
            return Err(SafetyError::InstructionLimitExceeded {
                count: self.instruction_count,
            });
        }
        
        // Check timeout
        let elapsed_ms = self.start_time.elapsed().as_millis() as u64;
        if elapsed_ms > self.limits.timeout_ms {
            return Err(SafetyError::ExecutionTimeout {
                limit_ms: self.limits.timeout_ms,
            });
        }
        
        // Check memory usage (approximate)
        if self.memory_usage_mb > self.limits.memory_limit_mb {
            return Err(SafetyError::MemoryLimitExceeded {
                current_mb: self.memory_usage_mb,
                limit_mb: self.limits.memory_limit_mb,
            });
        }
        
        Ok(())
    }

    /// Check recursion depth before function call
    pub fn check_recursion_depth(&mut self) -> Result<(), SafetyError> {
        self.recursion_depth += 1;
        if self.recursion_depth > self.limits.max_recursion_depth {
            return Err(SafetyError::RecursionDepthExceeded {
                current: self.recursion_depth,
                limit: self.limits.max_recursion_depth,
            });
        }
        Ok(())
    }

    /// Called when returning from a function
    pub fn exit_recursion(&mut self) {
        if self.recursion_depth > 0 {
            self.recursion_depth -= 1;
        }
    }

    /// Check before starting a loop
    pub fn check_loop_start(&self, iterations: usize) -> Result<(), SafetyError> {
        if iterations > self.limits.max_loop_iterations {
            return Err(SafetyError::LoopLimitExceeded {
                current: iterations,
                limit: self.limits.max_loop_iterations,
            });
        }
        Ok(())
    }

    /// Check during loop iteration
    pub fn check_loop_iteration(&mut self, iteration: usize) -> Result<(), SafetyError> {
        if iteration >= self.limits.max_loop_iterations {
            return Err(SafetyError::LoopLimitExceeded {
                current: iteration,
                limit: self.limits.max_loop_iterations,
            });
        }
        
        // Also check general execution limits
        self.check_execution_continue()
    }

    /// Update memory usage estimate
    pub fn update_memory_usage(&mut self, mb: usize) {
        self.memory_usage_mb = mb;
    }

    /// Get current execution stats
    pub fn get_stats(&self) -> SafetyStats {
        SafetyStats {
            elapsed_ms: self.start_time.elapsed().as_millis() as u64,
            instruction_count: self.instruction_count,
            recursion_depth: self.recursion_depth,
            memory_usage_mb: self.memory_usage_mb,
            limits: self.limits.clone(),
        }
    }

    /// Check if execution is getting close to limits (for warnings)
    pub fn check_warning_thresholds(&self) -> Vec<SafetyWarning> {
        let mut warnings = Vec::new();
        let stats = self.get_stats();
        
        // Time warning at 80% of limit
        if stats.elapsed_ms > (self.limits.timeout_ms * 80 / 100) {
            warnings.push(SafetyWarning::NearTimeout {
                current_ms: stats.elapsed_ms,
                limit_ms: self.limits.timeout_ms,
            });
        }
        
        // Memory warning at 80% of limit
        if self.memory_usage_mb > (self.limits.memory_limit_mb * 80 / 100) {
            warnings.push(SafetyWarning::NearMemoryLimit {
                current_mb: self.memory_usage_mb,
                limit_mb: self.limits.memory_limit_mb,
            });
        }
        
        // Recursion warning at 90% of limit
        if self.recursion_depth > (self.limits.max_recursion_depth * 90 / 100) {
            warnings.push(SafetyWarning::DeepRecursion {
                current: self.recursion_depth,
                limit: self.limits.max_recursion_depth,
            });
        }
        
        warnings
    }

    /// Get execution limits
    pub fn get_limits(&self) -> &ExecutionLimits {
        &self.limits
    }

    /// Update execution limits
    pub fn update_limits(&mut self, limits: ExecutionLimits) {
        self.limits = limits;
    }
}

/// Current execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyStats {
    pub elapsed_ms: u64,
    pub instruction_count: u64,
    pub recursion_depth: usize,
    pub memory_usage_mb: usize,
    pub limits: ExecutionLimits,
}

impl SafetyStats {
    /// Calculate percentage of timeout used
    pub fn timeout_percentage(&self) -> f64 {
        (self.elapsed_ms as f64 / self.limits.timeout_ms as f64) * 100.0
    }

    /// Calculate percentage of memory used
    pub fn memory_percentage(&self) -> f64 {
        (self.memory_usage_mb as f64 / self.limits.memory_limit_mb as f64) * 100.0
    }

    /// Calculate percentage of recursion depth used
    pub fn recursion_percentage(&self) -> f64 {
        (self.recursion_depth as f64 / self.limits.max_recursion_depth as f64) * 100.0
    }

    /// Check if any metric is above warning threshold (80%)
    pub fn has_warnings(&self) -> bool {
        self.timeout_percentage() > 80.0
            || self.memory_percentage() > 80.0
            || self.recursion_percentage() > 90.0
    }
}

/// Safety warnings (non-fatal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyWarning {
    NearTimeout { current_ms: u64, limit_ms: u64 },
    NearMemoryLimit { current_mb: usize, limit_mb: usize },
    DeepRecursion { current: usize, limit: usize },
}

impl SafetyWarning {
    /// Get a kid-friendly warning message
    pub fn friendly_message(&self) -> String {
        match self {
            SafetyWarning::NearTimeout { current_ms, limit_ms } => {
                format!("⏰ Your code is taking a while ({}/{}ms). Try to make it simpler!", current_ms, limit_ms)
            }
            SafetyWarning::NearMemoryLimit { current_mb, limit_mb } => {
                format!("💾 Your code is using lots of memory ({}/{}MB). Try using fewer variables!", current_mb, limit_mb)
            }
            SafetyWarning::DeepRecursion { current, limit } => {
                format!("🔄 Your functions are calling each other a lot ({}/{}). Watch out for infinite loops!", current, limit)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_execution_limits_creation() {
        let limits = ExecutionLimits::new();
        assert_eq!(limits.timeout_ms, 5000);
        assert_eq!(limits.memory_limit_mb, 64);
        assert_eq!(limits.max_recursion_depth, 100);
        assert_eq!(limits.max_loop_iterations, 10000);
    }

    #[test]
    fn test_safety_monitor_basic() {
        let limits = ExecutionLimits::new();
        let mut monitor = SafetyMonitor::new(limits);
        
        // Should start successfully
        assert!(monitor.check_execution_start().is_ok());
        
        // Should continue successfully for a while
        for _ in 0..100 {
            assert!(monitor.check_execution_continue().is_ok());
        }
    }

    #[test]
    fn test_instruction_limit() {
        let limits = ExecutionLimits::new().with_timeout(60000); // Long timeout
        let mut monitor = SafetyMonitor::new(limits);
        
        monitor.check_execution_start().unwrap();
        
        // Should eventually hit instruction limit
        for i in 0..200000 {
            match monitor.check_execution_continue() {
                Ok(()) => continue,
                Err(SafetyError::InstructionLimitExceeded { .. }) => {
                    assert!(i > 100000); // Should hit after max_instructions
                    return;
                }
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }
        panic!("Should have hit instruction limit");
    }

    #[test]
    fn test_recursion_depth() {
        let limits = ExecutionLimits::new();
        let mut monitor = SafetyMonitor::new(limits);
        
        // Should allow recursion up to limit
        for _ in 0..100 {
            assert!(monitor.check_recursion_depth().is_ok());
        }
        
        // Should fail on next call
        assert!(matches!(
            monitor.check_recursion_depth(),
            Err(SafetyError::RecursionDepthExceeded { .. })
        ));
    }

    #[test]
    fn test_loop_limits() {
        let limits = ExecutionLimits::new();
        let monitor = SafetyMonitor::new(limits);
        
        // Should reject loops that are too large
        assert!(matches!(
            monitor.check_loop_start(20000),
            Err(SafetyError::LoopLimitExceeded { .. })
        ));
        
        // Should accept reasonable loops
        assert!(monitor.check_loop_start(100).is_ok());
    }

    #[test]
    fn test_memory_tracking() {
        let limits = ExecutionLimits::new();
        let mut monitor = SafetyMonitor::new(limits);
        
        monitor.update_memory_usage(100); // Over limit
        
        assert!(matches!(
            monitor.check_execution_continue(),
            Err(SafetyError::MemoryLimitExceeded { .. })
        ));
    }

    #[test]
    fn test_safety_stats() {
        let limits = ExecutionLimits::new();
        let mut monitor = SafetyMonitor::new(limits);
        
        monitor.check_execution_start().unwrap();
        for _ in 0..50 {
            monitor.check_execution_continue().unwrap();
        }
        
        let stats = monitor.get_stats();
        assert_eq!(stats.instruction_count, 50);
        assert!(stats.elapsed_ms < 1000); // Should be very fast
    }

    #[test]
    fn test_warning_thresholds() {
        let limits = ExecutionLimits::new();
        let mut monitor = SafetyMonitor::new(limits);
        
        // Set memory to warning threshold
        monitor.update_memory_usage(52); // 52/64 = ~81%
        
        let warnings = monitor.check_warning_thresholds();
        assert!(!warnings.is_empty());
        assert!(matches!(warnings[0], SafetyWarning::NearMemoryLimit { .. }));
    }

    #[test]
    fn test_beginner_vs_advanced_limits() {
        let beginner = ExecutionLimits::for_beginners();
        let advanced = ExecutionLimits::for_advanced();
        
        assert!(beginner.timeout_ms < advanced.timeout_ms);
        assert!(beginner.memory_limit_mb < advanced.memory_limit_mb);
        assert!(beginner.max_recursion_depth < advanced.max_recursion_depth);
        assert!(beginner.max_loop_iterations < advanced.max_loop_iterations);
    }

    #[test]
    fn test_reset_functionality() {
        let limits = ExecutionLimits::new();
        let mut monitor = SafetyMonitor::new(limits);
        
        // Do some work
        monitor.check_execution_start().unwrap();
        for _ in 0..100 {
            monitor.check_execution_continue().unwrap();
        }
        monitor.check_recursion_depth().unwrap();
        
        let stats_before = monitor.get_stats();
        assert!(stats_before.instruction_count > 0);
        assert!(stats_before.recursion_depth > 0);
        
        // Reset
        monitor.reset();
        
        let stats_after = monitor.get_stats();
        assert_eq!(stats_after.instruction_count, 0);
        assert_eq!(stats_after.recursion_depth, 0);
    }
}