defmodule Ellex.SafetyMonitor do
  @moduledoc """
  Safety monitoring for Ellex code execution.
  
  Monitors:
  - Memory usage
  - Execution time
  - Infinite loops
  - Resource consumption
  """
  
  use GenServer
  
  require Logger
  
  defstruct [
    :max_memory,
    :max_execution_time,
    :active_executions,
    :warnings_issued
  ]
  
  @max_memory 64 * 1024 * 1024  # 64MB
  @max_execution_time 5_000      # 5 seconds
  @check_interval 100            # Check every 100ms
  
  def start_link(opts) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end
  
  @impl true
  def init(_opts) do
    state = %__MODULE__{
      max_memory: @max_memory,
      max_execution_time: @max_execution_time,
      active_executions: %{},
      warnings_issued: 0
    }
    
    # Start periodic safety checks
    :timer.send_interval(@check_interval, :safety_check)
    
    {:ok, state}
  end
  
  @doc """
  Start monitoring an execution.
  """
  def start_execution(pid) when is_pid(pid) do
    GenServer.call(__MODULE__, {:start_execution, pid})
  end
  
  @doc """
  End monitoring an execution.
  """
  def end_execution(pid) when is_pid(pid) do
    GenServer.call(__MODULE__, {:end_execution, pid})
  end
  
  @doc """
  Check if execution is safe to continue.
  """
  def check_safety(pid) when is_pid(pid) do
    GenServer.call(__MODULE__, {:check_safety, pid})
  end
  
  @doc """
  Get current safety statistics.
  """
  def get_stats do
    GenServer.call(__MODULE__, :get_stats)
  end
  
  @impl true
  def handle_call({:start_execution, pid}, _from, state) do
    execution_info = %{
      pid: pid,
      start_time: System.monotonic_time(:millisecond),
      initial_memory: get_process_memory(pid),
      warnings: 0
    }
    
    new_executions = Map.put(state.active_executions, pid, execution_info)
    new_state = %{state | active_executions: new_executions}
    
    Logger.debug("[SafetyMonitor] Started monitoring execution for #{inspect(pid)}")
    
    {:reply, :ok, new_state}
  end
  
  @impl true
  def handle_call({:end_execution, pid}, _from, state) do
    case Map.get(state.active_executions, pid) do
      nil ->
        {:reply, :not_found, state}
      
      exec_info ->
        duration = System.monotonic_time(:millisecond) - exec_info.start_time
        final_memory = get_process_memory(pid)
        
        Logger.debug("[SafetyMonitor] Execution completed for #{inspect(pid)} in #{duration}ms, memory: #{final_memory} bytes")
        
        new_executions = Map.delete(state.active_executions, pid)
        new_state = %{state | active_executions: new_executions}
        
        {:reply, :ok, new_state}
    end
  end
  
  @impl true
  def handle_call({:check_safety, pid}, _from, state) do
    case Map.get(state.active_executions, pid) do
      nil ->
        {:reply, {:error, :not_monitored}, state}
      
      exec_info ->
        current_time = System.monotonic_time(:millisecond)
        duration = current_time - exec_info.start_time
        current_memory = get_process_memory(pid)
        
        safety_result = check_execution_safety(exec_info, duration, current_memory, state)
        
        {:reply, safety_result, state}
    end
  end
  
  @impl true
  def handle_call(:get_stats, _from, state) do
    stats = %{
      active_executions: map_size(state.active_executions),
      warnings_issued: state.warnings_issued,
      max_memory: state.max_memory,
      max_execution_time: state.max_execution_time
    }
    
    {:reply, stats, state}
  end
  
  @impl true
  def handle_info(:safety_check, state) do
    new_state = perform_safety_checks(state)
    {:noreply, new_state}
  end
  
  @impl true
  def handle_info(_msg, state) do
    {:noreply, state}
  end
  
  # Perform periodic safety checks on all active executions
  defp perform_safety_checks(state) do
    current_time = System.monotonic_time(:millisecond)
    
    {violations, new_warnings} = 
      Enum.reduce(state.active_executions, {[], 0}, fn {pid, exec_info}, {violations, warnings} ->
        duration = current_time - exec_info.start_time
        current_memory = get_process_memory(pid)
        
        case check_execution_safety(exec_info, duration, current_memory, state) do
          :ok ->
            {violations, warnings}
          
          {:warning, reason} ->
            Logger.warning("[SafetyMonitor] Safety warning for #{inspect(pid)}: #{reason}")
            {violations, warnings + 1}
          
          {:violation, reason} ->
            Logger.error("[SafetyMonitor] Safety violation for #{inspect(pid)}: #{reason}")
            # Terminate the violating process
            Process.exit(pid, {:safety_violation, reason})
            {[{pid, reason} | violations], warnings + 1}
        end
      end)
    
    # Remove terminated processes from monitoring
    new_executions = 
      Enum.reduce(violations, state.active_executions, fn {pid, _reason}, acc ->
        Map.delete(acc, pid)
      end)
    
    %{state | 
      active_executions: new_executions,
      warnings_issued: state.warnings_issued + new_warnings
    }
  end
  
  # Check if a specific execution is within safety limits
  defp check_execution_safety(exec_info, duration, current_memory, state) do
    cond do
      # Hard limit: execution time exceeded
      duration > state.max_execution_time ->
        {:violation, "Execution time exceeded #{state.max_execution_time}ms (took #{duration}ms)"}
      
      # Hard limit: memory exceeded
      current_memory > state.max_memory ->
        {:violation, "Memory usage exceeded #{state.max_memory} bytes (using #{current_memory} bytes)"}
      
      # Warning: approaching time limit
      duration > state.max_execution_time * 0.8 ->
        {:warning, "Execution time approaching limit (#{duration}ms of #{state.max_execution_time}ms)"}
      
      # Warning: approaching memory limit
      current_memory > state.max_memory * 0.8 ->
        {:warning, "Memory usage approaching limit (#{current_memory} of #{state.max_memory} bytes)"}
      
      # Warning: too many warnings for this execution
      exec_info.warnings > 5 ->
        {:violation, "Too many safety warnings issued for this execution"}
      
      true ->
        :ok
    end
  end
  
  # Get memory usage for a process (simplified)
  defp get_process_memory(pid) do
    try do
      case Process.info(pid, :memory) do
        {:memory, memory} -> memory
        nil -> 0  # Process might have died
      end
    rescue
      _ -> 0
    end
  end
  
  @doc """
  Create kid-friendly safety error messages.
  """
  def format_safety_error({:safety_violation, reason}) do
    case reason do
      "Execution time exceeded" <> _ ->
        %{
          type: "Your code is taking too long! ⏰",
          message: "Code should finish quickly so we can see the results!",
          emoji: "⏰",
          suggestion: "Try making your code simpler, or avoid repeating things too many times!"
        }
      
      "Memory usage exceeded" <> _ ->
        %{
          type: "Your code is using too much memory! 💾",
          message: "Let's keep things simple so everything runs smoothly!",
          emoji: "💾", 
          suggestion: "Try working with smaller numbers or shorter lists!"
        }
      
      reason ->
        %{
          type: "Safety check failed 🛡️",
          message: "We stopped your code to keep things safe: #{reason}",
          emoji: "🛡️",
          suggestion: "Try a simpler version of your code!"
        }
    end
  end
  
  def format_safety_error(error) do
    %{
      type: "Unknown safety error 🤷",
      message: inspect(error),
      emoji: "🤷",
      suggestion: "Something unexpected happened - try again!"
    }
  end
end