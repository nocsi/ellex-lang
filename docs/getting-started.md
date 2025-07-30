# Getting Started with Ellex

Welcome to Ellex! This guide will help you get up and running with the natural language programming environment designed for kids.

## What is Ellex?

Ellex is a programming language that reads like English and is designed to be safe, fun, and educational. It combines:

- **Natural language syntax** that's easy to understand
- **Built-in safety features** to prevent crashes and infinite loops
- **Visual programming** with turtle graphics
- **AI assistance** to help you learn
- **Real-time feedback** so you see results immediately

## Installation

### Prerequisites

- Rust 1.70 or higher
- Git
- A terminal or command prompt

### Setup

1. Clone the repository:
```bash
git clone https://github.com/your-org/ellex-lang.git
cd ellex-lang
```

2. Run the setup script:
```bash
./scripts/setup.sh
```

3. Build the project:
```bash
make build
```

4. Test the installation:
```bash
make test
```

## Your First Ellex Program

Let's start with the classic "Hello, World!" program.

Create a file called `hello.ellex`:

```ellex
tell "Hello, World!"
```

Run it:
```bash
cargo run --bin ellex_cli run hello.ellex
```

You should see:
```
Hello, World!
```

Congratulations! You've written your first Ellex program! 🎉

## Interactive Learning with REPL

The best way to learn Ellex is through the interactive REPL (Read-Eval-Print Loop):

```bash
cargo run --bin ellex_cli repl
```

This will start an interactive session where you can type commands and see results immediately:

```
🌿 Welcome to Ellex!
Type 'tell "Hello world!"' to get started
AI assistance is enabled 🤖
> tell "Hello there!"
Hello there!
> ask "What's your name?" → name
What's your name?: Alice
> tell "Nice to meet you, {name}!"
Nice to meet you, Alice!
```

## Basic Commands

### 1. Telling (Output)

The `tell` command displays messages:

```ellex
tell "Hello, everyone!"
tell "The answer is 42"
tell 3.14159
```

### 2. Asking (Input)

The `ask` command gets input from the user:

```ellex
ask "What's your favorite color?" → color
tell "I like {color} too!"
```

The `→` symbol stores the answer in a variable called `color`. You can then use `{color}` in other messages.

### 3. Repeating (Loops)

The `repeat` command does something multiple times:

```ellex
repeat 3 times do
    tell "Hip hip hooray!"
end
```

### 4. When (Conditionals)

The `when` command does different things based on conditions:

```ellex
ask "How old are you?" → age
when age is 16 do
    tell "You can drive!"
otherwise do
    tell "Not quite old enough to drive yet!"
end
```

### 5. Making Functions

The `make` command creates reusable pieces of code:

```ellex
make greet_person do
    ask "What's your name?" → name
    tell "Hello, {name}! Welcome to Ellex!"
end

greet_person
```

## Hands-On Tutorial

Let's build increasingly complex programs step by step.

### Tutorial 1: Personal Greeter

**Goal**: Create a program that asks for your name and age, then gives you a personalized greeting.

**Step 1**: Basic greeting
```ellex
ask "What's your name?" → name
tell "Hello, {name}!"
```

**Step 2**: Add age
```ellex
ask "What's your name?" → name
ask "How old are you?" → age
tell "Hello, {name}! You are {age} years old."
```

**Step 3**: Add conditional response
```ellex
ask "What's your name?" → name
ask "How old are you?" → age

when age is 10 do
    tell "Hello, {name}! Double digits - you're growing up!"
when age is 13 do
    tell "Hello, {name}! Welcome to being a teenager!"
otherwise do
    tell "Hello, {name}! You are {age} years old."
end
```

### Tutorial 2: Counting Game

**Goal**: Create a counting game that counts up to a number the user chooses.

```ellex
make counting_game do
    ask "Pick a number from 1 to 10" → target
    tell "Let's count to {target}!"
    
    repeat target times do
        tell "Counting... almost there!"
    end
    
    tell "We made it to {target}! Great job!"
end

counting_game
```

### Tutorial 3: Color Artist

**Goal**: Use turtle graphics to draw colorful shapes.

```ellex
make draw_square do
    repeat 4 times do
        move forward 100
        turn right 90
    end
end

ask "What's your favorite color?" → color
use color color

tell "Drawing a {color} square for you!"
draw_square

tell "How about a circle too?"
draw circle with radius 50
```

### Tutorial 4: Smart Assistant

**Goal**: Create a helpful assistant that can do different tasks.

```ellex
make smart_assistant do
    tell "Hi! I'm your smart assistant. What would you like to do?"
    ask "Type 'count', 'draw', or 'greet'" → choice
    
    when choice matches "count" do
        ask "Count to what number?" → num
        repeat num times do
            tell "Counting!"
        end
        tell "All done counting!"
    end
    
    when choice matches "draw" do
        ask "What color should I use?" → color
        use color color
        repeat 6 times do
            move forward 60
            turn right 60
        end
        tell "I drew a {color} hexagon!"
    end
    
    when choice matches "greet" do
        ask "Who should I greet?" → person
        tell "Hello there, {person}! Hope you're having a great day!"
    end
    
    otherwise do
        tell "I don't know how to do that yet, but I'm learning!"
    end
end

smart_assistant
```

## Advanced Features

### String Interpolation

You can put variables inside strings using `{variable_name}`:

```ellex
ask "What's your name?" → name  
ask "What's your age?" → age
tell "Hi {name}, you've been alive for {age} years!"
```

### Comments

Use `#` to add comments that explain your code:

```ellex
# This is a comment - it doesn't do anything
tell "Hello!"  # You can also put comments at the end of lines

# Comments help explain what your code does
# This is especially helpful for complex programs
```

### Lists

You can work with lists of things:

```ellex
# This will be more fully supported in future versions
tell [1, 2, 3, 4, 5]
tell ["apple", "banana", "orange"]
```

## Turtle Graphics

Ellex includes a built-in turtle graphics system for visual programming:

### Basic Movement
```ellex
move forward 100    # Move forward 100 pixels
turn right 90       # Turn right 90 degrees
turn left 45        # Turn left 45 degrees
```

### Colors and Drawing
```ellex
use color "red"     # Set the drawing color
use color "blue"
use color "green"
```

### Shapes
```ellex
draw circle with radius 50
```

### Complete Example
```ellex
make colorful_square do
    use color "purple"
    repeat 4 times do
        move forward 100
        turn right 90
    end
end

make rainbow_circles do
    use color "red"
    draw circle with radius 30
    
    use color "orange"  
    move forward 40
    draw circle with radius 30
    
    use color "yellow"
    move forward 40
    draw circle with radius 30
end

colorful_square
rainbow_circles
```

## Modal Programming

Ellex supports different "modes" for different types of activities:

### Listen Mode - Exploring Code
```ellex
@listen do
    /tell                    # Learn about the tell command
    around greet_person      # Explore the greet_person function
end
```

### Think Mode - Understanding Code  
```ellex
@think do
    what does greet_person do?
    where is name used?
end
```

### Build Mode - Changing Code
```ellex
@build do
    rename old_name → new_name
    extract 10,20 → helper_function
end
```

## Safety and Error Handling

Ellex is designed to be safe and forgiving:

### Automatic Safety Features
- **Time limits**: Programs automatically stop after 5 seconds
- **Memory limits**: Programs can't use too much memory
- **Loop limits**: Loops can't run forever
- **Recursion limits**: Functions can't call themselves too many times

### Friendly Error Messages
Instead of scary technical errors, Ellex gives helpful messages:

- ❌ **Bad**: `SyntaxError: unexpected token at line 5`
- ✅ **Good**: `I didn't understand 'tel' - did you mean 'tell'? 🤔`

### Getting Help
If you make a mistake, Ellex will:
1. Explain what went wrong in simple terms
2. Suggest how to fix it
3. Give you an example of the correct way

## AI Assistance

When AI assistance is enabled, Ellex can:

- **Suggest completions** as you type
- **Explain errors** in kid-friendly language  
- **Recommend improvements** to make your code better
- **Answer questions** about how things work
- **Provide examples** when you're stuck

To enable AI assistance:
```bash
cargo run --bin ellex_cli repl --ai=true
```

## Tips for Success

### 1. Start Small
Begin with simple programs and gradually add complexity:
- Start with `tell` statements
- Add `ask` for input
- Try `repeat` for loops
- Use `when` for decisions
- Create `make` functions when you have repeated code

### 2. Experiment Freely
The REPL is perfect for trying things out:
- Test individual commands
- Try different values
- See what happens when you change things
- Don't worry about breaking anything!

### 3. Read Your Code Aloud
One of Ellex's strengths is that code reads like English:
- "Tell the user hello"
- "Ask what's your name and store it in name"
- "Repeat 5 times, move forward and turn right"

### 4. Use Comments
Explain what your code does:
```ellex
# Ask the user for their favorite things
ask "What's your favorite color?" → color
ask "What's your favorite number?" → number

# Create a personalized message
tell "I love {color} and the number {number} too!"
```

### 5. Break Complex Tasks Down
Use functions to organize your code:
```ellex
make get_user_info do
    ask "What's your name?" → name
    ask "How old are you?" → age
end

make greet_user do
    tell "Hello, {name}! You are {age} years old."
end

# Main program
get_user_info
greet_user
```

## What's Next?

Once you're comfortable with the basics:

1. **Try the TUI Interface**: `cargo run --bin ellex_cli tui` for a visual programming environment
2. **Explore the Web Playground**: `cargo run --bin ellex_cli serve` for browser-based coding
3. **Check out the Examples**: Look in the `examples/` directory for inspiration
4. **Join the Community**: Share your creations and get help from other learners
5. **Advanced Topics**: Learn about service definitions, advanced AI features, and transitioning to full programming languages

## Common Patterns

### Input Validation
```ellex
make get_positive_number do
    ask "Enter a positive number" → num
    when num is 0 do
        tell "Zero isn't positive! Try again."
        get_positive_number
    end
    # Note: This will be enhanced with proper number validation
end
```

### Menu Systems
```ellex
make show_menu do
    tell "What would you like to do?"
    tell "1. Play a game"
    tell "2. Draw a picture"  
    tell "3. Tell a joke"
    ask "Enter your choice (1, 2, or 3)" → choice
    
    when choice is "1" do
        play_game
    when choice is "2" do
        draw_picture  
    when choice is "3" do
        tell_joke
    otherwise do
        tell "I don't understand that choice."
        show_menu
    end
end
```

### Drawing Patterns
```ellex
make draw_polygon do
    ask "How many sides?" → sides
    ask "What color?" → color
    
    use color color
    repeat sides times do
        move forward 80
        turn right (360 / sides)
    end
end
```

## Troubleshooting

### Common Issues

**Problem**: "I typed something and nothing happened"
**Solution**: Make sure to press Enter after typing your command

**Problem**: "It says it doesn't understand my command"  
**Solution**: Check spelling and make sure you're using the right syntax

**Problem**: "My program keeps running forever"
**Solution**: Ellex will automatically stop it after 5 seconds. Check your loop conditions.

**Problem**: "I made a mistake and want to start over"
**Solution**: Type `exit` in the REPL to quit, then start again

### Getting Help

1. **Use the help command**: Type `/help` in the REPL
2. **Check the examples**: Look in the `examples/` directory
3. **Read the error messages**: They're designed to be helpful!
4. **Ask the AI**: When AI is enabled, ask questions like "How do I make a loop?"

## Conclusion

Congratulations on starting your journey with Ellex! Remember:

- **Programming is creative** - there are many ways to solve problems
- **Mistakes are learning opportunities** - every programmer makes them
- **Practice makes progress** - the more you code, the better you get
- **Have fun!** - programming should be enjoyable

Happy coding! 🚀

---

*For more advanced topics, check out the [Language Specification](language-specification.md) and [API Reference](api-reference.md).*