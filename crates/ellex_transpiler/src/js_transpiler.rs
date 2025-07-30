//! High-performance Ellex to JavaScript transpiler

use crate::{TranspilerError, TranspilerOptions, Target};
use crate::ast::{AstTransformer, TypeInfo};
use ellex_core::values::{EllexValue, Statement};
use std::collections::HashMap;

/// JavaScript code generation context
pub struct JsCodegen {
    /// Generated code buffer
    output: Vec<String>,
    /// Current indentation level
    indent_level: usize,
    /// Variable name mapping for optimization
    var_mapping: HashMap<String, String>,
    /// Function definitions
    functions: HashMap<String, String>,
    /// Import statements
    imports: Vec<String>,
    /// Optimization flags
    optimize: bool,
}

impl JsCodegen {
    pub fn new(optimize: bool) -> Self {
        Self {
            output: Vec::new(),
            indent_level: 0,
            var_mapping: HashMap::new(),
            functions: HashMap::new(),
            imports: Vec::new(),
            optimize,
        }
    }
    
    /// Add indented line to output
    fn emit_line(&mut self, line: &str) {
        let indent = "  ".repeat(self.indent_level);
        self.output.push(format!("{}{}", indent, line));
    }
    
    /// Add line without indentation
    fn emit_raw(&mut self, line: &str) {
        self.output.push(line.to_string());
    }
    
    /// Increase indentation
    fn indent(&mut self) {
        self.indent_level += 1;
    }
    
    /// Decrease indentation
    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }
    
    /// Generate runtime helpers
    fn emit_runtime_helpers(&mut self) {
        self.emit_raw("// Ellex Runtime Helpers");
        self.emit_raw("const EllexRuntime = {");
        self.indent();
        
        // Input/output helpers
        self.emit_line("tell: (value) => {");
        self.indent();
        self.emit_line("if (typeof value === 'object' && value !== null) {");
        self.indent();
        self.emit_line("console.log(JSON.stringify(value, null, 2));");
        self.dedent();
        self.emit_line("} else {");
        self.indent();
        self.emit_line("console.log(String(value));");
        self.dedent();
        self.emit_line("}");
        self.dedent();
        self.emit_line("},");
        
        // Async input helper
        self.emit_line("ask: async (question) => {");
        self.indent();
        self.emit_line("if (typeof process !== 'undefined' && process.stdin) {");
        self.indent();
        self.emit_line("// Node.js environment");
        self.emit_line("const readline = require('readline');");
        self.emit_line("const rl = readline.createInterface({");
        self.indent();
        self.emit_line("input: process.stdin,");
        self.emit_line("output: process.stdout");
        self.dedent();
        self.emit_line("});");
        self.emit_line("return new Promise(resolve => {");
        self.indent();
        self.emit_line("rl.question(question + ' ', answer => {");
        self.indent();
        self.emit_line("rl.close();");
        self.emit_line("resolve(answer);");
        self.dedent();
        self.emit_line("});");
        self.dedent();
        self.emit_line("});");
        self.dedent();
        self.emit_line("} else {");
        self.indent();
        self.emit_line("// Browser environment");
        self.emit_line("return Promise.resolve(prompt(question) || '');");
        self.dedent();
        self.emit_line("}");
        self.dedent();
        self.emit_line("},");
        
        // Type checking helpers
        self.emit_line("isNumber: (value) => typeof value === 'number' && !isNaN(value),");
        self.emit_line("isString: (value) => typeof value === 'string',");
        self.emit_line("isList: (value) => Array.isArray(value),");
        
        // Safe math operations
        self.emit_line("safeAdd: (a, b) => {");
        self.indent();
        self.emit_line("const numA = Number(a);");
        self.emit_line("const numB = Number(b);");
        self.emit_line("if (isNaN(numA) || isNaN(numB)) throw new Error('Cannot add non-numbers');");
        self.emit_line("return numA + numB;");
        self.dedent();
        self.emit_line("},");
        
        self.dedent();
        self.emit_line("};");
        self.emit_raw("");
    }
    
    /// Generate main execution function
    fn emit_main_function(&mut self, statements: &[Statement]) -> Result<(), TranspilerError> {
        self.emit_line("async function main() {");
        self.indent();
        
        // Variable declarations
        self.emit_line("let variables = new Map();");
        self.emit_raw("");
        
        // Generate statements
        for stmt in statements {
            self.emit_statement(stmt)?;
        }
        
        self.dedent();
        self.emit_line("}");
        self.emit_raw("");
        
        // Execute main function
        self.emit_line("// Execute program");
        self.emit_line("main().catch(error => {");
        self.indent();
        self.emit_line("console.error('Ellex Error:', error.message);");
        self.emit_line("process.exit(1);");
        self.dedent();
        self.emit_line("});");
        
        Ok(())
    }
    
    /// Generate code for a statement
    fn emit_statement(&mut self, stmt: &Statement) -> Result<(), TranspilerError> {
        match stmt {
            Statement::Tell(value) => {
                let js_value = self.emit_value(value)?;
                self.emit_line(&format!("EllexRuntime.tell({});", js_value));
            }
            
            Statement::Ask(var, type_hint) => {
                let question = format!("\"What is {}?\"", var);
                let sanitized_var = self.sanitize_var_name(var);
                self.emit_line(&format!("const {} = await EllexRuntime.ask({});", 
                    sanitized_var, question));
                
                // Type validation if hint provided
                if let Some(hint) = type_hint {
                    match hint.as_str() {
                        "number" => {
                            let sanitized_var_check = self.sanitize_var_name(var);
                            self.emit_line(&format!("if (!EllexRuntime.isNumber(parseFloat({}))) {{", 
                                sanitized_var_check));
                            self.indent();
                            self.emit_line(&format!("throw new Error('Expected number for {}');", var));
                            self.dedent();
                            self.emit_line("}");
                            let sanitized_var = self.sanitize_var_name(var);
                            self.emit_line(&format!("{} = parseFloat({});", 
                                sanitized_var, sanitized_var));
                        }
                        "string" => {
                            let sanitized_var = self.sanitize_var_name(var);
                            self.emit_line(&format!("{} = String({});", 
                                sanitized_var, sanitized_var));
                        }
                        _ => {}
                    }
                }
                
                // Store in variables map for compatibility
                let sanitized_var = self.sanitize_var_name(var);
                self.emit_line(&format!("variables.set('{}', {});", 
                    var, sanitized_var));
            }
            
            Statement::Repeat(count, body) => {
                if self.optimize && *count <= 5 {
                    // Unroll small loops for performance
                    for _ in 0..*count {
                        for body_stmt in body {
                            self.emit_statement(body_stmt)?;
                        }
                    }
                } else {
                    // Use regular loop
                    self.emit_line(&format!("for (let i = 0; i < {}; i++) {{", count));
                    self.indent();
                    for body_stmt in body {
                        self.emit_statement(body_stmt)?;
                    }
                    self.dedent();
                    self.emit_line("}");
                }
            }
            
            Statement::When(var, condition, then_body, else_body) => {
                let condition_js = self.emit_value(condition)?;
                let var_js = self.sanitize_var_name(var);
                
                self.emit_line(&format!("if ({} === {}) {{", var_js, condition_js));
                self.indent();
                for stmt in then_body {
                    self.emit_statement(stmt)?;
                }
                self.dedent();
                
                if let Some(else_stmts) = else_body {
                    self.emit_line("} else {");
                    self.indent();
                    for stmt in else_stmts {
                        self.emit_statement(stmt)?;
                    }
                    self.dedent();
                }
                
                self.emit_line("}");
            }
            
            Statement::Call(func_name) => {
                if let Some(func_code) = self.functions.get(func_name).cloned() {
                    if self.optimize {
                        // Inline small functions
                        self.emit_line(&format!("// Inlined function: {}", func_name));
                        self.emit_raw(&func_code);
                    } else {
                        let sanitized_func = self.sanitize_var_name(func_name);
                        self.emit_line(&format!("{}();", sanitized_func));
                    }
                } else {
                    // Generate function call
                    let sanitized_func = self.sanitize_var_name(func_name);
                    self.emit_line(&format!("{}();", sanitized_func));
                }
            }
        }
        
        Ok(())
    }
    
    /// Generate JavaScript code for a value
    fn emit_value(&mut self, value: &EllexValue) -> Result<String, TranspilerError> {
        match value {
            EllexValue::String(s) => {
                // Escape string and handle interpolation
                let escaped = s.replace('\\', "\\\\")
                              .replace('"', "\\\"")
                              .replace('\n', "\\n")
                              .replace('\r', "\\r")
                              .replace('\t', "\\t");
                
                // Handle simple interpolation {var}
                if escaped.contains('{') && escaped.contains('}') {
                    // Convert to template literal
                    let mut result = escaped.clone();
                    
                    // Simple regex-like replacement for {var} -> ${var}
                    while let Some(start) = result.find('{') {
                        if let Some(end) = result[start..].find('}') {
                            let end = start + end;
                            let var_name = &result[start + 1..end];
                            let js_var = self.sanitize_var_name(var_name);
                            result.replace_range(start..=end, &format!("${{{}}}", js_var));
                        } else {
                            break;
                        }
                    }
                    
                    Ok(format!("`{}`", result))
                } else {
                    Ok(format!("\"{}\"", escaped))
                }
            }
            
            EllexValue::Number(n) => {
                if n.fract() == 0.0 && *n >= i32::MIN as f64 && *n <= i32::MAX as f64 {
                    // Integer
                    Ok(format!("{}", *n as i32))
                } else {
                    // Float
                    Ok(format!("{}", n))
                }
            }
            
            EllexValue::List(items) => {
                let js_items: Result<Vec<String>, TranspilerError> = items
                    .iter()
                    .map(|item| self.emit_value(item))
                    .collect();
                
                Ok(format!("[{}]", js_items?.join(", ")))
            }
            
            EllexValue::Function(_) => {
                Err(TranspilerError::UnsupportedFeature(
                    "Function values not yet supported in JS transpilation".to_string()
                ))
            }
            
            EllexValue::Nil => Ok("null".to_string()),
        }
    }
    
    /// Sanitize variable names for JavaScript
    fn sanitize_var_name(&mut self, name: &str) -> String {
        // Check if already mapped
        if let Some(mapped) = self.var_mapping.get(name) {
            return mapped.clone();
        }
        
        // Create safe JS variable name
        let mut result = String::new();
        let mut chars = name.chars();
        
        // First character must be letter or underscore
        if let Some(first) = chars.next() {
            if first.is_alphabetic() || first == '_' {
                result.push(first);
            } else {
                result.push('_');
                if first.is_numeric() {
                    result.push(first);
                }
            }
        }
        
        // Remaining characters can be alphanumeric or underscore
        for c in chars {
            if c.is_alphanumeric() || c == '_' {
                result.push(c);
            } else {
                result.push('_');
            }
        }
        
        // Avoid JS reserved words
        let js_reserved = [
            "break", "case", "catch", "class", "const", "continue", "debugger",
            "default", "delete", "do", "else", "export", "extends", "finally",
            "for", "function", "if", "import", "in", "instanceof", "new",
            "return", "super", "switch", "this", "throw", "try", "typeof",
            "var", "void", "while", "with", "yield", "let", "static",
            "enum", "implements", "interface", "package", "private",
            "protected", "public"
        ];
        
        if js_reserved.contains(&result.as_str()) {
            result = format!("ellex_{}", result);
        }
        
        // Cache mapping
        self.var_mapping.insert(name.to_string(), result.clone());
        result
    }
    
    /// Get final generated code
    pub fn get_code(&self) -> String {
        self.output.join("\n")
    }
}

/// Transpile Ellex AST to JavaScript
pub fn transpile(ast: &[Statement], options: &TranspilerOptions) -> Result<String, TranspilerError> {
    let optimize = match &options.target {
        Target::JavaScript { optimize, .. } => *optimize,
        _ => false,
    };
    
    let mut codegen = JsCodegen::new(optimize);
    
    // Apply AST transformations if optimizing
    if optimize {
        let transformer = AstTransformer::new();
        let mut nodes = transformer.transform(ast.to_vec());
        transformer.infer_types(&mut nodes);
        transformer.apply_perf_hints(&mut nodes);
        
        // Use transformed AST for better optimization
        let optimized_ast: Vec<Statement> = nodes
            .into_iter()
            .map(|node| node.statement)
            .collect();
        
        codegen.emit_runtime_helpers();
        codegen.emit_main_function(&optimized_ast)?;
    } else {
        codegen.emit_runtime_helpers();
        codegen.emit_main_function(ast)?;
    }
    
    let mut code = codegen.get_code();
    
    // Add source map comment if requested
    if options.source_maps {
        code.push_str("\n//# sourceMappingURL=ellex-output.js.map");
    }
    
    // Minify if requested
    if options.minify {
        code = minify_js(&code)?;
    }
    
    Ok(code)
}

/// Simple JavaScript minification
fn minify_js(code: &str) -> Result<String, TranspilerError> {
    // Basic minification: remove comments and extra whitespace
    let lines: Vec<&str> = code.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .collect();
    
    Ok(lines.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EsVersion;

    #[test]
    fn test_simple_tell_statement() {
        let ast = vec![
            Statement::Tell(EllexValue::String("Hello, world!".to_string())),
        ];
        
        let options = TranspilerOptions {
            target: Target::JavaScript {
                async_support: true,
                es_version: EsVersion::Es2020,
                optimize: false,
            },
            ..Default::default()
        };
        
        let result = transpile(&ast, &options).unwrap();
        assert!(result.contains("EllexRuntime.tell"));
        assert!(result.contains("Hello, world!"));
        assert!(result.contains("async function main"));
    }

    #[test]
    fn test_ask_statement_with_type_hint() {
        let ast = vec![
            Statement::Ask("age".to_string(), Some("number".to_string())),
        ];
        
        let options = TranspilerOptions::default();
        let result = transpile(&ast, &options).unwrap();
        
        assert!(result.contains("EllexRuntime.ask"));
        assert!(result.contains("parseFloat"));
        assert!(result.contains("isNumber"));
    }

    #[test]
    fn test_repeat_statement() {
        let ast = vec![
            Statement::Repeat(3, vec![
                Statement::Tell(EllexValue::String("Hello".to_string())),
            ]),
        ];
        
        let options = TranspilerOptions::default();
        let result = transpile(&ast, &options).unwrap();
        
        assert!(result.contains("for (let i = 0; i < 3; i++)"));
    }

    #[test]
    fn test_optimization_unrolling() {
        let ast = vec![
            Statement::Repeat(2, vec![
                Statement::Tell(EllexValue::String("Optimized".to_string())),
            ]),
        ];
        
        let options = TranspilerOptions {
            target: Target::JavaScript {
                async_support: true,
                es_version: EsVersion::Es2020,
                optimize: true,
            },
            ..Default::default()
        };
        
        let result = transpile(&ast, &options).unwrap();
        // Small loops should be unrolled when optimizing
        assert!(!result.contains("for (let i = 0;"));
    }

    #[test]
    fn test_variable_sanitization() {
        let mut codegen = JsCodegen::new(false);
        
        assert_eq!(codegen.sanitize_var_name("user_name"), "user_name");
        assert_eq!(codegen.sanitize_var_name("123invalid"), "_123invalid");
        assert_eq!(codegen.sanitize_var_name("function"), "ellex_function");
        assert_eq!(codegen.sanitize_var_name("my-var"), "my_var");
    }

    #[test]
    fn test_string_interpolation() {
        let mut codegen = JsCodegen::new(false);
        
        let value = EllexValue::String("Hello {name}!".to_string());
        let result = codegen.emit_value(&value).unwrap();
        
        assert!(result.contains("`"));
        assert!(result.contains("${"));
    }

    #[test]
    fn test_when_statement() {
        let ast = vec![
            Statement::When(
                "answer".to_string(),
                EllexValue::String("yes".to_string()),
                vec![Statement::Tell(EllexValue::String("Great!".to_string()))],
                Some(vec![Statement::Tell(EllexValue::String("Okay".to_string()))]),
            ),
        ];
        
        let options = TranspilerOptions::default();
        let result = transpile(&ast, &options).unwrap();
        
        assert!(result.contains("if (answer === \"yes\")"));
        assert!(result.contains("} else {"));
    }

    #[test]
    fn test_minification() {
        let code = r#"
            // This is a comment
            function test() {
                console.log("hello");
            }
            
            test();
        "#;
        
        let minified = minify_js(code).unwrap();
        assert!(!minified.contains("//"));
        assert!(!minified.contains("\n"));
    }
}