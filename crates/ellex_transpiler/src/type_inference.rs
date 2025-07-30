//! Advanced type inference for optimization

use crate::ast::{TypeInfo, TranspilerNode};
use ellex_core::values::{EllexValue, Statement};
use std::collections::HashMap;

/// Type environment for tracking variable types
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    /// Variable type mappings
    variables: HashMap<String, TypeInfo>,
    /// Function signatures
    functions: HashMap<String, FunctionType>,
    /// Current scope level
    scope_level: usize,
}

/// Function type signature
#[derive(Debug, Clone)]
pub struct FunctionType {
    pub params: Vec<TypeInfo>,
    pub returns: TypeInfo,
    pub pure: bool, // No side effects
}

/// Type inference engine
pub struct TypeInference {
    /// Current type environment
    env: TypeEnvironment,
    /// Type constraints collected during inference
    constraints: Vec<TypeConstraint>,
    /// Inferred types cache
    type_cache: HashMap<String, TypeInfo>,
}

/// Type constraint for unification
#[derive(Debug, Clone)]
pub enum TypeConstraint {
    /// Two types must be equal
    Equality(TypeInfo, TypeInfo),
    /// Type must be a subtype of another
    Subtype(TypeInfo, TypeInfo),
    /// Type must support an operation
    Operation(TypeInfo, Operation),
}

/// Supported operations for type checking
#[derive(Debug, Clone)]
pub enum Operation {
    Concatenation, // String + String
    Addition,      // Number + Number
    Comparison,    // Any == Any
    Indexing,      // List[T] -> T
    Interpolation, // String with variables
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            scope_level: 0,
        }
    }
    
    /// Enter new scope
    pub fn enter_scope(&mut self) {
        self.scope_level += 1;
    }
    
    /// Exit current scope
    pub fn exit_scope(&mut self) {
        self.scope_level -= 1;
        // In a full implementation, would remove scope-local variables
    }
    
    /// Add variable type
    pub fn add_variable(&mut self, name: String, ty: TypeInfo) {
        self.variables.insert(name, ty);
    }
    
    /// Get variable type
    pub fn get_variable(&self, name: &str) -> Option<&TypeInfo> {
        self.variables.get(name)
    }
    
    /// Add function signature
    pub fn add_function(&mut self, name: String, func_type: FunctionType) {
        self.functions.insert(name, func_type);
    }
    
    /// Get function type
    pub fn get_function(&self, name: &str) -> Option<&FunctionType> {
        self.functions.get(name)
    }
}

impl TypeInference {
    pub fn new() -> Self {
        let mut env = TypeEnvironment::new();
        
        // Add built-in function types
        env.add_function("tell".to_string(), FunctionType {
            params: vec![TypeInfo::Union(vec![
                TypeInfo::String,
                TypeInfo::Number,
                TypeInfo::List(Box::new(TypeInfo::Unknown)),
            ])],
            returns: TypeInfo::Unknown, // Void-like
            pure: false, // Has side effects (output)
        });
        
        Self {
            env,
            constraints: Vec::new(),
            type_cache: HashMap::new(),
        }
    }
    
    /// Infer types for all nodes in AST
    pub fn infer_types(&mut self, nodes: &mut [TranspilerNode]) -> anyhow::Result<()> {
        // First pass: collect variable declarations and function signatures
        for node in nodes.iter() {
            self.collect_declarations(&node.statement)?;
        }
        
        // Second pass: infer expression types
        for node in nodes.iter_mut() {
            let inferred_type = self.infer_statement_type(&node.statement)?;
            node.metadata.type_info = Some(inferred_type);
        }
        
        // Third pass: solve constraints
        self.solve_constraints()?;
        
        // Fourth pass: propagate refined types
        for node in nodes.iter_mut() {
            if let Some(ref mut type_info) = node.metadata.type_info {
                *type_info = self.refine_type(type_info.clone())?;
            }
        }
        
        Ok(())
    }
    
    /// Collect variable and function declarations
    fn collect_declarations(&mut self, stmt: &Statement) -> anyhow::Result<()> {
        match stmt {
            Statement::Ask(var_name, type_hint) => {
                let var_type = match type_hint {
                    Some(hint) => self.parse_type_hint(hint),
                    None => TypeInfo::String, // Default assumption
                };
                self.env.add_variable(var_name.clone(), var_type);
            }
            
            Statement::Repeat(_, body) => {
                for stmt in body {
                    self.collect_declarations(stmt)?;
                }
            }
            
            Statement::When(var_name, _condition, then_body, else_body) => {
                // Variable should already be declared, but ensure it exists
                if self.env.get_variable(var_name).is_none() {
                    self.env.add_variable(var_name.clone(), TypeInfo::Unknown);
                }
                
                for stmt in then_body {
                    self.collect_declarations(stmt)?;
                }
                
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.collect_declarations(stmt)?;
                    }
                }
            }
            
            Statement::Call(func_name) => {
                // For user-defined functions, would collect signature here
                if !self.env.get_function(func_name).is_some() {
                    // Add placeholder function type
                    self.env.add_function(func_name.clone(), FunctionType {
                        params: vec![],
                        returns: TypeInfo::Unknown,
                        pure: true,
                    });
                }
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    /// Infer type of a statement
    fn infer_statement_type(&mut self, stmt: &Statement) -> anyhow::Result<TypeInfo> {
        match stmt {
            Statement::Tell(value) => {
                let value_type = self.infer_value_type(value)?;
                
                // Add constraint: tell accepts this type
                self.constraints.push(TypeConstraint::Operation(
                    value_type.clone(),
                    Operation::Concatenation, // Tell can output anything
                ));
                
                Ok(TypeInfo::Unknown) // Tell doesn't return a value
            }
            
            Statement::Ask(var_name, type_hint) => {
                let expected_type = match type_hint {
                    Some(hint) => self.parse_type_hint(hint),
                    None => TypeInfo::String,
                };
                
                self.env.add_variable(var_name.clone(), expected_type.clone());
                Ok(expected_type)
            }
            
            Statement::Repeat(_count, body) => {
                // Infer types for body statements
                for stmt in body {
                    self.infer_statement_type(stmt)?;
                }
                Ok(TypeInfo::Unknown) // Repeat doesn't return a value
            }
            
            Statement::When(var_name, condition, then_body, else_body) => {
                // Get variable type
                let var_type = self.env.get_variable(var_name)
                    .cloned()
                    .unwrap_or(TypeInfo::Unknown);
                
                // Infer condition type
                let condition_type = self.infer_value_type(condition)?;
                
                // Add constraint: variable and condition must be comparable
                self.constraints.push(TypeConstraint::Operation(
                    var_type,
                    Operation::Comparison,
                ));
                self.constraints.push(TypeConstraint::Operation(
                    condition_type,
                    Operation::Comparison,
                ));
                
                // Infer branch types
                let mut then_types = Vec::new();
                for stmt in then_body {
                    then_types.push(self.infer_statement_type(stmt)?);
                }
                
                let mut else_types = Vec::new();
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        else_types.push(self.infer_statement_type(stmt)?);
                    }
                }
                
                // Union of possible return types
                let mut all_types = then_types;
                all_types.extend(else_types);
                
                if all_types.is_empty() {
                    Ok(TypeInfo::Unknown)
                } else if all_types.len() == 1 {
                    Ok(all_types[0].clone())
                } else {
                    Ok(TypeInfo::Union(all_types))
                }
            }
            
            Statement::Call(func_name) => {
                if let Some(func_type) = self.env.get_function(func_name) {
                    Ok(func_type.returns.clone())
                } else {
                    Ok(TypeInfo::Unknown)
                }
            }
        }
    }
    
    /// Infer type of a value
    fn infer_value_type(&mut self, value: &EllexValue) -> anyhow::Result<TypeInfo> {
        let type_key = format!("{:?}", value);
        
        if let Some(cached) = self.type_cache.get(&type_key) {
            return Ok(cached.clone());
        }
        
        let inferred_type = match value {
            EllexValue::String(s) => {
                if self.has_interpolation(s) {
                    // String with variable interpolation
                    self.constraints.push(TypeConstraint::Operation(
                        TypeInfo::String,
                        Operation::Interpolation,
                    ));
                    TypeInfo::String
                } else {
                    TypeInfo::String
                }
            }
            
            EllexValue::Number(n) => {
                if n.fract() == 0.0 && *n >= i32::MIN as f64 && *n <= i32::MAX as f64 {
                    TypeInfo::Integer
                } else {
                    TypeInfo::Float
                }
            }
            
            EllexValue::List(items) => {
                if items.is_empty() {
                    TypeInfo::List(Box::new(TypeInfo::Unknown))
                } else {
                    // Infer element type
                    let mut element_types = Vec::new();
                    for item in items {
                        element_types.push(self.infer_value_type(item)?);
                    }
                    
                    // Find common type or create union
                    let element_type = if element_types.iter().all(|t| *t == element_types[0]) {
                        element_types[0].clone()
                    } else {
                        TypeInfo::Union(element_types)
                    };
                    
                    TypeInfo::List(Box::new(element_type))
                }
            }
            
            EllexValue::Function(_) => {
                TypeInfo::Function {
                    params: vec![TypeInfo::Unknown],
                    returns: Box::new(TypeInfo::Unknown),
                }
            }
            
            EllexValue::Nil => TypeInfo::Unknown,
        };
        
        self.type_cache.insert(type_key, inferred_type.clone());
        Ok(inferred_type)
    }
    
    /// Parse type hint string
    fn parse_type_hint(&self, hint: &str) -> TypeInfo {
        match hint {
            "string" => TypeInfo::String,
            "number" => TypeInfo::Number,
            "integer" => TypeInfo::Integer,
            "float" => TypeInfo::Float,
            "list" => TypeInfo::List(Box::new(TypeInfo::Unknown)),
            _ => TypeInfo::Unknown,
        }
    }
    
    /// Check if string has variable interpolation
    fn has_interpolation(&self, s: &str) -> bool {
        s.contains('{') && s.contains('}')
    }
    
    /// Solve type constraints using unification
    fn solve_constraints(&mut self) -> anyhow::Result<()> {
        // Simplified constraint solver
        for constraint in &self.constraints {
            match constraint {
                TypeConstraint::Equality(type1, type2) => {
                    // In a full implementation, would unify these types
                    if !self.types_compatible(type1, type2) {
                        return Err(anyhow::anyhow!(
                            "Type constraint violation: {:?} != {:?}",
                            type1, type2
                        ));
                    }
                }
                
                TypeConstraint::Subtype(subtype, supertype) => {
                    if !self.is_subtype(subtype, supertype) {
                        return Err(anyhow::anyhow!(
                            "Subtype constraint violation: {:?} not <: {:?}",
                            subtype, supertype
                        ));
                    }
                }
                
                TypeConstraint::Operation(type_info, operation) => {
                    if !self.supports_operation(type_info, operation) {
                        return Err(anyhow::anyhow!(
                            "Operation constraint violation: {:?} doesn't support {:?}",
                            type_info, operation
                        ));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if two types are compatible
    fn types_compatible(&self, type1: &TypeInfo, type2: &TypeInfo) -> bool {
        match (type1, type2) {
            (TypeInfo::Unknown, _) | (_, TypeInfo::Unknown) => true,
            (TypeInfo::String, TypeInfo::String) => true,
            (TypeInfo::Number, TypeInfo::Number) => true,
            (TypeInfo::Integer, TypeInfo::Integer) => true,
            (TypeInfo::Float, TypeInfo::Float) => true,
            (TypeInfo::Integer, TypeInfo::Number) => true, // Integer is subtype of Number
            (TypeInfo::Number, TypeInfo::Integer) => true, // For compatibility
            (TypeInfo::List(t1), TypeInfo::List(t2)) => self.types_compatible(t1, t2),
            _ => false,
        }
    }
    
    /// Check if type1 is a subtype of type2
    fn is_subtype(&self, type1: &TypeInfo, type2: &TypeInfo) -> bool {
        match (type1, type2) {
            (_, TypeInfo::Unknown) => true,
            (TypeInfo::Integer, TypeInfo::Number) => true,
            (TypeInfo::Integer, TypeInfo::Float) => true,
            _ => self.types_compatible(type1, type2),
        }
    }
    
    /// Check if type supports operation
    fn supports_operation(&self, type_info: &TypeInfo, operation: &Operation) -> bool {
        match (type_info, operation) {
            (TypeInfo::String, Operation::Concatenation) => true,
            (TypeInfo::String, Operation::Interpolation) => true,
            (TypeInfo::Number, Operation::Addition) => true,
            (TypeInfo::Integer, Operation::Addition) => true,
            (TypeInfo::Float, Operation::Addition) => true,
            (_, Operation::Comparison) => true, // Most types support comparison
            (TypeInfo::List(_), Operation::Indexing) => true,
            _ => false,
        }
    }
    
    /// Refine type based on constraints
    fn refine_type(&self, type_info: TypeInfo) -> anyhow::Result<TypeInfo> {
        match type_info {
            TypeInfo::Union(types) => {
                // Try to simplify unions
                let mut unique_types = Vec::new();
                for ty in types {
                    if !unique_types.contains(&ty) {
                        unique_types.push(ty);
                    }
                }
                
                if unique_types.len() == 1 {
                    Ok(unique_types[0].clone())
                } else {
                    Ok(TypeInfo::Union(unique_types))
                }
            }
            
            TypeInfo::List(element_type) => {
                let refined_element = self.refine_type(*element_type)?;
                Ok(TypeInfo::List(Box::new(refined_element)))
            }
            
            _ => Ok(type_info),
        }
    }
    
    /// Get most specific type for optimization
    pub fn get_specific_type(&self, general_type: &TypeInfo) -> TypeInfo {
        match general_type {
            TypeInfo::Number => TypeInfo::Float, // Assume float for safety
            TypeInfo::Union(types) => {
                // Pick the most specific type
                if types.contains(&TypeInfo::Integer) {
                    TypeInfo::Integer
                } else if types.contains(&TypeInfo::Float) {
                    TypeInfo::Float
                } else if types.contains(&TypeInfo::String) {
                    TypeInfo::String
                } else {
                    types.first().unwrap_or(&TypeInfo::Unknown).clone()
                }
            }
            _ => general_type.clone(),
        }
    }
}

impl Default for TypeInference {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstTransformer, NodeMetadata};

    #[test]
    fn test_type_environment() {
        let mut env = TypeEnvironment::new();
        
        env.add_variable("x".to_string(), TypeInfo::Integer);
        env.add_variable("name".to_string(), TypeInfo::String);
        
        assert!(matches!(env.get_variable("x"), Some(TypeInfo::Integer)));
        assert!(matches!(env.get_variable("name"), Some(TypeInfo::String)));
        assert!(env.get_variable("unknown").is_none());
    }

    #[test]
    fn test_value_type_inference() {
        let mut inference = TypeInference::new();
        
        // String literal
        let string_type = inference.infer_value_type(&EllexValue::String("hello".to_string())).unwrap();
        assert!(matches!(string_type, TypeInfo::String));
        
        // Integer
        let int_type = inference.infer_value_type(&EllexValue::Number(42.0)).unwrap();
        assert!(matches!(int_type, TypeInfo::Integer));
        
        // Float
        let float_type = inference.infer_value_type(&EllexValue::Number(3.14)).unwrap();
        assert!(matches!(float_type, TypeInfo::Float));
        
        // List
        let list_type = inference.infer_value_type(&EllexValue::List(vec![
            EllexValue::Number(1.0),
            EllexValue::Number(2.0),
        ])).unwrap();
        assert!(matches!(list_type, TypeInfo::List(_)));
    }

    #[test]
    fn test_interpolated_string() {
        let mut inference = TypeInference::new();
        
        let interpolated = EllexValue::String("Hello {name}!".to_string());
        let type_info = inference.infer_value_type(&interpolated).unwrap();
        
        assert!(matches!(type_info, TypeInfo::String));
        assert!(!inference.constraints.is_empty()); // Should have added interpolation constraint
    }

    #[test]
    fn test_statement_type_inference() {
        let mut inference = TypeInference::new();
        
        // Tell statement
        let tell_type = inference.infer_statement_type(&Statement::Tell(
            EllexValue::String("Hello".to_string())
        )).unwrap();
        assert!(matches!(tell_type, TypeInfo::Unknown)); // Tell doesn't return
        
        // Ask statement
        let ask_type = inference.infer_statement_type(&Statement::Ask(
            "name".to_string(),
            Some("string".to_string())
        )).unwrap();
        assert!(matches!(ask_type, TypeInfo::String));
    }

    #[test]
    fn test_constraint_solving() {
        let mut inference = TypeInference::new();
        
        // Add compatible constraint
        inference.constraints.push(TypeConstraint::Equality(
            TypeInfo::Integer,
            TypeInfo::Number,
        ));
        
        let result = inference.solve_constraints();
        assert!(result.is_ok());
        
        // Add incompatible constraint
        inference.constraints.push(TypeConstraint::Equality(
            TypeInfo::String,
            TypeInfo::Integer,
        ));
        
        let result = inference.solve_constraints();
        assert!(result.is_err());
    }

    #[test]
    fn test_type_compatibility() {
        let inference = TypeInference::new();
        
        assert!(inference.types_compatible(&TypeInfo::Integer, &TypeInfo::Number));
        assert!(inference.types_compatible(&TypeInfo::String, &TypeInfo::String));
        assert!(!inference.types_compatible(&TypeInfo::String, &TypeInfo::Integer));
        assert!(inference.types_compatible(&TypeInfo::Unknown, &TypeInfo::String));
    }

    #[test]
    fn test_operation_support() {
        let inference = TypeInference::new();
        
        assert!(inference.supports_operation(&TypeInfo::String, &Operation::Concatenation));
        assert!(inference.supports_operation(&TypeInfo::Number, &Operation::Addition));
        assert!(inference.supports_operation(&TypeInfo::List(Box::new(TypeInfo::String)), &Operation::Indexing));
        assert!(!inference.supports_operation(&TypeInfo::String, &Operation::Addition));
    }

    #[test]
    fn test_full_inference_pipeline() {
        let mut inference = TypeInference::new();
        let transformer = AstTransformer::new();
        
        let ast = vec![
            Statement::Ask("age".to_string(), Some("number".to_string())),
            Statement::Tell(EllexValue::String("Hello {name}!".to_string())),
            Statement::When("age".to_string(), EllexValue::Number(18.0), vec![
                Statement::Tell(EllexValue::String("Adult".to_string())),
            ], None),
        ];
        
        let mut nodes = transformer.transform(ast);
        let result = inference.infer_types(&mut nodes);
        
        assert!(result.is_ok());
        
        // Check that types were inferred
        for node in &nodes {
            assert!(node.metadata.type_info.is_some());
        }
    }

    #[test]
    fn test_type_refinement() {
        let inference = TypeInference::new();
        
        // Refine union type
        let union_type = TypeInfo::Union(vec![
            TypeInfo::String,
            TypeInfo::String, // Duplicate
        ]);
        
        let refined = inference.refine_type(union_type).unwrap();
        assert!(matches!(refined, TypeInfo::String)); // Should be simplified
        
        // Refine list type
        let list_type = TypeInfo::List(Box::new(TypeInfo::Union(vec![
            TypeInfo::Integer,
            TypeInfo::Integer,
        ])));
        
        let refined = inference.refine_type(list_type).unwrap();
        if let TypeInfo::List(element_type) = refined {
            assert!(matches!(*element_type, TypeInfo::Integer));
        } else {
            panic!("Expected refined list type");
        }
    }
}