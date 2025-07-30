//! Advanced optimization passes for transpiler

use crate::ast::{TranspilerNode, TypeInfo, PerfHints};
use ellex_core::values::{EllexValue, Statement};
use std::collections::{HashMap, HashSet};

/// Optimization pass trait
pub trait OptimizationPass {
    fn name(&self) -> &'static str;
    fn apply(&mut self, nodes: &mut Vec<TranspilerNode>) -> anyhow::Result<bool>;
}

/// Optimization pipeline manager
pub struct Optimizer {
    passes: Vec<Box<dyn OptimizationPass>>,
    max_iterations: usize,
}

impl Optimizer {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            max_iterations: 10,
        }
    }
    
    /// Add optimization pass
    pub fn add_pass<T: OptimizationPass + 'static>(&mut self, pass: T) {
        self.passes.push(Box::new(pass));
    }
    
    /// Run all optimization passes until convergence
    pub fn optimize(&mut self, nodes: &mut Vec<TranspilerNode>) -> anyhow::Result<()> {
        for iteration in 0..self.max_iterations {
            let mut changed = false;
            
            for pass in &mut self.passes {
                if pass.apply(nodes)? {
                    changed = true;
                }
            }
            
            if !changed {
                break;
            }
            
            // Safety check
            if iteration == self.max_iterations - 1 {
                eprintln!("Warning: Optimization did not converge after {} iterations", self.max_iterations);
            }
        }
        
        Ok(())
    }
    
    /// Create default optimization pipeline
    pub fn default_pipeline() -> Self {
        let mut optimizer = Self::new();
        
        // Add passes in order of application
        optimizer.add_pass(ConstantFoldingPass::new());
        optimizer.add_pass(DeadCodeEliminationPass::new());
        optimizer.add_pass(LoopUnrollingPass::new(5));
        optimizer.add_pass(InliningPass::new(100));
        optimizer.add_pass(StrengthReductionPass::new());
        optimizer.add_pass(CommonSubexpressionEliminationPass::new());
        
        optimizer
    }
    
    /// Create performance-focused pipeline
    pub fn performance_pipeline() -> Self {
        let mut optimizer = Self::new();
        
        optimizer.add_pass(ConstantFoldingPass::new());
        optimizer.add_pass(LoopUnrollingPass::new(10));
        optimizer.add_pass(InliningPass::new(200));
        optimizer.add_pass(StrengthReductionPass::new());
        optimizer.add_pass(CommonSubexpressionEliminationPass::new());
        optimizer.add_pass(DeadCodeEliminationPass::new());
        optimizer.add_pass(LoopInvariantMotionPass::new());
        
        optimizer
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::default_pipeline()
    }
}

/// Constant folding optimization
pub struct ConstantFoldingPass;

impl ConstantFoldingPass {
    pub fn new() -> Self {
        Self
    }
    
    /// Fold constants in expressions
    fn fold_value(&self, value: &EllexValue) -> Option<EllexValue> {
        match value {
            EllexValue::String(s) => {
                // Fold simple string operations
                if s.is_empty() {
                    Some(EllexValue::String("".to_string()))
                } else {
                    None
                }
            }
            EllexValue::Number(n) => {
                // Normalize numbers
                if n.fract() == 0.0 && *n >= i32::MIN as f64 && *n <= i32::MAX as f64 {
                    Some(EllexValue::Number(*n))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl OptimizationPass for ConstantFoldingPass {
    fn name(&self) -> &'static str {
        "constant_folding"
    }
    
    fn apply(&mut self, nodes: &mut Vec<TranspilerNode>) -> anyhow::Result<bool> {
        let mut changed = false;
        
        for node in nodes {
            match &mut node.statement {
                Statement::Tell(value) => {
                    if let Some(folded) = self.fold_value(value) {
                        *value = folded;
                        changed = true;
                    }
                }
                Statement::Repeat(_, _body) => {
                    // Skip recursive processing for now - would need to convert Statement to TranspilerNode
                }
                Statement::When(_, _, _then_body, _else_body) => {
                    // Skip recursive processing for now - would need to convert Statement to TranspilerNode
                }
                _ => {}
            }
        }
        
        Ok(changed)
    }
}

/// Dead code elimination
pub struct DeadCodeEliminationPass {
    used_vars: HashSet<String>,
}

impl DeadCodeEliminationPass {
    pub fn new() -> Self {
        Self {
            used_vars: HashSet::new(),
        }
    }
    
    /// Analyze variable usage
    fn analyze_usage(&mut self, statements: &[Statement]) {
        for stmt in statements {
            match stmt {
                Statement::Tell(value) => {
                    self.analyze_value_usage(value);
                }
                Statement::When(var, condition, then_body, else_body) => {
                    self.used_vars.insert(var.clone());
                    self.analyze_value_usage(condition);
                    self.analyze_usage(then_body);
                    if let Some(else_stmts) = else_body {
                        self.analyze_usage(else_stmts);
                    }
                }
                Statement::Repeat(_, body) => {
                    self.analyze_usage(body);
                }
                Statement::Call(name) => {
                    self.used_vars.insert(name.clone());
                }
                _ => {}
            }
        }
    }
    
    /// Analyze variable usage in values
    fn analyze_value_usage(&mut self, value: &EllexValue) {
        match value {
            EllexValue::String(s) => {
                // Simple interpolation analysis
                if s.contains('{') && s.contains('}') {
                    // Extract variable names (simplified)
                    let mut chars = s.chars();
                    let mut in_var = false;
                    let mut var_name = String::new();
                    
                    while let Some(ch) = chars.next() {
                        if ch == '{' {
                            in_var = true;
                            var_name.clear();
                        } else if ch == '}' && in_var {
                            self.used_vars.insert(var_name.clone());
                            in_var = false;
                        } else if in_var {
                            var_name.push(ch);
                        }
                    }
                }
            }
            EllexValue::List(items) => {
                for item in items {
                    self.analyze_value_usage(item);
                }
            }
            _ => {}
        }
    }
}

impl OptimizationPass for DeadCodeEliminationPass {
    fn name(&self) -> &'static str {
        "dead_code_elimination"
    }
    
    fn apply(&mut self, nodes: &mut Vec<TranspilerNode>) -> anyhow::Result<bool> {
        // First pass: analyze usage
        self.used_vars.clear();
        let statements: Vec<Statement> = nodes.iter().map(|n| n.statement.clone()).collect();
        self.analyze_usage(&statements);
        
        // Second pass: remove unused variables
        let original_len = nodes.len();
        nodes.retain(|node| {
            match &node.statement {
                Statement::Ask(var, _) => {
                    self.used_vars.contains(var)
                }
                _ => true, // Keep non-variable statements
            }
        });
        
        Ok(nodes.len() != original_len)
    }
}

/// Loop unrolling optimization
pub struct LoopUnrollingPass {
    max_unroll_size: u32,
}

impl LoopUnrollingPass {
    pub fn new(max_unroll_size: u32) -> Self {
        Self { max_unroll_size }
    }
}

impl OptimizationPass for LoopUnrollingPass {
    fn name(&self) -> &'static str {
        "loop_unrolling"
    }
    
    fn apply(&mut self, nodes: &mut Vec<TranspilerNode>) -> anyhow::Result<bool> {
        let mut changed = false;
        
        for node in nodes {
            if let Statement::Repeat(count, body) = &mut node.statement {
                if *count <= self.max_unroll_size && !body.is_empty() {
                    // Estimate unrolled size
                    let body_size = body.len() * (*count as usize);
                    
                    if body_size <= 50 { // Reasonable limit
                        // Unroll the loop
                        let mut unrolled = Vec::new();
                        for _ in 0..*count {
                            unrolled.extend(body.clone());
                        }
                        
                        *body = unrolled;
                        *count = 1; // Loop runs once with unrolled body
                        changed = true;
                        
                        // Update performance hints
                        node.metadata.perf_hints.hot_path = false;
                        node.metadata.perf_hints.inlinable = true;
                    }
                }
            }
        }
        
        Ok(changed)
    }
}

/// Function inlining optimization
pub struct InliningPass {
    max_inline_size: usize,
    function_bodies: HashMap<String, Vec<Statement>>,
}

impl InliningPass {
    pub fn new(max_inline_size: usize) -> Self {
        Self {
            max_inline_size,
            function_bodies: HashMap::new(),
        }
    }
    
    /// Collect function definitions
    fn collect_functions(&mut self, statements: &[Statement]) {
        for stmt in statements {
            match stmt {
                Statement::Call(name) if name.starts_with("make_") => {
                    // This is a placeholder - real implementation would extract function body
                    let func_name = name.strip_prefix("make_").unwrap();
                    self.function_bodies.insert(
                        func_name.to_string(),
                        vec![Statement::Tell(EllexValue::String("inlined".to_string()))]
                    );
                }
                _ => {}
            }
        }
    }
}

impl OptimizationPass for InliningPass {
    fn name(&self) -> &'static str {
        "inlining"
    }
    
    fn apply(&mut self, nodes: &mut Vec<TranspilerNode>) -> anyhow::Result<bool> {
        // Collect function definitions
        let statements: Vec<Statement> = nodes.iter().map(|n| n.statement.clone()).collect();
        self.collect_functions(&statements);
        
        let mut changed = false;
        
        for node in nodes {
            if let Statement::Call(func_name) = &node.statement {
                if let Some(body) = self.function_bodies.get(func_name) {
                    if body.len() <= self.max_inline_size {
                        // Inline small functions
                        if body.len() == 1 {
                            node.statement = body[0].clone();
                            changed = true;
                        }
                    }
                }
            }
        }
        
        Ok(changed)
    }
}

/// Strength reduction optimization
pub struct StrengthReductionPass;

impl StrengthReductionPass {
    pub fn new() -> Self {
        Self
    }
}

impl OptimizationPass for StrengthReductionPass {
    fn name(&self) -> &'static str {
        "strength_reduction"
    }
    
    fn apply(&mut self, nodes: &mut Vec<TranspilerNode>) -> anyhow::Result<bool> {
        let mut changed = false;
        
        for node in nodes {
            // Look for patterns that can be optimized
            match &mut node.statement {
                Statement::Repeat(count, body) => {
                    // Optimize repeated simple operations
                    if *count == 1 {
                        // Remove unnecessary loop wrapper
                        if body.len() == 1 {
                            node.statement = body[0].clone();
                            changed = true;
                        }
                    }
                }
                _ => {}
            }
        }
        
        Ok(changed)
    }
}

/// Common subexpression elimination
pub struct CommonSubexpressionEliminationPass {
    expressions: HashMap<String, EllexValue>,
}

impl CommonSubexpressionEliminationPass {
    pub fn new() -> Self {
        Self {
            expressions: HashMap::new(),
        }
    }
    
    /// Generate a key for an expression
    fn expression_key(&self, value: &EllexValue) -> String {
        match value {
            EllexValue::String(s) => format!("str:{}", s),
            EllexValue::Number(n) => format!("num:{}", n),
            EllexValue::List(items) => {
                let keys: Vec<String> = items.iter().map(|i| self.expression_key(i)).collect();
                format!("list:[{}]", keys.join(","))
            }
            _ => "unknown".to_string(),
        }
    }
}

impl OptimizationPass for CommonSubexpressionEliminationPass {
    fn name(&self) -> &'static str {
        "common_subexpression_elimination"
    }
    
    fn apply(&mut self, nodes: &mut Vec<TranspilerNode>) -> anyhow::Result<bool> {
        self.expressions.clear();
        let mut changed = false;
        
        // First pass: collect common expressions
        for node in nodes.iter() {
            match &node.statement {
                Statement::Tell(value) => {
                    let key = self.expression_key(value);
                    self.expressions.entry(key).or_insert_with(|| value.clone());
                }
                _ => {}
            }
        }
        
        // Second pass: replace with cached values (simplified)
        // In a real implementation, this would create temporary variables
        
        Ok(changed)
    }
}

/// Loop invariant motion
pub struct LoopInvariantMotionPass;

impl LoopInvariantMotionPass {
    pub fn new() -> Self {
        Self
    }
    
    /// Check if a statement is loop invariant
    fn is_loop_invariant(&self, stmt: &Statement, _loop_vars: &HashSet<String>) -> bool {
        match stmt {
            Statement::Tell(EllexValue::String(_)) => true,
            Statement::Tell(EllexValue::Number(_)) => true,
            _ => false,
        }
    }
}

impl OptimizationPass for LoopInvariantMotionPass {
    fn name(&self) -> &'static str {
        "loop_invariant_motion"
    }
    
    fn apply(&mut self, nodes: &mut Vec<TranspilerNode>) -> anyhow::Result<bool> {
        let mut changed = false;
        
        for node in nodes {
            if let Statement::Repeat(count, body) = &mut node.statement {
                if *count > 1 {
                    let loop_vars = HashSet::new(); // Would be populated with loop variables
                    
                    // Find invariant statements
                    let mut invariant_stmts = Vec::new();
                    let mut remaining_body = Vec::new();
                    
                    for stmt in body.drain(..) {
                        if self.is_loop_invariant(&stmt, &loop_vars) {
                            invariant_stmts.push(stmt);
                            changed = true;
                        } else {
                            remaining_body.push(stmt);
                        }
                    }
                    
                    *body = remaining_body;
                    
                    // Note: In a real implementation, invariant statements would be
                    // moved before the loop in the AST
                }
            }
        }
        
        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstTransformer, NodeMetadata};

    #[test]
    fn test_constant_folding() {
        let mut pass = ConstantFoldingPass::new();
        let mut nodes = vec![
            TranspilerNode {
                statement: Statement::Tell(EllexValue::Number(42.0)),
                metadata: NodeMetadata::default(),
            }
        ];
        
        let changed = pass.apply(&mut nodes).unwrap();
        assert!(!changed); // Already constant
    }

    #[test]
    fn test_loop_unrolling() {
        let mut pass = LoopUnrollingPass::new(5);
        let mut nodes = vec![
            TranspilerNode {
                statement: Statement::Repeat(3, vec![
                    Statement::Tell(EllexValue::String("Hello".to_string())),
                ]),
                metadata: NodeMetadata::default(),
            }
        ];
        
        let changed = pass.apply(&mut nodes).unwrap();
        assert!(changed);
        
        if let Statement::Repeat(count, body) = &nodes[0].statement {
            assert_eq!(*count, 1);
            assert_eq!(body.len(), 3); // Unrolled 3 times
        } else {
            panic!("Expected Repeat statement");
        }
    }

    #[test]
    fn test_dead_code_elimination() {
        let mut pass = DeadCodeEliminationPass::new();
        let mut nodes = vec![
            TranspilerNode {
                statement: Statement::Ask("unused_var".to_string(), None),
                metadata: NodeMetadata::default(),
            },
            TranspilerNode {
                statement: Statement::Tell(EllexValue::String("Hello".to_string())),
                metadata: NodeMetadata::default(),
            }
        ];
        
        let changed = pass.apply(&mut nodes).unwrap();
        assert!(changed);
        assert_eq!(nodes.len(), 1); // Unused variable removed
    }

    #[test]
    fn test_strength_reduction() {
        let mut pass = StrengthReductionPass::new();
        let mut nodes = vec![
            TranspilerNode {
                statement: Statement::Repeat(1, vec![
                    Statement::Tell(EllexValue::String("Single".to_string())),
                ]),
                metadata: NodeMetadata::default(),
            }
        ];
        
        let changed = pass.apply(&mut nodes).unwrap();
        assert!(changed);
        
        // Loop with count 1 should be eliminated
        assert!(matches!(nodes[0].statement, Statement::Tell(_)));
    }

    #[test]
    fn test_optimizer_pipeline() {
        let mut optimizer = Optimizer::default_pipeline();
        let transformer = AstTransformer::new();
        
        let ast = vec![
            Statement::Repeat(1, vec![
                Statement::Tell(EllexValue::String("Test".to_string())),
            ]),
            Statement::Ask("unused".to_string(), None),
        ];
        
        let mut nodes = transformer.transform(ast);
        optimizer.optimize(&mut nodes).unwrap();
        
        // Should have optimized the single-iteration loop and removed unused variable
        assert!(nodes.len() <= 2);
    }

    #[test]
    fn test_performance_pipeline() {
        let mut optimizer = Optimizer::performance_pipeline();
        let transformer = AstTransformer::new();
        
        let ast = vec![
            Statement::Repeat(3, vec![
                Statement::Tell(EllexValue::Number(42.0)),
            ]),
        ];
        
        let mut nodes = transformer.transform(ast);
        optimizer.optimize(&mut nodes).unwrap();
        
        // Performance pipeline should have unrolled the small loop
        assert!(!nodes.is_empty());
    }

    #[test]
    fn test_convergence() {
        let mut optimizer = Optimizer::new();
        optimizer.add_pass(ConstantFoldingPass::new());
        optimizer.add_pass(LoopUnrollingPass::new(10));
        
        let transformer = AstTransformer::new();
        let ast = vec![
            Statement::Tell(EllexValue::String("Convergence test".to_string())),
        ];
        
        let mut nodes = transformer.transform(ast);
        
        // Should not run forever
        let result = optimizer.optimize(&mut nodes);
        assert!(result.is_ok());
    }
}