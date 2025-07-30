# 🌿 Ellex Language

> A natural language programming environment for kids with AI-powered assistance and modal editing

Ellex is designed to be the perfect first programming language for young learners while providing a natural pathway to full Elixir development. It combines the safety and fault-tolerance of the BEAM VM with intuitive natural language syntax and intelligent AI assistance.

## ✨ Features

### Core Language
- **Natural Language Syntax**: Code that reads like speaking - `tell "Hello!"` instead of `print("Hello")`
- **AI-Powered Learning**: Intelligent daemon that watches your code and suggests improvements
- **Modal Programming**: Different modes for speaking, listening, thinking, and building
- **Pattern Recognition**: Automatically detects repeated code and suggests functions
- **Real-Time Execution**: See results immediately as you type
- **Built-in Graphics**: Turtle graphics for visual programming
- **Smart Autocompletion**: Context-aware suggestions that understand what you're building
- **Crash-Resistant**: BEAM VM supervision ensures nothing breaks permanently
- **Progressive Learning**: Start with natural language, grow into full Elixir

### 🚀 Full Transpiler System (NEW!)
- **Ellex → JavaScript**: High-performance compilation to modern JavaScript
- **JavaScript → Ellex**: Convert JS back to natural language for learning
- **WebAssembly Support**: Maximum performance compilation to WASM
- **Multi-target**: Python, Go, TypeScript support
- **Advanced Optimizations**: Dead code elimination, loop unrolling, constant folding, type inference
- **Source Maps**: Debug support for transpiled code
- **Performance**: <1ms transpilation for simple programs, <100ms for complex ASTs

## 🚀 Quick Start

### Interactive REPL
```bash
# Start the interactive environment
ellex repl

# Start with AI assistance
ellex repl --ai
```

### Transpilation Examples

#### Ellex → JavaScript
```bash
# Basic transpilation
ellex transpile -i hello_world.ellex -t javascript

# With optimizations and minification
ellex transpile -i program.ellex -t javascript --optimize --minify -o output.js
```

#### JavaScript → Ellex
```bash
# Convert JavaScript to natural language
ellex from-js -i script.js -o converted.ellex
```

#### WebAssembly
```bash
# Compile to WebAssembly with loader
ellex transpile -i program.ellex -t wasm -o program.wat
```

### Basic Ellex Code

```ellex
tell "Hello, world!"
ask "What's your name?" → my_name
tell "Nice to meet you, {my_name}!"

repeat 3 times:
  tell "Programming is fun!"

make greet_friend:
  ask "Who should I greet?" → friend
  tell "Hello there, {friend}!"

greet_friend
```

### Modal Commands

```ellex
# Switch to listen mode (exploration)
@listen
  /tell           # Find all "tell" commands
  around function # Select around function
  
# Switch to think mode (query and understand)
@think
  what does greet_friend do?
  where is my_name used?

# Switch to build mode (create and transform)
@build
  rename my_name → user_name
  extract 1,5 → greeting_sequence
```

## 🛠️ Installation

### Prerequisites

- Rust 1.70+ 
- Elixir 1.15+ with OTP 26+
- Node.js 18+ (for web playground)

### Quick Setup

```bash
git clone https://github.com/nocsi/ellex-language.git
cd ellex-language
./scripts/setup.sh
```

### Build from Source

```bash
# Build Rust components
cd crates
cargo build --release

# Build Elixir backend
cd ../elixir_backend
mix deps.get
mix compile

# Build web playground
cd ../playground
npm install
npm run build
```

## 📚 Documentation

- [Language Specification](docs/language_spec.md)
- [Modal Editing Guide](docs/modal_editing.md)
- [Architecture Overview](docs/architecture.md)
- [Examples](examples/)

## 🎯 Project Goals

1. **Make programming accessible** to children aged 6-16
2. **Eliminate setup friction** with zero-install web playground
3. **Teach real concepts** using industry-proven languages (Elixir/BEAM)
4. **Support modal thinking** for advanced users
5. **Scale from beginner to expert** without language switching
6. **AI-Enhanced Learning** with intelligent assistance

## 📖 Examples

### Beginner: Simple Conversation

```ellex
tell "Welcome to my program!"
ask "How are you feeling today?" → mood
when mood is "happy":
  tell "That's wonderful! 😊"
when mood is "sad":
  tell "I hope your day gets better! 🌈"
otherwise:
  tell "Thanks for sharing!"
```

### Intermediate: Drawing Art

```ellex
make draw_flower:
  # Draw the stem
  use color green
  repeat 2 times:
    move forward 100
    turn right 90
    move forward 5
    turn right 90
  
  # Draw petals
  use color pink
  repeat 8 times:
    draw circle with radius 20
    turn right 45

draw_flower
```

### Advanced: Interactive Game

```ellex
make guessing_game:
  secret = random number from 1 to 100
  guesses = 0
  
  repeat until correct:
    ask "Guess the number (1-100):" → guess
    guesses = guesses + 1
    
    when guess < secret:
      tell "Too low! Try higher."
    when guess > secret:
      tell "Too high! Try lower."
    when guess equals secret:
      tell "Perfect! You got it in {guesses} guesses! 🎉"
      correct = true

guessing_game
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🌟 Acknowledgments

- Inspired by Elixir and the BEAM VM
- Modal editing concepts from vim
- Educational programming principles from Scratch and Logo
- Natural language processing advances in AI

---

**Made with 🌿 for young programmers everywhere**
