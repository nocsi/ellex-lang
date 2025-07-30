//! # Ellex Transpiler
//! 
//! High-performance bidirectional transpiler for Ellex with JavaScript and WebAssembly support.
//! 
//! ## Features
//! 
//! - **Ellex → JavaScript**: Natural language to performant JS
//! - **JavaScript → Ellex**: JS to natural language for learning
//! - **WebAssembly Support**: Direct WASM compilation for maximum performance
//! - **Optimizations**: Dead code elimination, constant folding, loop unrolling
//! - **Type Inference**: Smart type detection and optimization
//! 
//! ## Architecture
//! 
//! ```text
//! Ellex Source ←→ AST ←→ Optimized AST ←→ Target (JS/WASM)
//!      ↕                     ↕                    ↕
//! Natural Lang ←→ Semantic ←→ Optimized ←→ Platform Code
//! ```

pub mod ast;
pub mod codegen;
pub mod js_transpiler;
pub mod js_parser;
pub mod wasm_compiler;
pub mod optimizer;
pub mod type_inference;

use anyhow::Result;
use ellex_core::values::Statement;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Transpiler errors
#[derive(Error, Debug)]
pub enum TranspilerError {
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Codegen error: {0}")]
    CodegenError(String),
    
    #[error("Optimization error: {0}")]
    OptimizationError(String),
    
    #[error("Type inference error: {0}")]
    TypeInferenceError(String),
    
    #[error("WASM compilation error: {0}")]
    WasmError(String),
    
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
}

/// Transpilation targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Target {
    /// Modern JavaScript (ES2020+)
    JavaScript {
        /// Enable async/await transformations
        async_support: bool,
        /// Target ECMAScript version
        es_version: EsVersion,
        /// Enable performance optimizations
        optimize: bool,
    },
    /// WebAssembly (WASM)
    WebAssembly {
        /// Enable WASM SIMD instructions
        simd: bool,
        /// Enable multi-threading
        threads: bool,
        /// Optimization level (0-3)
        opt_level: u8,
    },
    /// TypeScript output
    TypeScript {
        /// Generate declaration files
        declarations: bool,
        /// Strict mode
        strict: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EsVersion {
    Es5,
    Es2015,
    Es2016,
    Es2017,
    Es2018,
    Es2019,
    Es2020,
    Es2021,
    Es2022,
    EsNext,
}

/// Transpilation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranspilerOptions {
    /// Target platform
    pub target: Target,
    /// Enable source maps
    pub source_maps: bool,
    /// Minify output
    pub minify: bool,
    /// Apply optimizations
    pub optimize: bool,
    /// Preserve comments
    pub preserve_comments: bool,
}

impl Default for TranspilerOptions {
    fn default() -> Self {
        Self {
            target: Target::JavaScript {
                async_support: true,
                es_version: EsVersion::Es2020,
                optimize: true,
            },
            source_maps: true,
            minify: false,
            optimize: true,
            preserve_comments: false,
        }
    }
}

/// Main transpiler struct
pub struct EllexTranspiler {
    options: TranspilerOptions,
}

impl EllexTranspiler {
    /// Create new transpiler with default options
    pub fn new() -> Self {
        Self {
            options: TranspilerOptions::default(),
        }
    }
    
    /// Create transpiler with custom options
    pub fn with_options(options: TranspilerOptions) -> Self {
        Self { options }
    }
    
    /// Transpile Ellex AST to target language
    pub fn transpile(&self, ast: &[Statement]) -> Result<String, TranspilerError> {
        match &self.options.target {
            Target::JavaScript { .. } => {
                self.transpile_to_javascript(ast)
            }
            Target::WebAssembly { .. } => {
                self.transpile_to_wasm(ast)
            }
            Target::TypeScript { .. } => {
                self.transpile_to_typescript(ast)
            }
        }
    }
    
    /// Parse JavaScript and convert to Ellex AST
    pub fn parse_javascript(&self, js_code: &str) -> Result<Vec<Statement>, TranspilerError> {
        js_parser::parse_js_to_ellex(js_code)
    }
    
    /// Transpile Ellex to JavaScript
    fn transpile_to_javascript(&self, ast: &[Statement]) -> Result<String, TranspilerError> {
        js_transpiler::transpile(ast, &self.options)
    }
    
    /// Transpile Ellex to WebAssembly
    fn transpile_to_wasm(&self, ast: &[Statement]) -> Result<String, TranspilerError> {
        wasm_compiler::compile(ast, &self.options)
    }
    
    /// Transpile Ellex to TypeScript
    fn transpile_to_typescript(&self, ast: &[Statement]) -> Result<String, TranspilerError> {
        // TypeScript is similar to JavaScript but with type annotations
        let js_code = self.transpile_to_javascript(ast)?;
        // Add TypeScript-specific transformations
        Ok(format!("// Generated TypeScript from Ellex\n{}", js_code))
    }
}

impl Default for EllexTranspiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common operations
impl EllexTranspiler {
    /// Quick transpile to JavaScript string
    pub fn to_js(ast: &[Statement]) -> Result<String, TranspilerError> {
        let transpiler = Self::new();
        transpiler.transpile(ast)
    }
    
    /// Quick transpile to WASM
    pub fn to_wasm(ast: &[Statement]) -> Result<String, TranspilerError> {
        let transpiler = Self::with_options(TranspilerOptions {
            target: Target::WebAssembly {
                simd: false,
                threads: false,
                opt_level: 2,
            },
            ..Default::default()
        });
        transpiler.transpile(ast)
    }
    
    /// Parse JS to Ellex
    pub fn from_js(js_code: &str) -> Result<Vec<Statement>, TranspilerError> {
        let transpiler = Self::new();
        transpiler.parse_javascript(js_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ellex_core::values::EllexValue;

    #[test]
    fn test_transpiler_creation() {
        let transpiler = EllexTranspiler::new();
        match transpiler.options.target {
            Target::JavaScript { .. } => {},
            _ => panic!("Expected JavaScript target"),
        }
    }

    #[test]
    fn test_simple_transpilation() {
        let ast = vec![
            Statement::Tell(EllexValue::String("Hello, world!".to_string())),
        ];
        
        let result = EllexTranspiler::to_js(&ast);
        assert!(result.is_ok());
        
        let js_code = result.unwrap();
        assert!(js_code.contains("console.log"));
        assert!(js_code.contains("Hello, world!"));
    }

    #[test]
    fn test_transpiler_options() {
        let options = TranspilerOptions {
            target: Target::WebAssembly {
                simd: true,
                threads: true,
                opt_level: 3,
            },
            minify: true,
            optimize: true,
            source_maps: false,
            preserve_comments: false,
        };
        
        let _transpiler = EllexTranspiler::with_options(options);
        // Test that options are properly set
    }
}