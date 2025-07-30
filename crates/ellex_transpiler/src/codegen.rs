//! Code generation utilities and backends

use crate::{TranspilerError, TranspilerOptions, Target};
use crate::ast::{TranspilerNode, TypeInfo};
use ellex_core::values::{EllexValue, Statement};
use std::collections::HashMap;

/// Code generation trait for different backends
pub trait CodeGenerator {
    /// Target language/platform
    fn target_name(&self) -> &'static str;
    
    /// Generate code for a list of statements
    fn generate(&mut self, nodes: &[TranspilerNode], options: &TranspilerOptions) -> Result<String, TranspilerError>;
    
    /// Generate code for a single statement
    fn generate_statement(&mut self, node: &TranspilerNode) -> Result<String, TranspilerError>;
    
    /// Generate code for a value
    fn generate_value(&mut self, value: &EllexValue, type_hint: Option<&TypeInfo>) -> Result<String, TranspilerError>;
    
    /// Generate runtime setup code
    fn generate_runtime(&mut self) -> Result<String, TranspilerError> {
        Ok(String::new()) // Default: no runtime
    }
    
    /// Generate imports/dependencies
    fn generate_imports(&mut self) -> Result<String, TranspilerError> {
        Ok(String::new()) // Default: no imports
    }
    
    /// Post-process generated code
    fn post_process(&mut self, code: String, options: &TranspilerOptions) -> Result<String, TranspilerError> {
        Ok(code) // Default: no post-processing
    }
}

/// Multi-target code generator
pub struct MultiTargetCodegen {
    generators: HashMap<String, Box<dyn CodeGenerator>>,
}

impl MultiTargetCodegen {
    pub fn new() -> Self {
        let mut generators: HashMap<String, Box<dyn CodeGenerator>> = HashMap::new();
        
        // Register built-in generators
        generators.insert("javascript".to_string(), Box::new(JavaScriptGenerator::new()));
        generators.insert("typescript".to_string(), Box::new(TypeScriptGenerator::new()));
        generators.insert("python".to_string(), Box::new(PythonGenerator::new()));
        generators.insert("go".to_string(), Box::new(GoGenerator::new()));
        
        Self { generators }
    }
    
    /// Register a custom code generator
    pub fn register<T: CodeGenerator + 'static>(&mut self, name: String, generator: T) {
        self.generators.insert(name, Box::new(generator));
    }
    
    /// Generate code for target
    pub fn generate_for_target(&mut self, target: &str, nodes: &[TranspilerNode], options: &TranspilerOptions) -> Result<String, TranspilerError> {
        if let Some(generator) = self.generators.get_mut(target) {
            generator.generate(nodes, options)
        } else {
            Err(TranspilerError::UnsupportedFeature(format!("No generator for target: {}", target)))
        }
    }
    
    /// Get available targets
    pub fn available_targets(&self) -> Vec<&String> {
        self.generators.keys().collect()
    }
}

impl Default for MultiTargetCodegen {
    fn default() -> Self {
        Self::new()
    }
}

/// JavaScript code generator
pub struct JavaScriptGenerator {
    indent_level: usize,
    use_strict: bool,
    es_version: String,
}

impl JavaScriptGenerator {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            use_strict: true,
            es_version: "ES2020".to_string(),
        }
    }
    
    fn indent(&self) -> String {
        "  ".repeat(self.indent_level)
    }
    
    fn escape_string(&self, s: &str) -> String {
        s.replace('\\', "\\\\")
         .replace('"', "\\\"")
         .replace('\n', "\\n")
         .replace('\r', "\\r")
         .replace('\t', "\\t")
    }
}

impl CodeGenerator for JavaScriptGenerator {
    fn target_name(&self) -> &'static str {
        "JavaScript"
    }
    
    fn generate(&mut self, nodes: &[TranspilerNode], options: &TranspilerOptions) -> Result<String, TranspilerError> {
        let mut code = Vec::new();
        
        if self.use_strict {
            code.push("'use strict';".to_string());
            code.push("".to_string());
        }
        
        // Generate imports
        let imports = self.generate_imports()?;
        if !imports.is_empty() {
            code.push(imports);
            code.push("".to_string());
        }
        
        // Generate runtime
        let runtime = self.generate_runtime()?;
        if !runtime.is_empty() {
            code.push(runtime);
            code.push("".to_string());
        }
        
        // Generate main code
        code.push("async function main() {".to_string());
        self.indent_level += 1;
        
        for node in nodes {
            let stmt_code = self.generate_statement(node)?;
            if !stmt_code.is_empty() {
                for line in stmt_code.lines() {
                    code.push(format!("{}{}", self.indent(), line));
                }
            }
        }
        
        self.indent_level -= 1;
        code.push("}".to_string());
        code.push("".to_string());
        code.push("main().catch(console.error);".to_string());
        
        let result = code.join("\n");
        self.post_process(result, options)
    }
    
    fn generate_statement(&mut self, node: &TranspilerNode) -> Result<String, TranspilerError> {
        match &node.statement {
            Statement::Tell(value) => {
                let value_code = self.generate_value(value, node.metadata.type_info.as_ref())?;
                Ok(format!("console.log({});", value_code))
            }
            
            Statement::Ask(var_name, _) => {
                Ok(format!("const {} = await ask('Enter {}:');", var_name, var_name))
            }
            
            Statement::Repeat(count, body) => {
                let mut code = Vec::new();
                code.push(format!("for (let i = 0; i < {}; i++) {{", count));
                
                self.indent_level += 1;
                for stmt in body {
                    let node = TranspilerNode {
                        statement: stmt.clone(),
                        metadata: Default::default(),
                    };
                    let stmt_code = self.generate_statement(&node)?;
                    for line in stmt_code.lines() {
                        code.push(format!("{}{}", self.indent(), line));
                    }
                }
                self.indent_level -= 1;
                
                code.push("}".to_string());
                Ok(code.join("\n"))
            }
            
            Statement::When(var, condition, then_body, else_body) => {
                let condition_code = self.generate_value(condition, None)?;
                let mut code = Vec::new();
                
                code.push(format!("if ({} === {}) {{", var, condition_code));
                
                self.indent_level += 1;
                for stmt in then_body {
                    let node = TranspilerNode {
                        statement: stmt.clone(),
                        metadata: Default::default(),
                    };
                    let stmt_code = self.generate_statement(&node)?;
                    for line in stmt_code.lines() {
                        code.push(format!("{}{}", self.indent(), line));
                    }
                }
                self.indent_level -= 1;
                
                if let Some(else_stmts) = else_body {
                    code.push("} else {".to_string());
                    self.indent_level += 1;
                    for stmt in else_stmts {
                        let node = TranspilerNode {
                            statement: stmt.clone(),
                            metadata: Default::default(),
                        };
                        let stmt_code = self.generate_statement(&node)?;
                        for line in stmt_code.lines() {
                            code.push(format!("{}{}", self.indent(), line));
                        }
                    }
                    self.indent_level -= 1;
                }
                
                code.push("}".to_string());
                Ok(code.join("\n"))
            }
            
            Statement::Call(func_name) => {
                Ok(format!("{}();", func_name))
            }
        }
    }
    
    fn generate_value(&mut self, value: &EllexValue, type_hint: Option<&TypeInfo>) -> Result<String, TranspilerError> {
        match value {
            EllexValue::String(s) => {
                if s.contains('{') && s.contains('}') {
                    // Template literal
                    let mut result = s.clone();
                    // Convert {var} to ${var}
                    while let Some(start) = result.find('{') {
                        if let Some(end) = result[start..].find('}') {
                            let end = start + end;
                            let var_name = &result[start + 1..end];
                            result.replace_range(start..=end, &format!("${{{}}}", var_name));
                        } else {
                            break;
                        }
                    }
                    Ok(format!("`{}`", result))
                } else {
                    Ok(format!("\"{}\"", self.escape_string(s)))
                }
            }
            
            EllexValue::Number(n) => {
                match type_hint {
                    Some(TypeInfo::Integer) => Ok(format!("{}", *n as i64)),
                    Some(TypeInfo::Float) => Ok(format!("{}", n)),
                    _ => {
                        if n.fract() == 0.0 {
                            Ok(format!("{}", *n as i64))
                        } else {
                            Ok(format!("{}", n))
                        }
                    }
                }
            }
            
            EllexValue::List(items) => {
                let item_codes: Result<Vec<String>, TranspilerError> = items
                    .iter()
                    .map(|item| self.generate_value(item, None))
                    .collect();
                Ok(format!("[{}]", item_codes?.join(", ")))
            }
            
            EllexValue::Function(_) => {
                Err(TranspilerError::UnsupportedFeature("Function values not supported in JS generation".to_string()))
            }
            
            EllexValue::Nil => Ok("null".to_string()),
        }
    }
    
    fn generate_runtime(&mut self) -> Result<String, TranspilerError> {
        Ok(r#"
// Ellex Runtime for JavaScript
async function ask(prompt) {
    if (typeof process !== 'undefined' && process.stdin) {
        // Node.js environment
        const readline = require('readline');
        const rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout
        });
        return new Promise(resolve => {
            rl.question(prompt + ' ', answer => {
                rl.close();
                resolve(answer);
            });
        });
    } else {
        // Browser environment
        return prompt(prompt) || '';
    }
}
"#.to_string())
    }
}

/// TypeScript code generator
pub struct TypeScriptGenerator {
    js_generator: JavaScriptGenerator,
    emit_types: bool,
}

impl TypeScriptGenerator {
    pub fn new() -> Self {
        Self {
            js_generator: JavaScriptGenerator::new(),
            emit_types: true,
        }
    }
    
    fn generate_type_annotation(&self, type_info: &TypeInfo) -> String {
        match type_info {
            TypeInfo::String => ": string".to_string(),
            TypeInfo::Number | TypeInfo::Integer | TypeInfo::Float => ": number".to_string(),
            TypeInfo::List(element_type) => {
                format!(": {}[]", self.generate_type_annotation(element_type).trim_start_matches(": "))
            }
            TypeInfo::Function { params, returns } => {
                let param_types: Vec<String> = params.iter()
                    .enumerate()
                    .map(|(i, t)| format!("p{}{}", i, self.generate_type_annotation(t)))
                    .collect();
                let return_type = self.generate_type_annotation(returns).trim_start_matches(": ").to_string();
                format!(": ({}) => {}", param_types.join(", "), return_type)
            }
            TypeInfo::Union(types) => {
                let type_strs: Vec<String> = types.iter()
                    .map(|t| self.generate_type_annotation(t).trim_start_matches(": ").to_string())
                    .collect();
                format!(": {}", type_strs.join(" | "))
            }
            TypeInfo::Unknown => ": any".to_string(),
        }
    }
}

impl CodeGenerator for TypeScriptGenerator {
    fn target_name(&self) -> &'static str {
        "TypeScript"
    }
    
    fn generate(&mut self, nodes: &[TranspilerNode], options: &TranspilerOptions) -> Result<String, TranspilerError> {
        // Generate JavaScript code first
        let js_code = self.js_generator.generate(nodes, options)?;
        
        // Add TypeScript-specific modifications
        let mut ts_code = js_code;
        
        if self.emit_types {
            // Add type annotations (simplified)
            ts_code = format!("// Generated TypeScript from Ellex\n{}", ts_code);
        }
        
        Ok(ts_code)
    }
    
    fn generate_statement(&mut self, node: &TranspilerNode) -> Result<String, TranspilerError> {
        match &node.statement {
            Statement::Ask(var_name, _) => {
                let type_annotation = if let Some(type_info) = &node.metadata.type_info {
                    self.generate_type_annotation(type_info)
                } else {
                    ": string".to_string()
                };
                Ok(format!("const {}{} = await ask('Enter {}:');", var_name, type_annotation, var_name))
            }
            _ => self.js_generator.generate_statement(node),
        }
    }
    
    fn generate_value(&mut self, value: &EllexValue, type_hint: Option<&TypeInfo>) -> Result<String, TranspilerError> {
        self.js_generator.generate_value(value, type_hint)
    }
    
    fn generate_runtime(&mut self) -> Result<String, TranspilerError> {
        Ok(r#"
// Ellex Runtime for TypeScript
async function ask(prompt: string): Promise<string> {
    if (typeof process !== 'undefined' && process.stdin) {
        // Node.js environment
        const readline = require('readline');
        const rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout
        });
        return new Promise<string>(resolve => {
            rl.question(prompt + ' ', (answer: string) => {
                rl.close();
                resolve(answer);
            });
        });
    } else {
        // Browser environment
        return prompt(prompt) || '';
    }
}
"#.to_string())
    }
}

/// Python code generator
pub struct PythonGenerator {
    indent_level: usize,
}

impl PythonGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }
    
    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }
}

impl CodeGenerator for PythonGenerator {
    fn target_name(&self) -> &'static str {
        "Python"
    }
    
    fn generate(&mut self, nodes: &[TranspilerNode], _options: &TranspilerOptions) -> Result<String, TranspilerError> {
        let mut code = Vec::new();
        
        code.push("#!/usr/bin/env python3".to_string());
        code.push("# Generated Python from Ellex".to_string());
        code.push("".to_string());
        
        let runtime = self.generate_runtime()?;
        code.push(runtime);
        code.push("".to_string());
        
        code.push("def main():".to_string());
        self.indent_level += 1;
        
        for node in nodes {
            let stmt_code = self.generate_statement(node)?;
            for line in stmt_code.lines() {
                code.push(format!("{}{}", self.indent(), line));
            }
        }
        
        if nodes.is_empty() {
            code.push(format!("{}pass", self.indent()));
        }
        
        self.indent_level -= 1;
        code.push("".to_string());
        code.push("if __name__ == '__main__':".to_string());
        code.push("    main()".to_string());
        
        Ok(code.join("\n"))
    }
    
    fn generate_statement(&mut self, node: &TranspilerNode) -> Result<String, TranspilerError> {
        match &node.statement {
            Statement::Tell(value) => {
                let value_code = self.generate_value(value, None)?;
                Ok(format!("print({})", value_code))
            }
            
            Statement::Ask(var_name, _) => {
                Ok(format!("{} = input('Enter {}: ')", var_name, var_name))
            }
            
            Statement::Repeat(count, body) => {
                let mut code = Vec::new();
                code.push(format!("for i in range({}):", count));
                
                self.indent_level += 1;
                for stmt in body {
                    let node = TranspilerNode {
                        statement: stmt.clone(),
                        metadata: Default::default(),
                    };
                    let stmt_code = self.generate_statement(&node)?;
                    for line in stmt_code.lines() {
                        code.push(format!("{}{}", self.indent(), line));
                    }
                }
                self.indent_level -= 1;
                
                Ok(code.join("\n"))
            }
            
            Statement::When(var, condition, then_body, else_body) => {
                let condition_code = self.generate_value(condition, None)?;
                let mut code = Vec::new();
                
                code.push(format!("if {} == {}:", var, condition_code));
                
                self.indent_level += 1;
                for stmt in then_body {
                    let node = TranspilerNode {
                        statement: stmt.clone(),
                        metadata: Default::default(),
                    };
                    let stmt_code = self.generate_statement(&node)?;
                    for line in stmt_code.lines() {
                        code.push(format!("{}{}", self.indent(), line));
                    }
                }
                self.indent_level -= 1;
                
                if let Some(else_stmts) = else_body {
                    code.push("else:".to_string());
                    self.indent_level += 1;
                    for stmt in else_stmts {
                        let node = TranspilerNode {
                            statement: stmt.clone(),
                            metadata: Default::default(),
                        };
                        let stmt_code = self.generate_statement(&node)?;
                        for line in stmt_code.lines() {
                            code.push(format!("{}{}", self.indent(), line));
                        }
                    }
                    self.indent_level -= 1;
                }
                
                Ok(code.join("\n"))
            }
            
            Statement::Call(func_name) => {
                Ok(format!("{}()", func_name))
            }
        }
    }
    
    fn generate_value(&mut self, value: &EllexValue, _type_hint: Option<&TypeInfo>) -> Result<String, TranspilerError> {
        match value {
            EllexValue::String(s) => {
                if s.contains('{') && s.contains('}') {
                    // f-string
                    Ok(format!("f\"{}\"", s.replace('{', "{").replace('}', "}")))
                } else {
                    Ok(format!("\"{}\"", s.replace('"', "\\\"")))
                }
            }
            
            EllexValue::Number(n) => {
                Ok(format!("{}", n))
            }
            
            EllexValue::List(items) => {
                let item_codes: Result<Vec<String>, TranspilerError> = items
                    .iter()
                    .map(|item| self.generate_value(item, None))
                    .collect();
                Ok(format!("[{}]", item_codes?.join(", ")))
            }
            
            EllexValue::Function(_) => {
                Err(TranspilerError::UnsupportedFeature("Function values not supported in Python generation".to_string()))
            }
            
            EllexValue::Nil => Ok("None".to_string()),
        }
    }
    
    fn generate_runtime(&mut self) -> Result<String, TranspilerError> {
        Ok("# Ellex Runtime for Python".to_string())
    }
}

/// Go code generator
pub struct GoGenerator {
    indent_level: usize,
    package_name: String,
}

impl GoGenerator {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            package_name: "main".to_string(),
        }
    }
    
    fn indent(&self) -> String {
        "\t".repeat(self.indent_level)
    }
}

impl CodeGenerator for GoGenerator {
    fn target_name(&self) -> &'static str {
        "Go"
    }
    
    fn generate(&mut self, nodes: &[TranspilerNode], _options: &TranspilerOptions) -> Result<String, TranspilerError> {
        let mut code = Vec::new();
        
        code.push(format!("package {}", self.package_name));
        code.push("".to_string());
        code.push("import (".to_string());
        code.push("\t\"fmt\"".to_string());
        code.push(")".to_string());
        code.push("".to_string());
        
        code.push("func main() {".to_string());
        self.indent_level += 1;
        
        for node in nodes {
            let stmt_code = self.generate_statement(node)?;
            for line in stmt_code.lines() {
                code.push(format!("{}{}", self.indent(), line));
            }
        }
        
        self.indent_level -= 1;
        code.push("}".to_string());
        
        Ok(code.join("\n"))
    }
    
    fn generate_statement(&mut self, node: &TranspilerNode) -> Result<String, TranspilerError> {
        match &node.statement {
            Statement::Tell(value) => {
                let value_code = self.generate_value(value, None)?;
                Ok(format!("fmt.Println({})", value_code))
            }
            
            Statement::Ask(var_name, _) => {
                Ok(format!("var {}\nfmt.Print(\"Enter {}: \")\nfmt.Scanln(&{})", var_name, var_name, var_name))
            }
            
            Statement::Repeat(count, body) => {
                let mut code = Vec::new();
                code.push(format!("for i := 0; i < {}; i++ {{", count));
                
                self.indent_level += 1;
                for stmt in body {
                    let node = TranspilerNode {
                        statement: stmt.clone(),
                        metadata: Default::default(),
                    };
                    let stmt_code = self.generate_statement(&node)?;
                    for line in stmt_code.lines() {
                        code.push(format!("{}{}", self.indent(), line));
                    }
                }
                self.indent_level -= 1;
                
                code.push("}".to_string());
                Ok(code.join("\n"))
            }
            
            Statement::When(var, condition, then_body, else_body) => {
                let condition_code = self.generate_value(condition, None)?;
                let mut code = Vec::new();
                
                code.push(format!("if {} == {} {{", var, condition_code));
                
                self.indent_level += 1;
                for stmt in then_body {
                    let node = TranspilerNode {
                        statement: stmt.clone(),
                        metadata: Default::default(),
                    };
                    let stmt_code = self.generate_statement(&node)?;
                    for line in stmt_code.lines() {
                        code.push(format!("{}{}", self.indent(), line));
                    }
                }
                self.indent_level -= 1;
                
                if let Some(else_stmts) = else_body {
                    code.push("} else {".to_string());
                    self.indent_level += 1;
                    for stmt in else_stmts {
                        let node = TranspilerNode {
                            statement: stmt.clone(),
                            metadata: Default::default(),
                        };
                        let stmt_code = self.generate_statement(&node)?;
                        for line in stmt_code.lines() {
                            code.push(format!("{}{}", self.indent(), line));
                        }
                    }
                    self.indent_level -= 1;
                }
                
                code.push("}".to_string());
                Ok(code.join("\n"))
            }
            
            Statement::Call(func_name) => {
                Ok(format!("{}()", func_name))
            }
        }
    }
    
    fn generate_value(&mut self, value: &EllexValue, _type_hint: Option<&TypeInfo>) -> Result<String, TranspilerError> {
        match value {
            EllexValue::String(s) => {
                Ok(format!("\"{}\"", s.replace('"', "\\\"")))
            }
            
            EllexValue::Number(n) => {
                Ok(format!("{}", n))
            }
            
            EllexValue::List(items) => {
                let item_codes: Result<Vec<String>, TranspilerError> = items
                    .iter()
                    .map(|item| self.generate_value(item, None))
                    .collect();
                Ok(format!("[]interface{{{{{}}}}}", item_codes?.join(", ")))
            }
            
            EllexValue::Function(_) => {
                Err(TranspilerError::UnsupportedFeature("Function values not supported in Go generation".to_string()))
            }
            
            EllexValue::Nil => Ok("nil".to_string()),
        }
    }
    
    fn generate_runtime(&mut self) -> Result<String, TranspilerError> {
        Ok("// Ellex Runtime for Go".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::NodeMetadata;

    #[test]
    fn test_javascript_generator() {
        let mut generator = JavaScriptGenerator::new();
        let nodes = vec![
            TranspilerNode {
                statement: Statement::Tell(EllexValue::String("Hello, world!".to_string())),
                metadata: NodeMetadata::default(),
            }
        ];
        
        let options = TranspilerOptions::default();
        let result = generator.generate(&nodes, &options).unwrap();
        
        assert!(result.contains("console.log"));
        assert!(result.contains("Hello, world!"));
        assert!(result.contains("async function main"));
    }

    #[test]
    fn test_typescript_generator() {
        let mut generator = TypeScriptGenerator::new();
        let nodes = vec![
            TranspilerNode {
                statement: Statement::Ask("name".to_string(), None),
                metadata: NodeMetadata {
                    type_info: Some(TypeInfo::String),
                    ..Default::default()
                },
            }
        ];
        
        let options = TranspilerOptions::default();
        let result = generator.generate(&nodes, &options).unwrap();
        
        assert!(result.contains("Generated TypeScript"));
        assert!(result.contains(": string"));
    }

    #[test]
    fn test_python_generator() {
        let mut generator = PythonGenerator::new();
        let nodes = vec![
            TranspilerNode {
                statement: Statement::Tell(EllexValue::Number(42.0)),
                metadata: NodeMetadata::default(),
            }
        ];
        
        let options = TranspilerOptions::default();
        let result = generator.generate(&nodes, &options).unwrap();
        
        assert!(result.contains("print(42)"));
        assert!(result.contains("def main():"));
        assert!(result.contains("if __name__ == '__main__':"));
    }

    #[test]
    fn test_go_generator() {
        let mut generator = GoGenerator::new();
        let nodes = vec![
            TranspilerNode {
                statement: Statement::Tell(EllexValue::String("Hello, Go!".to_string())),
                metadata: NodeMetadata::default(),
            }
        ];
        
        let options = TranspilerOptions::default();
        let result = generator.generate(&nodes, &options).unwrap();
        
        assert!(result.contains("package main"));
        assert!(result.contains("fmt.Println"));
        assert!(result.contains("Hello, Go!"));
    }

    #[test]
    fn test_multi_target_codegen() {
        let mut codegen = MultiTargetCodegen::new();
        let nodes = vec![
            TranspilerNode {
                statement: Statement::Tell(EllexValue::String("Multi-target test".to_string())),
                metadata: NodeMetadata::default(),
            }
        ];
        let options = TranspilerOptions::default();
        
        let targets = codegen.available_targets();
        assert!(targets.contains(&&"javascript".to_string()));
        assert!(targets.contains(&&"python".to_string()));
        
        let js_result = codegen.generate_for_target("javascript", &nodes, &options);
        assert!(js_result.is_ok());
        
        let py_result = codegen.generate_for_target("python", &nodes, &options);
        assert!(py_result.is_ok());
    }

    #[test]
    fn test_complex_statements() {
        let mut generator = JavaScriptGenerator::new();
        let nodes = vec![
            TranspilerNode {
                statement: Statement::Repeat(3, vec![
                    Statement::Tell(EllexValue::String("Loop iteration".to_string())),
                ]),
                metadata: NodeMetadata::default(),
            },
            TranspilerNode {
                statement: Statement::When(
                    "answer".to_string(),
                    EllexValue::String("yes".to_string()),
                    vec![Statement::Tell(EllexValue::String("Great!".to_string()))],
                    Some(vec![Statement::Tell(EllexValue::String("Okay".to_string()))]),
                ),
                metadata: NodeMetadata::default(),
            }
        ];
        
        let options = TranspilerOptions::default();
        let result = generator.generate(&nodes, &options).unwrap();
        
        assert!(result.contains("for (let i = 0; i < 3; i++)"));
        assert!(result.contains("if (answer === \"yes\")"));
        assert!(result.contains("} else {"));
    }
}