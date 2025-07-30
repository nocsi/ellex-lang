use crate::minielixir::{MiniElixirExpr, BinaryOperator, UnaryOperator};
use crate::minielixir_cached_runtime::CachedMiniElixirRuntime;
use crate::values::{EllexValue, EllexFunction, Statement};
use crate::{EllexError, EllexResult};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Bridge to Elixir MiniElixir implementation
pub struct ElixirMiniElixirBridge;

impl ElixirMiniElixirBridge {
    pub fn new() -> Self {
        Self
    }
    
    /// Execute natural language Ellex code using the Elixir backend
    pub fn execute_natural_language(&self, code: &str, context: HashMap<String, EllexValue>) -> EllexResult<EllexValue> {
        // This would communicate with the Elixir MiniElixir via NIFs
        // For now, we'll implement a simple fallback
        
        if code.trim().starts_with("tell ") {
            let message = code.trim_start_matches("tell ").trim();
            if message.starts_with('"') && message.ends_with('"') {
                let content = &message[1..message.len()-1];
                return Ok(EllexValue::String(content.to_string()));
            }
        }
        
        // In the full implementation, this would call the Elixir backend:
        // ellex_nif::execute_ellex_code(code, context)
        
        Err(EllexError::NotYetImplemented { 
            feature: "ElixirMiniElixirBridge communication".to_string() 
        })
    }
    
    /// Check if Elixir backend is available
    pub fn is_available(&self) -> bool {
        // This would check if the NIFs are loaded and Elixir backend is running
        false // For now, always false until NIFs are implemented
    }
}

impl Default for ElixirMiniElixirBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Bridge between Ellex syntax and MiniElixir interpreter
pub struct EllexMiniElixirBridge {
    runtime: CachedMiniElixirRuntime,
}

impl EllexMiniElixirBridge {
    pub fn new() -> Self {
        Self {
            runtime: CachedMiniElixirRuntime::new(),
        }
    }
    
    /// Convert Ellex code to MiniElixir AST and execute
    pub fn execute_ellex_code(&mut self, code: &str) -> EllexResult<EllexValue> {
        // First parse as Ellex (simplified parsing for now)
        let statements = self.parse_ellex_simple(code)?;
        
        // Convert to MiniElixir AST
        let minielixir_exprs = self.statements_to_minielixir(&statements)?;
        
        // Execute with cached runtime
        let mut ctx = crate::minielixir::EvaluationContext::new();
        
        if minielixir_exprs.len() == 1 {
            self.runtime.eval_expr(&minielixir_exprs[0], &mut ctx)
        } else {
            self.runtime.eval_block(&minielixir_exprs, &mut ctx)
        }
    }
    
    /// Convert Ellex statements to MiniElixir expressions
    pub fn statements_to_minielixir(&self, statements: &[Statement]) -> EllexResult<Vec<MiniElixirExpr>> {
        let mut minielixir_exprs = Vec::new();
        
        for stmt in statements {
            let expr = self.statement_to_minielixir(stmt)?;
            minielixir_exprs.push(expr);
        }
        
        Ok(minielixir_exprs)
    }
    
    /// Convert a single Ellex statement to MiniElixir expression
    pub fn statement_to_minielixir(&self, stmt: &Statement) -> EllexResult<MiniElixirExpr> {
        match stmt {
            Statement::Tell(value) => {
                // tell "Hello" -> IO.puts("Hello") equivalent
                let arg = self.ellex_value_to_minielixir(value)?;
                Ok(MiniElixirExpr::Call {
                    function: "to_string".to_string(), // Use our safe built-in
                    args: vec![arg],
                })
            }
            
            Statement::Ask(var_name, _type_hint) => {
                // ask "Name?" → name -> get_input("Name?") |> assign to variable
                Ok(MiniElixirExpr::Call {
                    function: "get_input".to_string(), // Would need to implement this
                    args: vec![MiniElixirExpr::String(format!("Enter {}: ", var_name))],
                })
            }
            
            Statement::Assignment(var_name, value) => {
                // var = value -> assign variable
                let arg = self.ellex_value_to_minielixir(value)?;
                Ok(MiniElixirExpr::Call {
                    function: "assign".to_string(),
                    args: vec![MiniElixirExpr::String(var_name.clone()), arg],
                })
            }
            
            Statement::Repeat(times, body) => {
                // repeat 3 times do ... end -> Enum.each(1..3, fn _ -> ... end)
                let body_exprs = self.statements_to_minielixir(body)?;
                let body_block = if body_exprs.len() == 1 {
                    body_exprs[0].clone()
                } else {
                    MiniElixirExpr::Block(body_exprs)
                };
                
                // For now, represent as a simple loop construct
                Ok(MiniElixirExpr::Call {
                    function: "repeat".to_string(),
                    args: vec![
                        MiniElixirExpr::Integer(*times as i64),
                        body_block,
                    ],
                })
            }
            
            Statement::When(var_name, condition_value, then_body, else_body) => {
                // when var == value do ... else ... end -> if var == value, do: ..., else: ...
                let var_expr = MiniElixirExpr::Variable(var_name.clone());
                let condition_expr = self.ellex_value_to_minielixir(condition_value)?;
                
                let condition = MiniElixirExpr::BinaryOp {
                    op: BinaryOperator::Eq,
                    left: Box::new(var_expr),
                    right: Box::new(condition_expr),
                };
                
                let then_exprs = self.statements_to_minielixir(then_body)?;
                let then_block = if then_exprs.len() == 1 {
                    Box::new(then_exprs[0].clone())
                } else {
                    Box::new(MiniElixirExpr::Block(then_exprs))
                };
                
                let else_block = if let Some(else_stmts) = else_body {
                    let else_exprs = self.statements_to_minielixir(else_stmts)?;
                    if else_exprs.len() == 1 {
                        Some(Box::new(else_exprs[0].clone()))
                    } else {
                        Some(Box::new(MiniElixirExpr::Block(else_exprs)))
                    }
                } else {
                    None
                };
                
                Ok(MiniElixirExpr::If {
                    condition: Box::new(condition),
                    then_branch: then_block,
                    else_branch: else_block,
                })
            }
            
            Statement::Call(function_name) => {
                // Simple function call
                Ok(MiniElixirExpr::Call {
                    function: function_name.clone(),
                    args: vec![],
                })
            }
        }
    }
    
    /// Convert EllexValue to MiniElixir expression
    pub fn ellex_value_to_minielixir(&self, value: &EllexValue) -> EllexResult<MiniElixirExpr> {
        match value {
            EllexValue::String(s) => Ok(MiniElixirExpr::String(s.clone())),
            EllexValue::Number(n) => {
                if n.fract() == 0.0 {
                    Ok(MiniElixirExpr::Integer(*n as i64))
                } else {
                    Ok(MiniElixirExpr::Float(*n))
                }
            }
            EllexValue::List(elements) => {
                let mut minielixir_elements = Vec::new();
                for elem in elements {
                    minielixir_elements.push(self.ellex_value_to_minielixir(elem)?);
                }
                Ok(MiniElixirExpr::List(minielixir_elements))
            }
            EllexValue::Function(func) => {
                // Convert function to a call expression for now
                Ok(MiniElixirExpr::Call {
                    function: func.name.clone(),
                    args: vec![],
                })
            }
            EllexValue::Nil => Ok(MiniElixirExpr::Nil),
        }
    }
    
    /// Convert MiniElixir result back to EllexValue
    pub fn minielixir_to_ellex_value(&self, expr: &MiniElixirExpr) -> EllexValue {
        match expr {
            MiniElixirExpr::String(s) => EllexValue::String(s.clone()),
            MiniElixirExpr::Integer(n) => EllexValue::Number(*n as f64),
            MiniElixirExpr::Float(f) => EllexValue::Number(*f),
            MiniElixirExpr::Boolean(b) => EllexValue::String(b.to_string()),
            MiniElixirExpr::Nil => EllexValue::Nil,
            MiniElixirExpr::Atom(a) => EllexValue::String(format!(":{}", a)),
            MiniElixirExpr::List(elements) => {
                let ellex_elements = elements.iter()
                    .map(|e| self.minielixir_to_ellex_value(e))
                    .collect();
                EllexValue::List(ellex_elements)
            }
            _ => EllexValue::String(format!("{}", expr)), // Fallback
        }
    }
    
    /// Convert Ellex AST to JSON representation (as shown in your example)
    pub fn ellex_to_json(&self, statements: &[Statement]) -> EllexResult<Vec<Value>> {
        let mut json_ast = Vec::new();
        
        for stmt in statements {
            let json_stmt = self.statement_to_json(stmt)?;
            json_ast.push(json_stmt);
        }
        
        Ok(json_ast)
    }
    
    fn statement_to_json(&self, stmt: &Statement) -> EllexResult<Value> {
        match stmt {
            Statement::Assignment(var_name, value) => {
                // Handle assignment statement
                Ok(json!({
                    "op": "assign",
                    "variable": var_name,
                    "value": self.ellex_value_to_json(value)?
                }))
            }
            Statement::Tell(value) => {
                Ok(json!({
                    "op": "call",
                    "module": "Kernel",
                    "function": "inspect",
                    "args": [self.ellex_value_to_json(value)?]
                }))
            }
            
            Statement::Ask(var_name, _type_hint) => {
                Ok(json!({
                    "op": "assign",
                    "var": var_name,
                    "value": {
                        "op": "call",
                        "module": "IO",
                        "function": "gets",
                        "args": [json!(format!("Enter {}: ", var_name))]
                    }
                }))
            }
            
            Statement::Repeat(times, body) => {
                let body_json: Result<Vec<_>, _> = body.iter()
                    .map(|s| self.statement_to_json(s))
                    .collect();
                
                Ok(json!({
                    "op": "call",
                    "module": "Enum",
                    "function": "each",
                    "args": [
                        {
                            "op": "range",
                            "start": 1,
                            "end": times
                        },
                        {
                            "op": "fn",
                            "args": ["_"],
                            "body": body_json?
                        }
                    ]
                }))
            }
            
            Statement::When(var_name, condition_value, then_body, else_body) => {
                let then_json: Result<Vec<_>, _> = then_body.iter()
                    .map(|s| self.statement_to_json(s))
                    .collect();
                
                let else_json = if let Some(else_stmts) = else_body {
                    let else_result: Result<Vec<_>, _> = else_stmts.iter()
                        .map(|s| self.statement_to_json(s))
                        .collect();
                    Some(else_result?)
                } else {
                    None
                };
                
                Ok(json!({
                    "op": "if",
                    "condition": {
                        "op": "==",
                        "left": {"var": var_name},
                        "right": self.ellex_value_to_json(condition_value)?
                    },
                    "then": then_json?,
                    "else": else_json
                }))
            }
            
            Statement::Call(function_name) => {
                Ok(json!({
                    "op": "call",
                    "function": function_name,
                    "args": []
                }))
            }
        }
    }
    
    fn ellex_value_to_json(&self, value: &EllexValue) -> EllexResult<Value> {
        match value {
            EllexValue::String(s) => Ok(json!(s)),
            EllexValue::Number(n) => Ok(json!(n)),
            EllexValue::List(elements) => {
                let json_elements: Result<Vec<_>, _> = elements.iter()
                    .map(|e| self.ellex_value_to_json(e))
                    .collect();
                Ok(json!(json_elements?))
            }
            EllexValue::Function(func) => {
                Ok(json!({
                    "type": "function",
                    "name": func.name,
                    "params": func.params
                }))
            }
            EllexValue::Nil => Ok(json!(null)),
        }
    }
    
    /// Simple Ellex parser (placeholder - in real implementation this would use ellex_parser)
    fn parse_ellex_simple(&self, code: &str) -> EllexResult<Vec<Statement>> {
        let mut statements = Vec::new();
        
        for line in code.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some(stmt) = self.parse_line(line)? {
                statements.push(stmt);
            }
        }
        
        Ok(statements)
    }
    
    fn parse_line(&self, line: &str) -> EllexResult<Option<Statement>> {
        // Simple line-by-line parser for demo purposes
        if line.starts_with("tell ") {
            let content = &line[5..].trim();
            if content.starts_with('"') && content.ends_with('"') {
                let message = &content[1..content.len()-1];
                return Ok(Some(Statement::Tell(EllexValue::String(message.to_string()))));
            } else if let Ok(num) = content.parse::<f64>() {
                return Ok(Some(Statement::Tell(EllexValue::Number(num))));
            } else {
                return Ok(Some(Statement::Tell(EllexValue::String(content.to_string()))));
            }
        }
        
        if line.starts_with("ask ") {
            let content = &line[4..].trim();
            if content.starts_with('"') && content.ends_with('"') {
                let var_name = &content[1..content.len()-1];
                return Ok(Some(Statement::Ask(var_name.to_string(), None)));
            }
        }
        
        if line.starts_with("repeat ") && line.contains(" times") {
            if let Some(times_pos) = line.find(" times") {
                let times_str = &line[7..times_pos].trim();
                if let Ok(times) = times_str.parse::<u32>() {
                    // For simplicity, assume empty body for now
                    return Ok(Some(Statement::Repeat(times, vec![])));
                }
            }
        }
        
        // Function calls (simple case)
        if line.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Ok(Some(Statement::Call(line.to_string())));
        }
        
        Ok(None)
    }
    
    /// Get performance statistics from the underlying runtime
    pub fn performance_stats(&self) -> crate::minielixir_cached_runtime::CachedRuntimeStats {
        self.runtime.performance_stats()
    }
    
    /// Get cache analysis
    pub fn cache_analysis(&self) -> crate::minielixir_cached_runtime::CacheAnalysis {
        self.runtime.cache_analysis()
    }
    
    /// Clear all caches
    pub fn clear_caches(&mut self) {
        self.runtime.clear_caches();
    }
}

impl Default for EllexMiniElixirBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Extended version that integrates with ellex_parser when available
#[cfg(feature = "full_parser")]
impl EllexMiniElixirBridge {
    pub fn execute_with_full_parser(&mut self, code: &str) -> EllexResult<EllexValue> {
        // This would use the full ellex_parser when available
        use crate::ellex_parser;
        
        let ast = ellex_parser::parse(code)
            .map_err(|e| EllexError::ParseError {
                line: 0,
                column: 0,
                message: format!("Parse error: {}", e),
            })?;
        
        let minielixir_exprs = self.statements_to_minielixir(&ast)?;
        let mut ctx = crate::minielixir::EvaluationContext::new();
        
        if minielixir_exprs.len() == 1 {
            self.runtime.eval_expr(&minielixir_exprs[0], &mut ctx)
        } else {
            self.runtime.eval_block(&minielixir_exprs, &mut ctx)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_basic_tell() {
        let mut bridge = EllexMiniElixirBridge::new();
        let result = bridge.execute_ellex_code("tell \"Hello World\"").unwrap();
        assert_eq!(result, EllexValue::String("Hello World".to_string()));
    }
    
    #[test]
    fn test_bridge_number_tell() {
        let mut bridge = EllexMiniElixirBridge::new();
        let result = bridge.execute_ellex_code("tell 42").unwrap();
        assert_eq!(result, EllexValue::String("42".to_string()));
    }
    
    #[test]
    fn test_statement_to_minielixir() {
        let bridge = EllexMiniElixirBridge::new();
        let stmt = Statement::Tell(EllexValue::String("test".to_string()));
        let minielixir_expr = bridge.statement_to_minielixir(&stmt).unwrap();
        
        match minielixir_expr {
            MiniElixirExpr::Call { function, args } => {
                assert_eq!(function, "to_string");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected function call"),
        }
    }
    
    #[test]
    fn test_ellex_to_json() {
        let bridge = EllexMiniElixirBridge::new();
        let statements = vec![
            Statement::Tell(EllexValue::String("Hello".to_string())),
        ];
        
        let json_ast = bridge.ellex_to_json(&statements).unwrap();
        assert_eq!(json_ast.len(), 1);
        
        let json_stmt = &json_ast[0];
        assert_eq!(json_stmt["op"], "call");
        assert_eq!(json_stmt["module"], "Kernel");
        assert_eq!(json_stmt["function"], "inspect");
    }
    
    #[test]
    fn test_value_conversion() {
        let bridge = EllexMiniElixirBridge::new();
        
        let ellex_val = EllexValue::Number(3.14);
        let minielixir_expr = bridge.ellex_value_to_minielixir(&ellex_val).unwrap();
        
        match minielixir_expr {
            MiniElixirExpr::Float(f) => assert_eq!(f, 3.14),
            _ => panic!("Expected float"),
        }
    }
    
    #[test]
    fn test_when_statement_conversion() {
        let bridge = EllexMiniElixirBridge::new();
        let stmt = Statement::When(
            "x".to_string(),
            EllexValue::String("yes".to_string()),
            vec![Statement::Tell(EllexValue::String("Match!".to_string()))],
            None,
        );
        
        let minielixir_expr = bridge.statement_to_minielixir(&stmt).unwrap();
        
        match minielixir_expr {
            MiniElixirExpr::If { condition, then_branch, else_branch } => {
                assert!(matches!(condition.as_ref(), MiniElixirExpr::BinaryOp { .. }));
                // then_branch is a Box, so it's always "some"
                assert!(else_branch.is_none());
            }
            _ => panic!("Expected if expression"),
        }
    }
}