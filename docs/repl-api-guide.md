# Ellex REPL API Guide

## Overview

The Ellex REPL (Read-Eval-Print Loop) API provides a comprehensive interface for interactive programming sessions. It supports both terminal-based and web-based interactions with persistent session management, variable storage, and comprehensive error handling.

## Quick Start

### Starting a REPL Session

```bash
# Terminal REPL
cargo run --bin ellex_cli repl

# Web API Server
cargo run --bin ellex_api
# Then visit http://localhost:8080
```

### Basic API Usage

```javascript
// Create a session
const response = await fetch('/api/repl/sessions', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({})
});
const { session_id } = await response.json();

// Execute code
const execResponse = await fetch('/api/repl/execute', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },  
    body: JSON.stringify({
        code: 'tell "Hello, world!"',
        session_id: session_id
    })
});
const result = await execResponse.json();
console.log(result.output); // ["Hello, world!"]
```

## REST API Endpoints

### Session Management

#### `POST /api/repl/sessions`
Create a new REPL session.

**Request Body:**
```json
{
  "config": {
    "execution_timeout_ms": 5000,
    "memory_limit_mb": 64,
    "enable_turtle": true,
    "enable_ai": true,
    "max_recursion_depth": 100,
    "max_loop_iterations": 10000
  }
}
```

**Response:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "config": {
    "execution_timeout_ms": 5000,
    "memory_limit_mb": 64,
    "enable_turtle": true,
    "enable_ai": true,
    "max_recursion_depth": 100,
    "max_loop_iterations": 10000
  }
}
```

#### `GET /api/repl/sessions/{id}`
Get session information and current state.

**Response:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "execution_count": 5,
  "variables": {
    "name": {"String": "Alice"},
    "age": {"Number": 16}
  },
  "functions": ["greet_user", "count_to_ten"],
  "config": { /* ... */ }
}
```

#### `DELETE /api/repl/sessions/{id}`
Delete a session and clean up resources.

**Response:** `204 No Content`

### Code Execution

#### `POST /api/repl/execute`
Execute Ellex code in a session.

**Request Body:**
```json
{
  "code": "tell \"Hello, {name}!\"",
  "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Response:**
```json
{
  "output": ["Hello, Alice!"],
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "execution_count": 6,
  "variables": {
    "name": {"String": "Alice"},
    "age": {"Number": 16}
  }
}
```

**Error Response:**
```json
{
  "output": ["Error: I didn't understand 'tel'. Did you mean 'tell'? 🤔"],
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "execution_count": 6,
  "variables": {}
}
```

### Interactive Input

#### `POST /api/repl/input`
Provide input for interactive statements (like `ask`).

**Request Body:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "variable": "user_name",
  "value": "Bob"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Set user_name = Bob"
}
```

## REPL Session Object

### ReplSession Structure

```rust
pub struct ReplSession {
    pub variables: HashMap<String, EllexValue>,
    pub functions: HashMap<String, Statement>,
    pub history: Vec<String>,
    pub output_buffer: Vec<String>,
    pub execution_count: usize,
    pub config: EllexConfig,
}
```

### Methods

#### `execute_line(input: &str) -> Result<Vec<String>>`
Execute a single line of Ellex code or REPL command.

**Examples:**
```rust
let mut session = ReplSession::new();

// Execute Ellex code
let result = session.execute_line("tell \"Hello!\"");
assert_eq!(result.unwrap(), vec!["Hello!"]);

// Execute REPL commands
let result = session.execute_line("/help");
assert!(result.unwrap()[0].contains("Ellex REPL Help"));

// Set variables
let result = session.execute_line("/set name \"Alice\"");
assert_eq!(result.unwrap(), vec!["Set name = \"Alice\""]);
```

#### `interactive_ask(question: &str, var_name: &str) -> String`
Handle interactive input for terminal REPL.

```rust
let mut session = ReplSession::new();
let response = session.interactive_ask("What's your name?", "name");
// User input is stored in session.variables["name"]
```

## REPL Commands

### Built-in Commands

| Command | Description | Example |
|---------|-------------|---------|
| `/help` | Show help information | `/help` |
| `/clear` | Clear output buffer | `/clear` |
| `/history` | Show command history | `/history` |
| `/vars` | Show all variables | `/vars` |
| `/funcs` | Show all functions | `/funcs` |
| `/config` | Show configuration | `/config` |
| `/set var value` | Set variable directly | `/set name "Alice"` |
| `/reset` | Reset session | `/reset` |
| `/exit` | Exit REPL | `/exit` |

### Variable Management

```ellex
# Set variables using REPL commands
/set name "Alice"
/set age 16
/set colors ["red", "green", "blue"]

# Use variables in code
tell "Hello, {name}!"
tell "You are {age} years old"

# View all variables
/vars
```

## Interactive Programming

### Basic Flow

```ellex
# 1. Output messages
tell "Welcome to Ellex!"

# 2. Get user input
ask "What's your name?" → name

# 3. Use the input
tell "Nice to meet you, {name}!"

# 4. Make decisions
when name is "Alice" do
    tell "That's my favorite name!"
end
```

### Creating Functions

```ellex
# Define a function
make greet_user do
    ask "What's your name?" → user_name
    tell "Hello, {user_name}! Welcome to Ellex!"
end

# Call the function
greet_user
```

### Loops and Repetition

```ellex
# Simple counting
repeat 5 times do
    tell "Counting..."
end

# Interactive loops
ask "How many times should I count?" → count
repeat count times do
    tell "Count number {count}"
end
```

## Error Handling

### Kid-Friendly Error Messages

The REPL provides friendly error messages with emojis and suggestions:

```ellex
# Typo in command
tel "Hello!"
# Output: "Error: I didn't understand 'tel'. Did you mean 'tell'? 🤔"

# Undefined variable
tell "Hello, {unknown_name}!"
# Output: "Error: I don't know what 'unknown_name' is. Use 'ask' to get input first! 🤷"

# Too many loop iterations
repeat 50000 times do tell "Too much!" end
# Output: "Whoa! That's a lot of repetitions. Let's try something smaller! 🐌"
```

### Error Recovery

The REPL continues running even after errors:

```ellex
tell "This works"
invalid_command
tell "This still works after the error"
```

## Advanced Features

### Session Persistence

```rust
// Save session to file
let session = ReplSession::new();
session.save_to_file(&PathBuf::from("my_session.json"))?;

// Load session from file
let loaded_session = ReplSession::load_from_file(&PathBuf::from("my_session.json"))?;
```

### Custom Configuration

```rust
let config = EllexConfig {
    execution_timeout_ms: 10000,  // 10 seconds
    memory_limit_mb: 128,         // 128 MB
    enable_turtle: true,          // Enable turtle graphics
    enable_ai: false,             // Disable AI assistance
    max_recursion_depth: 50,      // Limit recursion
    max_loop_iterations: 5000,    // Limit loop iterations
};

let session = ReplSession::with_config(config);
```

### Modal Programming Integration

```ellex
# Explore code (future feature)
@listen do
    tell                    # Learn about the tell command
    around greet_user      # Explore around a function
end

# Analyze code (future feature)
@think do
    what does greet_user do?    # AI-powered analysis
    where is name used?         # Variable usage analysis
end

# Refactor code (future feature)
@build do
    rename old_name → new_name     # Rename variables
    extract 10,20 → new_function  # Extract functions
end
```

## Integration Examples

### Web Application Integration

```javascript
class EllexREPL {
    constructor() {
        this.apiBase = '/api/repl';
        this.sessionId = null;
    }
    
    async createSession() {
        const response = await fetch(`${this.apiBase}/sessions`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({})
        });
        const data = await response.json();
        this.sessionId = data.session_id;
        return data;
    }
    
    async execute(code) {
        const response = await fetch(`${this.apiBase}/execute`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                code: code,
                session_id: this.sessionId
            })
        });
        return await response.json();
    }
}

// Usage
const repl = new EllexREPL();
await repl.createSession();
const result = await repl.execute('tell "Hello from web!"');
console.log(result.output); // ["Hello from web!"]
```

### Educational Platform Integration

```javascript
// Lesson progression tracking
class EllexLesson {
    constructor(lessonId) {
        this.lessonId = lessonId;
        this.repl = new EllexREPL();
        this.currentStep = 0;
        this.studentProgress = [];
    }
    
    async startLesson() {
        await this.repl.createSession();
        return this.getNextChallenge();
    }
    
    async submitCode(code) {
        const result = await this.repl.execute(code);
        const success = this.validateOutput(result.output);
        
        this.studentProgress.push({
            step: this.currentStep,
            code: code,
            output: result.output,
            success: success,
            timestamp: new Date()
        });
        
        if (success) {
            this.currentStep++;
            return this.getNextChallenge();
        } else {
            return this.getHint();
        }
    }
}
```

## Testing the REPL

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_execution() {
        let mut session = ReplSession::new();
        let result = session.execute_line("tell \"Hello!\"").unwrap();
        assert_eq!(result, vec!["Hello!"]);
    }

    #[test]
    fn test_variable_setting() {
        let mut session = ReplSession::new();
        session.execute_line("/set name \"Alice\"").unwrap();
        let result = session.execute_line("tell \"Hello, {name}!\"").unwrap();
        assert_eq!(result, vec!["Hello, Alice!"]);
    }

    #[test]
    fn test_session_persistence() {
        let mut session = ReplSession::new();
        session.execute_line("/set test_var 42").unwrap();
        
        let temp_file = std::env::temp_dir().join("test_session.json");
        session.save_to_file(&temp_file).unwrap();
        
        let loaded_session = ReplSession::load_from_file(&temp_file).unwrap();
        assert_eq!(loaded_session.variables.get("test_var"), 
                   Some(&EllexValue::Number(42.0)));
    }
}
```

### API Testing

```bash
# Test session creation
curl -X POST http://localhost:8080/api/repl/sessions \
  -H "Content-Type: application/json" \
  -d '{}'

# Test code execution
curl -X POST http://localhost:8080/api/repl/execute \
  -H "Content-Type: application/json" \
  -d '{"code": "tell \"Hello!\"", "session_id": "your-session-id"}'

# Test session info
curl http://localhost:8080/api/repl/sessions/your-session-id
```

## Performance Considerations

### Memory Management
- Sessions automatically clean up unused variables
- Output buffers have configurable size limits
- Function definitions are garbage collected when sessions end

### Execution Limits
- Default 5-second timeout per execution
- 64MB memory limit per session
- Maximum 100 recursion depth
- Maximum 10,000 loop iterations

### Scaling
- Each session is isolated and thread-safe
- Sessions can be distributed across multiple servers
- Database persistence available for long-term storage

## Best Practices

### For Educators
1. **Start Simple**: Begin with basic `tell` statements
2. **Encourage Experimentation**: REPL recovers from errors gracefully
3. **Use Variables Early**: Show immediate value of variable storage
4. **Build Up Complexity**: Gradually introduce functions and loops
5. **Celebrate Errors**: Use them as learning opportunities

### For Developers
1. **Handle Async Operations**: All API calls return promises
2. **Implement Error Handling**: Display friendly error messages
3. **Track Session State**: Monitor variables and execution count
4. **Provide Context**: Help users understand what happened
5. **Enable Exploration**: Encourage trying different approaches

## Troubleshooting

### Common Issues

**Session Not Found**
```json
{"error": "Session not found", "status": 404}
```
Solution: Create a new session or verify the session ID.

**Execution Timeout**
```json
{"output": ["Whoa! Your code is taking a really long time. Let's try something simpler! 🐌"]}
```
Solution: Reduce loop iterations or simplify the code.

**Parse Error**
```json
{"output": ["I didn't understand that. Try checking your spelling! 🤔"]}
```
Solution: Check syntax and refer to language documentation.

### Getting Help

- Use `/help` command in any REPL session
- Check the language specification documentation
- Review example programs in the `/examples` directory
- Join the community forum for peer support

## Changelog

### Version 0.1.0
- Initial REPL implementation
- Basic statement execution (`tell`, `ask`, `repeat`, `when`, `make`)
- Session management with persistence
- REST API endpoints
- Kid-friendly error messages
- Interactive terminal and web interfaces

### Upcoming Features
- Modal programming commands (`@listen`, `@think`, `@build`)
- AI-powered code suggestions and explanations
- Turtle graphics integration
- Collaborative sessions
- Visual programming interface
- Mobile app support