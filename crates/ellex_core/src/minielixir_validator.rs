use crate::minielixir::{MiniElixirExpr, BinaryOperator, UnaryOperator, CaseClause};
use crate::{EllexError, EllexResult};
use std::collections::HashSet;

/// Validator for MiniElixir expressions - ensures safe execution
pub struct MiniElixirValidator {
    max_depth: usize,
    max_list_size: usize,
    allowed_functions: HashSet<String>,
    forbidden_patterns: Vec<String>,
}

impl MiniElixirValidator {
    pub fn new() -> Self {
        let mut allowed_functions = HashSet::new();
        
        // Built-in safe functions
        allowed_functions.insert("length".to_string());
        allowed_functions.insert("hd".to_string());
        allowed_functions.insert("tl".to_string());
        allowed_functions.insert("to_string".to_string());
        allowed_functions.insert("is_list".to_string());
        allowed_functions.insert("is_atom".to_string());
        allowed_functions.insert("is_number".to_string());
        allowed_functions.insert("is_binary".to_string());
        allowed_functions.insert("elem".to_string());
        allowed_functions.insert("tuple_size".to_string());
        
        Self {
            max_depth: 50,           // Maximum AST depth to prevent stack overflow
            max_list_size: 1000,     // Maximum list size to prevent memory issues
            allowed_functions,
            forbidden_patterns: vec![
                "System".to_string(),
                "File".to_string(),
                "Process".to_string(),
                ":os".to_string(),
                "spawn".to_string(),
            ],
        }
    }
    
    pub fn with_custom_functions(mut self, functions: Vec<String>) -> Self {
        for func in functions {
            self.allowed_functions.insert(func);
        }
        self
    }
    
    /// Validate an expression for safety and correctness
    pub fn validate(&self, expr: &MiniElixirExpr) -> EllexResult<()> {
        self.validate_recursive(expr, 0)
    }
    
    fn validate_recursive(&self, expr: &MiniElixirExpr, depth: usize) -> EllexResult<()> {
        // Check maximum depth to prevent stack overflow
        if depth > self.max_depth {
            return Err(EllexError::SafetyViolation {
                reason: format!("Expression too deeply nested (max: {})", self.max_depth),
            });
        }
        
        match expr {
            // Literals are always safe
            MiniElixirExpr::Nil |
            MiniElixirExpr::Boolean(_) |
            MiniElixirExpr::Integer(_) |
            MiniElixirExpr::Float(_) => Ok(()),
            
            // Validate string contents
            MiniElixirExpr::String(s) => {
                self.validate_string_content(s)?;
                Ok(())
            }
            
            // Validate atom names
            MiniElixirExpr::Atom(atom) => {
                self.validate_atom_name(atom)?;
                Ok(())
            }
            
            // Variables are safe (just identifiers)
            MiniElixirExpr::Variable(name) => {
                self.validate_variable_name(name)?;
                Ok(())
            }
            
            // Validate list contents and size
            MiniElixirExpr::List(elements) => {
                if elements.len() > self.max_list_size {
                    return Err(EllexError::SafetyViolation {
                        reason: format!("List too large (max: {})", self.max_list_size),
                    });
                }
                
                for element in elements {
                    self.validate_recursive(element, depth + 1)?;
                }
                Ok(())
            }
            
            // Validate tuple contents
            MiniElixirExpr::Tuple(elements) => {
                if elements.len() > self.max_list_size {
                    return Err(EllexError::SafetyViolation {
                        reason: format!("Tuple too large (max: {})", self.max_list_size),
                    });
                }
                
                for element in elements {
                    self.validate_recursive(element, depth + 1)?;
                }
                Ok(())
            }
            
            // Validate map key-value pairs
            MiniElixirExpr::Map(pairs) => {
                if pairs.len() > self.max_list_size {
                    return Err(EllexError::SafetyViolation {
                        reason: format!("Map too large (max: {})", self.max_list_size),
                    });
                }
                
                for (key, value) in pairs {
                    self.validate_recursive(key, depth + 1)?;
                    self.validate_recursive(value, depth + 1)?;
                }
                Ok(())
            }
            
            // Validate binary operations
            MiniElixirExpr::BinaryOp { op, left, right } => {
                self.validate_binary_operator(op)?;
                self.validate_recursive(left, depth + 1)?;
                self.validate_recursive(right, depth + 1)?;
                Ok(())
            }
            
            // Validate unary operations
            MiniElixirExpr::UnaryOp { op, operand } => {
                self.validate_unary_operator(op)?;
                self.validate_recursive(operand, depth + 1)?;
                Ok(())
            }
            
            // Validate function calls
            MiniElixirExpr::Call { function, args } => {
                self.validate_function_call(function, args)?;
                for arg in args {
                    self.validate_recursive(arg, depth + 1)?;
                }
                Ok(())
            }
            
            // Validate pipe operations
            MiniElixirExpr::Pipe { value, function } => {
                self.validate_recursive(value, depth + 1)?;
                self.validate_recursive(function, depth + 1)?;
                Ok(())
            }
            
            // Validate pattern matching
            MiniElixirExpr::Match { pattern, value } => {
                self.validate_recursive(pattern, depth + 1)?;
                self.validate_recursive(value, depth + 1)?;
                Ok(())
            }
            
            // Validate case expressions
            MiniElixirExpr::Case { expr, clauses } => {
                self.validate_recursive(expr, depth + 1)?;
                for clause in clauses {
                    self.validate_case_clause(clause, depth + 1)?;
                }
                Ok(())
            }
            
            // Validate if expressions
            MiniElixirExpr::If { condition, then_branch, else_branch } => {
                self.validate_recursive(condition, depth + 1)?;
                self.validate_recursive(then_branch, depth + 1)?;
                if let Some(else_expr) = else_branch {
                    self.validate_recursive(else_expr, depth + 1)?;
                }
                Ok(())
            }
            
            // Validate block expressions
            MiniElixirExpr::Block(exprs) => {
                if exprs.len() > self.max_list_size {
                    return Err(EllexError::SafetyViolation {
                        reason: format!("Block too large (max: {})", self.max_list_size),
                    });
                }
                
                for expr in exprs {
                    self.validate_recursive(expr, depth + 1)?;
                }
                Ok(())
            }
            
            // Validate access operations
            MiniElixirExpr::Access { object, key } => {
                self.validate_recursive(object, depth + 1)?;
                self.validate_recursive(key, depth + 1)?;
                Ok(())
            }
        }
    }
    
    fn validate_string_content(&self, content: &str) -> EllexResult<()> {
        // Check for forbidden patterns in strings
        for pattern in &self.forbidden_patterns {
            if content.contains(pattern) {
                return Err(EllexError::SafetyViolation {
                    reason: format!("Forbidden pattern '{}' found in string", pattern),
                });
            }
        }
        
        // Check string length
        if content.len() > 10000 {
            return Err(EllexError::SafetyViolation {
                reason: "String too long (max: 10000 characters)".to_string(),
            });
        }
        
        Ok(())
    }
    
    fn validate_atom_name(&self, atom: &str) -> EllexResult<()> {
        // Check for forbidden patterns in atom names
        for pattern in &self.forbidden_patterns {
            if atom.contains(pattern) {
                return Err(EllexError::SafetyViolation {
                    reason: format!("Forbidden pattern '{}' found in atom", pattern),
                });
            }
        }
        
        // Validate atom name format
        if !atom.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(EllexError::ParseError {
                line: 0,
                column: 0,
                message: "Invalid atom name format".to_string(),
            });
        }
        
        Ok(())
    }
    
    fn validate_variable_name(&self, name: &str) -> EllexResult<()> {
        // Variable names must start with lowercase letter or underscore
        if let Some(first_char) = name.chars().next() {
            if !first_char.is_lowercase() && first_char != '_' {
                return Err(EllexError::ParseError {
                    line: 0,
                    column: 0,
                    message: "Variable names must start with lowercase letter or underscore".to_string(),
                });
            }
        }
        
        // Check for valid characters
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(EllexError::ParseError {
                line: 0,
                column: 0,
                message: "Invalid characters in variable name".to_string(),
            });
        }
        
        Ok(())
    }
    
    fn validate_binary_operator(&self, op: &BinaryOperator) -> EllexResult<()> {
        // All currently defined binary operators are safe
        match op {
            BinaryOperator::Add | BinaryOperator::Sub | BinaryOperator::Mul | 
            BinaryOperator::Div | BinaryOperator::Rem |
            BinaryOperator::Eq | BinaryOperator::Ne | BinaryOperator::Lt | 
            BinaryOperator::Le | BinaryOperator::Gt | BinaryOperator::Ge |
            BinaryOperator::StrictEq | BinaryOperator::StrictNe |
            BinaryOperator::And | BinaryOperator::Or |
            BinaryOperator::Concat | BinaryOperator::Cons | BinaryOperator::Put => Ok(()),
        }
    }
    
    fn validate_unary_operator(&self, op: &UnaryOperator) -> EllexResult<()> {
        // All currently defined unary operators are safe
        match op {
            UnaryOperator::Not | UnaryOperator::Minus => Ok(()),
        }
    }
    
    fn validate_function_call(&self, function: &str, args: &[MiniElixirExpr]) -> EllexResult<()> {
        // Check if function is in allowed list
        if !self.allowed_functions.contains(function) {
            return Err(EllexError::SafetyViolation {
                reason: format!("Function '{}' is not allowed", function),
            });
        }
        
        // Validate argument count for known functions
        match function {
            "length" | "hd" | "tl" | "to_string" | "is_list" | 
            "is_atom" | "is_number" | "is_binary" | "tuple_size" => {
                if args.len() != 1 {
                    return Err(EllexError::LogicError {
                        message: format!("Function '{}/1' expects exactly one argument", function),
                    });
                }
            }
            "elem" => {
                if args.len() != 2 {
                    return Err(EllexError::LogicError {
                        message: "Function 'elem/2' expects exactly two arguments".to_string(),
                    });
                }
            }
            _ => {} // Unknown functions (should have been caught above)
        }
        
        Ok(())
    }
    
    fn validate_case_clause(&self, clause: &CaseClause, depth: usize) -> EllexResult<()> {
        self.validate_recursive(&clause.pattern, depth)?;
        if let Some(guard) = &clause.guard {
            self.validate_recursive(guard, depth)?;
        }
        self.validate_recursive(&clause.body, depth)?;
        Ok(())
    }
    
    /// Check if an expression is deterministic (no side effects)
    pub fn is_deterministic(&self, expr: &MiniElixirExpr) -> bool {
        match expr {
            // Literals are always deterministic
            MiniElixirExpr::Nil | MiniElixirExpr::Boolean(_) | 
            MiniElixirExpr::Integer(_) | MiniElixirExpr::Float(_) |
            MiniElixirExpr::String(_) | MiniElixirExpr::Atom(_) |
            MiniElixirExpr::Variable(_) => true,
            
            // Collections are deterministic if their contents are
            MiniElixirExpr::List(elements) | MiniElixirExpr::Tuple(elements) => {
                elements.iter().all(|e| self.is_deterministic(e))
            }
            
            MiniElixirExpr::Map(pairs) => {
                pairs.iter().all(|(k, v)| self.is_deterministic(k) && self.is_deterministic(v))
            }
            
            // Operations are deterministic if their operands are
            MiniElixirExpr::BinaryOp { left, right, .. } => {
                self.is_deterministic(left) && self.is_deterministic(right)
            }
            
            MiniElixirExpr::UnaryOp { operand, .. } => {
                self.is_deterministic(operand)
            }
            
            // Most function calls are deterministic (for our safe subset)
            MiniElixirExpr::Call { function, args } => {
                match function.as_str() {
                    "length" | "hd" | "tl" | "to_string" | "is_list" | 
                    "is_atom" | "is_number" | "is_binary" | "elem" | "tuple_size" => {
                        args.iter().all(|arg| self.is_deterministic(arg))
                    }
                    _ => false, // Unknown functions assumed non-deterministic
                }
            }
            
            // Other expressions
            MiniElixirExpr::Pipe { value, function } => {
                self.is_deterministic(value) && self.is_deterministic(function)
            }
            
            MiniElixirExpr::Match { pattern, value } => {
                self.is_deterministic(pattern) && self.is_deterministic(value)
            }
            
            MiniElixirExpr::If { condition, then_branch, else_branch } => {
                self.is_deterministic(condition) && 
                self.is_deterministic(then_branch) &&
                else_branch.as_ref().map_or(true, |e| self.is_deterministic(e))
            }
            
            MiniElixirExpr::Block(exprs) => {
                exprs.iter().all(|e| self.is_deterministic(e))
            }
            
            MiniElixirExpr::Access { object, key } => {
                self.is_deterministic(object) && self.is_deterministic(key)
            }
            
            // Case expressions are complex, assume non-deterministic for safety
            MiniElixirExpr::Case { .. } => false,
        }
    }
    
    /// Get complexity score for an expression (for caching decisions)
    pub fn complexity_score(&self, expr: &MiniElixirExpr) -> usize {
        match expr {
            // Literals have minimal complexity
            MiniElixirExpr::Nil | MiniElixirExpr::Boolean(_) | 
            MiniElixirExpr::Integer(_) | MiniElixirExpr::Float(_) |
            MiniElixirExpr::String(_) | MiniElixirExpr::Atom(_) |
            MiniElixirExpr::Variable(_) => 1,
            
            // Collections add complexity based on size
            MiniElixirExpr::List(elements) | MiniElixirExpr::Tuple(elements) => {
                1 + elements.iter().map(|e| self.complexity_score(e)).sum::<usize>()
            }
            
            MiniElixirExpr::Map(pairs) => {
                1 + pairs.iter().map(|(k, v)| self.complexity_score(k) + self.complexity_score(v)).sum::<usize>()
            }
            
            // Operations add to the complexity of their operands
            MiniElixirExpr::BinaryOp { left, right, .. } => {
                2 + self.complexity_score(left) + self.complexity_score(right)
            }
            
            MiniElixirExpr::UnaryOp { operand, .. } => {
                2 + self.complexity_score(operand)
            }
            
            // Function calls have higher complexity
            MiniElixirExpr::Call { args, .. } => {
                5 + args.iter().map(|arg| self.complexity_score(arg)).sum::<usize>()
            }
            
            // Other complex expressions
            MiniElixirExpr::Pipe { value, function } => {
                3 + self.complexity_score(value) + self.complexity_score(function)
            }
            
            MiniElixirExpr::Match { pattern, value } => {
                4 + self.complexity_score(pattern) + self.complexity_score(value)
            }
            
            MiniElixirExpr::Case { expr, clauses } => {
                10 + self.complexity_score(expr) + 
                clauses.iter().map(|c| {
                    self.complexity_score(&c.pattern) + 
                    c.guard.as_ref().map_or(0, |g| self.complexity_score(g)) +
                    self.complexity_score(&c.body)
                }).sum::<usize>()
            }
            
            MiniElixirExpr::If { condition, then_branch, else_branch } => {
                5 + self.complexity_score(condition) + 
                self.complexity_score(then_branch) +
                else_branch.as_ref().map_or(0, |e| self.complexity_score(e))
            }
            
            MiniElixirExpr::Block(exprs) => {
                2 + exprs.iter().map(|e| self.complexity_score(e)).sum::<usize>()
            }
            
            MiniElixirExpr::Access { object, key } => {
                3 + self.complexity_score(object) + self.complexity_score(key)
            }
        }
    }
}

impl Default for MiniElixirValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minielixir::{MiniElixirExpr, BinaryOperator};

    #[test]
    fn test_validate_literals() {
        let validator = MiniElixirValidator::new();
        
        assert!(validator.validate(&MiniElixirExpr::Nil).is_ok());
        assert!(validator.validate(&MiniElixirExpr::Boolean(true)).is_ok());
        assert!(validator.validate(&MiniElixirExpr::Integer(42)).is_ok());
        assert!(validator.validate(&MiniElixirExpr::String("hello".to_string())).is_ok());
        assert!(validator.validate(&MiniElixirExpr::Atom("test".to_string())).is_ok());
    }
    
    #[test]
    fn test_validate_forbidden_strings() {
        let validator = MiniElixirValidator::new();
        
        let forbidden_expr = MiniElixirExpr::String("System.cmd('rm -rf /')".to_string());
        assert!(validator.validate(&forbidden_expr).is_err());
    }
    
    #[test]
    fn test_validate_allowed_functions() {
        let validator = MiniElixirValidator::new();
        
        let length_call = MiniElixirExpr::Call {
            function: "length".to_string(),
            args: vec![MiniElixirExpr::List(vec![MiniElixirExpr::Integer(1)])],
        };
        assert!(validator.validate(&length_call).is_ok());
    }
    
    #[test]
    fn test_validate_forbidden_functions() {
        let validator = MiniElixirValidator::new();
        
        let spawn_call = MiniElixirExpr::Call {
            function: "spawn".to_string(),
            args: vec![],
        };
        assert!(validator.validate(&spawn_call).is_err());
    }
    
    #[test]
    fn test_complexity_scoring() {
        let validator = MiniElixirValidator::new();
        
        let simple_expr = MiniElixirExpr::Integer(42);
        let complex_expr = MiniElixirExpr::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(MiniElixirExpr::Integer(1)),
            right: Box::new(MiniElixirExpr::Integer(2)),
        };
        
        assert!(validator.complexity_score(&simple_expr) < validator.complexity_score(&complex_expr));
    }
    
    #[test]
    fn test_deterministic_check() {
        let validator = MiniElixirValidator::new();
        
        let deterministic = MiniElixirExpr::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(MiniElixirExpr::Integer(1)),
            right: Box::new(MiniElixirExpr::Integer(2)),
        };
        
        assert!(validator.is_deterministic(&deterministic));
    }
}