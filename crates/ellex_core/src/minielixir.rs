use crate::cached_ast::{CachedStatement, CachedType, CacheEntry};
use crate::values::EllexValue;
use crate::{EllexError, EllexResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// MiniElixir AST node - inspired by the original minielixir implementation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MiniElixirExpr {
    // Literals
    Atom(String),
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Nil,
    
    // Collections
    List(Vec<MiniElixirExpr>),
    Tuple(Vec<MiniElixirExpr>),
    Map(Vec<(MiniElixirExpr, MiniElixirExpr)>),
    
    // Variables and identifiers
    Variable(String),
    
    // Binary operations
    BinaryOp {
        op: BinaryOperator,
        left: Box<MiniElixirExpr>,
        right: Box<MiniElixirExpr>,
    },
    
    // Unary operations
    UnaryOp {
        op: UnaryOperator,
        operand: Box<MiniElixirExpr>,
    },
    
    // Function calls
    Call {
        function: String,
        args: Vec<MiniElixirExpr>,
    },
    
    // Pipe operator
    Pipe {
        value: Box<MiniElixirExpr>,
        function: Box<MiniElixirExpr>,
    },
    
    // Pattern matching
    Match {
        pattern: Box<MiniElixirExpr>,
        value: Box<MiniElixirExpr>,
    },
    
    // Case expressions
    Case {
        expr: Box<MiniElixirExpr>,
        clauses: Vec<CaseClause>,
    },
    
    // If expressions
    If {
        condition: Box<MiniElixirExpr>,
        then_branch: Box<MiniElixirExpr>,
        else_branch: Option<Box<MiniElixirExpr>>,
    },
    
    // Blocks
    Block(Vec<MiniElixirExpr>),
    
    // Access operations (for maps/structs)
    Access {
        object: Box<MiniElixirExpr>,
        key: Box<MiniElixirExpr>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    // Arithmetic
    Add, Sub, Mul, Div, Rem,
    
    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,
    
    // Strict comparison
    StrictEq, StrictNe,
    
    // Logical
    And, Or,
    
    // String operations
    Concat,
    
    // List operations
    Cons,
    
    // Map operations
    Put,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Not,
    Minus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseClause {
    pub pattern: MiniElixirExpr,
    pub guard: Option<MiniElixirExpr>,
    pub body: MiniElixirExpr,
}

/// Evaluation context with variable bindings
#[derive(Debug)]
pub struct EvaluationContext {
    bindings: HashMap<String, EllexValue>,
    functions: HashMap<String, String>, // Just store function names for now
    recursion_depth: usize,
    max_recursion: usize,
}

impl Clone for EvaluationContext {
    fn clone(&self) -> Self {
        Self {
            bindings: self.bindings.clone(),
            functions: self.functions.clone(),
            recursion_depth: self.recursion_depth,
            max_recursion: self.max_recursion,
        }
    }
}

impl EvaluationContext {
    pub fn new() -> Self {
        let mut ctx = Self {
            bindings: HashMap::new(),
            functions: HashMap::new(),
            recursion_depth: 0,
            max_recursion: 100,
        };
        
        // Add built-in functions
        ctx.add_builtins();
        ctx
    }
    
    pub fn with_bindings(bindings: HashMap<String, EllexValue>) -> Self {
        let mut ctx = Self::new();
        ctx.bindings = bindings;
        ctx
    }
    
    pub fn bind(&mut self, name: String, value: EllexValue) {
        self.bindings.insert(name, value);
    }
    
    pub fn get(&self, name: &str) -> Option<&EllexValue> {
        self.bindings.get(name)
    }
    
    pub fn enter_recursion(&mut self) -> EllexResult<()> {
        if self.recursion_depth >= self.max_recursion {
            return Err(EllexError::SafetyViolation {
                reason: format!("Maximum recursion depth of {} exceeded", self.max_recursion),
            });
        }
        self.recursion_depth += 1;
        Ok(())
    }
    
    pub fn exit_recursion(&mut self) {
        if self.recursion_depth > 0 {
            self.recursion_depth -= 1;
        }
    }
    
    fn add_builtins(&mut self) {
        // Add built-in functions here (we'll implement these as function pointers for now)
        // In a real implementation, these would be proper closures
    }
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// MiniElixir interpreter with inline caching support
pub struct MiniElixirInterpreter {
    cached_expressions: HashMap<String, (MiniElixirExpr, CacheEntry)>,
    evaluation_count: u64,
}

impl MiniElixirInterpreter {
    pub fn new() -> Self {
        Self {
            cached_expressions: HashMap::new(),
            evaluation_count: 0,
        }
    }
    
    /// Parse MiniElixir code into AST (placeholder - would need real parser)
    pub fn parse(&self, code: &str) -> EllexResult<MiniElixirExpr> {
        // This is a simplified parser for demonstration
        // In a real implementation, this would be a proper recursive descent parser
        // or use a parser generator like nom or pest
        
        let trimmed = code.trim();
        
        // Simple literal parsing
        if trimmed == "nil" {
            return Ok(MiniElixirExpr::Nil);
        }
        
        if trimmed == "true" {
            return Ok(MiniElixirExpr::Boolean(true));
        }
        
        if trimmed == "false" {
            return Ok(MiniElixirExpr::Boolean(false));
        }
        
        // String literals
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            let content = &trimmed[1..trimmed.len()-1];
            return Ok(MiniElixirExpr::String(content.to_string()));
        }
        
        // Atom literals
        if trimmed.starts_with(':') {
            let atom_name = &trimmed[1..];
            return Ok(MiniElixirExpr::Atom(atom_name.to_string()));
        }
        
        // Integer literals
        if let Ok(num) = trimmed.parse::<i64>() {
            return Ok(MiniElixirExpr::Integer(num));
        }
        
        // Float literals
        if let Ok(num) = trimmed.parse::<f64>() {
            return Ok(MiniElixirExpr::Float(num));
        }
        
        // Simple function calls (simplified)
        if trimmed.contains('(') && trimmed.ends_with(')') {
            let paren_pos = trimmed.find('(').unwrap();
            let func_name = trimmed[..paren_pos].trim().to_string();
            let args_str = &trimmed[paren_pos+1..trimmed.len()-1];
            
            let args = if args_str.trim().is_empty() {
                Vec::new()
            } else {
                // Simple comma-separated argument parsing
                args_str.split(',')
                    .map(|arg| self.parse(arg.trim()))
                    .collect::<Result<Vec<_>, _>>()?
            };
            
            return Ok(MiniElixirExpr::Call {
                function: func_name,
                args,
            });
        }
        
        // Variables (anything else that looks like an identifier)
        if trimmed.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Ok(MiniElixirExpr::Variable(trimmed.to_string()));
        }
        
        // Fallback - treat as string literal
        Ok(MiniElixirExpr::String(trimmed.to_string()))
    }
    
    /// Evaluate MiniElixir expression with caching
    pub fn eval(&mut self, expr: &MiniElixirExpr, ctx: &mut EvaluationContext) -> EllexResult<EllexValue> {
        self.evaluation_count += 1;
        
        // Check for cached results (simple string-based caching for now)
        let expr_key = format!("{:?}", expr);
        
        match expr {
            MiniElixirExpr::Nil => Ok(EllexValue::Nil),
            
            MiniElixirExpr::Boolean(b) => Ok(EllexValue::String(b.to_string())),
            
            MiniElixirExpr::Integer(n) => Ok(EllexValue::Number(*n as f64)),
            
            MiniElixirExpr::Float(f) => Ok(EllexValue::Number(*f)),
            
            MiniElixirExpr::String(s) => Ok(EllexValue::String(s.clone())),
            
            MiniElixirExpr::Atom(a) => Ok(EllexValue::String(format!(":{}", a))),
            
            MiniElixirExpr::Variable(name) => {
                ctx.get(name)
                    .cloned()
                    .ok_or_else(|| EllexError::LogicError {
                        message: format!("Variable '{}' is undefined", name),
                    })
            }
            
            MiniElixirExpr::List(elements) => {
                let mut result = Vec::new();
                for elem in elements {
                    result.push(self.eval(elem, ctx)?);
                }
                Ok(EllexValue::List(result))
            }
            
            MiniElixirExpr::BinaryOp { op, left, right } => {
                self.eval_binary_op(op, left, right, ctx)
            }
            
            MiniElixirExpr::UnaryOp { op, operand } => {
                self.eval_unary_op(op, operand, ctx)
            }
            
            MiniElixirExpr::Call { function, args } => {
                self.eval_function_call(function, args, ctx)
            }
            
            MiniElixirExpr::If { condition, then_branch, else_branch } => {
                let cond_value = self.eval(condition, ctx)?;
                if self.is_truthy(&cond_value) {
                    self.eval(then_branch, ctx)
                } else if let Some(else_expr) = else_branch {
                    self.eval(else_expr, ctx)
                } else {
                    Ok(EllexValue::Nil)
                }
            }
            
            MiniElixirExpr::Block(exprs) => {
                let mut result = EllexValue::Nil;
                for expr in exprs {
                    result = self.eval(expr, ctx)?;
                }
                Ok(result)
            }
            
            MiniElixirExpr::Access { object, key } => {
                let obj_value = self.eval(object, ctx)?;
                let key_value = self.eval(key, ctx)?;
                self.eval_access(&obj_value, &key_value)
            }
            
            // Simplified implementations for other expression types
            MiniElixirExpr::Tuple(elements) => {
                let mut result = Vec::new();
                for elem in elements {
                    result.push(self.eval(elem, ctx)?);
                }
                // Represent tuple as a list for now
                Ok(EllexValue::List(result))
            }
            
            MiniElixirExpr::Map(_) => {
                // Simplified map handling
                Ok(EllexValue::String("map".to_string()))
            }
            
            MiniElixirExpr::Pipe { value, function } => {
                let val = self.eval(value, ctx)?;
                // For pipes, we need to inject the value as the first argument
                match function.as_ref() {
                    MiniElixirExpr::Call { function: func_name, args } => {
                        let mut new_args = vec![MiniElixirExpr::String(val.to_string())];
                        new_args.extend(args.clone());
                        self.eval_function_call(func_name, &new_args, ctx)
                    }
                    _ => Err(EllexError::LogicError {
                        message: "Pipe target must be a function call".to_string(),
                    })
                }
            }
            
            MiniElixirExpr::Match { pattern: _, value } => {
                // Simplified pattern matching - just evaluate the value
                self.eval(value, ctx)
            }
            
            MiniElixirExpr::Case { expr: _, clauses: _ } => {
                // Simplified case handling
                Ok(EllexValue::Nil)
            }
        }
    }
    
    fn eval_binary_op(
        &mut self,
        op: &BinaryOperator,
        left: &MiniElixirExpr,
        right: &MiniElixirExpr,
        ctx: &mut EvaluationContext
    ) -> EllexResult<EllexValue> {
        let left_val = self.eval(left, ctx)?;
        let right_val = self.eval(right, ctx)?;
        
        match op {
            BinaryOperator::Add => self.add_values(&left_val, &right_val),
            BinaryOperator::Sub => self.sub_values(&left_val, &right_val),
            BinaryOperator::Mul => self.mul_values(&left_val, &right_val),
            BinaryOperator::Div => self.div_values(&left_val, &right_val),
            BinaryOperator::Eq => Ok(EllexValue::String((left_val == right_val).to_string())),
            BinaryOperator::Ne => Ok(EllexValue::String((left_val != right_val).to_string())),
            BinaryOperator::Lt => self.compare_values(&left_val, &right_val, |a, b| a < b),
            BinaryOperator::Le => self.compare_values(&left_val, &right_val, |a, b| a <= b),
            BinaryOperator::Gt => self.compare_values(&left_val, &right_val, |a, b| a > b),
            BinaryOperator::Ge => self.compare_values(&left_val, &right_val, |a, b| a >= b),
            BinaryOperator::And => {
                let left_truthy = self.is_truthy(&left_val);
                if left_truthy {
                    Ok(right_val)
                } else {
                    Ok(left_val)
                }
            }
            BinaryOperator::Or => {
                let left_truthy = self.is_truthy(&left_val);
                if left_truthy {
                    Ok(left_val)
                } else {
                    Ok(right_val)
                }
            }
            BinaryOperator::Concat => {
                Ok(EllexValue::String(format!("{}{}", left_val, right_val)))
            }
            _ => Err(EllexError::LogicError {
                message: format!("Binary operator {:?} not implemented", op),
            })
        }
    }
    
    fn eval_unary_op(
        &mut self,
        op: &UnaryOperator,
        operand: &MiniElixirExpr,
        ctx: &mut EvaluationContext
    ) -> EllexResult<EllexValue> {
        let value = self.eval(operand, ctx)?;
        
        match op {
            UnaryOperator::Not => {
                let truthy = self.is_truthy(&value);
                Ok(EllexValue::String((!truthy).to_string()))
            }
            UnaryOperator::Minus => match value {
                EllexValue::Number(n) => Ok(EllexValue::Number(-n)),
                _ => Err(EllexError::LogicError {
                    message: "Cannot negate non-numeric value".to_string(),
                })
            }
        }
    }
    
    fn eval_function_call(
        &mut self,
        function: &str,
        args: &[MiniElixirExpr],
        ctx: &mut EvaluationContext
    ) -> EllexResult<EllexValue> {
        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.eval(arg, ctx)?);
        }
        
        // Built-in functions
        match function {
            "length" => {
                if arg_values.len() != 1 {
                    return Err(EllexError::LogicError {
                        message: "length/1 expects exactly one argument".to_string(),
                    });
                }
                match &arg_values[0] {
                    EllexValue::List(list) => Ok(EllexValue::Number(list.len() as f64)),
                    EllexValue::String(s) => Ok(EllexValue::Number(s.len() as f64)),
                    _ => Err(EllexError::LogicError {
                        message: "length/1 expects a list or string".to_string(),
                    })
                }
            }
            
            "hd" => {
                if arg_values.len() != 1 {
                    return Err(EllexError::LogicError {
                        message: "hd/1 expects exactly one argument".to_string(),
                    });
                }
                match &arg_values[0] {
                    EllexValue::List(list) => {
                        if list.is_empty() {
                            Err(EllexError::LogicError {
                                message: "hd/1 called on empty list".to_string(),
                            })
                        } else {
                            Ok(list[0].clone())
                        }
                    }
                    _ => Err(EllexError::LogicError {
                        message: "hd/1 expects a list".to_string(),
                    })
                }
            }
            
            "tl" => {
                if arg_values.len() != 1 {
                    return Err(EllexError::LogicError {
                        message: "tl/1 expects exactly one argument".to_string(),
                    });
                }
                match &arg_values[0] {
                    EllexValue::List(list) => {
                        if list.is_empty() {
                            Err(EllexError::LogicError {
                                message: "tl/1 called on empty list".to_string(),
                            })
                        } else {
                            Ok(EllexValue::List(list[1..].to_vec()))
                        }
                    }
                    _ => Err(EllexError::LogicError {
                        message: "tl/1 expects a list".to_string(),
                    })
                }
            }
            
            "to_string" => {
                if arg_values.len() != 1 {
                    return Err(EllexError::LogicError {
                        message: "to_string/1 expects exactly one argument".to_string(),
                    });
                }
                Ok(EllexValue::String(arg_values[0].to_string()))
            }
            
            _ => Err(EllexError::LogicError {
                message: format!("Unknown function '{}'", function),
            })
        }
    }
    
    fn eval_access(&self, object: &EllexValue, key: &EllexValue) -> EllexResult<EllexValue> {
        // Simplified access operation
        match (object, key) {
            (EllexValue::List(list), EllexValue::Number(index)) => {
                let idx = *index as usize;
                if idx < list.len() {
                    Ok(list[idx].clone())
                } else {
                    Ok(EllexValue::Nil)
                }
            }
            _ => Ok(EllexValue::Nil)
        }
    }
    
    // Helper functions for operations
    fn add_values(&self, left: &EllexValue, right: &EllexValue) -> EllexResult<EllexValue> {
        match (left, right) {
            (EllexValue::Number(a), EllexValue::Number(b)) => Ok(EllexValue::Number(a + b)),
            (EllexValue::String(a), EllexValue::String(b)) => Ok(EllexValue::String(format!("{}{}", a, b))),
            _ => Err(EllexError::LogicError {
                message: "Cannot add these types".to_string(),
            })
        }
    }
    
    fn sub_values(&self, left: &EllexValue, right: &EllexValue) -> EllexResult<EllexValue> {
        match (left, right) {
            (EllexValue::Number(a), EllexValue::Number(b)) => Ok(EllexValue::Number(a - b)),
            _ => Err(EllexError::LogicError {
                message: "Cannot subtract these types".to_string(),
            })
        }
    }
    
    fn mul_values(&self, left: &EllexValue, right: &EllexValue) -> EllexResult<EllexValue> {
        match (left, right) {
            (EllexValue::Number(a), EllexValue::Number(b)) => Ok(EllexValue::Number(a * b)),
            _ => Err(EllexError::LogicError {
                message: "Cannot multiply these types".to_string(),
            })
        }
    }
    
    fn div_values(&self, left: &EllexValue, right: &EllexValue) -> EllexResult<EllexValue> {
        match (left, right) {
            (EllexValue::Number(a), EllexValue::Number(b)) => {
                if *b == 0.0 {
                    Err(EllexError::LogicError {
                        message: "Division by zero".to_string(),
                    })
                } else {
                    Ok(EllexValue::Number(a / b))
                }
            }
            _ => Err(EllexError::LogicError {
                message: "Cannot divide these types".to_string(),
            })
        }
    }
    
    fn compare_values<F>(&self, left: &EllexValue, right: &EllexValue, op: F) -> EllexResult<EllexValue>
    where
        F: Fn(f64, f64) -> bool,
    {
        match (left, right) {
            (EllexValue::Number(a), EllexValue::Number(b)) => {
                Ok(EllexValue::String(op(*a, *b).to_string()))
            }
            _ => Err(EllexError::LogicError {
                message: "Cannot compare these types".to_string(),
            })
        }
    }
    
    fn is_truthy(&self, value: &EllexValue) -> bool {
        match value {
            EllexValue::Nil => false,
            EllexValue::String(s) => s != "false" && !s.is_empty(),
            EllexValue::Number(n) => *n != 0.0,
            EllexValue::List(l) => !l.is_empty(),
            _ => true,
        }
    }
    
    /// Get evaluation statistics
    pub fn stats(&self) -> InterpreterStats {
        InterpreterStats {
            evaluation_count: self.evaluation_count,
            cached_expressions: self.cached_expressions.len(),
        }
    }
}

impl Default for MiniElixirInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about interpreter performance
#[derive(Debug, Clone)]
pub struct InterpreterStats {
    pub evaluation_count: u64,
    pub cached_expressions: usize,
}

impl fmt::Display for MiniElixirExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MiniElixirExpr::Nil => write!(f, "nil"),
            MiniElixirExpr::Boolean(b) => write!(f, "{}", b),
            MiniElixirExpr::Integer(n) => write!(f, "{}", n),
            MiniElixirExpr::Float(fl) => write!(f, "{}", fl),
            MiniElixirExpr::String(s) => write!(f, r#""{}""#, s),
            MiniElixirExpr::Atom(a) => write!(f, ":{}", a),
            MiniElixirExpr::Variable(v) => write!(f, "{}", v),
            MiniElixirExpr::List(elements) => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            MiniElixirExpr::Call { function, args } => {
                write!(f, "{}(", function)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            _ => write!(f, "{:?}", self), // Fallback for complex expressions
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_literals() {
        let interpreter = MiniElixirInterpreter::new();
        
        assert_eq!(interpreter.parse("nil").unwrap(), MiniElixirExpr::Nil);
        assert_eq!(interpreter.parse("true").unwrap(), MiniElixirExpr::Boolean(true));
        assert_eq!(interpreter.parse("42").unwrap(), MiniElixirExpr::Integer(42));
        assert_eq!(interpreter.parse("3.14").unwrap(), MiniElixirExpr::Float(3.14));
        assert_eq!(interpreter.parse(r#""hello""#).unwrap(), MiniElixirExpr::String("hello".to_string()));
        assert_eq!(interpreter.parse(":atom").unwrap(), MiniElixirExpr::Atom("atom".to_string()));
    }
    
    #[test]
    fn test_eval_literals() {
        let mut interpreter = MiniElixirInterpreter::new();
        let mut ctx = EvaluationContext::new();
        
        assert_eq!(interpreter.eval(&MiniElixirExpr::Nil, &mut ctx).unwrap(), EllexValue::Nil);
        assert_eq!(interpreter.eval(&MiniElixirExpr::Integer(42), &mut ctx).unwrap(), EllexValue::Number(42.0));
        assert_eq!(interpreter.eval(&MiniElixirExpr::String("hello".to_string()), &mut ctx).unwrap(), 
                  EllexValue::String("hello".to_string()));
    }
    
    #[test]
    fn test_binary_operations() {
        let mut interpreter = MiniElixirInterpreter::new();
        let mut ctx = EvaluationContext::new();
        
        let add_expr = MiniElixirExpr::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(MiniElixirExpr::Integer(5)),
            right: Box::new(MiniElixirExpr::Integer(3)),
        };
        
        assert_eq!(interpreter.eval(&add_expr, &mut ctx).unwrap(), EllexValue::Number(8.0));
    }
    
    #[test]
    fn test_variables() {
        let mut interpreter = MiniElixirInterpreter::new();
        let mut ctx = EvaluationContext::new();
        ctx.bind("x".to_string(), EllexValue::Number(42.0));
        
        let var_expr = MiniElixirExpr::Variable("x".to_string());
        assert_eq!(interpreter.eval(&var_expr, &mut ctx).unwrap(), EllexValue::Number(42.0));
    }
    
    #[test]
    fn test_function_calls() {
        let mut interpreter = MiniElixirInterpreter::new();
        let mut ctx = EvaluationContext::new();
        
        let length_expr = MiniElixirExpr::Call {
            function: "length".to_string(),
            args: vec![MiniElixirExpr::List(vec![
                MiniElixirExpr::Integer(1),
                MiniElixirExpr::Integer(2),
                MiniElixirExpr::Integer(3),
            ])],
        };
        
        assert_eq!(interpreter.eval(&length_expr, &mut ctx).unwrap(), EllexValue::Number(3.0));
    }
    
    #[test]
    fn test_if_expression() {
        let mut interpreter = MiniElixirInterpreter::new();
        let mut ctx = EvaluationContext::new();
        
        let if_expr = MiniElixirExpr::If {
            condition: Box::new(MiniElixirExpr::Boolean(true)),
            then_branch: Box::new(MiniElixirExpr::String("yes".to_string())),
            else_branch: Some(Box::new(MiniElixirExpr::String("no".to_string()))),
        };
        
        assert_eq!(interpreter.eval(&if_expr, &mut ctx).unwrap(), EllexValue::String("yes".to_string()));
    }
}