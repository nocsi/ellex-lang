//! WebAssembly compiler for maximum performance Ellex execution

use crate::{TranspilerError, TranspilerOptions, Target};
use ellex_core::values::{EllexValue, Statement};
use walrus::{Module, ModuleConfig, ValType, FunctionBuilder, LocalId};
use std::collections::HashMap;

/// WebAssembly code generator
pub struct WasmCompiler {
    /// Walrus module being built
    module: Module,
    /// Variable mappings to WASM locals
    locals: HashMap<String, LocalId>,
    /// String constants table
    strings: HashMap<String, u32>,
    /// Memory offset for string storage
    string_offset: u32,
    /// Optimization level
    opt_level: u8,
}

impl WasmCompiler {
    pub fn new(opt_level: u8) -> Self {
        let config = ModuleConfig::new();
        let module = Module::with_config(config);
        
        Self {
            module,
            locals: HashMap::new(),
            strings: HashMap::new(),
            string_offset: 1024, // Start after first KB for stack
            opt_level,
        }
    }
    
    /// Compile Ellex AST to WebAssembly
    pub fn compile(&mut self, ast: &[Statement]) -> Result<Vec<u8>, TranspilerError> {
        // Initialize memory for string storage (64KB default)
        let memory = self.module.memories.add_local(false, 1, Some(64));
        
        // Add memory export
        self.module.exports.add("memory", memory);
        
        // Create main function
        let mut main_builder = FunctionBuilder::new(&mut self.module.types, &[], &[]);
        
        // Pre-allocate some locals for variables (simplified approach)
        // Note: locals should be added to the FunctionBuilder, not InstrSeqBuilder
        
        // Compile statements
        for stmt in ast {
            self.compile_statement(&mut main_builder, stmt)?;
        }
        
        // Finish main function
        let main_func = main_builder.finish(vec![], &mut self.module.funcs);
        
        // Export main function
        self.module.exports.add("main", main_func);
        
        // Add runtime helper functions
        self.add_runtime_helpers()?;
        
        // Optimize if requested
        if self.opt_level > 0 {
            self.optimize()?;
        }
        
        // Generate final WASM bytes
        Ok(self.module.emit_wasm())
    }
    
    /// Compile a single statement to WASM instructions
    fn compile_statement(&mut self, builder: &mut FunctionBuilder, stmt: &Statement) -> Result<(), TranspilerError> {
        match stmt {
            Statement::Tell(value) => {
                self.compile_tell(builder, value)?;
            }
            
            Statement::Ask(var_name, _type_hint) => {
                self.compile_ask(builder, var_name)?;
            }
            
            Statement::Repeat(count, body) => {
                self.compile_repeat(builder, *count, body)?;
            }
            
            Statement::When(var, condition, then_body, else_body) => {
                self.compile_when(builder, var, condition, then_body, else_body.as_deref())?;
            }
            
            Statement::Call(func_name) => {
                self.compile_call(builder, func_name)?;
            }
        }
        
        Ok(())
    }
    
    /// Compile a single statement to an InstrSeqBuilder (for use in loops/conditionals)
    fn compile_statement_to_builder(&mut self, builder: &mut walrus::InstrSeqBuilder, stmt: &Statement) -> Result<(), TranspilerError> {
        match stmt {
            Statement::Tell(value) => {
                self.compile_tell_to_builder(builder, value)?;
            }
            
            Statement::Ask(var_name, _type_hint) => {
                self.compile_ask_to_builder(builder, var_name)?;
            }
            
            Statement::Repeat(count, body) => {
                self.compile_repeat_to_builder(builder, *count, body)?;
            }
            
            Statement::When(var, condition, then_body, else_body) => {
                self.compile_when_to_builder(builder, var, condition, then_body, else_body.as_deref())?;
            }
            
            Statement::Call(func_name) => {
                self.compile_call_to_builder(builder, func_name)?;
            }
        }
        
        Ok(())
    }
    
    /// Compile tell statement (output)
    fn compile_tell(&mut self, builder: &mut FunctionBuilder, value: &EllexValue) -> Result<(), TranspilerError> {
        match value {
            EllexValue::String(s) => {
                // Store string in memory and call print helper
                let str_offset = self.add_string_constant(s);
                let str_len = s.len() as u32;
                
                // Push string offset and length to stack
                builder.func_body().i32_const(str_offset as i32);
                builder.func_body().i32_const(str_len as i32);
                
                // Call print_string helper
                let print_func = self.get_or_create_print_string_func()?;
                builder.func_body().call(print_func);
            }
            
            EllexValue::Number(n) => {
                // Call print_number helper
                if n.fract() == 0.0 {
                    // Integer
                    builder.func_body().i32_const(*n as i32);
                    let print_func = self.get_or_create_print_i32_func()?;
                    builder.func_body().call(print_func);
                } else {
                    // Float
                    builder.func_body().f64_const(*n);
                    let print_func = self.get_or_create_print_f64_func()?;
                    builder.func_body().call(print_func);
                }
            }
            
            EllexValue::List(items) => {
                // Print each item
                for item in items {
                    self.compile_tell(builder, item)?;
                }
            }
            
            _ => {
                return Err(TranspilerError::UnsupportedFeature(
                    "Unsupported value type in WASM tell".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Compile ask statement (input)
    fn compile_ask(&mut self, builder: &mut FunctionBuilder, var_name: &str) -> Result<(), TranspilerError> {
        // For simplicity, just add a placeholder - real implementation would need 
        // proper local variable management
        builder.func_body().i32_const(0);
        builder.func_body().drop();
        
        Ok(())
    }
    
    /// Compile repeat statement (loop)
    fn compile_repeat(&mut self, builder: &mut FunctionBuilder, count: u32, body: &[Statement]) -> Result<(), TranspilerError> {
        if self.opt_level > 1 && count <= 5 {
            // Unroll small loops for performance
            for _ in 0..count {
                for stmt in body {
                    self.compile_statement(builder, stmt)?;
                }
            }
        } else {
            // For larger loops, just unroll for now (simplified)
            // A proper implementation would use walrus loop constructs
            for _ in 0..count {
                for stmt in body {
                    self.compile_statement(builder, stmt)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Compile when statement (conditional)
    fn compile_when(&mut self, builder: &mut FunctionBuilder, var: &str, condition: &EllexValue, then_body: &[Statement], else_body: Option<&[Statement]>) -> Result<(), TranspilerError> {
        // Simplified conditional - just execute then_body for now
        // A real implementation would need proper variable lookup and comparison
        for stmt in then_body {
            self.compile_statement(builder, stmt)?;
        }
        
        Ok(())
    }
    
    /// Compile function call
    fn compile_call(&mut self, _builder: &mut FunctionBuilder, _func_name: &str) -> Result<(), TranspilerError> {
        // Function calls would need a function table, skip for now
        Ok(())
    }
    
    /// Compile tell statement to InstrSeqBuilder
    fn compile_tell_to_builder(&mut self, builder: &mut walrus::InstrSeqBuilder, value: &EllexValue) -> Result<(), TranspilerError> {
        match value {
            EllexValue::String(s) => {
                // Store string in memory and call print helper
                let str_offset = self.add_string_constant(s);
                let str_len = s.len() as u32;
                
                // Push string offset and length to stack
                builder.i32_const(str_offset as i32);
                builder.i32_const(str_len as i32);
                
                // Call print_string helper
                let print_func = self.get_or_create_print_string_func()?;
                builder.call(print_func);
            }
            
            EllexValue::Number(n) => {
                // Call print_number helper
                if n.fract() == 0.0 {
                    // Integer
                    builder.i32_const(*n as i32);
                    let print_func = self.get_or_create_print_i32_func()?;
                    builder.call(print_func);
                } else {
                    // Float
                    builder.f64_const(*n);
                    let print_func = self.get_or_create_print_f64_func()?;
                    builder.call(print_func);
                }
            }
            
            EllexValue::List(items) => {
                // Print each item
                for item in items {
                    self.compile_tell_to_builder(builder, item)?;
                }
            }
            
            _ => {
                return Err(TranspilerError::UnsupportedFeature(
                    "Unsupported value type in WASM tell".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Compile ask statement to InstrSeqBuilder
    fn compile_ask_to_builder(&mut self, builder: &mut walrus::InstrSeqBuilder, var_name: &str) -> Result<(), TranspilerError> {
        // For now, create a local variable and set to default value
        // Note: This is simplified since we can't easily add locals from InstrSeqBuilder
        // In a real implementation, locals would be managed at the function level
        
        // Set default value (0) - assuming we have a way to reference the local
        builder.i32_const(0);
        builder.drop(); // Drop the value since we can't set a local here
        
        Ok(())
    }
    
    /// Compile repeat statement to InstrSeqBuilder
    fn compile_repeat_to_builder(&mut self, builder: &mut walrus::InstrSeqBuilder, count: u32, body: &[Statement]) -> Result<(), TranspilerError> {
        if count <= 5 {
            // Unroll small loops for performance
            for _ in 0..count {
                for stmt in body {
                    self.compile_statement_to_builder(builder, stmt)?;
                }
            }
        } else {
            // For larger loops, we'd need to implement a proper loop structure
            // This is simplified for now
            for _ in 0..count {
                for stmt in body {
                    self.compile_statement_to_builder(builder, stmt)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Compile when statement to InstrSeqBuilder
    fn compile_when_to_builder(&mut self, builder: &mut walrus::InstrSeqBuilder, _var: &str, _condition: &EllexValue, then_body: &[Statement], _else_body: Option<&[Statement]>) -> Result<(), TranspilerError> {
        // Simplified conditional - just execute then_body for now
        for stmt in then_body {
            self.compile_statement_to_builder(builder, stmt)?;
        }
        
        Ok(())
    }
    
    /// Compile function call to InstrSeqBuilder
    fn compile_call_to_builder(&mut self, _builder: &mut walrus::InstrSeqBuilder, _func_name: &str) -> Result<(), TranspilerError> {
        // Function calls would need a function table, skip for now
        Ok(())
    }
    
    /// Add string constant to memory and return offset
    fn add_string_constant(&mut self, s: &str) -> u32 {
        if let Some(&offset) = self.strings.get(s) {
            return offset;
        }
        
        let offset = self.string_offset;
        let bytes = s.as_bytes();
        
        // Add to data section (simplified for compilation)
        let _data_id = self.module.data.add(
            walrus::DataKind::Passive,
            bytes.to_vec(),
        );
        
        self.strings.insert(s.to_string(), offset);
        self.string_offset += bytes.len() as u32 + 1; // +1 for null terminator
        
        offset
    }
    
    /// Get or create print_string helper function
    fn get_or_create_print_string_func(&mut self) -> Result<walrus::FunctionId, TranspilerError> {
        // This would be an imported function from the host environment
        let func_type = self.module.types.add(&[ValType::I32, ValType::I32], &[]);
        let (func_id, _import_id) = self.module.add_import_func("env", "print_string", func_type);
        Ok(func_id)
    }
    
    /// Get or create print_i32 helper function
    fn get_or_create_print_i32_func(&mut self) -> Result<walrus::FunctionId, TranspilerError> {
        let func_type = self.module.types.add(&[ValType::I32], &[]);
        let (func_id, _import_id) = self.module.add_import_func("env", "print_i32", func_type);
        Ok(func_id)
    }
    
    /// Get or create print_f64 helper function
    fn get_or_create_print_f64_func(&mut self) -> Result<walrus::FunctionId, TranspilerError> {
        let func_type = self.module.types.add(&[ValType::F64], &[]);
        let (func_id, _import_id) = self.module.add_import_func("env", "print_f64", func_type);
        Ok(func_id)
    }
    
    /// Add runtime helper functions
    fn add_runtime_helpers(&mut self) -> Result<(), TranspilerError> {
        // Helper functions would be imported from the host environment
        // They handle I/O operations that WASM cannot do directly
        Ok(())
    }
    
    /// Apply optimizations to the WASM module
    fn optimize(&mut self) -> Result<(), TranspilerError> {
        // Basic optimizations
        match self.opt_level {
            1 => {
                // Light optimizations: dead code elimination
                walrus::passes::gc::run(&mut self.module);
            }
            2 => {
                // Medium optimizations: GC + local CSE
                walrus::passes::gc::run(&mut self.module);
                // Additional passes would go here
            }
            3 => {
                // Aggressive optimizations
                walrus::passes::gc::run(&mut self.module);
                // More aggressive passes
            }
            _ => {}
        }
        
        Ok(())
    }
}

/// Compile Ellex AST to WebAssembly
pub fn compile(ast: &[Statement], options: &TranspilerOptions) -> Result<String, TranspilerError> {
    let (opt_level, _simd, _threads) = match &options.target {
        Target::WebAssembly { opt_level, simd, threads } => (*opt_level, *simd, *threads),
        _ => (2, false, false),
    };
    
    let mut compiler = WasmCompiler::new(opt_level);
    let wasm_bytes = compiler.compile(ast)?;
    
    // Convert to WAT (WebAssembly Text) format for readability
    let wat_string = wasmprinter::print_bytes(&wasm_bytes)
        .map_err(|e| TranspilerError::WasmError(format!("WAT conversion error: {}", e)))?;
    
    Ok(wat_string)
}

/// Generate JavaScript WASM loader
pub fn generate_wasm_loader(wasm_wat: &str) -> String {
    format!(r#"
// Ellex WebAssembly Runtime Loader
const EllexWasmRuntime = {{
    instance: null,
    memory: null,
    decoder: new TextDecoder(),
    
    // Import functions that WASM calls
    imports: {{
        env: {{
            print_string: (offset, length) => {{
                const bytes = new Uint8Array(EllexWasmRuntime.memory.buffer, offset, length);
                const str = EllexWasmRuntime.decoder.decode(bytes);
                console.log(str);
            }},
            
            print_i32: (value) => {{
                console.log(value);
            }},
            
            print_f64: (value) => {{
                console.log(value);
            }},
            
            // Input functions (would need async handling)
            input_string: () => {{
                // Placeholder - real implementation would be async
                return 0;
            }}
        }}
    }},
    
    // Load and instantiate WASM module
    async init() {{
        // Compile WAT to WASM bytes
        const wasmBytes = await WebAssembly.compile(await this.watToWasm(`{}`));
        
        // Instantiate with imports
        const wasmModule = await WebAssembly.instantiate(wasmBytes, this.imports);
        this.instance = wasmModule;
        this.memory = wasmModule.exports.memory;
        
        return this;
    }},
    
    // Execute main function
    run() {{
        if (!this.instance) {{
            throw new Error('WASM module not initialized');
        }}
        
        return this.instance.exports.main();
    }},
    
    // Convert WAT to WASM bytes (would use wabt.js in real implementation)
    async watToWasm(watString) {{
        // This is a placeholder - in real implementation you'd use:
        // const wabt = await import('wabt');
        // const wabtModule = await wabt();
        // const parsed = wabtModule.parseWat('inline', watString);
        // return parsed.toBinary({{}}).buffer;
        
        throw new Error('WAT to WASM conversion requires wabt.js');
    }}
}};

// Usage:
// EllexWasmRuntime.init().then(runtime => {{
//     runtime.run();
// }});

export default EllexWasmRuntime;
"#, wasm_wat.replace('`', r"\`"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TranspilerOptions;

    #[test]
    fn test_wasm_compiler_creation() {
        let compiler = WasmCompiler::new(2);
        assert_eq!(compiler.opt_level, 2);
        assert!(compiler.locals.is_empty());
        assert!(compiler.strings.is_empty());
    }

    #[test]
    fn test_simple_tell_compilation() {
        let ast = vec![
            Statement::Tell(EllexValue::String("Hello WASM!".to_string())),
        ];
        
        let options = TranspilerOptions {
            target: Target::WebAssembly {
                simd: false,
                threads: false,
                opt_level: 1,
            },
            ..Default::default()
        };
        
        let result = compile(&ast, &options);
        assert!(result.is_ok());
        
        let wat = result.unwrap();
        assert!(wat.contains("module"));
        assert!(wat.contains("memory"));
        assert!(wat.contains("export"));
    }

    #[test]
    fn test_number_tell_compilation() {
        let ast = vec![
            Statement::Tell(EllexValue::Number(42.0)),
        ];
        
        let options = TranspilerOptions {
            target: Target::WebAssembly {
                simd: false,
                threads: false,
                opt_level: 0,
            },
            ..Default::default()
        };
        
        let result = compile(&ast, &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_repeat_compilation() {
        let ast = vec![
            Statement::Repeat(3, vec![
                Statement::Tell(EllexValue::String("Loop".to_string())),
            ]),
        ];
        
        let options = TranspilerOptions {
            target: Target::WebAssembly {
                simd: false,
                threads: false,
                opt_level: 2,
            },
            ..Default::default()
        };
        
        let result = compile(&ast, &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimization_levels() {
        let ast = vec![
            Statement::Tell(EllexValue::String("Test".to_string())),
        ];
        
        for opt_level in 0..=3 {
            let options = TranspilerOptions {
                target: Target::WebAssembly {
                    simd: false,
                    threads: false,
                    opt_level,
                },
                ..Default::default()
            };
            
            let result = compile(&ast, &options);
            assert!(result.is_ok(), "Optimization level {} failed", opt_level);
        }
    }

    #[test]
    fn test_wasm_loader_generation() {
        let wat = "(module (export \"main\" (func 0)) (func))";
        let loader = generate_wasm_loader(wat);
        
        assert!(loader.contains("EllexWasmRuntime"));
        assert!(loader.contains("print_string"));
        assert!(loader.contains("print_i32"));
        assert!(loader.contains("print_f64"));
        assert!(loader.contains("async init"));
        assert!(loader.contains("run()"));
    }

    #[test]
    fn test_string_constant_management() {
        let mut compiler = WasmCompiler::new(1);
        
        let offset1 = compiler.add_string_constant("Hello");
        let offset2 = compiler.add_string_constant("World");
        let offset3 = compiler.add_string_constant("Hello"); // Should reuse
        
        assert_eq!(offset1, offset3); // Same string should reuse offset
        assert_ne!(offset1, offset2); // Different strings should have different offsets
        assert_eq!(compiler.strings.len(), 2); // Only 2 unique strings
    }

    #[test]
    fn test_multiple_statements() {
        let ast = vec![
            Statement::Tell(EllexValue::String("First".to_string())),
            Statement::Tell(EllexValue::Number(123.0)),
            Statement::Ask("name".to_string(), None),
            Statement::Tell(EllexValue::String("Done".to_string())),
        ];
        
        let options = TranspilerOptions {
            target: Target::WebAssembly {
                simd: false,
                threads: false,
                opt_level: 1,
            },
            ..Default::default()
        };
        
        let result = compile(&ast, &options);
        assert!(result.is_ok());
        
        let wat = result.unwrap();
        assert!(wat.contains("module"));
        assert!(wat.contains("main"));
    }
}