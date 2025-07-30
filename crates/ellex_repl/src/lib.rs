use ellex_core::{EllexRuntime, EllexConfig, EllexValue, Statement, friendly_error_message, EllexError};
use ellex_parser::parse;
use rustyline::{Editor, error::ReadlineError};
use rustyline::history::FileHistory;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;

/// REPL session state that can be shared across interfaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplSession {
    pub variables: HashMap<String, EllexValue>,
    pub functions: HashMap<String, Statement>,
    pub history: Vec<String>,
    pub output_buffer: Vec<String>,
    pub execution_count: usize,
    pub config: EllexConfig,
}

impl Default for ReplSession {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            history: Vec::new(),
            output_buffer: Vec::new(),
            execution_count: 0,
            config: EllexConfig::default(),
        }
    }
}

impl ReplSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: EllexConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Execute a single line of Ellex code
    pub fn execute_line(&mut self, input: &str) -> Result<Vec<String>> {
        // Skip empty lines and comments
        let trimmed = input.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Ok(vec![]);
        }

        // Add to history
        self.history.push(input.to_string());
        self.execution_count += 1;

        // Handle special REPL commands
        if let Some(output) = self.handle_repl_command(trimmed)? {
            return Ok(vec![output]);
        }

        // Parse and execute Ellex code
        match parse(input) {
            Ok(statements) => {
                let mut runtime = EllexRuntime::with_config(self.config.clone());
                let mut output = Vec::new();

                for stmt in statements {
                    match self.execute_statement(&mut runtime, stmt) {
                        Ok(result) => {
                            if let Some(result) = result {
                                output.push(result.clone());
                                self.output_buffer.push(result);
                            }
                        }
                        Err(e) => {
                            let friendly_msg = friendly_error_message(&e);
                            output.push(format!("Error: {}", friendly_msg));
                            self.output_buffer.push(format!("Error: {}", friendly_msg));
                        }
                    }
                }

                Ok(output)
            }
            Err(e) => {
                let error_msg = format!("Parse error: {}", e);
                self.output_buffer.push(error_msg.clone());
                Ok(vec![error_msg])
            }
        }
    }

    /// Execute a statement and return output if any
    fn execute_statement(&mut self, runtime: &mut EllexRuntime, stmt: Statement) -> Result<Option<String>, EllexError> {
        match stmt {
            Statement::Tell(value) => {
                let interpolated = value.interpolate(&self.variables);
                Ok(Some(interpolated.to_string()))
            }
            Statement::Ask(var, _type_hint) => {
                // For non-interactive contexts, we'll return a prompt
                // Interactive contexts will override this behavior
                Ok(Some(format!("Input requested for variable '{}'", var)))
            }
            Statement::Call(name) => {
                if let Some(func_stmt) = self.functions.get(&name).cloned() {
                    self.execute_statement(runtime, func_stmt)
                } else {
                    Err(EllexError::UnknownCommand {
                        input: name.clone(),
                        suggestion: format!("Function '{}' is not defined. Use 'make {}' to define it.", name, name),
                    })
                }
            }
            Statement::Repeat(count, body) => {
                let mut outputs = Vec::new();
                for _ in 0..count {
                    for stmt in &body {
                        if let Some(output) = self.execute_statement(runtime, stmt.clone())? {
                            outputs.push(output);
                        }
                    }
                }
                Ok(if outputs.is_empty() { None } else { Some(outputs.join("\n")) })
            }
            Statement::When(var, condition, then_body, else_body) => {
                let var_value = self.variables.get(&var).cloned().unwrap_or(EllexValue::Nil);
                let should_execute = match (&var_value, &condition) {
                    (EllexValue::String(v), EllexValue::String(c)) => v == c,
                    (EllexValue::Number(v), EllexValue::Number(c)) => (v - c).abs() < f64::EPSILON,
                    _ => false,
                };

                let body = if should_execute {
                    &then_body
                } else if let Some(else_body) = &else_body {
                    else_body
                } else {
                    return Ok(None);
                };

                let mut outputs = Vec::new();
                for stmt in body {
                    if let Some(output) = self.execute_statement(runtime, stmt.clone())? {
                        outputs.push(output);
                    }
                }
                Ok(if outputs.is_empty() { None } else { Some(outputs.join("\n")) })
            }
        }
    }

    /// Handle special REPL commands
    fn handle_repl_command(&mut self, input: &str) -> Result<Option<String>> {
        match input {
            "help" | "/help" => Ok(Some(self.show_help())),
            "clear" | "/clear" => {
                self.output_buffer.clear();
                Ok(Some("Output cleared.".to_string()))
            }
            "history" | "/history" => Ok(Some(self.show_history())),
            "variables" | "/variables" | "/vars" => Ok(Some(self.show_variables())),
            "functions" | "/functions" | "/funcs" => Ok(Some(self.show_functions())),
            "config" | "/config" => Ok(Some(self.show_config())),
            "reset" | "/reset" => {
                *self = Self::with_config(self.config.clone());
                Ok(Some("Session reset.".to_string()))
            }
            "exit" | "/exit" | "quit" | "/quit" => {
                Ok(Some("exit".to_string())) // Special signal for exit
            }
            _ if input.starts_with("/set ") => self.handle_set_command(input),
            _ => Ok(None), // Not a REPL command
        }
    }

    fn handle_set_command(&mut self, input: &str) -> Result<Option<String>> {
        let parts: Vec<&str> = input.splitn(3, ' ').collect();
        if parts.len() != 3 {
            return Ok(Some("Usage: /set variable_name value".to_string()));
        }

        let var_name = parts[1].to_string();
        let value_str = parts[2];

        // Try to parse as different types
        let value = if value_str.starts_with('"') && value_str.ends_with('"') {
            EllexValue::String(value_str[1..value_str.len()-1].to_string())
        } else if let Ok(num) = value_str.parse::<f64>() {
            EllexValue::Number(num)
        } else if value_str == "nil" {
            EllexValue::Nil
        } else {
            EllexValue::String(value_str.to_string())
        };

        self.variables.insert(var_name.clone(), value);
        Ok(Some(format!("Set {} = {}", var_name, value_str)))
    }

    fn show_help(&self) -> String {
        format!(
            r#"🌿 Ellex REPL Help

Basic Commands:
  tell "message"              - Output a message
  ask "question" → variable   - Get input and store in variable
  repeat N times do ... end   - Loop N times
  when var is value do ... end - Conditional execution
  make function_name do ... end - Define a function

REPL Commands:
  /help                       - Show this help
  /clear                      - Clear output buffer
  /history                    - Show command history
  /variables or /vars         - Show all variables
  /functions or /funcs        - Show all functions
  /config                     - Show current configuration
  /set var value              - Set a variable directly
  /reset                      - Reset session
  /exit or /quit              - Exit REPL

Examples:
  tell "Hello, world!"
  ask "What's your name?" → name
  tell "Hello, {{name}}!"
  repeat 3 times do tell "Hi!" end

Execution count: {}
Variables: {}
Functions: {}"#,
            self.execution_count,
            self.variables.len(),
            self.functions.len()
        )
    }

    fn show_history(&self) -> String {
        if self.history.is_empty() {
            "No command history.".to_string()
        } else {
            let mut result = "Command History:\n".to_string();
            for (i, cmd) in self.history.iter().enumerate().rev().take(10) {
                result.push_str(&format!("  {}: {}\n", i + 1, cmd));
            }
            result
        }
    }

    fn show_variables(&self) -> String {
        if self.variables.is_empty() {
            "No variables defined.".to_string()
        } else {
            let mut result = "Variables:\n".to_string();
            for (name, value) in &self.variables {
                result.push_str(&format!("  {} = {}\n", name, value));
            }
            result
        }
    }

    fn show_functions(&self) -> String {
        if self.functions.is_empty() {
            "No functions defined.".to_string()
        } else {
            let mut result = "Functions:\n".to_string();
            for name in self.functions.keys() {
                result.push_str(&format!("  {}\n", name));
            }
            result
        }
    }

    fn show_config(&self) -> String {
        format!(
            r#"Configuration:
  Execution timeout: {}ms
  Memory limit: {}MB
  Turtle graphics: {}
  AI assistance: {}
  Max recursion depth: {}
  Max loop iterations: {}"#,
            self.config.execution_timeout_ms,
            self.config.memory_limit_mb,
            if self.config.enable_turtle { "enabled" } else { "disabled" },
            if self.config.enable_ai { "enabled" } else { "disabled" },
            self.config.max_recursion_depth,
            self.config.max_loop_iterations
        )
    }

    /// Get the current output buffer
    pub fn get_output(&self) -> &[String] {
        &self.output_buffer
    }

    /// Clear the output buffer
    pub fn clear_output(&mut self) {
        self.output_buffer.clear();
    }

    /// Save session to file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load session from file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let session: ReplSession = serde_json::from_str(&json)?;
        Ok(session)
    }

    /// Interactive ask implementation for terminal REPL
    pub fn interactive_ask(&mut self, question: &str, var_name: &str) -> String {
        println!("{}", question);
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            let value = input.trim().to_string();
            // Try to parse as number first
            if let Ok(num) = value.parse::<f64>() {
                self.variables.insert(var_name.to_string(), EllexValue::Number(num));
            } else {
                self.variables.insert(var_name.to_string(), EllexValue::String(value.clone()));
            }
            format!("Stored '{}' in variable '{}'", value, var_name)
        } else {
            "Failed to read input".to_string()
        }
    }
}

/// Interactive terminal REPL
pub struct InteractiveRepl {
    session: ReplSession,
    editor: Editor<(), FileHistory>,
    ai_enabled: bool,
}

impl InteractiveRepl {
    pub fn new(config: EllexConfig, ai_enabled: bool) -> Result<Self> {
        let mut editor = Editor::<(), FileHistory>::new()?;
        
        // Load history if it exists
        let history_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".el_history");
        
        if let Err(e) = editor.load_history(&history_path) {
            // History file doesn't exist yet, that's fine
            eprintln!("Note: Could not load history: {}", e);
        }

        Ok(Self {
            session: ReplSession::with_config(config),
            editor,
            ai_enabled,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.print_welcome();

        loop {
            let prompt = self.get_prompt().to_string();
            let readline = self.editor.readline(&prompt);
            match readline {
                Ok(line) => {
                    self.editor.add_history_entry(&line)?;
                    
                    match self.execute_line(&line) {
                        Ok(outputs) => {
                            if outputs.len() == 1 && outputs[0] == "exit" {
                                break;
                            }
                            for output in outputs {
                                println!("{}", output);
                            }
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("Goodbye! 👋");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }

        // Save history
        let history_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".el_history");
        
        if let Err(e) = self.editor.save_history(&history_path) {
            eprintln!("Warning: Could not save history: {}", e);
        }

        Ok(())
    }

    fn execute_line(&mut self, input: &str) -> Result<Vec<String>> {
        // Handle ask statements interactively
        if input.trim().starts_with("ask ") {
            return self.handle_interactive_ask(input);
        }

        self.session.execute_line(input)
    }

    fn handle_interactive_ask(&mut self, input: &str) -> Result<Vec<String>> {
        // Parse the ask statement
        match parse(input) {
            Ok(statements) => {
                if let Some(Statement::Ask(var, _type_hint)) = statements.first() {
                    // Extract the question from the original input
                    if let Some(question_start) = input.find('"') {
                        if let Some(question_end) = input[question_start + 1..].find('"') {
                            let question = &input[question_start + 1..question_start + 1 + question_end];
                            let response = self.session.interactive_ask(question, var);
                            return Ok(vec![response]);
                        }
                    }
                }
            }
            Err(e) => {
                return Ok(vec![format!("Parse error: {}", e)]);
            }
        }

        // Fallback to regular execution
        self.session.execute_line(input)
    }

    fn print_welcome(&self) {
        println!("🌿 Welcome to Ellex REPL!");
        println!("Type 'tell \"Hello world!\"' to get started");
        if self.ai_enabled {
            println!("AI assistance is enabled 🤖");
        }
        println!("Type '/help' for help, '/exit' to quit");
        println!();
    }

    fn get_prompt(&self) -> &str {
        if self.session.execution_count == 0 {
            "ellex> "
        } else {
            "ellex> "
        }
    }

    /// Get access to the session for external use
    pub fn session(&self) -> &ReplSession {
        &self.session
    }

    /// Get mutable access to the session
    pub fn session_mut(&mut self) -> &mut ReplSession {
        &mut self.session
    }
}

/// Non-interactive REPL for web/API use
pub struct ApiRepl {
    session: ReplSession,
}

impl ApiRepl {
    pub fn new(config: EllexConfig) -> Self {
        Self {
            session: ReplSession::with_config(config),
        }
    }

    pub fn execute(&mut self, input: &str) -> Result<Vec<String>> {
        self.session.execute_line(input)
    }

    pub fn get_session(&self) -> &ReplSession {
        &self.session
    }

    pub fn get_session_mut(&mut self) -> &mut ReplSession {
        &mut self.session
    }

    pub fn load_session(&mut self, session: ReplSession) {
        self.session = session;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_execution() {
        let mut session = ReplSession::new();
        let result = session.execute_line(r#"tell "Hello, world!""#).unwrap();
        assert_eq!(result, vec!["Hello, world!"]);
    }

    #[test]
    fn test_variable_setting() {
        let mut session = ReplSession::new();
        session.execute_line("/set name \"Alice\"").unwrap();
        let result = session.execute_line(r#"tell "Hello, {name}!""#).unwrap();
        assert_eq!(result, vec!["Hello, Alice!"]);
    }

    #[test]
    fn test_help_command() {
        let mut session = ReplSession::new();
        let result = session.execute_line("/help").unwrap();
        assert!(result[0].contains("Ellex REPL Help"));
    }

    #[test]
    fn test_repeat_statement() {
        let mut session = ReplSession::new();
        let result = session.execute_line(r#"repeat 3 times do tell "Hi!" end"#).unwrap();
        assert_eq!(result, vec!["Hi!\nHi!\nHi!"]);
    }

    #[test]
    fn test_session_persistence() {
        let mut session = ReplSession::new();
        session.execute_line("/set test_var 42").unwrap();
        
        let temp_file = std::env::temp_dir().join("test_session.json");
        session.save_to_file(&temp_file).unwrap();
        
        let loaded_session = ReplSession::load_from_file(&temp_file).unwrap();
        assert_eq!(loaded_session.variables.get("test_var"), Some(&EllexValue::Number(42.0)));
        
        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_empty_and_comment_lines() {
        let mut session = ReplSession::new();
        
        let result1 = session.execute_line("").unwrap();
        assert!(result1.is_empty());
        
        let result2 = session.execute_line("# This is a comment").unwrap();
        assert!(result2.is_empty());
        
        let result3 = session.execute_line("   ").unwrap();
        assert!(result3.is_empty());
    }
}