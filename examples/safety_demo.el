# Safety Limits Demonstration
# This example shows how Ellex protects young programmers with built-in safety features

tell "🛡️ Welcome to the Ellex Safety Demo!"
tell "This program demonstrates how Ellex keeps code execution safe for kids."
tell ""

# 1. Loop Safety - Preventing Infinite Loops
tell "1. Loop Safety Protection:"
tell "Ellex prevents infinite loops by limiting iterations..."

# This loop is safe and will complete
repeat 3 times do
    tell "Safe loop iteration"
end

# This would be caught by safety limits if too large
# repeat 50000 times do
#     tell "This would be stopped by safety limits"
# end

tell "✅ Loop completed safely within limits"
tell ""

# 2. Execution Time Limits
tell "2. Execution Time Protection:"
tell "Ellex stops programs that run too long..."

# Simulate some work
repeat 10 times do
    tell "Working... (this finishes quickly)"
end

tell "✅ Program completed within time limit"
tell ""

# 3. Memory Usage Monitoring
tell "3. Memory Usage Protection:"
tell "Ellex monitors memory to prevent system overload..."

# Create some variables (safe amount)
ask "What's your favorite color?" → color
ask "What's your age?" → age
ask "What's your favorite animal?" → animal

tell "Stored variables safely: {color}, {age}, {animal}"
tell "✅ Memory usage within safe limits"
tell ""

# 4. Recursion Depth Protection
tell "4. Recursion Protection:"
tell "Ellex prevents functions from calling themselves too deeply..."

make simple_function do
    tell "This is a simple, safe function"
    tell "No dangerous recursion here!"
end

simple_function

# This would be caught if it tried to recurse infinitely:
# make dangerous_recursion do
#     dangerous_recursion  # This would be stopped
# end

tell "✅ Function executed safely"
tell ""

# 5. Input Validation
tell "5. Input Safety:"
tell "Ellex helps validate and sanitize user input..."

ask "Enter a safe number (1-100):" → safe_number
when safe_number is "42" do
    tell "Great choice! 42 is the answer to everything."
end

tell "✅ Input processed safely"
tell ""

# 6. Turtle Graphics Safety
tell "6. Turtle Graphics Safety:"
tell "Drawing commands are bounded to prevent infinite movement..."

pen_down
repeat 4 times do
    forward
    right
end
pen_up

tell "✅ Turtle drawing completed within canvas bounds"
tell ""

# 7. Resource Cleanup
tell "7. Automatic Resource Management:"
tell "Ellex automatically cleans up resources..."

make cleanup_demo do
    tell "Creating temporary resources..."
    tell "Resources automatically cleaned up when function ends"
end

cleanup_demo
tell "✅ Resources cleaned up automatically"
tell ""

# 8. Error Recovery
tell "8. Graceful Error Handling:"
tell "Ellex provides helpful error messages instead of crashes..."

# This demonstrates how undefined variables are handled gracefully
tell "Trying to use undefined variable: {undefined_var}"
tell "(Notice how Ellex handles this gracefully!)"
tell ""

# 9. Concurrent Safety
tell "9. Concurrent Execution Safety:"
tell "Ellex ensures safe execution even with multiple operations..."

repeat 3 times do
    tell "Operation 1: Drawing"
    forward
    tell "Operation 2: Calculating" 
    tell "Operation 3: Displaying"
end

tell "✅ All operations completed safely"
tell ""

# 10. System Protection
tell "10. System Protection:"
tell "Ellex prevents access to dangerous system operations..."

tell "✅ Only safe, kid-friendly operations allowed"
tell "✅ No file system access without permission"
tell "✅ No network operations without supervision"
tell "✅ No system command execution"
tell ""

# Safety Summary
tell "🎯 Safety Features Summary:"
tell ""
tell "Time Limits:"
tell "  • Maximum execution time: 5 seconds (configurable)"
tell "  • Prevents programs from running forever"
tell ""
tell "Memory Limits:"
tell "  • Maximum memory usage: 64MB (configurable)"
tell "  • Prevents system resource exhaustion"
tell ""
tell "Loop Protection:"
tell "  • Maximum loop iterations: 10,000 (configurable)"
tell "  • Prevents infinite loops"
tell ""
tell "Recursion Protection:"
tell "  • Maximum recursion depth: 100 levels (configurable)"
tell "  • Prevents stack overflow"
tell ""
tell "Input Safety:"
tell "  • Automatic input validation"
tell "  • Safe string handling"
tell "  • Type checking"
tell ""
tell "Graphics Safety:"
tell "  • Bounded drawing area"
tell "  • Safe color values"
tell "  • Automatic resource cleanup"
tell ""
tell "Error Handling:"
tell "  • Friendly error messages with emojis"
tell "  • Helpful suggestions for fixes"
tell "  • Graceful degradation"
tell ""

# Configuration Examples
tell "🔧 Safety Configuration Examples:"
tell ""
tell "For Beginners (Ages 6-10):"
tell "  • Timeout: 3 seconds"
tell "  • Memory: 32MB"
tell "  • Loops: 1,000 iterations max"
tell "  • Recursion: 50 levels max"
tell ""
tell "For Intermediate (Ages 11-14):"
tell "  • Timeout: 5 seconds (default)"
tell "  • Memory: 64MB (default)"
tell "  • Loops: 10,000 iterations max (default)"
tell "  • Recursion: 100 levels max (default)"
tell ""
tell "For Advanced (Ages 15+):"
tell "  • Timeout: 30 seconds"
tell "  • Memory: 256MB"
tell "  • Loops: 100,000 iterations max"
tell "  • Recursion: 500 levels max"
tell ""

tell "🌟 Safety Demo Complete!"
tell "Ellex keeps young programmers safe while they learn and explore!"
tell "All safety features work automatically in the background."
tell ""
tell "Remember: Safety first, fun second, learning always! 🚀"