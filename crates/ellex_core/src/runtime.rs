use crate::safety::{ExecutionLimits, SafetyMonitor};
use crate::values::{EllexFunction, EllexValue, Statement};
use crate::{EllexConfig, EllexError};
use crate::ellex_minielixir_bridge::{EllexMiniElixirBridge, ElixirMiniElixirBridge};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

// Trait for a compiler pass (functional attachment)
pub trait Pass: Send + Sync {
    fn apply(&self, ast: &mut Vec<Statement>) -> Result<()>;
}

// Pipeline: Chain of passes, arbitrarily attachable
pub struct Pipeline {
    passes: Vec<Box<dyn Pass>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline { passes: Vec::new() }
    }

    pub fn attach(&mut self, pass: Box<dyn Pass>) {
        self.passes.push(pass);
    }

    pub fn run(&self, ast: &mut Vec<Statement>) -> Result<()> {
        for pass in &self.passes {
            pass.apply(ast)?;
        }
        Ok(())
    }
}

// Example Pass: Constant Folding (evaluates constants at compile-time)
pub struct ConstantFolding;

impl Pass for ConstantFolding {
    fn apply(&self, ast: &mut Vec<Statement>) -> Result<()> {
        for stmt in ast.iter_mut() {
            if let Statement::Tell(EllexValue::Number(_n)) = stmt {
                // Already constant, but could fold expressions (expand for real exprs)
            }
            // Recurse into blocks (e.g., repeat/when)
            if let Statement::Repeat(_, body) = stmt {
                self.apply(body)?;
            }
            // Add for When, etc.
        }
        Ok(())
    }
}

// Example Pass: Dead Code Elimination (remove unused vars/statements)
pub struct DeadCodeElim {
    used_vars: HashMap<String, bool>, // Track usage (populate via analysis)
}

impl DeadCodeElim {
    pub fn new() -> Self {
        DeadCodeElim {
            used_vars: HashMap::new(),
        }
    }
}

impl Pass for DeadCodeElim {
    fn apply(&self, ast: &mut Vec<Statement>) -> Result<()> {
        // First pass: analyze usage (e.g., scan for var refs)
        // Second pass: remove unused
        ast.retain(|stmt| {
            if let Statement::Ask(var, _) = stmt {
                self.used_vars.get(var).cloned().unwrap_or(false)
            } else {
                true
            }
        });
        Ok(())
    }
}

// Example Pass: Inlining (replace function calls with body)
pub struct Inlining {
    functions: HashMap<String, EllexFunction>, // From AST analysis
}

impl Inlining {
    pub fn new() -> Self {
        Inlining {
            functions: HashMap::new(),
        }
    }
}

impl Pass for Inlining {
    fn apply(&self, ast: &mut Vec<Statement>) -> Result<()> {
        // Collect functions first
        // Then replace Call(name) with body clone
        let mut replacements = Vec::new();

        for (i, stmt) in ast.iter().enumerate() {
            if let Statement::Call(name) = stmt {
                if let Some(func) = self.functions.get(name) {
                    replacements.push((i, func.body.clone()));
                }
            }
        }

        // Apply replacements in reverse order to maintain indices
        for (i, replacement) in replacements.into_iter().rev() {
            ast.splice(i..i + 1, replacement);
        }
        Ok(())
    }
}

// Example Pass: Loop Unrolling (for small repeat counts)
pub struct LoopUnroll {
    max_unroll: usize, // Configurable, e.g., 5
}

impl LoopUnroll {
    pub fn new(max_unroll: usize) -> Self {
        LoopUnroll { max_unroll }
    }
}

impl Pass for LoopUnroll {
    fn apply(&self, ast: &mut Vec<Statement>) -> Result<()> {
        for stmt in ast.iter_mut() {
            if let Statement::Repeat(count, body) = stmt {
                if *count as usize <= self.max_unroll {
                    let unrolled: Vec<Statement> = (0..*count).flat_map(|_| body.clone()).collect();
                    *stmt = Statement::Repeat(1, unrolled); // Or replace with flat list
                }
            }
        }
        Ok(())
    }
}

// Transpilation Pass: To Elixir Code (generate efficient Elixir)
pub struct ElixirTranspile;

impl Pass for ElixirTranspile {
    fn apply(&self, _ast: &mut Vec<Statement>) -> Result<()> {
        // Transform AST to Elixir AST or string
        // E.g., Tell(str) -> IO.puts(str)
        // Ensure efficient: Use Elixir's pattern matching for When
        // Output as new "AST" or directly to file
        Err(anyhow!("Transpilation not implemented yet")) // Placeholder
    }
}

// Superoptimization Pass (AI-assisted, stub for LLM integration)
pub struct SuperOpt;

impl Pass for SuperOpt {
    fn apply(&self, _ast: &mut Vec<Statement>) -> Result<()> {
        // Use ellex_ai to suggest optimal variants
        Ok(())
    }
}

pub struct EllexRuntime {
    variables: HashMap<String, EllexValue>,
    functions: HashMap<String, EllexFunction>,
    safety: SafetyMonitor,
    elixir_bridge: ElixirMiniElixirBridge,
    rust_bridge: EllexMiniElixirBridge,
    prefer_elixir: bool,
    config: EllexConfig,
    turtle: Option<crate::turtle::TurtleGraphics>,
}

impl EllexRuntime {
    pub fn new() -> Self {
        Self::with_config(EllexConfig::default())
    }

    pub fn with_config(config: EllexConfig) -> Self {
        let elixir_bridge = ElixirMiniElixirBridge::new();
        let prefer_elixir = elixir_bridge.is_available();
        
        let limits = ExecutionLimits {
            timeout_ms: config.execution_timeout_ms,
            memory_limit_mb: config.memory_limit_mb,
            max_recursion_depth: config.max_recursion_depth,
            max_loop_iterations: config.max_loop_iterations,
            max_instructions: 100000,
        };
        
        let turtle = if config.enable_turtle {
            Some(crate::turtle::TurtleGraphics::new())
        } else {
            None
        };
        
        EllexRuntime {
            variables: HashMap::new(),
            functions: HashMap::new(),
            safety: SafetyMonitor::new(limits),
            elixir_bridge,
            rust_bridge: EllexMiniElixirBridge::new(),
            prefer_elixir,
            config,
            turtle,
        }
    }

    /// Execute Ellex code using the best available interpreter
    pub fn execute(&mut self, stmts: Vec<Statement>) -> Result<EllexValue> {
        self.safety.reset();
        
        // Check safety before execution
        self.safety.check_execution_start()?;
        
        // Try Elixir MiniElixir first if available and preferred
        if self.prefer_elixir && self.elixir_bridge.is_available() {
            match self.execute_with_elixir_backend(&stmts) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    eprintln!("Elixir backend failed, falling back to Rust: {:?}", e);
                    // Fall through to Rust implementation
                }
            }
        }
        
        // Use direct statement execution
        self.execute_statements_directly(&stmts)
    }
    
    /// Execute natural language Ellex code directly
    pub fn execute_natural_language(&mut self, code: &str) -> Result<EllexValue, EllexError> {
        self.safety.reset();
        
        let context = self.variables.clone();
        
        // Try Elixir backend for natural language (it has better NL support)
        if self.elixir_bridge.is_available() {
            match self.elixir_bridge.execute_natural_language(code, context.clone()) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    eprintln!("Elixir natural language processing failed: {:?}", e);
                    // Fall through to basic Rust parsing
                }
            }
        }
        
        // Basic fallback for simple commands
        self.execute_simple_natural_language(code)
    }
    
    /// Execute using Elixir MiniElixir backend
    fn execute_with_elixir_backend(&mut self, _stmts: &[Statement]) -> Result<EllexValue> {
        // This would convert statements to code and send to Elixir
        // For now, just return success
        Ok(EllexValue::String("Executed with Elixir backend (placeholder)".to_string()))
    }
    
    /// Execute statements directly in the runtime
    fn execute_statements_directly(&mut self, stmts: &[Statement]) -> Result<EllexValue> {
        let mut last_result = EllexValue::Nil;
        
        for stmt in stmts {
            self.safety.check_execution_continue()?;
            last_result = self.execute_single_statement(stmt)?;
        }
        
        Ok(last_result)
    }
    
    /// Execute using Rust MiniElixir backend
    fn execute_with_rust_backend(&mut self, stmts: &[Statement]) -> Result<EllexValue> {
        match self.rust_bridge.execute_ellex_code(&format_statements_as_code(stmts)) {
            Ok(result) => Ok(result),
            Err(e) => Err(anyhow!("Rust backend execution failed: {:?}", e)),
        }
    }
    
    /// Execute a single statement
    fn execute_single_statement(&mut self, stmt: &Statement) -> Result<EllexValue> {
        match stmt {
            Statement::Tell(value) => {
                let output = self.evaluate_value(value)?;
                println!("{}", self.format_output(&output));
                Ok(output)
            }
            
            Statement::Ask(var_name, _type_hint) => {
                use std::io::{self, Write};
                print!("? ");
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim().to_string();
                
                // Try to parse as number, otherwise treat as string
                let value = if let Ok(num) = input.parse::<f64>() {
                    EllexValue::Number(num)
                } else {
                    EllexValue::String(input)
                };
                
                self.variables.insert(var_name.clone(), value.clone());
                Ok(value)
            }
            
            Statement::Assignment(var_name, value) => {
                let evaluated_value = self.evaluate_value(value)?;
                self.variables.insert(var_name.clone(), evaluated_value.clone());
                Ok(evaluated_value)
            }
            
            Statement::Repeat(count, body) => {
                self.safety.check_loop_start(*count as usize)?;
                let mut last_result = EllexValue::Nil;
                
                for i in 0..*count {
                    self.safety.check_loop_iteration(i as usize)?;
                    last_result = self.execute_statements_directly(body)?;
                }
                
                Ok(last_result)
            }
            
            Statement::When(var_name, condition, then_body, else_body) => {
                let var_value = self.variables.get(var_name)
                    .ok_or_else(|| anyhow!("Variable '{}' not found", var_name))?;
                
                let condition_value = self.evaluate_value(condition)?;
                
                if self.values_equal(var_value, &condition_value) {
                    self.execute_statements_directly(then_body)
                } else if let Some(else_body) = else_body {
                    self.execute_statements_directly(else_body)
                } else {
                    Ok(EllexValue::Nil)
                }
            }
            
            Statement::Call(func_name) => {
                if let Some(function) = self.functions.get(func_name).cloned() {
                    self.safety.check_recursion_depth()?;
                    self.execute_statements_directly(&function.body)
                } else {
                    // Check if it's a turtle graphics command
                    if self.turtle.is_some() {
                        self.execute_turtle_command(func_name)
                    } else {
                        Err(anyhow!("Function '{}' not found", func_name))
                    }
                }
            }
        }
    }
    
    /// Execute turtle graphics commands
    fn execute_turtle_command(&mut self, command: &str) -> Result<EllexValue> {
        if let Some(ref mut turtle) = self.turtle {
            match command {
                "move_forward" | "forward" => {
                    turtle.move_forward(10.0);
                    Ok(EllexValue::String("Turtle moved forward".to_string()))
                }
                "turn_right" | "right" => {
                    turtle.turn_right(90.0);
                    Ok(EllexValue::String("Turtle turned right".to_string()))
                }
                "turn_left" | "left" => {
                    turtle.turn_left(90.0);
                    Ok(EllexValue::String("Turtle turned left".to_string()))
                }
                "pen_up" => {
                    turtle.pen_up();
                    Ok(EllexValue::String("Pen up".to_string()))
                }
                "pen_down" => {
                    turtle.pen_down();
                    Ok(EllexValue::String("Pen down".to_string()))
                }
                _ => Err(anyhow!("Unknown turtle command: {}", command))
            }
        } else {
            Err(anyhow!("Turtle graphics not enabled"))
        }
    }
    
    /// Evaluate a value (resolve variables, etc.)
    fn evaluate_value(&self, value: &EllexValue) -> Result<EllexValue> {
        match value {
            EllexValue::String(s) => {
                // Handle variable interpolation with full string processing
                let interpolated = self.interpolate_string(s);
                Ok(EllexValue::String(interpolated))
            }
            _ => Ok(value.clone())
        }
    }
    
    /// Interpolate variables in a string
    fn interpolate_string(&self, s: &str) -> String {
        // Handle multiple variable interpolations in a single string
        let mut result = s.to_string();
        
        // Find all {variable} patterns and replace them
        let mut start = 0;
        while let Some(open_pos) = result[start..].find('{') {
            let open_pos = start + open_pos;
            if let Some(close_pos) = result[open_pos..].find('}') {
                let close_pos = open_pos + close_pos;
                let var_name = &result[open_pos + 1..close_pos];
                
                let replacement = if let Some(var_value) = self.variables.get(var_name) {
                    self.format_output(var_value)
                } else {
                    format!("undefined:{}", var_name)
                };
                
                result.replace_range(open_pos..=close_pos, &replacement);
                start = open_pos + replacement.len();
            } else {
                break;
            }
        }
        
        result
    }
    
    /// Check if two values are equal
    fn values_equal(&self, a: &EllexValue, b: &EllexValue) -> bool {
        match (a, b) {
            (EllexValue::String(s1), EllexValue::String(s2)) => s1 == s2,
            (EllexValue::Number(n1), EllexValue::Number(n2)) => (n1 - n2).abs() < f64::EPSILON,
            (EllexValue::Nil, EllexValue::Nil) => true,
            _ => false,
        }
    }
    
    /// Format output for display
    fn format_output(&self, value: &EllexValue) -> String {
        match value {
            EllexValue::String(s) => s.clone(),
            EllexValue::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            EllexValue::List(items) => {
                let formatted: Vec<String> = items.iter()
                    .map(|item| self.format_output(item))
                    .collect();
                format!("[{}]", formatted.join(", "))
            }
            EllexValue::Function(f) => format!("function {}", f.name),
            EllexValue::Nil => "nil".to_string(),
        }
    }
    
    /// Simple natural language fallback
    fn execute_simple_natural_language(&mut self, code: &str) -> Result<EllexValue, EllexError> {
        let code = code.trim();
        
        if code.starts_with("tell ") {
            let message = code.trim_start_matches("tell ").trim();
            if message.starts_with('"') && message.ends_with('"') {
                let content = &message[1..message.len()-1];
                return Ok(EllexValue::String(content.to_string()));
            }
            return Ok(EllexValue::String(message.to_string()));
        }
        
        if code.starts_with("ask ") {
            let question = code.trim_start_matches("ask ").trim();
            return Ok(EllexValue::String(format!("Asked: {}", question)));
        }
        
        if code.contains("move forward") {
            return Ok(EllexValue::String("Turtle moved forward".to_string()));
        }
        
        if code.contains("turn") {
            return Ok(EllexValue::String("Turtle turned".to_string()));
        }
        
        Err(EllexError::ParseError {
            line: 0,
            column: 0,
            message: format!("Don't understand: '{}'", code),
        })
    }
    
    /// Get current variables
    pub fn get_variables(&self) -> &HashMap<String, EllexValue> {
        &self.variables
    }
    
    /// Set a variable
    pub fn set_variable(&mut self, name: String, value: EllexValue) {
        self.variables.insert(name, value);
    }
    
    /// Define a function
    pub fn define_function(&mut self, name: String, function: EllexFunction) {
        self.functions.insert(name, function);
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &EllexConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: EllexConfig) {
        // Update safety limits
        let limits = ExecutionLimits {
            timeout_ms: config.execution_timeout_ms,
            memory_limit_mb: config.memory_limit_mb,
            max_recursion_depth: config.max_recursion_depth,
            max_loop_iterations: config.max_loop_iterations,
            max_instructions: 100000,
        };
        
        self.safety = SafetyMonitor::new(limits);
        
        // Update turtle graphics availability
        if config.enable_turtle && self.turtle.is_none() {
            self.turtle = Some(crate::turtle::TurtleGraphics::new());
        } else if !config.enable_turtle {
            self.turtle = None;
        }
        
        self.config = config;
    }
    
    /// Get turtle graphics (if enabled)
    pub fn get_turtle(&self) -> Option<&crate::turtle::TurtleGraphics> {
        self.turtle.as_ref()
    }
    
    /// Reset runtime state
    pub fn reset(&mut self) {
        self.variables.clear();
        self.functions.clear();
        self.safety.reset();
        if let Some(ref mut turtle) = self.turtle {
            turtle.reset();
        }
    }
    
    /// Check if Elixir backend is available
    pub fn has_elixir_backend(&self) -> bool {
        self.elixir_bridge.is_available()
    }
    
    /// Force use of a specific backend
    pub fn set_prefer_elixir(&mut self, prefer: bool) {
        self.prefer_elixir = prefer && self.elixir_bridge.is_available();
    }
}

/// Convert statements back to code for execution
fn format_statements_as_code(stmts: &[Statement]) -> String {
    stmts.iter().map(|stmt| match stmt {
        Statement::Tell(value) => format!("tell {}", format_value(value)),
        Statement::Ask(var, _) => format!("ask \"Enter value\" = {}", var),
        Statement::Assignment(var, value) => format!("{} = {}", var, format_value(value)),
        Statement::Repeat(count, body) => {
            let body_code = format_statements_as_code(body);
            format!("repeat {} times:\n{}", count, body_code)
        },
        Statement::When(var, condition, then_body, else_body) => {
            let then_code = format_statements_as_code(then_body);
            let else_code = else_body.as_ref()
                .map(|body| format!(" else:\n{}", format_statements_as_code(body)))
                .unwrap_or_default();
            format!("when {} == {}:\n{}{}", var, format_value(condition), then_code, else_code)
        },
        Statement::Call(name) => name.clone(),
    }).collect::<Vec<_>>().join("\n")
}

fn format_value(value: &EllexValue) -> String {
    match value {
        EllexValue::String(s) => format!("\"{}\"", s),
        EllexValue::Number(n) => n.to_string(),
        EllexValue::List(items) => {
            let formatted_items: Vec<String> = items.iter().map(format_value).collect();
            format!("[{}]", formatted_items.join(", "))
        },
        EllexValue::Function(f) => f.name.clone(),
        EllexValue::Nil => "nil".to_string(),
    }
}

impl From<anyhow::Error> for crate::EllexError {
    fn from(err: anyhow::Error) -> Self {
        crate::EllexError::LogicError {
            message: err.to_string(),
        }
    }
}


