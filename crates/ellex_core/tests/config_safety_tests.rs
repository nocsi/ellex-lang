use ellex_core::{
    EllexConfig, EllexRuntime, EllexValue, Statement,
    ExecutionLimits, SafetyMonitor,
};
use ellex_core::safety::{SafetyError, SafetyStats, SafetyWarning};

// Configuration Tests
#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        let config = EllexConfig::default();
        assert_eq!(config.execution_timeout_ms, 5000);
        assert_eq!(config.memory_limit_mb, 64);
        assert_eq!(config.max_recursion_depth, 100);
        assert_eq!(config.max_loop_iterations, 10000);
        assert!(config.enable_turtle);
        assert!(config.enable_ai);
    }

    #[test]
    fn test_config_serialization_deserialization() {
        let config = EllexConfig {
            execution_timeout_ms: 10000,
            memory_limit_mb: 128,
            enable_turtle: false,
            enable_ai: true,
            max_recursion_depth: 200,
            max_loop_iterations: 20000,
        };

        // Test JSON serialization
        let json = serde_json::to_string(&config).expect("Failed to serialize config");
        let deserialized: EllexConfig = serde_json::from_str(&json).expect("Failed to deserialize config");

        assert_eq!(config.execution_timeout_ms, deserialized.execution_timeout_ms);
        assert_eq!(config.memory_limit_mb, deserialized.memory_limit_mb);
        assert_eq!(config.enable_turtle, deserialized.enable_turtle);
        assert_eq!(config.enable_ai, deserialized.enable_ai);
        assert_eq!(config.max_recursion_depth, deserialized.max_recursion_depth);
        assert_eq!(config.max_loop_iterations, deserialized.max_loop_iterations);
    }

    #[test]
    fn test_runtime_config_application() {
        let config = EllexConfig {
            execution_timeout_ms: 15000,
            memory_limit_mb: 256,
            enable_turtle: false,
            enable_ai: false,
            max_recursion_depth: 50,
            max_loop_iterations: 5000,
        };

        let runtime = EllexRuntime::with_config(config.clone());
        
        assert_eq!(runtime.get_config().execution_timeout_ms, 15000);
        assert_eq!(runtime.get_config().memory_limit_mb, 256);
        assert!(!runtime.get_config().enable_turtle);
        assert!(!runtime.get_config().enable_ai);
        assert_eq!(runtime.get_config().max_recursion_depth, 50);
        assert_eq!(runtime.get_config().max_loop_iterations, 5000);
    }

    #[test]
    fn test_runtime_config_update() {
        let mut runtime = EllexRuntime::new();
        
        let initial_config = runtime.get_config().clone();
        assert_eq!(initial_config.execution_timeout_ms, 5000);

        let new_config = EllexConfig {
            execution_timeout_ms: 20000,
            memory_limit_mb: 512,
            enable_turtle: false,
            enable_ai: true,
            max_recursion_depth: 300,
            max_loop_iterations: 50000,
        };

        runtime.update_config(new_config.clone());
        
        assert_eq!(runtime.get_config().execution_timeout_ms, 20000);
        assert_eq!(runtime.get_config().memory_limit_mb, 512);
        assert!(!runtime.get_config().enable_turtle);
        assert!(runtime.get_config().enable_ai);
    }

    #[test]
    fn test_turtle_graphics_config_toggle() {
        // Start with turtle enabled
        let config_with_turtle = EllexConfig {
            enable_turtle: true,
            ..EllexConfig::default()
        };
        let runtime = EllexRuntime::with_config(config_with_turtle);
        assert!(runtime.get_turtle().is_some());

        // Test with turtle disabled
        let config_no_turtle = EllexConfig {
            enable_turtle: false,
            ..EllexConfig::default()
        };
        let runtime = EllexRuntime::with_config(config_no_turtle);
        assert!(runtime.get_turtle().is_none());
    }
}

// Safety System Tests
#[cfg(test)]
mod safety_tests {
    use super::*;

    #[test]
    fn test_execution_limits_creation() {
        let limits = ExecutionLimits::new();
        assert_eq!(limits.timeout_ms, 5000);
        assert_eq!(limits.memory_limit_mb, 64);
        assert_eq!(limits.max_recursion_depth, 100);
        assert_eq!(limits.max_loop_iterations, 10000);
        assert_eq!(limits.max_instructions, 100000);
    }

    #[test]
    fn test_execution_limits_builder_pattern() {
        let limits = ExecutionLimits::new()
            .with_timeout(10000)
            .with_memory_limit(128)
            .with_recursion_limit(200)
            .with_loop_limit(20000);

        assert_eq!(limits.timeout_ms, 10000);
        assert_eq!(limits.memory_limit_mb, 128);
        assert_eq!(limits.max_recursion_depth, 200);
        assert_eq!(limits.max_loop_iterations, 20000);
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
    fn test_safety_monitor_basic_operations() {
        let limits = ExecutionLimits::new();
        let mut monitor = SafetyMonitor::new(limits);

        // Should start successfully
        assert!(monitor.check_execution_start().is_ok());

        // Should continue for reasonable operations
        for _ in 0..100 {
            assert!(monitor.check_execution_continue().is_ok());
        }

        let stats = monitor.get_stats();
        assert_eq!(stats.instruction_count, 100);
        assert!(stats.elapsed_ms < 1000);
    }

    #[test]
    fn test_safety_monitor_instruction_limit() {
        let limits = ExecutionLimits::new()
            .with_timeout(60000) // Long timeout to focus on instructions
            .with_loop_limit(200000); // High loop limit
        let mut monitor = SafetyMonitor::new(limits);

        monitor.check_execution_start().unwrap();

        // Should eventually hit instruction limit
        let mut instruction_limit_hit = false;
        for _ in 0..150000 {
            match monitor.check_execution_continue() {
                Ok(()) => continue,
                Err(SafetyError::InstructionLimitExceeded { .. }) => {
                    instruction_limit_hit = true;
                    break;
                }
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }
        assert!(instruction_limit_hit);
    }

    #[test]
    fn test_safety_monitor_recursion_tracking() {
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

        // Test recursion exit
        monitor.exit_recursion();
        let stats = monitor.get_stats();
        assert_eq!(stats.recursion_depth, 100); // Should be back to 100
    }

    #[test]
    fn test_safety_monitor_loop_checking() {
        let limits = ExecutionLimits::new();
        let monitor = SafetyMonitor::new(limits);

        // Should reject loops that are too large
        assert!(matches!(
            monitor.check_loop_start(20000),
            Err(SafetyError::LoopLimitExceeded { .. })
        ));

        // Should accept reasonable loops
        assert!(monitor.check_loop_start(100).is_ok());
        assert!(monitor.check_loop_start(5000).is_ok());
        assert!(monitor.check_loop_start(10000).is_ok());
    }

    #[test]
    fn test_safety_monitor_memory_tracking() {
        let limits = ExecutionLimits::new();
        let mut monitor = SafetyMonitor::new(limits);

        // Should start with zero memory usage
        let initial_stats = monitor.get_stats();
        assert_eq!(initial_stats.memory_usage_mb, 0);

        // Should accept reasonable memory usage
        monitor.update_memory_usage(32);
        assert!(monitor.check_execution_continue().is_ok());

        // Should reject excessive memory usage
        monitor.update_memory_usage(100); // Over 64MB limit
        assert!(matches!(
            monitor.check_execution_continue(),
            Err(SafetyError::MemoryLimitExceeded { .. })
        ));
    }

    #[test]
    fn test_safety_monitor_reset() {
        let limits = ExecutionLimits::new();
        let mut monitor = SafetyMonitor::new(limits);

        // Do some operations
        monitor.check_execution_start().unwrap();
        for _ in 0..50 {
            monitor.check_execution_continue().unwrap();
        }
        monitor.check_recursion_depth().unwrap();
        monitor.update_memory_usage(20);

        let stats_before = monitor.get_stats();
        assert!(stats_before.instruction_count > 0);
        assert!(stats_before.recursion_depth > 0);
        assert!(stats_before.memory_usage_mb > 0);

        // Reset should clear everything
        monitor.reset();

        let stats_after = monitor.get_stats();
        assert_eq!(stats_after.instruction_count, 0);
        assert_eq!(stats_after.recursion_depth, 0);
        assert_eq!(stats_after.memory_usage_mb, 0);
    }

    #[test]
    fn test_safety_warning_thresholds() {
        let limits = ExecutionLimits::new();
        let mut monitor = SafetyMonitor::new(limits);

        // Should have no warnings initially
        let warnings = monitor.check_warning_thresholds();
        assert!(warnings.is_empty());

        // Set memory to warning threshold (80% of 64MB = ~51MB)
        monitor.update_memory_usage(52);
        let warnings = monitor.check_warning_thresholds();
        assert!(!warnings.is_empty());
        assert!(matches!(warnings[0], SafetyWarning::NearMemoryLimit { .. }));

        // Test friendly warning messages
        let friendly_message = warnings[0].friendly_message();
        assert!(friendly_message.contains("💾"));
        assert!(friendly_message.contains("memory"));
    }

    #[test]
    fn test_safety_stats_calculations() {
        let limits = ExecutionLimits::new();
        let mut monitor = SafetyMonitor::new(limits);

        monitor.update_memory_usage(32); // 50% of 64MB
        for _ in 0..50 {
            monitor.check_recursion_depth().unwrap(); // 50% of 100 levels
        }

        let stats = monitor.get_stats();
        
        let memory_percentage = stats.memory_percentage();
        assert!((memory_percentage - 50.0).abs() < 1.0); // ~50%

        let recursion_percentage = stats.recursion_percentage();
        assert!((recursion_percentage - 50.0).abs() < 1.0); // ~50%

        // Should not have warnings at 50%
        assert!(!stats.has_warnings());
    }

    #[test]
    fn test_safety_error_types() {
        // Test timeout error
        let timeout_error = SafetyError::ExecutionTimeout { limit_ms: 5000 };
        assert!(timeout_error.to_string().contains("timeout"));
        assert!(timeout_error.to_string().contains("5000ms"));

        // Test memory error
        let memory_error = SafetyError::MemoryLimitExceeded {
            current_mb: 100,
            limit_mb: 64,
        };
        assert!(memory_error.to_string().contains("Memory limit"));
        assert!(memory_error.to_string().contains("100MB"));
        assert!(memory_error.to_string().contains("64MB"));

        // Test recursion error
        let recursion_error = SafetyError::RecursionDepthExceeded {
            current: 150,
            limit: 100,
        };
        assert!(recursion_error.to_string().contains("Recursion"));
        assert!(recursion_error.to_string().contains("150"));
        assert!(recursion_error.to_string().contains("100"));

        // Test loop error
        let loop_error = SafetyError::LoopLimitExceeded {
            current: 20000,
            limit: 10000,
        };
        assert!(loop_error.to_string().contains("Loop"));
        assert!(loop_error.to_string().contains("20000"));
        assert!(loop_error.to_string().contains("10000"));
    }
}

// Integration Tests for Config + Safety
#[cfg(test)]
mod config_safety_integration_tests {
    use super::*;

    #[test]
    fn test_config_affects_safety_limits() {
        let strict_config = EllexConfig {
            execution_timeout_ms: 1000,
            memory_limit_mb: 16,
            max_recursion_depth: 10,
            max_loop_iterations: 100,
            ..EllexConfig::default()
        };

        let mut runtime = EllexRuntime::with_config(strict_config);

        // Test that strict limits are enforced
        let large_loop = Statement::Repeat(200, vec![
            Statement::Tell(EllexValue::String("This should fail".to_string()))
        ]);

        let result = runtime.execute(vec![large_loop]);
        assert!(result.is_err()); // Should fail due to loop limit
    }

    #[test]
    fn test_runtime_safety_integration() {
        let mut runtime = EllexRuntime::new();

        // Test that safety limits are integrated into runtime execution
        let statements = vec![
            Statement::Tell(EllexValue::String("Starting safety test".to_string())),
            Statement::Repeat(50, vec![
                Statement::Tell(EllexValue::String("Safe iteration".to_string())),
            ]),
            Statement::Tell(EllexValue::String("Safety test complete".to_string())),
        ];

        let result = runtime.execute(statements);
        assert!(result.is_ok()); // Should succeed within safety limits
    }

    #[test]
    fn test_config_update_affects_running_safety() {
        let mut runtime = EllexRuntime::new();

        // Start with default config
        let initial_config = runtime.get_config().clone();
        assert_eq!(initial_config.max_loop_iterations, 10000);

        // Update to stricter limits
        let strict_config = EllexConfig {
            max_loop_iterations: 5,
            ..initial_config
        };
        runtime.update_config(strict_config);

        // Test that new limits are enforced
        let large_loop = Statement::Repeat(10, vec![
            Statement::Tell(EllexValue::String("Should fail".to_string()))
        ]);

        let result = runtime.execute(vec![large_loop]);
        assert!(result.is_err()); // Should fail with new strict limits
    }

    #[test]
    fn test_beginner_config_safety() {
        let beginner_config = EllexConfig {
            execution_timeout_ms: 3000,
            memory_limit_mb: 32,
            max_recursion_depth: 50,
            max_loop_iterations: 1000,
            enable_turtle: true,
            enable_ai: true,
        };

        let mut runtime = EllexRuntime::with_config(beginner_config);

        // Test beginner-appropriate program
        let beginner_program = vec![
            Statement::Tell(EllexValue::String("Hello, beginner programmer!".to_string())),
            Statement::Repeat(5, vec![
                Statement::Tell(EllexValue::String("Learning is fun!".to_string())),
                Statement::Call("forward".to_string()),
            ]),
        ];

        let result = runtime.execute(beginner_program);
        assert!(result.is_ok()); // Should work fine for beginners
    }

    #[test]
    fn test_advanced_config_safety() {
        let advanced_config = EllexConfig {
            execution_timeout_ms: 30000,
            memory_limit_mb: 256,
            max_recursion_depth: 500,
            max_loop_iterations: 100000,
            enable_turtle: true,
            enable_ai: true,
        };

        let mut runtime = EllexRuntime::with_config(advanced_config);

        // Test more complex program that would fail with beginner limits
        let advanced_program = vec![
            Statement::Tell(EllexValue::String("Advanced programming concepts".to_string())),
            Statement::Repeat(200, vec![
                Statement::Tell(EllexValue::String("Complex iteration".to_string())),
                Statement::Call("forward".to_string()),
                Statement::Call("right".to_string()),
            ]),
        ];

        let result = runtime.execute(advanced_program);
        assert!(result.is_ok()); // Should work with advanced limits
    }

    #[test]
    fn test_config_validation() {
        // Test that runtime handles edge case configurations gracefully
        let edge_config = EllexConfig {
            execution_timeout_ms: 1, // Very short timeout
            memory_limit_mb: 1,       // Very low memory
            max_recursion_depth: 1,   // Minimal recursion
            max_loop_iterations: 1,   // Single iteration only
            enable_turtle: false,
            enable_ai: false,
        };

        let runtime = EllexRuntime::with_config(edge_config);
        
        // Runtime should handle extreme configurations without panicking
        assert_eq!(runtime.get_config().execution_timeout_ms, 1);
        assert_eq!(runtime.get_config().memory_limit_mb, 1);
        assert_eq!(runtime.get_config().max_recursion_depth, 1);
        assert_eq!(runtime.get_config().max_loop_iterations, 1);
    }

    #[test]
    fn test_safety_monitoring_during_execution() {
        let mut runtime = EllexRuntime::new();

        // Create a program that exercises different safety checks
        let test_program = vec![
            Statement::Tell(EllexValue::String("Safety monitoring test".to_string())),
            Statement::Repeat(10, vec![
                Statement::Tell(EllexValue::String("Loop safety check".to_string())),
            ]),
            Statement::Call("forward".to_string()), // Turtle command
            Statement::Tell(EllexValue::String("Test complete".to_string())),
        ];

        let result = runtime.execute(test_program);
        assert!(result.is_ok());
        
        // Safety monitoring should have tracked the execution
        // (In a real implementation, we might expose safety stats)
    }
}

// Performance Tests for Config/Safety
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_safety_monitoring_overhead() {
        let limits = ExecutionLimits::new();
        let mut monitor = SafetyMonitor::new(limits);

        let start = Instant::now();
        monitor.check_execution_start().unwrap();
        
        // Perform many safety checks
        for _ in 0..10000 {
            monitor.check_execution_continue().unwrap();
        }

        let duration = start.elapsed();
        
        // Safety checks should be very fast (under 10ms for 10k checks)
        assert!(duration.as_millis() < 10);
    }

    #[test]
    fn test_config_update_performance() {
        let mut runtime = EllexRuntime::new();

        let start = Instant::now();
        
        // Update configuration multiple times
        for i in 0..100 {
            let config = EllexConfig {
                execution_timeout_ms: 5000 + i * 100,
                memory_limit_mb: 64 + i as usize,
                max_recursion_depth: 100 + i as usize,
                max_loop_iterations: 10000 + i as usize * 100,
                enable_turtle: i % 2 == 0,
                enable_ai: i % 3 == 0,
            };
            runtime.update_config(config);
        }

        let duration = start.elapsed();
        
        // Config updates should be fast (under 50ms for 100 updates)
        assert!(duration.as_millis() < 50);
    }

    #[test]
    fn test_runtime_execution_with_safety_overhead() {
        let mut runtime = EllexRuntime::new();

        let statements = vec![
            Statement::Tell(EllexValue::String("Performance test".to_string())),
            Statement::Repeat(100, vec![
                Statement::Tell(EllexValue::String("Fast execution".to_string())),
            ]),
        ];

        let start = Instant::now();
        let result = runtime.execute(statements);
        let duration = start.elapsed();

        assert!(result.is_ok());
        // Execution should be fast even with safety monitoring
        assert!(duration.as_millis() < 100);
    }
}