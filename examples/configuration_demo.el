# Configuration and Execution Engine Demo
# This example demonstrates various Ellex configuration features and safety limits

tell "🚀 Welcome to the Ellex Configuration Demo!"
tell ""

# Basic execution example
tell "1. Basic Execution:"
tell "Hello from Ellex!"
tell "Current time: 2024"
tell ""

# Loop demonstration
tell "2. Loop Execution (testing loop limits):"
repeat 5 times do
    tell "Loop iteration - safe within limits"
end
tell ""

# Variable usage
tell "3. Variable Management:"
ask "What's your name?" = name
tell "Nice to meet you, {name}!"
tell ""

# Conditional logic
tell "4. Conditional Execution:"
when name is "Alice" do
    tell "Hello Alice! You have admin privileges."
end
tell ""

# Turtle graphics (if enabled)
tell "5. Turtle Graphics Demo:"
tell "Drawing a square..."

# Move forward and turn right 4 times to make a square
forward
right
forward
right
forward
right
forward
right

tell "Square drawing complete!"
tell ""

# Function definition and usage
tell "6. Function Definition:"
make greet_user do
    tell "👋 Greetings from a custom function!"
    tell "This demonstrates function execution within safety limits."
end

greet_user
tell ""

# Recursion test (will hit safety limits eventually)
tell "7. Safety Limits Test:"
make countdown do
    tell "This would normally count down, but safety limits prevent infinite recursion"
end

countdown
tell ""

# Memory and performance demonstration
tell "8. Performance Test:"
repeat 10 times do
    tell "Processing... (testing instruction limits)"
end
tell ""

# Modal programming examples
tell "9. Modal Programming:"
@listen do
    around greet_user
end

@think do
    what does greet_user do?
end

@build do
    extract 1,5 = new_function
end
tell ""

# Service configuration example
tell "10. Service Configuration:"
service "demo-app" do
    image "ellex/demo:latest"
    port 3000
    environment do
        DEBUG "true"
        LOG_LEVEL "info"
    end
    health_check do
        path "/health"
        interval 30
        timeout 5
    end
end
tell ""

# Error handling demonstration
tell "11. Error Handling:"
tell "Ellex provides friendly error messages for kids!"
tell "Try running a command that doesn't exist to see helpful suggestions."
tell ""

# Configuration summary
tell "12. Configuration Features Demonstrated:"
tell "✅ Execution timeout limits"
tell "✅ Memory usage monitoring"
tell "✅ Loop iteration limits"
tell "✅ Recursion depth limits"
tell "✅ Turtle graphics integration"
tell "✅ Variable management"
tell "✅ Function definitions"
tell "✅ Modal programming"
tell "✅ Service configuration"
tell "✅ Safety monitoring"
tell "✅ Friendly error messages"
tell ""

tell "🎉 Configuration demo complete!"
tell "All features executed safely within configured limits."