# Ellex API Reference

## Overview

This document provides comprehensive API documentation for the Ellex programming language runtime and libraries.

## Core Modules

### `ellex_core`

The core runtime module providing the fundamental Ellex execution environment.

#### `EllexRuntime`

The main runtime executor for Ellex programs.

**Creation:**
```rust
use ellex_core::EllexRuntime;

let mut runtime = EllexRuntime::new();
let mut runtime_with_config = EllexRuntime::with_config(config);
```

**Methods:**
- `new() -> Self`: Create a new runtime with default configuration
- `with_config(config: EllexConfig) -> Self`: Create runtime with custom configuration
- `execute(&mut self, statements: Vec<Statement>) -> Result<()>`: Execute a list of statements

#### `EllexConfig`

Configuration structure for runtime behavior.

**Fields:**
```rust
pub struct EllexConfig {
    pub execution_timeout_ms: u64,    // Default: 5000
    pub memory_limit_mb: usize,       // Default: 64
    pub enable_turtle: bool,          // Default: true
    pub enable_ai: bool,              // Default: true
    pub max_recursion_depth: usize,   // Default: 100
    pub max_loop_iterations: usize,   // Default: 10000
}
```

**Example:**
```rust
use ellex_core::EllexConfig;

let config = EllexConfig {
    execution_timeout_ms: 10000,
    memory_limit_mb: 128,
    enable_turtle: true,
    enable_ai: false,
    max_recursion_depth: 50,
    max_loop_iterations: 5000,
};
```

#### `EllexValue`

Represents values in the Ellex runtime.

**Variants:**
```rust
pub enum EllexValue {
    String(String),
    Number(f64),
    List(Vec<EllexValue>),
    Function(EllexFunction),
    Nil,
}
```

**Methods:**
- `from_str(s: &str) -> Self`: Create string value from literal
- `interpolate(&self, vars: &HashMap<String, EllexValue>) -> Self`: Interpolate variables in strings
- `to_string(&self) -> String`: Convert value to string representation

**Examples:**
```rust
use ellex_core::EllexValue;
use std::collections::HashMap;

// Create values
let text = EllexValue::String("Hello".to_string());
let number = EllexValue::Number(42.0);
let list = EllexValue::List(vec![
    EllexValue::Number(1.0),
    EllexValue::String("hello".to_string())
]);

// String interpolation
let mut vars = HashMap::new();
vars.insert("name".to_string(), EllexValue::String("Alice".to_string()));
let template = EllexValue::String("Hello, {name}!".to_string());
let result = template.interpolate(&vars); // "Hello, Alice!"
```

#### `Statement`

Represents executable statements in Ellex.

**Variants:**
```rust
pub enum Statement {
    Tell(EllexValue),                                           // Output statement
    Ask(String, Option<String>),                               // Input statement with optional type hint
    Repeat(u32, Vec<Statement>),                               // Loop statement
    When(String, EllexValue, Vec<Statement>, Option<Vec<Statement>>), // Conditional
    Call(String),                                              // Function call
}
```

#### `EllexFunction`

Represents user-defined functions.

**Fields:**
```rust
pub struct EllexFunction {
    pub name: String,
    pub body: Vec<Statement>,
    pub params: Vec<String>,
}
```

#### `EllexError`

Error types for Ellex execution.

**Variants:**
```rust
pub enum EllexError {
    Timeout { limit_ms: u64 },
    ParseError { line: usize, column: usize, message: String },
    UnknownCommand { input: String, suggestion: String },
    LogicError { message: String },
    SafetyViolation { reason: String },
}
```

#### Safety System

**`SafetyMonitor`**
Monitors execution for safety violations.

**`ExecutionLimits`**
Defines execution constraints.

```rust
use ellex_core::{ExecutionLimits, SafetyMonitor};

let limits = ExecutionLimits::new();
let mut monitor = SafetyMonitor::new(limits);
```

### `ellex_parser`

The parser module for converting Ellex source code to AST.

#### Functions

**`parse(input: &str) -> Result<Vec<Statement>, pest::error::Error<Rule>>`**

Parse Ellex source code into a statement list.

```rust
use ellex_parser::parse;

let code = r#"tell "Hello, world!""#;
let statements = parse(code)?;
```

**`parse_and_optimize(input: &str, pipeline: &Pipeline) -> Result<Vec<Statement>>`**

Parse and apply optimization pipeline.

```rust
use ellex_parser::parse_and_optimize;
use ellex_core::Pipeline;

let code = r#"tell "Hello!""#;
let pipeline = Pipeline::new();
let optimized = parse_and_optimize(code, &pipeline)?;
```

### `ellex_repl`

REPL (Read-Eval-Print Loop) functionality.

#### Features
- Interactive command execution
- Command history
- Syntax highlighting
- Auto-completion
- Multi-line input support

### `ellex_web`

Web server functionality for browser-based Ellex playground.

#### Features
- REST API endpoints
- WebSocket support for real-time execution
- File upload/download
- Session management

### `ellex_ai`

AI assistance and pattern recognition.

#### Features
- Code completion suggestions
- Error explanation
- Learning path recommendations
- Pattern recognition
- Code optimization suggestions

### `ellex_cli`

Command-line interface for Ellex.

#### Commands

**`ellex repl`**
Start interactive REPL session.

```bash
ellex repl --ai=true
```

**`ellex run <file>`**
Execute an Ellex file.

```bash
ellex run my_program.ellex
```

**`ellex serve`**
Start web playground server.

```bash  
ellex serve --port 3000
```

**`ellex tui`**
Start TUI interface with real-time metrics.

```bash
ellex tui
```

## Compiler Pipeline

### `Pipeline`

Manages compilation passes for optimization.

```rust
use ellex_core::runtime::{Pipeline, ConstantFolding, LoopUnroll};

let mut pipeline = Pipeline::new();
pipeline.attach(Box::new(ConstantFolding));
pipeline.attach(Box::new(LoopUnroll::new(5)));
```

### Built-in Passes

**`ConstantFolding`**
Evaluates constant expressions at compile time.

**`DeadCodeElim`**
Removes unused variables and statements.

**`Inlining`**
Replaces function calls with function bodies.

**`LoopUnroll`**
Unrolls small loops for performance.

**`ElixirTranspile`**
Transpiles to Elixir code.

**`SuperOpt`**
AI-assisted optimization.

## Turtle Graphics

### `TurtleGraphics`

Built-in graphics system for visual programming.

**Commands:**
- `move forward N`: Move turtle forward
- `turn right/left degrees`: Turn turtle
- `use color name`: Set drawing color
- `draw circle with radius N`: Draw circle

**Example:**
```ellex
use color "blue"
repeat 4 times do
    move forward 100
    turn right 90
end
```

## Error Handling

### Friendly Error Messages

The `friendly_error_message(error: &EllexError) -> String` function converts technical errors into kid-friendly messages:

```rust
use ellex_core::{EllexError, friendly_error_message};

let error = EllexError::Timeout { limit_ms: 5000 };
let message = friendly_error_message(&error);
// "Whoa! Your code is taking a really long time. Let's try something simpler! 🐌"
```

## Integration Examples

### Basic Usage

```rust
use ellex_core::{EllexRuntime, EllexValue, Statement};

fn main() -> anyhow::Result<()> {
    let mut runtime = EllexRuntime::new();
    
    let statements = vec![
        Statement::Tell(EllexValue::String("Hello, world!".to_string())),
        Statement::Ask("name".to_string(), None),
        Statement::Tell(EllexValue::String("Nice to meet you!".to_string())),
    ];
    
    runtime.execute(statements)?;
    Ok(())
}
```

### Parsing and Execution

```rust
use ellex_core::EllexRuntime;
use ellex_parser::parse;

fn run_ellex_code(code: &str) -> anyhow::Result<()> {
    let mut runtime = EllexRuntime::new();
    let statements = parse(code)?;
    runtime.execute(statements)?;
    Ok(())
}
```

### Custom Configuration

```rust
use ellex_core::{EllexRuntime, EllexConfig};

let config = EllexConfig {
    execution_timeout_ms: 10000,
    memory_limit_mb: 128,
    enable_turtle: false,
    enable_ai: true,
    max_recursion_depth: 200,
    max_loop_iterations: 20000,
};

let runtime = EllexRuntime::with_config(config);
```

## Testing

### Unit Test Helpers

The crates include comprehensive test utilities:

```rust
#[cfg(test)]
mod tests {
    use ellex_core::{EllexValue, Statement};
    use ellex_parser::parse;

    #[test]
    fn test_simple_program() {
        let code = r#"tell "Hello!""#;
        let statements = parse(code).unwrap();
        assert_eq!(statements.len(), 1);
        
        match &statements[0] {
            Statement::Tell(EllexValue::String(s)) => {
                assert_eq!(s, "Hello!");
            }
            _ => panic!("Expected Tell statement"),
        }
    }
}
```

## Performance Considerations

### Optimization Tips

1. **Use the Pipeline**: Apply optimization passes for better performance
2. **Limit Recursion**: Keep recursion depth reasonable for stack safety
3. **Batch Operations**: Group similar operations when possible
4. **Memory Management**: Monitor memory usage in long-running programs

### Benchmarking

```rust
use std::time::Instant;
use ellex_core::EllexRuntime;
use ellex_parser::parse;

fn benchmark_execution(code: &str) -> std::time::Duration {
    let start = Instant::now();
    let mut runtime = EllexRuntime::new();
    let statements = parse(code).unwrap();
    runtime.execute(statements).unwrap();
    start.elapsed()
}
```

## Version Information

**Current Version**: 0.1.0
**Build Info**: Available via `ellex_core::BUILD_INFO`

```rust
use ellex_core::{VERSION, BUILD_INFO};

println!("Ellex version: {}", VERSION);
println!("Build info: {}", BUILD_INFO);
```

## Support and Resources

- **GitHub Repository**: [ellex-lang](https://github.com/your-org/ellex-lang)
- **Documentation**: `/docs` directory
- **Examples**: `/examples` directory
- **Tests**: Each crate's `tests/` directory
- **Issue Tracker**: GitHub Issues