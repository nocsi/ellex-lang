use anyhow::Result;
use tokio::time::{Duration, Instant};

// Tracks execution safety
#[derive(Debug)]
pub struct SafetyMonitor {
    limits: ExecutionLimits,
    start_time: Instant,
    instruction_count: u64,
}

#[derive(Debug, Clone)]
pub struct ExecutionLimits {
    max_execution_time: Duration, // Max runtime per script
    max_instructions: u64,        // Max instructions to prevent loops
}

impl ExecutionLimits {
    pub fn new() -> Self {
        ExecutionLimits {
            max_execution_time: Duration::from_secs(5), // 5s for beginners
            max_instructions: 1000,                     // Prevent infinite loops
        }
    }

    pub fn with_timeout_and_instructions(timeout: Duration, max_instructions: u64) -> Self {
        ExecutionLimits {
            max_execution_time: timeout,
            max_instructions,
        }
    }
}

impl SafetyMonitor {
    pub fn new(limits: ExecutionLimits) -> Self {
        SafetyMonitor {
            limits,
            start_time: Instant::now(),
            instruction_count: 0,
        }
    }

    // Increment instruction count and check limits
    pub fn execute_step(&mut self) -> Result<()> {
        self.instruction_count += 1;
        if self.instruction_count > self.limits.max_instructions {
            anyhow::bail!(
                "Execution halted: exceeded {} instructions",
                self.limits.max_instructions
            );
        }
        if self.start_time.elapsed() > self.limits.max_execution_time {
            anyhow::bail!(
                "Execution halted: exceeded {} seconds",
                self.limits.max_execution_time.as_secs()
            );
        }
        Ok(())
    }

    // Reset for new execution
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.instruction_count = 0;
    }
}
