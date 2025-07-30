use axum::{
    extract::{Json, State, Path},
    http::StatusCode,
    response::{Json as ResponseJson, Html},
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use ellex_core::{Pipeline, Statement, EllexConfig};
use ellex_repl::ApiRepl;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use uuid::Uuid;

// Shared state for managing REPL sessions
#[derive(Clone)]
struct AppState {
    sessions: Arc<Mutex<HashMap<String, ApiRepl>>>,
    pipeline: Arc<Mutex<Pipeline>>,
}

// REPL API Request/Response structures
#[derive(Deserialize)]
struct CreateSessionRequest {
    config: Option<EllexConfig>,
}

#[derive(Serialize)]
struct CreateSessionResponse {
    session_id: String,
    config: EllexConfig,
}

#[derive(Deserialize)]
struct ExecuteRequest {
    code: String,
    session_id: Option<String>,
}

#[derive(Serialize)]
struct ExecuteResponse {
    output: Vec<String>,
    session_id: String,
    execution_count: usize,
    variables: serde_json::Value,
}

#[derive(Serialize)]
struct SessionInfoResponse {
    session_id: String,
    execution_count: usize,
    variables: serde_json::Value,
    functions: Vec<String>,
    config: EllexConfig,
}

#[derive(Deserialize)]
struct InteractiveInputRequest {
    session_id: String,
    variable: String,
    value: String,
}

#[derive(Serialize)]
struct InteractiveInputResponse {
    success: bool,
    message: String,
}

// Legacy structures for compatibility
#[derive(Deserialize)]
struct ParseRequest {
    code: String,
    visualize: bool,
}

#[derive(Serialize)]
struct ParseResponse {
    ast: Vec<Statement>,
    output: String,  // Parse tree or error
}

#[derive(Deserialize)]
struct IrRequest {
    ir_code: String,
}

#[derive(Serialize)]
struct IrResponse {
    graph: String,  // JSON or ASCII for viz
}

// REPL API Endpoints

// Create a new REPL session
async fn create_session(
    State(state): State<AppState>, 
    Json(req): Json<CreateSessionRequest>
) -> Result<ResponseJson<CreateSessionResponse>, StatusCode> {
    let session_id = Uuid::new_v4().to_string();
    let config = req.config.unwrap_or_default();
    let repl = ApiRepl::new(config.clone());
    
    state.sessions.lock().await.insert(session_id.clone(), repl);
    
    Ok(ResponseJson(CreateSessionResponse {
        session_id,
        config,
    }))
}

// Execute code in a REPL session
async fn execute_code(
    State(state): State<AppState>,
    Json(req): Json<ExecuteRequest>
) -> Result<ResponseJson<ExecuteResponse>, StatusCode> {
    let session_id = req.session_id.unwrap_or_else(|| {
        // Create a temporary session if none provided
        Uuid::new_v4().to_string()
    });
    
    let mut sessions = state.sessions.lock().await;
    
    // Get or create session
    let repl = sessions.entry(session_id.clone()).or_insert_with(|| {
        ApiRepl::new(EllexConfig::default())
    });
    
    match repl.execute(&req.code) {
        Ok(output) => {
            let session = repl.get_session();
            let variables = serde_json::to_value(&session.variables)
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
            
            Ok(ResponseJson(ExecuteResponse {
                output,
                session_id,
                execution_count: session.execution_count,
                variables,
            }))
        }
        Err(e) => {
            Ok(ResponseJson(ExecuteResponse {
                output: vec![format!("Error: {}", e)],
                session_id,
                execution_count: repl.get_session().execution_count,
                variables: serde_json::Value::Object(serde_json::Map::new()),
            }))
        }
    }
}

// Get session information
async fn get_session_info(
    State(state): State<AppState>,
    Path(session_id): Path<String>
) -> Result<ResponseJson<SessionInfoResponse>, StatusCode> {
    let sessions = state.sessions.lock().await;
    
    if let Some(repl) = sessions.get(&session_id) {
        let session = repl.get_session();
        let variables = serde_json::to_value(&session.variables)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        let functions: Vec<String> = session.functions.keys().cloned().collect();
        
        Ok(ResponseJson(SessionInfoResponse {
            session_id,
            execution_count: session.execution_count,
            variables,
            functions,
            config: session.config.clone(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Handle interactive input (for ask statements)
async fn handle_interactive_input(
    State(state): State<AppState>,
    Json(req): Json<InteractiveInputRequest>
) -> Result<ResponseJson<InteractiveInputResponse>, StatusCode> {
    let mut sessions = state.sessions.lock().await;
    
    if let Some(repl) = sessions.get_mut(&req.session_id) {
        let session = repl.get_session_mut();
        
        // Try to parse as number first, then as string
        let value = if let Ok(num) = req.value.parse::<f64>() {
            ellex_core::EllexValue::Number(num)
        } else {
            ellex_core::EllexValue::String(req.value.clone())
        };
        
        session.variables.insert(req.variable.clone(), value);
        
        Ok(ResponseJson(InteractiveInputResponse {
            success: true,
            message: format!("Set {} = {}", req.variable, req.value),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Delete a session
async fn delete_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>
) -> Result<StatusCode, StatusCode> {
    let mut sessions = state.sessions.lock().await;
    
    if sessions.remove(&session_id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Legacy endpoint for compatibility
async fn parse_code(State(state): State<AppState>, Json(req): Json<ParseRequest>) -> Json<ParseResponse> {
    let _pipeline = state.pipeline.lock().await;
    match ellex_parser::parse(&req.code) {
        Ok(ast) => Json(ParseResponse {
            ast,
            output: if req.visualize { "Visual tree...".to_string() } else { "Success".to_string() },
        }),
        Err(e) => Json(ParseResponse { ast: vec![], output: e.to_string() }),
    }
}

// Helper function to create IR visualization
fn visualize_ir_graph(ir_code: &str) -> String {
    // Placeholder for IR visualization - would parse LLVM IR and create graph
    format!("IR Graph for: {}", ir_code.lines().count())
}

// Endpoint for LLVM-IR Visual Editor (mirrors TUI IR pane + graph)
async fn visualize_ir(State(_state): State<AppState>, Json(req): Json<IrRequest>) -> Json<IrResponse> {
    // Parse IR, build graph (using previous visualize_ir logic)
    let graph = visualize_ir_graph(&req.ir_code);  // Returns JSON string for Cytoscape/Svelte
    Json(IrResponse { graph })
}

// Endpoint for Transpilation (attached to output pane)
async fn transpile(State(state): State<AppState>, Json(_req): Json<ParseRequest>) -> Json<String> {
    // Run pipeline with AstToLlvmIr, etc.
    let mut ast: Vec<Statement> = vec![];  // Parse first
    state.pipeline.lock().await.run(&mut ast).unwrap();
    Json("Generated Wasm/JS...".to_string())  // Return file path or content
}

#[tokio::main]
async fn main() {
    let state = AppState { 
        sessions: Arc::new(Mutex::new(HashMap::new())),
        pipeline: Arc::new(Mutex::new(Pipeline::new())) 
    };
    
    let app = Router::new()
        // Web playground
        .route("/", get(serve_playground))
        
        // New REPL API endpoints
        .route("/api/repl/sessions", post(create_session))
        .route("/api/repl/execute", post(execute_code))
        .route("/api/repl/sessions/:id", get(get_session_info))
        .route("/api/repl/sessions/:id", axum::routing::delete(delete_session))
        .route("/api/repl/input", post(handle_interactive_input))
        
        // Legacy endpoints for compatibility
        .route("/api/editor/parse", post(parse_code))
        .route("/api/ir/visualize", post(visualize_ir))
        .route("/api/transpile", post(transpile))
        
        // Health check endpoint
        .route("/health", get(health_check))
        
        .with_state(state);

    println!("🌿 Ellex API server starting on http://0.0.0.0:8080");
    println!("REPL endpoints available at /api/repl/");
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Serve the web playground
async fn serve_playground() -> Html<String> {
    let html = include_str!("../../../playground/index.html");
    Html(html.to_string())
}

// Health check endpoint
async fn health_check() -> ResponseJson<serde_json::Value> {
    ResponseJson(serde_json::json!({
        "status": "healthy",
        "service": "ellex-api",
        "version": ellex_core::VERSION,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
