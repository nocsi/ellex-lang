//! # Ellex Core
//! 
//! The core runtime for the Ellex programming language - a natural language
//! programming environment for kids with AI assistance and modal editing.
//!
//! ## Features
//!
//! - Safe execution environment with timeouts and memory limits
//! - Built-in turtle graphics for visual programming
//! - Real-time code evaluation
//! - Natural language syntax for beginners
//! - Modal programming interface
//! - AI-powered learning assistance
//!
//! ## Example
//!
//! ```rust
//! use ellex_core::{EllexRuntime, EllexValue, Statement};
//!
//! let mut runtime = EllexRuntime::new();
//! let statements = vec![
//!     Statement::Tell(EllexValue::String("Hello, world!".to_string())),
//! ];
//! runtime.execute(statements).unwrap();
//! ```

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod runtime;
pub mod values;
pub mod safety;
pub mod turtle;

pub use runtime::{EllexRuntime, Pipeline};
pub use values::{EllexValue, EllexFunction, Statement};
pub use safety::{SafetyMonitor, ExecutionLimits};
pub use turtle::TurtleGraphics;

/// Main error types for Ellex execution
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum EllexError {
    #[error("Execution timeout: code took longer than {limit_ms}ms")]
    Timeout { limit_ms: u64 },
    
    #[error("Parse error at line {line}, column {column}: {message}")]
    ParseError { line: usize, column: usize, message: String },
    
    #[error("I don't understand '{input}'. {suggestion}")]
    UnknownCommand { input: String, suggestion: String },
    
    #[error("That doesn't make sense here: {message}")]
    LogicError { message: String },
    
    #[error("Safety first! {reason}")]
    SafetyViolation { reason: String },
}

/// Result type for Ellex operations
pub type EllexResult<T> = Result<T, EllexError>;

/// Configuration for the Ellex runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EllexConfig {
    /// Maximum execution time in milliseconds
    pub execution_timeout_ms: u64,
    
    /// Maximum memory usage in MB
    pub memory_limit_mb: usize,
    
    /// Whether to enable turtle graphics
    pub enable_turtle: bool,
    
    /// Whether to enable AI assistance
    pub enable_ai: bool,
    
    /// Maximum recursion depth
    pub max_recursion_depth: usize,
    
    /// Maximum loop iterations
    pub max_loop_iterations: usize,
}

impl Default for EllexConfig {
    fn default() -> Self {
        Self {
            execution_timeout_ms: 5000,  // 5 seconds
            memory_limit_mb: 64,         // 64 MB
            enable_turtle: true,
            enable_ai: true,
            max_recursion_depth: 100,
            max_loop_iterations: 10000,
        }
    }
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build information
pub const BUILD_INFO: &str = concat!(
    "Ellex v", env!("CARGO_PKG_VERSION"),
    " (", env!("CARGO_PKG_REPOSITORY"), ")"
);

/// Initialize the Ellex runtime with default configuration
pub fn init() -> EllexRuntime {
    EllexRuntime::new()
}

/// Initialize the Ellex runtime with custom configuration
pub fn init_with_config(config: EllexConfig) -> EllexRuntime {
    EllexRuntime::with_config(config)
}

/// Create a friendly error message for kids
pub fn friendly_error_message(error: &EllexError) -> String {
    match error {
        EllexError::Timeout { .. } => {
            "Whoa! Your code is taking a really long time. Let's try something simpler! 🐌".to_string()
        }
        EllexError::ParseError { message, .. } => {
            format!("I didn't understand that. {} 🤔", message)
        }
        EllexError::UnknownCommand { input, suggestion } => {
            format!("I don't know how to '{}'. {} 💡", input, suggestion)
        }
        EllexError::LogicError { message } => {
            format!("That doesn't make sense: {} 🤷", message)
        }
        EllexError::SafetyViolation { reason } => {
            format!("Safety first! {} 🛡️", reason)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellex_config_default() {
        let config = EllexConfig::default();
        assert_eq!(config.execution_timeout_ms, 5000);
        assert_eq!(config.memory_limit_mb, 64);
        assert!(config.enable_turtle);
        assert!(config.enable_ai);
    }

    #[test]
    fn test_friendly_error_messages() {
        let timeout_error = EllexError::Timeout { limit_ms: 5000 };
        let friendly = friendly_error_message(&timeout_error);
        assert!(friendly.contains("long time"));
        assert!(friendly.contains("🐌"));
    }
}
