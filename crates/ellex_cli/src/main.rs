//! Ellex Command Line Interface

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod tui;

#[derive(Parser)]
#[command(name = "el")]
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
        /// The .el file to run
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
    /// Transpile Ellex to other languages
    Transpile {
        /// The .el file to transpile
        file: PathBuf,
        /// Target language (js, ts, go, rust, wasm)
        #[arg(short, long, default_value = "js")]
        target: String,
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
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
            println!("Web server functionality coming soon!");
        }
        Commands::Tui => {
            println!("🚀 Starting Ellex TUI with real-time metrics...");
            tui::run_tui().await?;
        }
        Commands::Transpile { file, target, output } => {
            println!("🔄 Transpiling {:?} to {}", file, target);
            transpile_ellex_file(&file, &target, output.as_ref()).await?;
        }
    }
    
    Ok(())
}

/// Transpile an Ellex file to another language
async fn transpile_ellex_file(file: &PathBuf, target: &str, output: Option<&PathBuf>) -> anyhow::Result<()> {
    use std::fs;
    
    let content = fs::read_to_string(file)?;
    println!("📝 Reading {}...", file.display());
    
    // Parse the Ellex file
    let statements = ellex_parser::parse(&content)?;
    println!("✅ Parsed {} statements", statements.len());
    
    // Set up transpiler options
    let target_enum = match target {
        "js" | "javascript" => ellex_transpiler::Target::JavaScript { 
            async_support: true,
            es_version: ellex_transpiler::EsVersion::Es2020,
            optimize: true,
        },
        "ts" | "typescript" => ellex_transpiler::Target::TypeScript { 
            declarations: false,
            strict: true,
        },
        "wasm" => ellex_transpiler::Target::WebAssembly { 
            simd: false, 
            threads: false, 
            opt_level: 2,
        },
        _ => return Err(anyhow::anyhow!("Unsupported target language: {}. Supported: js, ts, wasm", target)),
    };
    
    let options = ellex_transpiler::TranspilerOptions {
        target: target_enum,
        optimize: true,
        minify: false,
        source_maps: false,
        preserve_comments: false,
    };
    
    // Transpile
    println!("🔄 Transpiling to {}...", target);
    let transpiler = ellex_transpiler::EllexTranspiler::with_options(options);
    let transpiled = transpiler.transpile(&statements)?;
    
    // Determine output path
    let output_path = if let Some(out) = output {
        out.clone()
    } else {
        let mut path = file.clone();
        let extension = match target {
            "js" | "javascript" => "js",
            "ts" | "typescript" => "ts",
            "wasm" => "wat",
            _ => "txt",
        };
        path.set_extension(extension);
        path
    };
    
    // Write output
    fs::write(&output_path, transpiled)?;
    println!("✅ Transpiled code written to {}", output_path.display());
    
    Ok(())
}

/// Run an Ellex file
async fn run_ellex_file(file: &PathBuf) -> anyhow::Result<()> {
    use std::fs;
    
    let content = fs::read_to_string(file)?;
    println!("📝 Content of {}:", file.display());
    println!("{}", content);
    
    // Parse and execute the Ellex file
    let mut runtime = ellex_core::EllexRuntime::new();
    let statements = ellex_parser::parse(&content)?;
    
    println!("✅ Parsed {} statements", statements.len());
    println!("🚀 Execution would happen here (runtime methods not yet implemented)");
    
    // TODO: Implement proper statement execution
    for (i, _statement) in statements.iter().enumerate() {
        println!("  Statement {}: Ready for execution", i + 1);
    }
    
    Ok(())
}