defmodule Ellex.NIF do
  @moduledoc """
  Native Implemented Functions (NIFs) for interfacing with Rust components.
  
  This module provides the bridge between Elixir and Rust, allowing:
  - Safe execution of Ellex natural language code
  - Turtle graphics state management
  - Performance monitoring and safety checks
  """
  
  # use Rustler, otp_app: :ellex, crate: "ellex_nif" # Disabled for testing
  
  # Fallback implementations for when NIFs aren't loaded
  
  @doc """
  Execute Ellex code using the Rust runtime.
  Returns {:ok, result} or {:error, reason}.
  """
  def execute_ellex_code(_code, _context), do: :erlang.nif_error(:nif_not_loaded)
  
  @doc """
  Parse natural language Ellex code to AST.
  """
  def parse_natural_language(_code), do: :erlang.nif_error(:nif_not_loaded)
  
  @doc """
  Validate Ellex AST for safety.
  """
  def validate_ast(_ast), do: :erlang.nif_error(:nif_not_loaded)
  
  @doc """
  Get turtle graphics state.
  """
  def get_turtle_state(), do: :erlang.nif_error(:nif_not_loaded)
  
  @doc """
  Update turtle graphics state.
  """
  def update_turtle_state(_commands), do: :erlang.nif_error(:nif_not_loaded)
  
  @doc """
  Get performance statistics from Rust runtime.
  """
  def get_performance_stats(), do: :erlang.nif_error(:nif_not_loaded)
  
  @doc """
  Clear all caches in the Rust runtime.
  """
  def clear_caches(), do: :erlang.nif_error(:nif_not_loaded)
  
  @doc """
  Check if the Rust runtime is healthy.
  """
  def health_check(), do: :erlang.nif_error(:nif_not_loaded)
  
  @doc """
  Enable or disable debug mode in the Rust runtime.
  """
  def set_debug_mode(_enabled), do: :erlang.nif_error(:nif_not_loaded)
  
  @doc """
  Get available natural language commands.
  """
  def get_available_commands(), do: :erlang.nif_error(:nif_not_loaded)
  
  @doc """
  Benchmark execution performance.
  """
  def benchmark_execution(_code, _iterations), do: :erlang.nif_error(:nif_not_loaded)
end