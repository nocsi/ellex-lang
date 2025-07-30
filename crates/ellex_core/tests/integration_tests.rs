use ellex_core::{
    EllexConfig, EllexRuntime, EllexValue, Statement, EllexError,
    ExecutionLimits, SafetyMonitor,
};
use ellex_core::safety::SafetyError;

#[test]
fn test_basic_configuration_creation() {
    let config = EllexConfig::default();
    assert_eq!(config.execution_timeout_ms, 5000);
    assert_eq!(config.memory_limit_mb, 64);
    assert!(config.enable_turtle);
    assert!(config.enable_ai);
    assert_eq!(config.max_recursion_depth, 100);
    assert_eq!(config.max_loop_iterations, 10000);
}

#[test]
fn test_custom_configuration() {
    let config = EllexConfig {
        execution_timeout_ms: 10000,
        memory_limit_mb: 128,
        enable_turtle: false,
        enable_ai: false,
        max_recursion_depth: 50,
        max_loop_iterations: 5000,
    };
    
    let runtime = EllexRuntime::with_config(config.clone());
    assert_eq!(runtime.get_config().execution_timeout_ms, 10000);
    assert_eq!(runtime.get_config().memory_limit_mb, 128);
    assert!(!runtime.get_config().enable_turtle);
    assert!(!runtime.get_config().enable_ai);
}

#[test]
fn test_runtime_with_turtle_disabled() {
    let config = EllexConfig {
        enable_turtle: false,
        ..EllexConfig::default()
    };
    
    let runtime = EllexRuntime::with_config(config);
    assert!(runtime.get_turtle().is_none());
}

#[test]
fn test_runtime_with_turtle_enabled() {
    let config = EllexConfig {
        enable_turtle: true,
        ..EllexConfig::default()
    };
    
    let runtime = EllexRuntime::with_config(config);
    assert!(runtime.get_turtle().is_some());
}

#[test]
fn test_configuration_update() {
    let mut runtime = EllexRuntime::new();
    
    let new_config = EllexConfig {
        execution_timeout_ms: 15000,
        memory_limit_mb: 256,
        enable_turtle: false,
        enable_ai: true,
        max_recursion_depth: 200,
        max_loop_iterations: 20000,
    };
    
    runtime.update_config(new_config.clone());
    
    assert_eq!(runtime.get_config().execution_timeout_ms, 15000);
    assert_eq!(runtime.get_config().memory_limit_mb, 256);
    assert!(!runtime.get_config().enable_turtle);
    assert!(runtime.get_turtle().is_none());
}

#[test]
fn test_assignment_syntax() {
    let mut runtime = EllexRuntime::new();
    
    // Test basic assignment
    let assignment_stmt = Statement::Assignment(
        "name".to_string(),
        EllexValue::String("Alice".to_string())
    );
    
    let result = runtime.execute(vec![assignment_stmt]);
    assert!(result.is_ok());
    
    // Check that variable was set
    let variables = runtime.get_variables();
    match variables.get("name") {
        Some(EllexValue::String(s)) => assert_eq!(s, "Alice"),
        _ => panic!("Expected string variable to be set"),
    }
}

#[test]
fn test_assignment_with_numbers() {
    let mut runtime = EllexRuntime::new();
    
    let assignment_stmt = Statement::Assignment(
        "age".to_string(),
        EllexValue::Number(25.0)
    );
    
    let result = runtime.execute(vec![assignment_stmt]);
    assert!(result.is_ok());
    
    let variables = runtime.get_variables();
    match variables.get("age") {
        Some(EllexValue::Number(n)) => assert_eq!(*n, 25.0),
        _ => panic!("Expected number variable to be set"),
    }
}

#[test]
fn test_multiple_assignments() {
    let mut runtime = EllexRuntime::new();
    
    let statements = vec![
        Statement::Assignment("name".to_string(), EllexValue::String("Bob".to_string())),
        Statement::Assignment("age".to_string(), EllexValue::Number(30.0)),
        Statement::Assignment("active".to_string(), EllexValue::String("true".to_string())),
    ];
    
    let result = runtime.execute(statements);
    assert!(result.is_ok());
    
    let variables = runtime.get_variables();
    assert_eq!(variables.len(), 3);
    
    match variables.get("name") {
        Some(EllexValue::String(s)) => assert_eq!(s, "Bob"),
        _ => panic!("Expected name variable"),
    }
    
    match variables.get("age") {
        Some(EllexValue::Number(n)) => assert_eq!(*n, 30.0),
        _ => panic!("Expected age variable"),
    }
}

#[test]
fn test_assignment_with_interpolation() {
    let mut runtime = EllexRuntime::new();
    
    // First set a variable
    runtime.set_variable("base_name".to_string(), EllexValue::String("Alice".to_string()));
    
    // Then assign using interpolation
    let assignment_stmt = Statement::Assignment(
        "greeting".to_string(),
        EllexValue::String("Hello, {base_name}!".to_string())
    );
    
    let result = runtime.execute(vec![assignment_stmt]);
    assert!(result.is_ok());
    
    let variables = runtime.get_variables();
    match variables.get("greeting") {
        Some(EllexValue::String(s)) => assert_eq!(s, "Hello, Alice!"),
        _ => panic!("Expected interpolated greeting"),
    }
}

#[test]
fn test_assignment_overwrite() {
    let mut runtime = EllexRuntime::new();
    
    // First assignment
    let first_assignment = Statement::Assignment(
        "counter".to_string(),
        EllexValue::Number(1.0)
    );
    
    // Second assignment to same variable
    let second_assignment = Statement::Assignment(
        "counter".to_string(),
        EllexValue::Number(2.0)
    );
    
    let result = runtime.execute(vec![first_assignment, second_assignment]);
    assert!(result.is_ok());
    
    let variables = runtime.get_variables();
    match variables.get("counter") {
        Some(EllexValue::Number(n)) => assert_eq!(*n, 2.0),
        _ => panic!("Expected counter to be updated"),
    }
}

#[test]
fn test_tell_statement_execution() {
    let mut runtime = EllexRuntime::new();
    
    let statements = vec![
        Statement::Tell(EllexValue::String("Hello, world!".to_string())),
        Statement::Tell(EllexValue::Number(42.0)),
    ];
    
    let result = runtime.execute(statements);
    assert!(result.is_ok());
    
    // The last statement result should be returned
    match result.unwrap() {
        EllexValue::Number(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected number result"),
    }
}

#[test]
fn test_variable_assignment_and_retrieval() {
    let mut runtime = EllexRuntime::new();
    
    runtime.set_variable("name".to_string(), EllexValue::String("Alice".to_string()));
    runtime.set_variable("age".to_string(), EllexValue::Number(25.0));
    
    let variables = runtime.get_variables();
    assert_eq!(variables.len(), 2);
    
    match variables.get("name") {
        Some(EllexValue::String(s)) => assert_eq!(s, "Alice"),
        _ => panic!("Expected string variable"),
    }
    
    match variables.get("age") {
        Some(EllexValue::Number(n)) => assert_eq!(*n, 25.0),
        _ => panic!("Expected number variable"),
    }
}

#[test]
fn test_repeat_statement_execution() {
    let mut runtime = EllexRuntime::new();
    
    let repeat_stmt = Statement::Repeat(
        3,
        vec![Statement::Tell(EllexValue::String("Loop iteration".to_string()))],
    );
    
    let result = runtime.execute(vec![repeat_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_when_statement_execution() {
    let mut runtime = EllexRuntime::new();
    
    // Set up a variable for the condition
    runtime.set_variable("status".to_string(), EllexValue::String("ready".to_string()));
    
    let when_stmt = Statement::When(
        "status".to_string(),
        EllexValue::String("ready".to_string()),
        vec![Statement::Tell(EllexValue::String("System is ready!".to_string()))],
        Some(vec![Statement::Tell(EllexValue::String("System not ready".to_string()))]),
    );
    
    let result = runtime.execute(vec![when_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_turtle_commands() {
    let mut runtime = EllexRuntime::new();
    
    let turtle_commands = vec![
        Statement::Call("forward".to_string()),
        Statement::Call("right".to_string()),
        Statement::Call("pen_up".to_string()),
        Statement::Call("pen_down".to_string()),
    ];
    
    let result = runtime.execute(turtle_commands);
    assert!(result.is_ok());
    
    // Check turtle state
    if let Some(turtle) = runtime.get_turtle() {
        assert!(turtle.is_pen_down());
    }
}

#[test]
fn test_safety_monitor_basic_limits() {
    let limits = ExecutionLimits::new();
    let mut monitor = SafetyMonitor::new(limits);
    
    // Should start successfully
    assert!(monitor.check_execution_start().is_ok());
    
    // Should continue for reasonable number of steps
    for _ in 0..1000 {
        assert!(monitor.check_execution_continue().is_ok());
    }
    
    let stats = monitor.get_stats();
    assert_eq!(stats.instruction_count, 1000);
    assert!(stats.elapsed_ms < 1000); // Should be very fast
}

#[test]
fn test_safety_monitor_instruction_limit() {
    let limits = ExecutionLimits::new()
        .with_timeout(60000); // Long timeout to test instructions only
    let mut monitor = SafetyMonitor::new(limits);
    
    monitor.check_execution_start().unwrap();
    
    // Should eventually hit instruction limit
    let mut hit_limit = false;
    for _ in 0..200000 {
        match monitor.check_execution_continue() {
            Ok(()) => continue,
            Err(SafetyError::InstructionLimitExceeded { .. }) => {
                hit_limit = true;
                break;
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
    assert!(hit_limit, "Should have hit instruction limit");
}

#[test]
fn test_safety_monitor_recursion_limit() {
    let limits = ExecutionLimits::new();
    let mut monitor = SafetyMonitor::new(limits);
    
    // Should allow recursion up to limit
    for _ in 0..100 {
        assert!(monitor.check_recursion_depth().is_ok());
    }
    
    // Should fail on exceeding limit
    assert!(matches!(
        monitor.check_recursion_depth(),
        Err(SafetyError::RecursionDepthExceeded { .. })
    ));
}

#[test]
fn test_safety_monitor_loop_limit() {
    let limits = ExecutionLimits::new();
    let monitor = SafetyMonitor::new(limits);
    
    // Should reject very large loops
    assert!(matches!(
        monitor.check_loop_start(20000),
        Err(SafetyError::LoopLimitExceeded { .. })
    ));
    
    // Should accept reasonable loops
    assert!(monitor.check_loop_start(100).is_ok());
}

#[test]
fn test_safety_monitor_memory_limit() {
    let limits = ExecutionLimits::new();
    let mut monitor = SafetyMonitor::new(limits);
    
    // Set memory usage over limit
    monitor.update_memory_usage(100); // Over 64MB limit
    
    assert!(matches!(
        monitor.check_execution_continue(),
        Err(SafetyError::MemoryLimitExceeded { .. })
    ));
}

#[test]
fn test_beginner_vs_advanced_limits() {
    let beginner = ExecutionLimits::for_beginners();
    let advanced = ExecutionLimits::for_advanced();
    
    assert!(beginner.timeout_ms < advanced.timeout_ms);
    assert!(beginner.memory_limit_mb < advanced.memory_limit_mb);
    assert!(beginner.max_recursion_depth < advanced.max_recursion_depth);
    assert!(beginner.max_loop_iterations < advanced.max_loop_iterations);
    assert!(beginner.max_instructions < advanced.max_instructions);
}

#[test]
fn test_runtime_with_beginner_limits() {
    let config = EllexConfig {
        execution_timeout_ms: 3000,
        memory_limit_mb: 32,
        max_recursion_depth: 50,
        max_loop_iterations: 1000,
        ..EllexConfig::default()
    };
    
    let mut runtime = EllexRuntime::with_config(config);
    
    // Should handle basic operations
    let statements = vec![
        Statement::Tell(EllexValue::String("Hello!".to_string())),
        Statement::Repeat(5, vec![
            Statement::Tell(EllexValue::String("Count".to_string()))
        ]),
    ];
    
    let result = runtime.execute(statements);
    assert!(result.is_ok());
}

#[test]
fn test_runtime_with_strict_loop_limit() {
    let config = EllexConfig {
        max_loop_iterations: 5,
        ..EllexConfig::default()
    };
    
    let mut runtime = EllexRuntime::with_config(config);
    
    // Should fail with loop that's too large
    let large_loop = Statement::Repeat(10, vec![
        Statement::Tell(EllexValue::String("This should fail".to_string()))
    ]);
    
    let result = runtime.execute(vec![large_loop]);
    assert!(result.is_err());
}

#[test]
fn test_natural_language_execution() {
    let mut runtime = EllexRuntime::new();
    
    // Test basic tell command
    let result = runtime.execute_natural_language("tell \"Hello, world!\"");
    assert!(result.is_ok());
    
    // Test ask command
    let result = runtime.execute_natural_language("ask \"What's your name?\"");
    assert!(result.is_ok());
    
    // Test turtle commands
    let result = runtime.execute_natural_language("move forward");
    assert!(result.is_ok());
    
    let result = runtime.execute_natural_language("turn right");
    assert!(result.is_ok());
}

#[test]
fn test_runtime_reset() {
    let mut runtime = EllexRuntime::new();
    
    // Set up some state
    runtime.set_variable("test".to_string(), EllexValue::String("value".to_string()));
    
    // Execute some statements
    let statements = vec![
        Statement::Tell(EllexValue::String("Before reset".to_string())),
    ];
    runtime.execute(statements).unwrap();
    
    // Reset the runtime
    runtime.reset();
    
    // State should be cleared
    assert!(runtime.get_variables().is_empty());
}

#[test]
fn test_error_handling_and_friendly_messages() {
    use ellex_core::friendly_error_message;
    
    let timeout_error = EllexError::Timeout { limit_ms: 5000 };
    let friendly = friendly_error_message(&timeout_error);
    assert!(friendly.contains("long time"));
    assert!(friendly.contains("🐌"));
    
    let parse_error = EllexError::ParseError {
        line: 1,
        column: 5,
        message: "Unexpected token".to_string(),
    };
    let friendly = friendly_error_message(&parse_error);
    assert!(friendly.contains("didn't understand"));
    assert!(friendly.contains("🤔"));
    
    let unknown_error = EllexError::UnknownCommand {
        input: "foobar".to_string(),
        suggestion: "Try 'tell' instead".to_string(),
    };
    let friendly = friendly_error_message(&unknown_error);
    assert!(friendly.contains("don't know"));
    assert!(friendly.contains("💡"));
}

#[test]
fn test_safety_warning_thresholds() {
    let limits = ExecutionLimits::new();
    let mut monitor = SafetyMonitor::new(limits);
    
    // Set memory to warning threshold (80% of 64MB = ~51MB)
    monitor.update_memory_usage(52);
    
    let warnings = monitor.check_warning_thresholds();
    assert!(!warnings.is_empty());
    
    let friendly_message = warnings[0].friendly_message();
    assert!(friendly_message.contains("💾"));
    assert!(friendly_message.contains("memory"));
}

#[test]
fn test_execution_with_variable_interpolation() {
    let mut runtime = EllexRuntime::new();
    
    // Set a variable
    runtime.set_variable("name".to_string(), EllexValue::String("Alice".to_string()));
    
    // Use variable in tell statement (basic interpolation)
    let tell_stmt = Statement::Tell(EllexValue::String("{name}".to_string()));
    let result = runtime.execute(vec![tell_stmt]);
    
    assert!(result.is_ok());
    match result.unwrap() {
        EllexValue::String(s) => assert_eq!(s, "Alice"),
        _ => panic!("Expected interpolated string"),
    }
}

#[test]
fn test_function_definition_and_calling() {
    use ellex_core::EllexFunction;
    
    let mut runtime = EllexRuntime::new();
    
    // Define a simple function
    let function = EllexFunction {
        name: "greet".to_string(),
        params: vec![],
        body: vec![
            Statement::Tell(EllexValue::String("Hello from function!".to_string())),
        ],
    };
    
    runtime.define_function("greet".to_string(), function);
    
    // Call the function
    let call_stmt = Statement::Call("greet".to_string());
    let result = runtime.execute(vec![call_stmt]);
    
    assert!(result.is_ok());
}

#[test]
fn test_comprehensive_turtle_integration() {
    let mut runtime = EllexRuntime::new();
    
    let turtle_program = vec![
        Statement::Call("pen_down".to_string()),
        Statement::Call("forward".to_string()),
        Statement::Call("right".to_string()),
        Statement::Call("forward".to_string()),
        Statement::Call("right".to_string()),
        Statement::Call("forward".to_string()),
        Statement::Call("right".to_string()),
        Statement::Call("forward".to_string()),
        Statement::Call("pen_up".to_string()),
    ];
    
    let result = runtime.execute(turtle_program);
    assert!(result.is_ok());
    
    // Check that turtle has recorded lines
    if let Some(turtle) = runtime.get_turtle() {
        assert!(!turtle.is_pen_down()); // Should end with pen up
        assert!(!turtle.get_lines().is_empty()); // Should have drawn something
    }
}

#[test]
fn test_complex_program_execution() {
    let mut runtime = EllexRuntime::new();
    
    let complex_program = vec![
        Statement::Tell(EllexValue::String("Starting complex program".to_string())),
        Statement::Repeat(3, vec![
            Statement::Tell(EllexValue::String("Loop iteration".to_string())),
            Statement::Call("forward".to_string()),
            Statement::Call("right".to_string()),
        ]),
        Statement::Tell(EllexValue::String("Program complete".to_string())),
    ];
    
    let result = runtime.execute(complex_program);
    assert!(result.is_ok());
}

#[test]
fn test_performance_and_memory_tracking() {
    let mut runtime = EllexRuntime::new();
    
    // Create a program that should use some memory and time
    let large_program = vec![
        Statement::Repeat(100, vec![
            Statement::Tell(EllexValue::String("Performance test".to_string())),
        ]),
    ];
    
    let start_time = std::time::Instant::now();
    let result = runtime.execute(large_program);
    let duration = start_time.elapsed();
    
    assert!(result.is_ok());
    assert!(duration.as_millis() < 1000); // Should complete quickly
}

#[test]
fn test_configuration_serialization() {
    let config = EllexConfig {
        execution_timeout_ms: 10000,
        memory_limit_mb: 128,
        enable_turtle: false,
        enable_ai: true,
        max_recursion_depth: 200,
        max_loop_iterations: 20000,
    };
    
    // Test serialization
    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: EllexConfig = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(config.execution_timeout_ms, deserialized.execution_timeout_ms);
    assert_eq!(config.memory_limit_mb, deserialized.memory_limit_mb);
    assert_eq!(config.enable_turtle, deserialized.enable_turtle);
    assert_eq!(config.enable_ai, deserialized.enable_ai);
}