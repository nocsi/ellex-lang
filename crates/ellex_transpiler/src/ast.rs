//! Enhanced AST utilities for transpilation

use ellex_core::values::{EllexValue, Statement};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Extended AST node for transpilation with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranspilerNode {
    pub statement: Statement,
    pub metadata: NodeMetadata,
}

/// Metadata for transpilation optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// Inferred type information
    pub type_info: Option<TypeInfo>,
    /// Performance hints
    pub perf_hints: PerfHints,
    /// Source location (for source maps)
    pub source_loc: Option<SourceLocation>,
    /// Dependencies on other nodes
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfHints {
    /// Can be constant-folded
    pub const_foldable: bool,
    /// Can be inlined
    pub inlinable: bool,
    /// Hot path (executed frequently)
    pub hot_path: bool,
    /// Memory-intensive operation
    pub memory_intensive: bool,
}

/// Type information for optimization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TypeInfo {
    String,
    Number,
    Integer,
    Float,
    List(Box<TypeInfo>),
    Function {
        params: Vec<TypeInfo>,
        returns: Box<TypeInfo>,
    },
    Union(Vec<TypeInfo>),
    Unknown,
}

impl Default for NodeMetadata {
    fn default() -> Self {
        Self {
            type_info: None,
            perf_hints: PerfHints::default(),
            source_loc: None,
            dependencies: Vec::new(),
        }
    }
}

impl Default for PerfHints {
    fn default() -> Self {
        Self {
            const_foldable: false,
            inlinable: false,
            hot_path: false,
            memory_intensive: false,
        }
    }
}

/// AST visitor pattern for traversal and transformation
pub trait AstVisitor {
    fn visit_statement(&mut self, stmt: &mut Statement) -> anyhow::Result<()>;
    fn visit_value(&mut self, value: &mut EllexValue) -> anyhow::Result<()>;
}

/// AST transformer for applying optimizations
pub struct AstTransformer;

impl AstTransformer {
    pub fn new() -> Self {
        Self
    }
    
    /// Transform AST with metadata for transpilation
    pub fn transform(&self, ast: Vec<Statement>) -> Vec<TranspilerNode> {
        ast.into_iter()
            .map(|stmt| TranspilerNode {
                statement: stmt,
                metadata: NodeMetadata::default(),
            })
            .collect()
    }
    
    /// Apply type inference to AST
    pub fn infer_types(&self, nodes: &mut [TranspilerNode]) {
        for node in nodes {
            node.metadata.type_info = Some(self.infer_statement_type(&node.statement));
        }
    }
    
    /// Infer type of a statement
    fn infer_statement_type(&self, stmt: &Statement) -> TypeInfo {
        match stmt {
            Statement::Tell(value) => self.infer_value_type(value),
            Statement::Ask(_, type_hint) => match type_hint {
                Some(hint) => match hint.as_str() {
                    "number" => TypeInfo::Number,
                    "string" => TypeInfo::String,
                    "list" => TypeInfo::List(Box::new(TypeInfo::Unknown)),
                    _ => TypeInfo::Unknown,
                },
                None => TypeInfo::String, // Default assumption
            },
            Statement::Repeat(_, _) => TypeInfo::Unknown,
            Statement::When(_, _, _, _) => TypeInfo::Unknown,
            Statement::Call(_) => TypeInfo::Unknown,
        }
    }
    
    /// Infer type of a value
    fn infer_value_type(&self, value: &EllexValue) -> TypeInfo {
        match value {
            EllexValue::String(_) => TypeInfo::String,
            EllexValue::Number(n) => {
                if n.fract() == 0.0 {
                    TypeInfo::Integer
                } else {
                    TypeInfo::Float
                }
            },
            EllexValue::List(items) => {
                if items.is_empty() {
                    TypeInfo::List(Box::new(TypeInfo::Unknown))
                } else {
                    let first_type = self.infer_value_type(&items[0]);
                    // Check if all items have same type
                    let all_same = items.iter()
                        .skip(1)
                        .all(|item| self.infer_value_type(item) == first_type);
                    
                    if all_same {
                        TypeInfo::List(Box::new(first_type))
                    } else {
                        TypeInfo::List(Box::new(TypeInfo::Union(
                            items.iter().map(|item| self.infer_value_type(item)).collect()
                        )))
                    }
                }
            },
            EllexValue::Function(_) => TypeInfo::Function {
                params: vec![],
                returns: Box::new(TypeInfo::Unknown),
            },
            EllexValue::Nil => TypeInfo::Unknown,
        }
    }
    
    /// Apply performance hints
    pub fn apply_perf_hints(&self, nodes: &mut [TranspilerNode]) {
        for node in nodes {
            node.metadata.perf_hints = self.analyze_performance(&node.statement);
        }
    }
    
    /// Analyze performance characteristics
    fn analyze_performance(&self, stmt: &Statement) -> PerfHints {
        match stmt {
            Statement::Tell(EllexValue::String(s)) if s.len() < 100 => PerfHints {
                const_foldable: true,
                inlinable: true,
                hot_path: false,
                memory_intensive: false,
            },
            Statement::Tell(EllexValue::Number(_)) => PerfHints {
                const_foldable: true,
                inlinable: true,
                hot_path: false,
                memory_intensive: false,
            },
            Statement::Repeat(count, _) if *count > 1000 => PerfHints {
                const_foldable: false,
                inlinable: false,
                hot_path: true,
                memory_intensive: true,
            },
            Statement::Ask(_, _) => PerfHints {
                const_foldable: false,
                inlinable: false,
                hot_path: false,
                memory_intensive: false,
            },
            _ => PerfHints::default(),
        }
    }
}

/// Control flow graph builder for advanced optimizations
pub struct ControlFlowGraph {
    nodes: HashMap<usize, TranspilerNode>,
    edges: HashMap<usize, Vec<usize>>,
}

impl ControlFlowGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }
    
    /// Build CFG from AST
    pub fn build(&mut self, ast: &[TranspilerNode]) {
        for (i, node) in ast.iter().enumerate() {
            self.nodes.insert(i, node.clone());
            
            // Build edges based on control flow
            match &node.statement {
                Statement::When(_, _, then_body, else_body) => {
                    // Add edges for conditional branches
                    let then_start = i + 1;
                    self.edges.entry(i).or_default().push(then_start);
                    
                    if else_body.is_some() {
                        let else_start = then_start + then_body.len();
                        self.edges.entry(i).or_default().push(else_start);
                    }
                },
                Statement::Repeat(_, body) => {
                    // Add edge to loop body and back
                    let body_start = i + 1;
                    self.edges.entry(i).or_default().push(body_start);
                    let body_end = body_start + body.len() - 1;
                    self.edges.entry(body_end).or_default().push(i);
                },
                _ => {
                    // Sequential execution
                    if i + 1 < ast.len() {
                        self.edges.entry(i).or_default().push(i + 1);
                    }
                }
            }
        }
    }
    
    /// Find hot paths in the CFG
    pub fn find_hot_paths(&self) -> Vec<Vec<usize>> {
        // Simple implementation: find cycles (loops are hot)
        let mut hot_paths = Vec::new();
        
        for (&node_id, edges) in &self.edges {
            for &target in edges {
                if target <= node_id {
                    // Back edge detected (potential loop)
                    hot_paths.push(vec![target, node_id]);
                }
            }
        }
        
        hot_paths
    }
}

impl Default for ControlFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_transformer() {
        let transformer = AstTransformer::new();
        let ast = vec![
            Statement::Tell(EllexValue::String("Hello".to_string())),
            Statement::Tell(EllexValue::Number(42.0)),
        ];
        
        let mut nodes = transformer.transform(ast);
        transformer.infer_types(&mut nodes);
        transformer.apply_perf_hints(&mut nodes);
        
        assert_eq!(nodes.len(), 2);
        assert!(matches!(nodes[0].metadata.type_info, Some(TypeInfo::String)));
        assert!(matches!(nodes[1].metadata.type_info, Some(TypeInfo::Integer)));
    }

    #[test]
    fn test_type_inference() {
        let transformer = AstTransformer::new();
        
        // Test string type
        let string_type = transformer.infer_value_type(&EllexValue::String("test".to_string()));
        assert_eq!(string_type, TypeInfo::String);
        
        // Test integer type
        let int_type = transformer.infer_value_type(&EllexValue::Number(42.0));
        assert_eq!(int_type, TypeInfo::Integer);
        
        // Test float type
        let float_type = transformer.infer_value_type(&EllexValue::Number(3.14));
        assert_eq!(float_type, TypeInfo::Float);
        
        // Test list type
        let list_type = transformer.infer_value_type(&EllexValue::List(vec![
            EllexValue::Number(1.0),
            EllexValue::Number(2.0),
        ]));
        assert!(matches!(list_type, TypeInfo::List(_)));
    }

    #[test]
    fn test_control_flow_graph() {
        let mut cfg = ControlFlowGraph::new();
        let transformer = AstTransformer::new();
        
        let ast = vec![
            Statement::Tell(EllexValue::String("Start".to_string())),
            Statement::Repeat(3, vec![
                Statement::Tell(EllexValue::String("Loop".to_string())),
            ]),
        ];
        
        let nodes = transformer.transform(ast);
        cfg.build(&nodes);
        
        let hot_paths = cfg.find_hot_paths();
        assert!(!hot_paths.is_empty());
    }

    #[test]
    fn test_performance_hints() {
        let transformer = AstTransformer::new();
        
        // Small string should be inlinable
        let hints = transformer.analyze_performance(
            &Statement::Tell(EllexValue::String("Hi".to_string()))
        );
        assert!(hints.inlinable);
        assert!(hints.const_foldable);
        
        // Large loop should be marked as hot path
        let hints = transformer.analyze_performance(
            &Statement::Repeat(5000, vec![
                Statement::Tell(EllexValue::String("Loop".to_string())),
            ])
        );
        assert!(hints.hot_path);
        assert!(hints.memory_intensive);
    }
}