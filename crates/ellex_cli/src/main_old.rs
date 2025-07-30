//! Ellex Command Line Interface

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod tui;

#[derive(Parser)]
#[command(name = "ellex")]
#[command(about = "A natural language programming environment for kids")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive REPL
    Repl {
        /// Enable AI assistance
        #[arg(long, default_value_t = true)]
        ai: bool,
    },
    /// Run an Ellex file
    Run {
        /// The .ellex file to run
        file: PathBuf,
    },
    /// Start web playground server
    Serve {
        /// Port to serve on
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
    /// Start TUI interface with real-time metrics
    Tui,
    // Transpiler commands temporarily disabled due to compilation errors
    // /// Transpile Ellex code to other languages
    // Transpile {
    //     /// Input file (.ellex)
    //     #[arg(short, long)]
    //     input: PathBuf,
    //     /// Output file
    //     #[arg(short, long)]
    //     output: Option<PathBuf>,
    //     /// Target language (javascript, typescript, python, go, wasm)
    //     #[arg(short, long, default_value = "javascript")]
    //     target: String,
    //     /// Enable optimizations
    //     #[arg(long, default_value_t = true)]
    //     optimize: bool,
    //     /// Minify output
    //     #[arg(long, default_value_t = false)]
    //     minify: bool,
    // },
    // /// Parse JavaScript and convert to Ellex
    // FromJs {
    //     /// Input JavaScript file
    //     #[arg(short, long)]
    //     input: PathBuf,
    //     /// Output Ellex file
    //     #[arg(short, long)]
    //     output: Option<PathBuf>,
    // },
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Repl { ai } => {
            let config = ellex_core::EllexConfig::default();
            let mut repl = ellex_repl::InteractiveRepl::new(config, ai)?;
            repl.run()?;
        }
        Commands::Run { file } => {
            println!("Running {:?}", file);
            run_ellex_file(&file).await?;
        }
        Commands::Serve { port } => {
            println!("Starting Ellex playground server on port {}", port);
            // TODO: Start web server
        }
        Commands::Tui => {
            println!("🚀 Starting Ellex TUI with real-time metrics...");
            tui::run_tui().await?;
        }
        // Transpiler commands temporarily disabled
        // Commands::Transpile { input, output, target, optimize, minify } => {
        //     transpile_file(&input, output.as_ref(), &target, optimize, minify).await?;
        // }
        // Commands::FromJs { input, output } => {
        //     convert_from_js(&input, output.as_ref()).await?;
        // }
    }
    
    Ok(())
}

/// Run an Ellex file
async fn run_ellex_file(file: &PathBuf) -> anyhow::Result<()> {
    use std::fs;
    use ellex_core::runtime::Pipeline;
    
    let content = fs::read_to_string(file)?;
    let runtime = ellex_core::init();
    let pipeline = Pipeline::new();
    
    let ast = ellex_parser::parse_and_optimize(&content, &pipeline)?;
    
    println!("Executing Ellex program...");
    // TODO: Execute the AST
    println!("Program completed.");
    
    Ok(())
}

/// Transpile Ellex file to target language
async fn transpile_file(
    input: &PathBuf,
    output: Option<&PathBuf>,
    target: &str,
    optimize: bool,
    minify: bool,
) -> anyhow::Result<()> {
    use std::fs;
    use ellex_transpiler::{EllexTranspiler, TranspilerOptions, Target, EsVersion};
    
    println!("🔄 Transpiling {} to {}", input.display(), target);
    
    // Read input file
    let content = fs::read_to_string(input)?;
    
    // Parse Ellex code
    let runtime = ellex_core::runtime::Pipeline::new();
    let ast = ellex_parser::parse_and_optimize(&content, &runtime)?;
    
    // Configure transpiler
    let target_config = match target {
        "javascript" | "js" => Target::JavaScript {
            async_support: true,
            es_version: EsVersion::Es2020,
            optimize,
        },
        "typescript" | "ts" => Target::TypeScript {
            declarations: true,
            strict: true,
        },
        "wasm" | "webassembly" => Target::WebAssembly {
            simd: false,
            threads: false,
            opt_level: if optimize { 2 } else { 0 },
        },
        _ => return Err(anyhow::anyhow!("Unsupported target: {}", target)),
    };
    
    let options = TranspilerOptions {
        target: target_config,
        source_maps: true,
        minify,
        optimize,
        preserve_comments: false,
    };
    
    // Transpile
    let transpiler = EllexTranspiler::with_options(options);
    let transpiled_code = transpiler.transpile(&ast)?;
    
    // Determine output file
    let output_path = if let Some(path) = output {
        path.clone()
    } else {
        let extension = match target {
            "javascript" | "js" => "js",
            "typescript" | "ts" => "ts",
            "python" | "py" => "py",
            "go" => "go",
            "wasm" | "webassembly" => "wat",
            _ => "txt",
        };
        input.with_extension(extension)
    };
    
    // Write output
    fs::write(&output_path, transpiled_code)?;
    
    println!("✅ Transpiled to {}", output_path.display());
    
    // For WASM, also generate loader
    if target == "wasm" || target == "webassembly" {
        let loader_code = ellex_transpiler::wasm_compiler::generate_wasm_loader(&fs::read_to_string(&output_path)?);
        let loader_path = output_path.with_extension("js");
        fs::write(&loader_path, loader_code)?;
        println!("✅ Generated WASM loader: {}", loader_path.display());
    }
    
    Ok(())
}

/// Convert JavaScript file to Ellex
async fn convert_from_js(input: &PathBuf, output: Option<&PathBuf>) -> anyhow::Result<()> {
    use std::fs;
    use ellex_transpiler::EllexTranspiler;
    
    println!("🔄 Converting JavaScript to Ellex: {}", input.display());
    
    // Read JavaScript file
    let js_content = fs::read_to_string(input)?;
    
    // Parse and convert
    let ellex_ast = EllexTranspiler::from_js(&js_content)?;
    
    // Convert AST back to Ellex natural language (simplified)
    let mut ellex_code = String::new();
    for stmt in &ellex_ast {
        match stmt {
            ellex_core::values::Statement::Tell(value) => {
                ellex_code.push_str(&format!("tell {}\n", value));
            }
            ellex_core::values::Statement::Ask(var, _) => {
                ellex_code.push_str(&format!("ask \"What is {}?\" → {}\n", var, var));
            }
            ellex_core::values::Statement::Repeat(count, body) => {
                ellex_code.push_str(&format!("repeat {} times do\n", count));
                for body_stmt in body {
                    match body_stmt {
                        ellex_core::values::Statement::Tell(value) => {
                            ellex_code.push_str(&format!("  tell {}\n", value));
                        }
                        _ => {
                            ellex_code.push_str("  # Complex statement\n");
                        }
                    }
                }
                ellex_code.push_str("end\n");
            }
            _ => {
                ellex_code.push_str("# Complex statement converted from JavaScript\n");
            }
        }
    }
    
    // Determine output file
    let output_path = if let Some(path) = output {
        path.clone()
    } else {
        input.with_extension("ellex")
    };
    
    // Write output
    fs::write(&output_path, ellex_code)?;
    
    println!("✅ Converted to Ellex: {}", output_path.display());
    
    Ok(())
}
