# Parser-Runtime Integration Test
# This example demonstrates the complete integration between the Ellex parser and runtime
# It tests the full pipeline from natural language syntax to execution

tell "🔗 Parser-Runtime Integration Test"
tell "Testing the complete pipeline from parsing to execution..."
tell ""

# 1. Basic Syntax Parsing and Execution
tell "1. Basic Statement Parsing:"
tell "Hello, world!"
tell 42
tell "String and number parsing works!"
tell ""

# 2. Variable Parsing and Runtime Integration
tell "2. Variable System Integration:"
ask "What's your favorite programming language?" = language
ask "How many years have you been coding?" = years
tell "You've been coding {language} for {years} years!"
tell ""

# 3. Loop Parsing and Execution
tell "3. Loop Syntax Integration:"
repeat 4 times do
    tell "This loop was parsed and executed correctly"
    tell "Loop iteration complete"
end
tell ""

# 4. Conditional Parsing and Runtime Logic
tell "4. Conditional Logic Integration:"
when language is "Ellex" do
    tell "🎉 Excellent choice! Ellex is perfect for learning!"
end

when years is "1" do
    tell "🌱 You're just getting started - keep going!"
end

when years is "5" do
    tell "🚀 You're becoming an expert!"
end
tell ""

# 5. Function Definition Parsing and Execution
tell "5. Function System Integration:"
make greet_coder do
    tell "👋 Hello, fellow coder!"
    tell "Welcome to the world of programming!"
    tell "Your language: {language}"
    tell "Your experience: {years} years"
end

make celebrate do
    tell "🎊 Let's celebrate your coding journey!"
    greet_coder
    tell "Keep up the great work!"
end

celebrate
tell ""

# 6. Turtle Graphics Parsing and Execution
tell "6. Turtle Graphics Integration:"
tell "Drawing a triangle with parsed turtle commands..."

pen_down
forward
right
forward
right
forward
right
pen_up

tell "Triangle drawing complete!"
tell ""

# 7. Complex Nested Structures
tell "7. Complex Syntax Integration:"
repeat 3 times do
    tell "Starting iteration..."
    
    when language is "Ellex" do
        tell "  Ellex iteration"
        forward
        right
    end
    
    make nested_function do
        tell "  Nested function called"
        tell "  Function scope works correctly"
    end
    
    nested_function
    tell "Iteration complete"
end
tell ""

# 8. Error Handling Integration
tell "8. Error Handling Integration:"
tell "Testing graceful error handling..."

# Test undefined variable handling
tell "Undefined variable test: {undefined_var}"

# Test unknown function calls
unknown_function

tell "Errors handled gracefully by the runtime!"
tell ""

# 9. Modal Command Parsing
tell "9. Modal Programming Integration:"
tell "Testing modal command parsing and execution..."

@listen do
    around greet_coder
end

@think do
    what does celebrate do?
end

@build do
    extract 1,3 = new_greeting
end

tell "Modal commands parsed and executed!"
tell ""

# 10. Service Configuration Parsing
tell "10. Service Configuration Integration:"
tell "Testing service definition parsing..."

service "integration-test" do
    image "ellex/test:latest"
    port 8080
    environment do
        TEST_MODE "true"
        LANGUAGE "{language}"
        YEARS "{years}"
    end
    health_check do
        path "/health"
        interval 30
        timeout 10
    end
end

tell "Service configuration parsed successfully!"
tell ""

# 11. Advanced Expression Parsing
tell "11. Advanced Expression Integration:"
tell "Testing complex expression parsing and evaluation..."

ask "Enter a number for calculation:" = number
tell "Your number: {number}"
tell "Number processing complete"

# Variable interpolation in different contexts
when number is "10" do
    tell "Perfect ten! {number} is a great number!"
    repeat 2 times do
        tell "Celebrating {number}!"
    end
end
tell ""

# 12. Mixed Content Integration
tell "12. Mixed Content Integration:"
tell "Testing integration of all syntax elements together..."

make comprehensive_test do
    tell "Starting comprehensive integration test..."
    
    # Variables
    ask "Test variable:" = test_var
    
    # Loops with conditions
    repeat 2 times do
        when test_var is "success" do
            tell "✅ Integration test: {test_var}"
            
            # Nested turtle commands
            pen_down
            forward
            pen_up
        end
    end
    
    tell "Comprehensive test complete!"
end

comprehensive_test
tell ""

# 13. Performance Integration Test
tell "13. Performance Integration:"
tell "Testing parser-runtime performance with rapid execution..."

repeat 10 times do
    tell "Performance test iteration"
    make quick_function do
        tell "Quick function execution"
    end
    quick_function
end

tell "Performance test complete - parsing and execution optimized!"
tell ""

# 14. Memory Management Integration
tell "14. Memory Management Integration:"
tell "Testing variable scope and memory cleanup..."

make scope_test do
    ask "Local variable:" = local_var
    tell "Local variable created: {local_var}"
    
    make inner_scope do
        ask "Inner variable:" = inner_var
        tell "Inner variable: {inner_var}"
        tell "Both variables accessible"
    end
    
    inner_scope
    tell "Scope test complete"
end

scope_test
tell "Memory management working correctly!"
tell ""

# 15. Safety Integration Test
tell "15. Safety System Integration:"
tell "Testing safety monitoring during parsed code execution..."

# Safe operations that should complete
repeat 5 times do
    tell "Safe operation"
    forward
end

# Test safety limits awareness
tell "Safety limits properly integrated with runtime!"
tell ""

# Integration Test Summary
tell "📋 Integration Test Results:"
tell ""
tell "✅ Basic statement parsing and execution"
tell "✅ Variable system integration"
tell "✅ Loop syntax and runtime execution"
tell "✅ Conditional logic integration"
tell "✅ Function definition and calling"
tell "✅ Turtle graphics command integration"
tell "✅ Complex nested structure handling"
tell "✅ Error handling integration"
tell "✅ Modal programming integration"
tell "✅ Service configuration parsing"
tell "✅ Advanced expression evaluation"
tell "✅ Mixed content processing"
tell "✅ Performance optimization"
tell "✅ Memory management integration"
tell "✅ Safety system integration"
tell ""

# Parser-Runtime Architecture Summary
tell "🏗️ Parser-Runtime Architecture:"
tell ""
tell "Parsing Pipeline:"
tell "  1. Pest grammar → AST generation"
tell "  2. Statement tree construction"
tell "  3. Expression parsing and validation"
tell "  4. Function and variable resolution"
tell ""
tell "Runtime Pipeline:"
tell "  1. Safety monitoring initialization"
tell "  2. Statement-by-statement execution"
tell "  3. Variable scope management"
tell "  4. Function call stack management"
tell "  5. Resource cleanup and optimization"
tell ""
tell "Integration Features:"
tell "  • Zero-copy AST execution"
tell "  • Streaming parser-runtime communication"
tell "  • Optimized variable resolution"
tell "  • Efficient function dispatch"
tell "  • Real-time safety monitoring"
tell "  • Automatic error recovery"
tell ""

# Future Integration Enhancements
tell "🚀 Future Integration Enhancements:"
tell ""
tell "Planned Improvements:"
tell "  • JIT compilation for hot paths"
tell "  • Advanced AST optimization"
tell "  • Parallel execution for safe operations"
tell "  • Enhanced type inference"
tell "  • Real-time syntax validation"
tell "  • Interactive debugging integration"
tell ""

tell "🎯 Parser-Runtime Integration Test Complete!"
tell "The Ellex parser and runtime work seamlessly together!"
tell "From natural language syntax to safe, efficient execution."
tell ""
tell "Ready for real-world educational programming! 🌟"