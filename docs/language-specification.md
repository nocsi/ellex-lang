# Ellex Language Specification

## Overview

Ellex is a natural language programming environment designed specifically for kids (ages 6-16) with AI-powered assistance and modal editing. It combines the safety and fault-tolerance of the BEAM VM with intuitive natural language syntax, providing a pathway from simple natural language commands to full Elixir development.

## Design Principles

1. **Safety First**: Built-in execution limits, timeouts, memory constraints, and safe execution environment
2. **Natural Language Syntax**: Commands that read like English sentences
3. **Modal Programming**: Different modes for different types of interactions
4. **Progressive Learning**: Start with natural language, evolve to full programming languages
5. **Real-time Feedback**: Immediate execution and visual feedback through turtle graphics
6. **AI-Powered Learning**: Intelligent assistance and suggestions

## Core Language Features

### Basic Statements

#### Tell Statement
The `tell` statement outputs text or values to the user.

**Syntax:**
```ellex
tell "message"
tell variable_name
tell 42
```

**Examples:**
```ellex
tell "Hello, world!"
tell "The answer is 42"
tell user_name
tell 3.14159
```

#### Ask Statement
The `ask` statement prompts the user for input and stores it in a variable.

**Syntax:**
```ellex
ask "question" → variable_name
ask "question" → variable_name as type_hint
```

**Examples:**
```ellex
ask "What's your name?" → name
ask "How old are you?" → age as number
ask "What's your favorite color?" → color as string
```

**Type Hints:**
- `string`: Text input
- `number`: Numeric input
- `list`: List input

#### Repeat Statement
The `repeat` statement executes a block of code multiple times.

**Syntax:**
```ellex
repeat N times do
    statements...
end
```

**Examples:**
```ellex
repeat 3 times do
    tell "Hello!"
end

repeat 5 times do
    tell "Counting..."
    ask "Press enter to continue" → continue
end
```

#### When Statement (Conditionals)
The `when` statement executes code based on conditions.

**Syntax:**
```ellex
when variable is value do
    statements...
end

when variable matches pattern do
    statements...
otherwise do
    statements...
end
```

**Examples:**
```ellex
when age is 16 do
    tell "You can drive!"
end

when color matches "blue" do
    tell "Blue is a nice color!"
otherwise do
    tell "That's a different color!"
end
```

#### Make Statement (Function Definition)
The `make` statement creates reusable functions.

**Syntax:**
```ellex
make function_name do
    statements...
end
```

**Examples:**
```ellex
make greet_user do
    ask "What's your name?" → name
    tell "Hello, {name}!"
end

make count_to_ten do
    repeat 10 times do
        tell "Counting..."
    end
end
```

#### Function Calls
Execute previously defined functions by name.

**Examples:**
```ellex
greet_user
count_to_ten
```

### Data Types

#### String
Text values enclosed in double quotes.

**Examples:**
```ellex
"Hello, world!"
"This is a string with spaces"
"String with {variable} interpolation"
```

#### Number
Numeric values (integers and floating-point).

**Examples:**
```ellex
42
3.14159
-17
0.5
```

#### List
Collections of values.

**Examples:**
```ellex
[1, 2, 3, 4, 5]
["apple", "banana", "orange"]
[1, "hello", 3.14]
```

#### Nil
Represents empty or no value.

**Example:**
```ellex
nil
```

### String Interpolation

Variables can be embedded in strings using `{variable_name}` syntax.

**Examples:**
```ellex
ask "What's your name?" → name
tell "Hello, {name}!"

ask "How old are you?" → age
tell "You are {age} years old!"
```

### Comments

Comments start with `#` and continue to the end of the line.

**Examples:**
```ellex
# This is a comment
tell "Hello!" # This is also a comment

# You can have full-line comments
# that explain what the code does
```

## Modal Programming

Ellex supports different modes for different types of interactions:

### Listen Mode (@listen)
Exploration and discovery mode for understanding code.

**Syntax:**
```ellex
@listen do
    /tell                    # Explore the tell command
    around function_name     # Explore around a function
end
```

### Think Mode (@think)
Query and analysis mode for understanding code behavior.

**Syntax:**
```ellex
@think do
    what does greet_user do?
    where is name used?
end
```

### Build Mode (@build)
Creation and refactoring mode for modifying code.

**Syntax:**
```ellex
@build do
    rename old_name → new_name
    extract 10,20 → new_function
end
```

### Teach Mode (@teach)
Educational mode for learning and explanation.

**Syntax:**
```ellex
@teach do
    explain loops
    show examples of functions
end
```

## Turtle Graphics

Ellex includes built-in turtle graphics for visual programming:

```ellex
move forward 100
turn right 90
use color "red"
draw circle with radius 50
```

## Safety Features

### Execution Limits
- **Timeout**: 5 seconds maximum execution time
- **Memory**: 64MB memory limit
- **Recursion**: Maximum 100 recursion depth
- **Loops**: Maximum 10,000 loop iterations

### Error Handling
Ellex provides kid-friendly error messages with emojis and suggestions:

- **Timeout errors**: "Whoa! Your code is taking a really long time. Let's try something simpler! 🐌"
- **Parse errors**: "I didn't understand that. [explanation] 🤔"
- **Logic errors**: "That doesn't make sense: [explanation] 🤷"
- **Safety violations**: "Safety first! [reason] 🛡️"

## Configuration

Default runtime configuration:
- Execution timeout: 5000ms
- Memory limit: 64MB
- Turtle graphics: Enabled
- AI assistance: Enabled
- Max recursion depth: 100
- Max loop iterations: 10,000

## Examples

### Hello World
```ellex
tell "Hello, world!"
```

### Interactive Greeting
```ellex
ask "What's your name?" → name
tell "Hello, {name}! Nice to meet you!"
```

### Counting Game
```ellex
make counting_game do
    ask "Pick a number from 1 to 10" → target as number
    repeat target times do
        tell "Counting: {target}"
    end
    tell "All done counting!"
end

counting_game
```

### Color Chooser
```ellex
ask "What's your favorite color?" → color
when color matches "blue" do
    tell "Blue like the ocean! 🌊"
when color matches "red" do  
    tell "Red like a fire truck! 🚒"
otherwise do
    tell "{color} is a beautiful color! 🌈"
end
```

### Turtle Art
```ellex
make draw_square do
    repeat 4 times do
        move forward 100
        turn right 90
    end
end

use color "purple"
draw_square
move forward 50
use color "green"
draw_square
```

## Advanced Features

### Service Blocks (Future)
For more advanced users, Ellex will support service definitions:

```ellex
service "web-app" do
    image "node:16"
    port 3000
    environment do
        NODE_ENV "production"
        PORT "3000"
    end
    health_check do
        path "/health"
        interval 30
        timeout 5
    end
end
```

### AI Integration
- Code suggestions and completions
- Explain code functionality
- Debug assistance
- Learning recommendations
- Pattern recognition and optimization

### Progressive Learning Path
1. **Beginner**: Natural language commands, simple tell/ask
2. **Intermediate**: Functions, loops, conditionals, turtle graphics
3. **Advanced**: Modal programming, service definitions, AI assistance
4. **Transition**: Bridge to full Elixir programming

## Grammar Reference

The complete formal grammar is defined in `ellex.pest`:

```pest
program = { SOI ~ (statement | modal_block | service_block)* ~ EOI }
statement = { repeat_stmt | tell_stmt | ask_stmt | when_stmt | make_stmt | assignment | func_call }
tell_stmt = { "tell" ~ expression }
ask_stmt = { "ask" ~ expression ~ "→" ~ ident ~ ("as" ~ type_hint)? }
repeat_stmt = { "repeat" ~ integer ~ "times" ~ "do" ~ statement* ~ "end" }
# ... (see full grammar in ellex.pest)
```

## Error Recovery

Ellex includes intelligent error recovery:
- Suggests corrections for typos
- Explains syntax errors in kid-friendly language
- Provides examples of correct usage
- Offers alternative approaches

## Future Enhancements

- Web-based visual editor
- Mobile app support
- Collaborative programming features
- Integration with educational platforms
- Export to other programming languages
- Advanced AI tutoring capabilities