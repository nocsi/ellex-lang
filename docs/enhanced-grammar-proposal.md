# Enhanced Ellex Grammar Proposal

## Current State vs. Proposed Enhancement

### Current Grammar Limitations
- Only basic expressions (strings, numbers, identifiers)
- No arithmetic or logical operations
- Functions without parameters
- Limited control flow
- No proper variable assignment

### Proposed Enhanced Grammar

```pest
WHITESPACE = _{ " " | "\t" | "\n" | "\r" }
COMMENT = _{ "#" ~ (!NEWLINE ~ ANY)* ~ NEWLINE? }
NEWLINE = _{ "\n" | "\r\n" }

program = { SOI ~ (statement | modal_block | service_block)* ~ EOI }

// Enhanced statement types
statement = { 
    tell_stmt | ask_stmt | set_stmt | repeat_stmt | while_stmt | for_stmt |
    when_stmt | make_stmt | return_stmt | break_stmt | continue_stmt |
    func_call | expression_stmt
}

// Output statements
tell_stmt = { "tell" ~ expression }

// Input statements
ask_stmt = { "ask" ~ expression ~ "→" ~ ident ~ ("as" ~ type_hint)? }

// Variable assignment
set_stmt = { "set" ~ ident ~ "to" ~ expression }

// Enhanced loops
repeat_stmt = { "repeat" ~ expression ~ "times" ~ "do" ~ statement* ~ "end" }
while_stmt = { "while" ~ expression ~ "do" ~ statement* ~ "end" }
for_stmt = { "for" ~ ident ~ "in" ~ expression ~ "do" ~ statement* ~ "end" }

// Enhanced conditionals
when_stmt = { "when" ~ expression ~ "do" ~ statement* ~ elseif_clause* ~ else_clause? ~ "end" }
elseif_clause = { "otherwise" ~ "when" ~ expression ~ "do" ~ statement* }
else_clause = { "otherwise" ~ "do" ~ statement* }

// Enhanced functions
make_stmt = { "make" ~ ident ~ ("with" ~ param_list)? ~ "do" ~ statement* ~ "end" }
param_list = { ident ~ ("," ~ ident)* }

// Control flow
return_stmt = { "return" ~ expression? }
break_stmt = { "break" }
continue_stmt = { "continue" }

// Function calls with arguments
func_call = { ident ~ ("with" ~ arg_list)? }
arg_list = { expression ~ ("," ~ expression)* }

// Expression statement (for side effects)
expression_stmt = { expression }

// Enhanced expressions with operator precedence
expression = { logical_or }

logical_or = { logical_and ~ ("or" ~ logical_and)* }
logical_and = { equality ~ ("and" ~ equality)* }
equality = { comparison ~ (("is" | "is not") ~ comparison)* }
comparison = { addition ~ ((">" | ">=" | "<" | "<=") ~ addition)* }
addition = { multiplication ~ (("+" | "-") ~ multiplication)* }
multiplication = { unary ~ (("*" | "/" | "%") ~ unary)* }
unary = { ("not" | "-")? ~ primary }

primary = { 
    number | string | interpolated_string | boolean | nil |
    list | record | func_call | ident | 
    "(" ~ expression ~ ")"
}

// Enhanced data types
number = { "-"? ~ (float | integer) }
float = { ASCII_DIGIT+ ~ "." ~ ASCII_DIGIT+ }
integer = { ASCII_DIGIT+ }
boolean = { "true" | "false" }
nil = { "nil" }

string = { "\"" ~ (!"\"" ~ ANY)* ~ "\"" }
interpolated_string = { "\"" ~ (interp_part | (!"\"" ~ ANY))* ~ "\"" }
interp_part = { "{" ~ expression ~ "}" }

// Enhanced collections
list = { "[" ~ (expression ~ ("," ~ expression)*)? ~ "]" }
record = { "{" ~ (record_field ~ ("," ~ record_field)*)? ~ "}" }
record_field = { ident ~ ":" ~ expression }

// Turtle graphics (natural language)
turtle_stmt = { 
    move_stmt | turn_stmt | color_stmt | draw_stmt | pen_stmt
}

move_stmt = { "move" ~ direction ~ expression ~ unit? }
direction = { "forward" | "backward" | "up" | "down" | "left" | "right" }
unit = { "steps" | "pixels" | "units" }

turn_stmt = { "turn" ~ ("left" | "right") ~ expression ~ "degrees"? }
color_stmt = { "use" ~ "color" ~ expression }
draw_stmt = { "draw" ~ shape ~ ("with" ~ shape_props)? }
shape = { "circle" | "square" | "rectangle" | "line" | "triangle" }
shape_props = { shape_prop ~ ("and" ~ shape_prop)* }
shape_prop = { 
    ("radius" ~ expression) | 
    ("width" ~ expression) | 
    ("height" ~ expression) |
    ("size" ~ expression)
}

pen_stmt = { ("lift" | "drop") ~ "pen" }

// Modal programming (enhanced)
modal_block = { "@" ~ modal_mode ~ "do" ~ modal_content* ~ "end" }
modal_mode = { "listen" | "think" | "build" | "teach" | "explore" }

modal_content = { 
    listen_cmd | think_query | build_cmd | teach_cmd | explore_cmd
}

listen_cmd = { 
    "/" ~ ident |
    "around" ~ ident |
    "find" ~ string |
    "search" ~ "for" ~ string
}

think_query = { 
    "what" ~ "does" ~ ident ~ "do?" |
    "where" ~ "is" ~ ident ~ "used?" |
    "how" ~ "does" ~ expression ~ "work?" |
    "why" ~ "is" ~ expression ~ "needed?"
}

build_cmd = { 
    "rename" ~ ident ~ "to" ~ ident |
    "extract" ~ integer ~ "," ~ integer ~ "to" ~ ident |
    "move" ~ ident ~ "to" ~ ident |
    "copy" ~ ident ~ "as" ~ ident
}

teach_cmd = {
    "explain" ~ ident |
    "show" ~ "examples" ~ "of" ~ ident |
    "demonstrate" ~ ident
}

explore_cmd = {
    "try" ~ statement |
    "experiment" ~ "with" ~ expression
}

// Service definitions (for advanced users)
service_block = { "service" ~ string ~ "do" ~ service_prop* ~ "end" }
service_prop = { 
    "image" ~ string | 
    "port" ~ integer | 
    "environment" ~ "do" ~ env_pair* ~ "end" | 
    "health_check" ~ "do" ~ health_prop* ~ "end" |
    "depends" ~ "on" ~ string_list |
    "expose" ~ "port" ~ integer |
    "mount" ~ string ~ "at" ~ string
}

env_pair = { ident ~ string }
health_prop = { 
    "path" ~ string | 
    "interval" ~ integer | 
    "timeout" ~ integer |
    "retries" ~ integer
}

string_list = { "[" ~ (string ~ ("," ~ string)*)? ~ "]" }

// Enhanced identifiers and types
ident = { (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }
type_hint = { "number" | "string" | "list" | "record" | "boolean" }

// Error handling (future feature)
try_stmt = { "try" ~ "do" ~ statement* ~ "catch" ~ ident ~ "do" ~ statement* ~ "end" }
throw_stmt = { "throw" ~ expression }
```

## Key Enhancements

### 1. **Proper Expression System**
- Arithmetic operations: `+`, `-`, `*`, `/`, `%`
- Comparison operations: `>`, `>=`, `<`, `<=`, `is`, `is not`
- Logical operations: `and`, `or`, `not`
- Proper operator precedence

### 2. **Enhanced Functions**
```ellex
make greet_person with name, age do
    tell "Hello, {name}! You are {age} years old."
end

greet_person with "Alice", 25
```

### 3. **Better Control Flow**
```ellex
when age >= 18 do
    tell "You can vote!"
otherwise when age >= 16 do
    tell "You can drive!"
otherwise do
    tell "You're still growing up!"
end
```

### 4. **Enhanced Data Types**
```ellex
set numbers to [1, 2, 3, 4, 5]
set person to {name: "Alice", age: 25, city: "Portland"}
set is_student to true
```

### 5. **Natural Language Turtle Graphics**
```ellex
move forward 100 steps
turn right 90 degrees
use color "blue"
draw circle with radius 50
```

### 6. **Advanced Loops**
```ellex
# Enhanced repeat with expressions
set count to 5
repeat count times do
    tell "Counting: {count}"
    set count to count - 1
end

# While loops
while count > 0 do
    tell "Countdown: {count}"
    set count to count - 1
end

# For loops
for item in [1, 2, 3, 4, 5] do
    tell "Item: {item}"
end
```

## Implementation Priority

### Phase 1 (Core Enhancements)
1. ✅ Basic expressions with arithmetic operators
2. ✅ Variable assignment (`set x to value`)
3. ✅ Function parameters and arguments
4. ✅ Enhanced conditionals

### Phase 2 (Advanced Features)
1. ⏳ While and for loops
2. ⏳ Records/objects
3. ⏳ Enhanced turtle graphics
4. ⏳ Error handling

### Phase 3 (Professional Features)
1. 🔮 Modules and imports
2. 🔮 Advanced AI integration
3. 🔮 Debugging support
4. 🔮 Performance optimization

## Comparison After Enhancement

**With these enhancements, Ellex would be comparable to:**

### Educational Languages
- **Scratch**: ✅ More powerful (text-based, functions with params)
- **Logo**: ✅ Much more comprehensive
- **Alice**: ✅ Similar feature completeness

### Beginner-Friendly Languages
- **Python (basic subset)**: ✅ Comparable for educational use
- **JavaScript (basic subset)**: ✅ Similar expressiveness
- **Lua**: ✅ Similar simplicity with more safety

### Production Readiness
- **For education (K-12)**: ✅ Excellent
- **For hobbyist projects**: ✅ Good
- **For professional development**: ❌ Still limited (no modules, limited ecosystem)

## Conclusion

The enhanced grammar would make Ellex a **complete educational programming language** that could serve students from elementary through high school, providing a natural progression path to more advanced languages.

**Current completeness**: ~30%
**With proposed enhancements**: ~85% (for educational use)
**Timeline for full implementation**: 2-3 months of focused development