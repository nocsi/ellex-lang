use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Represents a value in the Ellex runtime
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EllexValue {
    String(String),
    Number(f64),
    List(Vec<EllexValue>),
    Function(EllexFunction),
    Nil,
}

// Represents a user-defined function
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EllexFunction {
    pub name: String,
    pub body: Vec<Statement>, // We'll define Statement below
    pub params: Vec<String>,  // Parameter names, if any
}

// Placeholder for statements (to be expanded with AST)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Statement {
    Tell(EllexValue),
    Ask(String, Option<String>), // Variable name, optional type hint
    Repeat(u32, Vec<Statement>),
    When(String, EllexValue, Vec<Statement>, Option<Vec<Statement>>), // var, condition, body, optional else
    Call(String),
    Assignment(String, EllexValue), // Variable name, value
}

impl EllexValue {
    // Helper to create string values from literals
    pub fn from_str(s: &str) -> Self {
        EllexValue::String(s.to_string())
    }

    // Helper to interpolate strings (e.g., "Hello, {name}!")
    pub fn interpolate(&self, vars: &HashMap<String, EllexValue>) -> Self {
        match self {
            EllexValue::String(s) => {
                let mut result = s.clone();
                for (var, value) in vars {
                    result = result.replace(&format!("{{{}}}", var), &value.to_string());
                }
                EllexValue::String(result)
            }
            _ => self.clone(),
        }
    }
}

impl std::fmt::Display for EllexValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EllexValue::String(s) => write!(f, "{}", s),
            EllexValue::Number(n) => write!(f, "{}", n),
            EllexValue::List(l) => write!(f, "[{:?}]", l),
            EllexValue::Function(_) => write!(f, "<function>"),
            EllexValue::Nil => write!(f, "nil"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), EllexValue::String("Alice".to_string()));
        let value = EllexValue::String("Hello, {name}!".to_string());
        assert_eq!(value.interpolate(&vars).to_string(), "Hello, Alice!");
    }

    #[test]
    fn test_interpolate_multiple_variables() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), EllexValue::String("Bob".to_string()));
        vars.insert("age".to_string(), EllexValue::Number(25.0));
        let value = EllexValue::String("Hi {name}, you are {age} years old!".to_string());
        assert_eq!(value.interpolate(&vars).to_string(), "Hi Bob, you are 25 years old!");
    }

    #[test]
    fn test_interpolate_no_variables() {
        let vars = HashMap::new();
        let value = EllexValue::String("Hello, world!".to_string());
        assert_eq!(value.interpolate(&vars).to_string(), "Hello, world!");
    }

    #[test]
    fn test_interpolate_missing_variable() {
        let vars = HashMap::new();
        let value = EllexValue::String("Hello, {missing}!".to_string());
        assert_eq!(value.interpolate(&vars).to_string(), "Hello, {missing}!");
    }

    #[test]
    fn test_ellex_value_display() {
        assert_eq!(EllexValue::String("test".to_string()).to_string(), "test");
        assert_eq!(EllexValue::Number(42.5).to_string(), "42.5");
        assert_eq!(EllexValue::Nil.to_string(), "nil");
        
        let list = EllexValue::List(vec![
            EllexValue::Number(1.0),
            EllexValue::String("hello".to_string())
        ]);
        assert!(list.to_string().contains("1"));
        assert!(list.to_string().contains("hello"));
    }

    #[test]
    fn test_ellex_function_creation() {
        let func = EllexFunction {
            name: "greet".to_string(),
            body: vec![Statement::Tell(EllexValue::String("Hello!".to_string()))],
            params: vec!["name".to_string()],
        };
        
        assert_eq!(func.name, "greet");
        assert_eq!(func.params.len(), 1);
        assert_eq!(func.body.len(), 1);
    }

    #[test]
    fn test_statement_equality() {
        let stmt1 = Statement::Tell(EllexValue::String("Hello".to_string()));
        let stmt2 = Statement::Tell(EllexValue::String("Hello".to_string()));
        let stmt3 = Statement::Tell(EllexValue::String("Hi".to_string()));
        
        assert_eq!(stmt1, stmt2);
        assert_ne!(stmt1, stmt3);
    }

    #[test]
    fn test_ellex_value_from_str() {
        let value = EllexValue::from_str("test string");
        match value {
            EllexValue::String(s) => assert_eq!(s, "test string"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_complex_statement_structures() {
        let repeat_stmt = Statement::Repeat(
            3,
            vec![
                Statement::Tell(EllexValue::String("Hello".to_string())),
                Statement::Ask("name".to_string(), Some("string".to_string())),
            ]
        );
        
        match repeat_stmt {
            Statement::Repeat(count, body) => {
                assert_eq!(count, 3);
                assert_eq!(body.len(), 2);
            }
            _ => panic!("Expected repeat statement"),
        }
    }

    #[test]
    fn test_when_statement() {
        let when_stmt = Statement::When(
            "user_input".to_string(),
            EllexValue::String("yes".to_string()),
            vec![Statement::Tell(EllexValue::String("Great!".to_string()))],
            Some(vec![Statement::Tell(EllexValue::String("Okay".to_string()))])
        );
        
        match when_stmt {
            Statement::When(var, condition, then_body, else_body) => {
                assert_eq!(var, "user_input");
                assert_eq!(condition, EllexValue::String("yes".to_string()));
                assert_eq!(then_body.len(), 1);
                assert!(else_body.is_some());
                assert_eq!(else_body.unwrap().len(), 1);
            }
            _ => panic!("Expected when statement"),
        }
    }

    #[test]
    fn test_nested_structures() {
        let nested_list = EllexValue::List(vec![
            EllexValue::Number(1.0),
            EllexValue::List(vec![
                EllexValue::String("nested".to_string()),
                EllexValue::Number(2.0),
            ]),
            EllexValue::String("end".to_string()),
        ]);
        
        match nested_list {
            EllexValue::List(items) => {
                assert_eq!(items.len(), 3);
                match &items[1] {
                    EllexValue::List(inner_items) => {
                        assert_eq!(inner_items.len(), 2);
                    }
                    _ => panic!("Expected nested list"),
                }
            }
            _ => panic!("Expected list"),
        }
    }
}
