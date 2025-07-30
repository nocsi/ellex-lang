#[cfg(feature = "llvm")]
use inkwell::context::Context;
#[cfg(feature = "llvm")]
use inkwell::module::Module;
#[cfg(feature = "llvm")]
use inkwell::passes::PassManager;
#[cfg(feature = "llvm")]
use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple};
#[cfg(feature = "llvm")]
use inkwell::OptimizationLevel;
use anyhow::{Result, anyhow};
use crate::values::{Statement, EllexValue};

// Existing Pass trait and Pipeline...

// New: AST to LLVM-IR Pass
#[cfg(feature = "llvm")]
pub struct AstToLlvmIr {
    context: Context,
}

#[cfg(feature = "llvm")]
impl AstToLlvmIr {
    pub fn new() -> Self {
        AstToLlvmIr { context: Context::create() }
    }
}

#[cfg(feature = "llvm")]
impl Pass for AstToLlvmIr {
    fn apply(&self, ast: &mut Vec<Statement>) -> Result<()> {
        let module = self.context.create_module("ellex_module");
        let builder = self.context.create_builder();
        let printf = module.add_function("printf", self.context.i32_type().fn_type(&[self.context.i8_type().ptr_type(inkwell::AddressSpace::default()).into()], true), None);

        // Map AST to IR
        for stmt in ast.iter() {
            match stmt {
                Statement::Tell(value) => {
                    let str_val = builder.build_global_string_ptr(&value.to_string(), "str");
                    builder.build_call(printf, &[str_val.as_operand().into()], "tell_call");
                }
                Statement::Repeat(count, body) => {
                    // Build loop: phi for counter, br, etc.
                    // Simplified; expand for full impl
                }
                // Add mappings for Ask, When, etc.
                _ => {}
            }
        }

        // Store IR in a custom field or output file
        module.print_to_file("ellex.ll").map_err(|e| anyhow!(e.to_string()))?;
        Ok(())
    }
}

// LLVM-IR Optimization Pass (runs on generated Module)
#[cfg(feature = "llvm")]
pub struct LlvmOptimizer {
    level: OptimizationLevel,
}

#[cfg(feature = "llvm")]
impl LlvmOptimizer {
    pub fn new(level: OptimizationLevel) -> Self {
        LlvmOptimizer { level }
    }
}

#[cfg(feature = "llvm")]
impl Pass for LlvmOptimizer {
    fn apply(&self, _ast: &mut Vec<Statement>) -> Result<()> {
        // Assume IR is in a file or module; load and optimize
        let context = Context::create();
        let module = Module::parse_bitcode_from_path("ellex.ll", &context)?;
        let pm = PassManager::create(&module);
        pm.add_verifier_pass();
        pm.add_new_pass_manager_passes(self.level);  // Inlining, const prop, etc.
        pm.run_on(&module);
        module.print_to_file("ellex_opt.ll")?;
        Ok(())
    }
}

// Backend: LLVM-IR to Wasm
#[cfg(feature = "llvm")]
pub struct LlvmToWasm;

#[cfg(feature = "llvm")]
impl Pass for LlvmToWasm {
    fn apply(&self, _ast: &mut Vec<Statement>) -> Result<()> {
        Target::initialize_webassembly(&InitializationConfig::default());
        let triple = TargetTriple::create("wasm32-unknown-unknown");
        let target = Target::get_first_defined_for_triple(&triple).ok_or(anyhow!("No Wasm target"))?;
        let machine = target.create_target_machine(&triple, "generic", "", OptimizationLevel::Aggressive, RelocMode::PIC, CodeModel::Default).ok_or(anyhow!("No machine"))?;
        machine.write_to_file(&Module::parse_bitcode_from_path("ellex_opt.ll", &Context::create())?, FileType::Object, "ellex.wasm".as_ref())?;
        Ok(())
    }
}

// Backend: LLVM-IR to JS (via Emscripten-like, but simplified; use external emcc for full)
#[cfg(feature = "llvm")]
pub struct LlvmToJs;

#[cfg(feature = "llvm")]
impl Pass for LlvmToJs {
    fn apply(&self, _ast: &mut Vec<Statement>) -> Result<()> {
        // Invoke emcc externally or integrate; placeholder
        std::process::Command::new("emcc").args(["ellex_opt.ll", "-o", "ellex.js", "-s", "WASM=0"]).output()?;
        Ok(())
    }
}

// Usage example in parser:
// #[cfg(feature = "llvm")]
// {
//     let mut pipeline = Pipeline::new();
//     pipeline.attach(Box::new(AstToLlvmIr::new()));
//     pipeline.attach(Box::new(LlvmOptimizer::new(OptimizationLevel::Aggressive)));
//     pipeline.attach(Box::new(LlvmToWasm));  // Or LlvmToJs:
// }
