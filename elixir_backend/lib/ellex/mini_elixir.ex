defmodule Ellex.MiniElixir do
  @moduledoc """
  MiniElixir interpreter for safe execution of Ellex natural language code.
  
  Based on Sequin's MiniElixir implementation but adapted for:
  - Natural language syntax support
  - Kid-friendly error messages
  - Turtle graphics integration
  - Educational safety constraints
  """
  
  use GenServer
  
  alias Ellex.MiniElixir.Validator
  alias Ellex.MiniElixir.NaturalLanguageTransformer
  alias Ellex.SafetyMonitor
  
  require Logger

  @timeout 5_000  # 5 seconds for kid-friendly timeout
  @max_recursion_depth 50  # Lower for safety
  
  # Kid-friendly execution context
  defstruct [
    :code,
    :variables,
    :turtle_state,
    :output,
    :mode,  # :speak, :listen, :think, :build
    :safety_limits
  ]
  
  def start_link(opts) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end
  
  @impl true
  def init(_opts) do
    {:ok, %{functions: %{}}}
  end
  
  @doc """
  Execute natural language Ellex code safely.
  
  ## Examples
  
      iex> Ellex.MiniElixir.execute("tell \"Hello, world!\"")
      {:ok, "Hello, world!"}
      
      iex> Ellex.MiniElixir.execute("ask \"What's your name?\" → name")
      {:ok, %{output: "What's your name?", binding: {:name, nil}}}
  """
  def execute(code, context \\ %{}) when is_binary(code) do
    execution_context = %__MODULE__{
      code: code,
      variables: context[:variables] || %{},
      turtle_state: context[:turtle_state] || Ellex.TurtleGraphics.initial_state(),
      output: [],
      mode: context[:mode] || :speak,
      safety_limits: %{
        timeout: @timeout,
        max_recursion_depth: @max_recursion_depth,
        memory_limit: 64 * 1024 * 1024  # 64MB
      }
    }
    
    task = Task.async(__MODULE__, :execute_with_safety, [execution_context])
    
    try do
      case Task.await(task, @timeout) do
        {:ok, result} -> {:ok, result}
        {:error, error} -> {:error, format_kid_friendly_error(error)}
      end
    catch
      :exit, {:timeout, _} ->
        Task.shutdown(task, :brutal_kill)
        {:error, kid_friendly_timeout_error()}
    end
  end
  
  @doc """
  Internal execution with safety monitoring.
  """
  def execute_with_safety(context) do
    SafetyMonitor.start_execution(self())
    
    try do
      # Transform natural language to Elixir AST
      with {:ok, elixir_ast} <- NaturalLanguageTransformer.transform(context.code),
           {:ok, validated_ast} <- Validator.validate_for_kids(elixir_ast),
           {:ok, result} <- evaluate_ast(validated_ast, context) do
        {:ok, result}
      else
        {:error, reason} -> {:error, reason}
      end
    after
      SafetyMonitor.end_execution(self())
    end
  rescue
    error ->
      Logger.error("[Ellex.MiniElixir] Execution error: #{Exception.message(error)}")
      {:error, error}
  end
  
  # Core evaluation function
  defp evaluate_ast(ast, context) do
    bindings = [
      # Standard Ellex bindings for kids
      name: context.variables[:name] || "friend",
      age: context.variables[:age] || 8,
      turtle: context.turtle_state,
      output: context.output,
      mode: context.mode
    ]
    
    try do
      {result, _new_bindings} = Code.eval_quoted(ast, bindings)
      {:ok, result}
    rescue
      error -> {:error, error}
    end
  end
  
  # Kid-friendly error formatting
  defp format_kid_friendly_error(%CompileError{description: desc}) do
    %{
      type: "Oops! I didn't understand that 🤔",
      message: simplify_error_message(desc),
      emoji: "🤔",
      suggestion: "Try writing it differently, like: tell \"Hello!\""
    }
  end
  
  defp format_kid_friendly_error(%RuntimeError{message: msg}) do
    %{
      type: "Something went wrong while running your code 🐛",
      message: simplify_error_message(msg),
      emoji: "🐛",
      suggestion: "Double-check your code and try again!"
    }
  end
  
  defp format_kid_friendly_error(%ArgumentError{message: msg}) do
    %{
      type: "Hmm, that doesn't look right 🤨",
      message: simplify_error_message(msg),
      emoji: "🤨",
      suggestion: "Check if you're using the right words and try again!"
    }
  end
  
  defp format_kid_friendly_error(error) when is_exception(error) do
    %{
      type: "Something unexpected happened 😅",
      message: "Don't worry, everyone makes mistakes when learning!",
      emoji: "😅",
      suggestion: "Try a simpler version first, then build up to what you want!"
    }
  end
  
  defp format_kid_friendly_error(error) do
    %{
      type: "Unknown error 🤷",
      message: inspect(error),
      emoji: "🤷",
      suggestion: "Ask for help if you're stuck!"
    }
  end
  
  defp simplify_error_message(msg) when is_binary(msg) do
    msg
    |> String.replace(~r/undefined function.*\/\d+/, "I don't know that command")
    |> String.replace(~r/\*\*.*\*\*/, "")
    |> String.replace("(CompileError)", "")
    |> String.trim()
  end
  
  defp kid_friendly_timeout_error do
    %{
      type: "Your code is taking too long! ⏰",
      message: "Code should finish quickly so we can see the results!",
      emoji: "⏰",
      suggestion: "Try breaking your code into smaller parts, or avoid infinite loops!"
    }
  end
  
  @doc """
  Parse natural language commands for execution.
  
  Supports Ellex natural language syntax like:
  - tell "message"
  - ask "question?" → variable
  - repeat N times: ...
  - when condition: ...
  - make function_name: ...
  """
  def parse_natural_language(code) do
    NaturalLanguageTransformer.transform(code)
  end
  
  @doc """
  Create a safe execution environment for kids.
  """
  def create_safe_environment do
    %{
      allowed_modules: [
        IO,
        Enum, 
        String,
        Integer,
        Float,
        Map,
        List,
        Ellex.TurtleGraphics
      ],
      blocked_modules: [
        System,
        Process,
        Node,
        File,
        Code
      ],
      memory_limit: 64 * 1024 * 1024,  # 64MB
      execution_time_limit: @timeout
    }
  end
  
  # GenServer callbacks for function management
  @impl true
  def handle_call({:register_function, name, ast}, _from, state) do
    new_functions = Map.put(state.functions, name, ast)
    {:reply, :ok, %{state | functions: new_functions}}
  end
  
  @impl true
  def handle_call({:get_function, name}, _from, state) do
    function = Map.get(state.functions, name)
    {:reply, function, state}
  end
  
  @impl true
  def handle_call(:list_functions, _from, state) do
    {:reply, Map.keys(state.functions), state}
  end
end