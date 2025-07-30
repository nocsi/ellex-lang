//! JavaScript to Ellex transpiler - converts JS code to natural language

use crate::TranspilerError;
use ellex_core::values::{EllexValue, Statement};
use swc_common::sync::Lrc;
use swc_common::SourceMap;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};
use swc_ecma_ast::*;
use std::collections::HashMap;

/// JavaScript to Ellex parser context
pub struct JsParser {
    /// Variable mappings from JS to Ellex names
    var_mappings: HashMap<String, String>,
    /// Function definitions discovered
    functions: HashMap<String, Vec<Statement>>,
    /// Current scope variables
    scope_vars: Vec<String>,
}

impl JsParser {
    pub fn new() -> Self {
        Self {
            var_mappings: HashMap::new(),
            functions: HashMap::new(),
            scope_vars: Vec::new(),
        }
    }
    
    /// Parse JavaScript AST node to Ellex statements
    fn parse_stmt(&mut self, stmt: &Stmt) -> Result<Vec<Statement>, TranspilerError> {
        match stmt {
            // console.log(...) -> tell ...
            Stmt::Expr(ExprStmt { expr, .. }) => {
                self.parse_expression_stmt(expr)
            }
            
            // Variable declarations: let x = y; -> (internal tracking)
            Stmt::Decl(Decl::Var(var_decl)) => {
                let decls = &var_decl.decls;
                let mut statements = Vec::new();
                for decl in decls {
                    if let Some(stmt) = self.parse_var_decl(decl)? {
                        statements.push(stmt);
                    }
                }
                Ok(statements)
            }
            
            // Function declarations: function name() { ... }
            Stmt::Decl(Decl::Fn(FnDecl { ident, function, .. })) => {
                let func_name = ident.sym.to_string();
                let body = self.parse_function_body(&function.body)?;
                self.functions.insert(func_name.clone(), body);
                
                // Convert to Ellex make statement (placeholder)
                Ok(vec![Statement::Call(format!("make_{}", func_name))])
            }
            
            // For loops: for(...) { ... } -> repeat ... times do ... end
            Stmt::For(ForStmt { test, update, body, .. }) => {
                self.parse_for_loop(test, update, body)
            }
            
            // If statements: if(...) { ... } -> when ... do ... end
            Stmt::If(IfStmt { test, cons, alt, .. }) => {
                self.parse_if_stmt(test, cons, alt.as_deref())
            }
            
            // Block statements: { ... }
            Stmt::Block(BlockStmt { stmts, .. }) => {
                let mut result = Vec::new();
                for stmt in stmts {
                    result.extend(self.parse_stmt(stmt)?);
                }
                Ok(result)
            }
            
            // Return statements (in functions)
            Stmt::Return(ReturnStmt { arg, .. }) => {
                if let Some(arg) = arg {
                    let value = self.parse_expr_to_value(arg)?;
                    Ok(vec![Statement::Tell(value)])
                } else {
                    Ok(vec![])
                }
            }
            
            _ => {
                // Unsupported statement type
                Ok(vec![Statement::Tell(EllexValue::String(
                    format!("# Unsupported JS: {:?}", stmt).to_string()
                ))])
            }
        }
    }
    
    /// Parse expression statement (function calls, assignments, etc.)
    fn parse_expression_stmt(&mut self, expr: &Expr) -> Result<Vec<Statement>, TranspilerError> {
        match expr {
            // console.log(x) -> tell x
            Expr::Call(CallExpr { callee, args, .. }) => {
                match callee {
                    Callee::Expr(expr) => match expr.as_ref() {
                        Expr::Member(member_expr) => {
                            if self.is_console_log(member_expr) {
                                return self.parse_console_log(args);
                            }
                        }
                        Expr::Ident(ident) => {
                            let func_name = ident.sym.to_string();
                            
                            // Check for special runtime functions
                            if func_name == "prompt" && args.len() == 1 {
                                // prompt("question") -> ask "question" → response
                                if let Some(arg) = args.first() {
                                    let question = self.parse_expr_to_value(&arg.expr)?;
                                    return Ok(vec![Statement::Ask("response".to_string(), None)]);
                                }
                            }
                            
                            // Regular function call
                            return Ok(vec![Statement::Call(func_name)]);
                        }
                        _ => {}
                    }
                    _ => {}
                }
                
                Ok(vec![])
            }
            
            // Assignment: x = y
            Expr::Assign(AssignExpr { left, right, .. }) => {
                match left {
                    PatOrExpr::Pat(pat) => {
                        if let Pat::Ident(ident_pat) = pat.as_ref() {
                            let ident = &ident_pat.id;
                            let var_name = ident.sym.to_string();
                            let value = self.parse_expr_to_value(right)?;
                            
                            // Store variable mapping
                            self.var_mappings.insert(var_name.clone(), var_name.clone());
                            
                            // This is an internal assignment, might not need direct Ellex equivalent
                            Ok(vec![])
                        } else {
                            Ok(vec![])
                        }
                    }
                    PatOrExpr::Expr(_) => {
                        // Expression assignment (like obj.prop = value), skip for now
                        Ok(vec![])
                    }
                }
            }
            
            _ => Ok(vec![])
        }
    }
    
    /// Parse variable declaration
    fn parse_var_decl(&mut self, decl: &VarDeclarator) -> Result<Option<Statement>, TranspilerError> {
        if let Pat::Ident(ident_pat) = &decl.name {
            let ident = &ident_pat.id;
            let var_name = ident.sym.to_string();
            self.scope_vars.push(var_name.clone());
            
            if let Some(init) = &decl.init {
                // Check for special patterns like await prompt(...)
                if let Expr::Await(AwaitExpr { arg, .. }) = &**init {
                    if let Expr::Call(CallExpr { callee, args, .. }) = &**arg {
                        if let Callee::Expr(expr) = callee {
                            if let Expr::Ident(ident) = expr.as_ref() {
                                if ident.sym == "prompt" && args.len() == 1 {
                                    let question = self.parse_expr_to_value(&args[0].expr)?;
                                    if let EllexValue::String(q) = question {
                                        return Ok(Some(Statement::Ask(var_name, None)));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Parse function body
    fn parse_function_body(&mut self, body: &Option<BlockStmt>) -> Result<Vec<Statement>, TranspilerError> {
        if let Some(block) = body {
            let mut statements = Vec::new();
            for stmt in &block.stmts {
                statements.extend(self.parse_stmt(stmt)?);
            }
            Ok(statements)
        } else {
            Ok(vec![])
        }
    }
    
    /// Parse for loop to repeat statement
    fn parse_for_loop(&mut self, test: &Option<Box<Expr>>, _update: &Option<Box<Expr>>, body: &Stmt) -> Result<Vec<Statement>, TranspilerError> {
        // Try to extract loop count from simple for loops
        let count = if let Some(test_expr) = test {
            self.extract_loop_count(test_expr).unwrap_or(1)
        } else {
            1
        };
        
        let body_stmts = self.parse_stmt(body)?;
        Ok(vec![Statement::Repeat(count, body_stmts)])
    }
    
    /// Parse if statement to when statement
    fn parse_if_stmt(&mut self, test: &Expr, cons: &Stmt, alt: Option<&Stmt>) -> Result<Vec<Statement>, TranspilerError> {
        // Try to extract variable and condition
        let (var_name, condition) = self.extract_condition(test)?;
        
        let then_body = self.parse_stmt(cons)?;
        let else_body = if let Some(alt_stmt) = alt {
            Some(self.parse_stmt(alt_stmt)?)
        } else {
            None
        };
        
        Ok(vec![Statement::When(var_name, condition, then_body, else_body)])
    }
    
    /// Check if member expression is console.log
    fn is_console_log(&self, member: &MemberExpr) -> bool {
        if let Expr::Ident(obj_ident) = member.obj.as_ref() {
            if let MemberProp::Ident(prop_ident) = &member.prop {
                return obj_ident.sym == "console" && prop_ident.sym == "log";
            }
        }
        false
    }
    
    /// Parse console.log arguments to tell statement
    fn parse_console_log(&mut self, args: &[ExprOrSpread]) -> Result<Vec<Statement>, TranspilerError> {
        let mut statements = Vec::new();
        
        for arg in args {
            let value = self.parse_expr_to_value(&arg.expr)?;
            statements.push(Statement::Tell(value));
        }
        
        Ok(statements)
    }
    
    /// Convert JavaScript expression to Ellex value
    fn parse_expr_to_value(&mut self, expr: &Expr) -> Result<EllexValue, TranspilerError> {
        match expr {
            // String literals
            Expr::Lit(Lit::Str(str_lit)) => {
                Ok(EllexValue::String(str_lit.value.to_string()))
            }
            
            // Number literals
            Expr::Lit(Lit::Num(num_lit)) => {
                Ok(EllexValue::Number(num_lit.value))
            }
            
            // Boolean literals (convert to string for Ellex)
            Expr::Lit(Lit::Bool(bool_lit)) => {
                Ok(EllexValue::String(bool_lit.value.to_string()))
            }
            
            // Array literals
            Expr::Array(ArrayLit { elems, .. }) => {
                let mut items = Vec::new();
                for elem in elems {
                    if let Some(elem) = elem {
                        items.push(self.parse_expr_to_value(&elem.expr)?);
                    }
                }
                Ok(EllexValue::List(items))
            }
            
            // Variable references
            Expr::Ident(ident) => {
                let var_name = ident.sym.to_string();
                // Return as string for now, could be enhanced with symbol table
                Ok(EllexValue::String(var_name))
            }
            
            // Template literals with interpolation
            Expr::Tpl(tpl) => {
                let mut result = String::new();
                
                for (i, quasi) in tpl.quasis.iter().enumerate() {
                    result.push_str(&quasi.raw);
                    
                    if i < tpl.exprs.len() {
                        // Add interpolation placeholder
                        if let Ok(EllexValue::String(var_name)) = self.parse_expr_to_value(&tpl.exprs[i]) {
                            result.push_str(&format!("{{{}}}", var_name));
                        }
                    }
                }
                
                Ok(EllexValue::String(result))
            }
            
            // Binary expressions (simplified)
            Expr::Bin(BinExpr { left, right, .. }) => {
                let left_val = self.parse_expr_to_value(left)?;
                let right_val = self.parse_expr_to_value(right)?;
                
                // For now, just concatenate as string
                Ok(EllexValue::String(format!("{} and {}", left_val, right_val)))
            }
            
            _ => {
                // Fallback for unsupported expressions
                Ok(EllexValue::String("unknown_value".to_string()))
            }
        }
    }
    
    /// Extract loop count from test expression (simple cases)
    fn extract_loop_count(&self, test: &Expr) -> Option<u32> {
        if let Expr::Bin(BinExpr { left, right, op, .. }) = test {
            if matches!(op, BinaryOp::Lt | BinaryOp::LtEq) {
                if let Expr::Lit(Lit::Num(num)) = &**right {
                    return Some(num.value as u32);
                }
            }
        }
        None
    }
    
    /// Extract condition from if test (simplified)
    fn extract_condition(&mut self, test: &Expr) -> Result<(String, EllexValue), TranspilerError> {
        match test {
            Expr::Bin(BinExpr { left, right, op, .. }) => {
                if matches!(op, BinaryOp::EqEq | BinaryOp::EqEqEq) {
                    if let Expr::Ident(var_ident) = left.as_ref() {
                        let var_name = var_ident.sym.to_string();
                        let condition = self.parse_expr_to_value(right)?;
                        return Ok((var_name, condition));
                    }
                }
            }
            
            Expr::Ident(ident) => {
                // Simple boolean variable
                let var_name = ident.sym.to_string();
                return Ok((var_name, EllexValue::String("true".to_string())));
            }
            
            _ => {}
        }
        
        // Fallback
        Ok(("condition".to_string(), EllexValue::String("true".to_string())))
    }
}

/// Parse JavaScript code to Ellex AST
pub fn parse_js_to_ellex(js_code: &str) -> Result<Vec<Statement>, TranspilerError> {
    let source_map: Lrc<SourceMap> = Default::default();
    let source_file = source_map.new_source_file(
        swc_common::FileName::Custom("input.js".into()),
        js_code.into(),
    );
    
    let lexer = Lexer::new(
        Syntax::Typescript(TsConfig {
            tsx: false,
            decorators: false,
            dts: false,
            no_early_errors: true,
            disallow_ambiguous_jsx_like: true,
        }),
        EsVersion::Es2020,
        StringInput::from(&*source_file),
        None,
    );
    
    let mut parser = Parser::new_from(lexer);
    
    let module = parser.parse_module()
        .map_err(|e| TranspilerError::ParseError(format!("SWC parse error: {:?}", e)))?;
    
    let mut js_parser = JsParser::new();
    let mut statements = Vec::new();
    
    for item in module.body {
        match item {
            ModuleItem::Stmt(stmt) => {
                statements.extend(js_parser.parse_stmt(&stmt)?);
            }
            ModuleItem::ModuleDecl(_) => {
                // Skip module declarations for now
            }
        }
    }
    
    Ok(statements)
}

impl Default for JsParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_log_parsing() {
        let js_code = r#"console.log("Hello, world!");"#;
        let result = parse_js_to_ellex(js_code).unwrap();
        
        assert_eq!(result.len(), 1);
        match &result[0] {
            Statement::Tell(EllexValue::String(s)) => {
                assert_eq!(s, "Hello, world!");
            }
            _ => panic!("Expected Tell statement"),
        }
    }

    #[test]
    fn test_multiple_console_logs() {
        let js_code = r#"
            console.log("First");
            console.log(42);
            console.log("Third");
        "#;
        let result = parse_js_to_ellex(js_code).unwrap();
        
        assert_eq!(result.len(), 3);
        assert!(matches!(result[0], Statement::Tell(EllexValue::String(_))));
        assert!(matches!(result[1], Statement::Tell(EllexValue::Number(_))));
        assert!(matches!(result[2], Statement::Tell(EllexValue::String(_))));
    }

    #[test]
    fn test_for_loop_parsing() {
        let js_code = r#"
            for (let i = 0; i < 3; i++) {
                console.log("Loop iteration");
            }
        "#;
        let result = parse_js_to_ellex(js_code).unwrap();
        
        assert_eq!(result.len(), 1);
        match &result[0] {
            Statement::Repeat(count, body) => {
                assert_eq!(*count, 3);
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected Repeat statement"),
        }
    }

    #[test]
    fn test_if_statement_parsing() {
        let js_code = r#"
            if (answer === "yes") {
                console.log("Great!");
            } else {
                console.log("Okay");
            }
        "#;
        let result = parse_js_to_ellex(js_code).unwrap();
        
        assert_eq!(result.len(), 1);
        match &result[0] {
            Statement::When(var, condition, then_body, else_body) => {
                assert_eq!(var, "answer");
                assert!(matches!(condition, EllexValue::String(_)));
                assert_eq!(then_body.len(), 1);
                assert!(else_body.is_some());
            }
            _ => panic!("Expected When statement"),
        }
    }

    #[test]
    fn test_function_declaration() {
        let js_code = r#"
            function greet() {
                console.log("Hello!");
            }
        "#;
        let result = parse_js_to_ellex(js_code).unwrap();
        
        assert_eq!(result.len(), 1);
        match &result[0] {
            Statement::Call(name) => {
                assert_eq!(name, "make_greet");
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_template_literal() {
        let js_code = r#"console.log(`Hello ${name}!`);"#;
        let result = parse_js_to_ellex(js_code).unwrap();
        
        assert_eq!(result.len(), 1);
        match &result[0] {
            Statement::Tell(EllexValue::String(s)) => {
                assert!(s.contains("{name}"));
            }
            _ => panic!("Expected Tell with interpolated string"),
        }
    }

    #[test]
    fn test_array_literal() {
        let js_code = r#"console.log([1, 2, "three"]);"#;
        let result = parse_js_to_ellex(js_code).unwrap();
        
        assert_eq!(result.len(), 1);
        match &result[0] {
            Statement::Tell(EllexValue::List(items)) => {
                assert_eq!(items.len(), 3);
            }
            _ => panic!("Expected Tell with list"),
        }
    }

    #[test]
    fn test_variable_assignment() {
        let js_code = r#"
            let message = "Hello";
            console.log(message);
        "#;
        let result = parse_js_to_ellex(js_code).unwrap();
        
        // Should generate Tell statement for console.log
        assert!(result.len() >= 1);
        assert!(matches!(result.last(), Some(Statement::Tell(_))));
    }
}