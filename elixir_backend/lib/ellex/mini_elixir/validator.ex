defmodule Ellex.MiniElixir.Validator do
  @moduledoc """
  AST validator for safe execution of kid-friendly Ellex code.
  
  Adapted from Sequin's MiniElixir validator with additional safety
  constraints and kid-friendly validation for natural language syntax.
  """
  
  alias Ellex.MiniElixir.Validator.PatternChecker
  
  # Kid-friendly function names that map to Elixir constructs
  @allowed_ellex_commands [
    :tell,      # IO.puts
    :ask,       # IO.gets with binding
    :repeat,    # Enum.each with range
    :when,      # if/case statements
    :make,      # function definition (limited)
    :move,      # turtle graphics
    :turn,      # turtle graphics
    :use,       # turtle color/pen
    :draw       # turtle graphics
  ]
  
  # Safe Elixir modules for kids
  @allowed_modules [
    IO,
    Enum,
    String,
    Integer,
    Float,
    Map,
    List,
    Range,
    Ellex.TurtleGraphics
  ]
  
  # Dangerous modules to block
  @blocked_modules [
    System,
    Process,
    Node,
    File,
    Code,
    :erlang,
    :os,
    Agent,
    Task,
    GenServer
  ]
  
  # Safe kernel functions
  @allowed_kernel_functions [
    :+, :-, :*, :/, :rem, :div,
    :==, :!=, :>, :<, :>=, :<=,
    :and, :or, :not,
    :is_integer, :is_float, :is_binary, :is_list, :is_map,
    :length, :hd, :tl, :elem, :tuple_size,
    :to_string, :inspect,
    :round, :trunc, :ceil, :floor
  ]
  
  # Pattern operations that are safe
  @safe_patterns [
    :=,           # pattern matching
    :{},          # tuples
    :%{},         # maps
    :<<>>,        # binaries (limited)
    :|,           # list cons
    :when         # guards
  ]
  
  @error_unsafe_operation "This operation isn't allowed in Ellex - it might not be safe for kids! 🛡️"
  @error_blocked_module "This module isn't available in Ellex to keep things safe 🔒"
  @error_too_complex "This code is too complex - try breaking it into smaller parts! 🧩"
  
  @doc """
  Validate AST for kid-friendly safety.
  """
  def validate_for_kids(ast) do
    case validate_ast(ast, 0) do
      :ok -> {:ok, ast}
      {:error, reason} -> {:error, format_kid_error(reason)}
    end
  end
  
  @doc """
  Main validation function - recursively validates AST nodes.
  """
  def validate_ast(_ast, depth) when depth > 100 do
    {:error, @error_too_complex}
  end
  
  # Literals are always safe
  def validate_ast(literal, _depth) when is_number(literal) or 
                                         is_binary(literal) or 
                                         is_boolean(literal) or 
                                         is_nil(literal) do
    :ok
  end
  
  # Variables are safe
  def validate_ast({var, _meta, context}, _depth) when is_atom(var) and is_atom(context) do
    :ok
  end
  
  # List literals
  def validate_ast(list, depth) when is_list(list) do
    Enum.reduce_while(list, :ok, fn item, :ok ->
      case validate_ast(item, depth + 1) do
        :ok -> {:cont, :ok}
        error -> {:halt, error}
      end
    end)
  end
  
  # Tuple literals
  def validate_ast({:{}, _meta, elements}, depth) do
    validate_ast(elements, depth + 1)
  end
  
  # Two-element tuples
  def validate_ast({left, right}, depth) do
    with :ok <- validate_ast(left, depth + 1),
         :ok <- validate_ast(right, depth + 1) do
      :ok
    end
  end
  
  # Map literals
  def validate_ast({:%{}, _meta, pairs}, depth) do
    Enum.reduce_while(pairs, :ok, fn {key, value}, :ok ->
      with :ok <- validate_ast(key, depth + 1),
           :ok <- validate_ast(value, depth + 1) do
        {:cont, :ok}
      else
        error -> {:halt, error}
      end
    end)
  end
  
  # Binary construction (limited for safety)
  def validate_ast({:<<>>, _meta, segments}, depth) do
    if length(segments) > 10 do
      {:error, "Binary patterns are limited to 10 segments for safety"}
    else
      validate_ast(segments, depth + 1)
    end
  end
  
  # Function calls - the main safety check
  def validate_ast({func, _meta, args}, depth) when is_atom(func) do
    cond do
      # Ellex natural language commands
      func in @allowed_ellex_commands ->
        validate_ellex_command(func, args, depth)
      
      # Safe kernel functions
      func in @allowed_kernel_functions ->
        validate_ast(args, depth + 1)
      
      # Pattern matching and safe operations
      func in @safe_patterns ->
        validate_ast(args, depth + 1)
      
      # Block structures
      func in [:__block__, :do, :else, :after, :catch, :rescue] ->
        validate_ast(args, depth + 1)
      
      # Control structures (limited)
      func in [:if, :unless, :case, :cond] ->
        validate_control_structure(func, args, depth)
      
      true ->
        {:error, "Function '#{func}' is not allowed"}
    end
  end
  
  # Remote function calls Module.function(args)
  def validate_ast({{:., _meta, [module, func]}, _call_meta, args}, depth) do
    validate_remote_call(module, func, args, depth)
  end
  
  # Pipe operator
  def validate_ast({:|>, _meta, [left, right]}, depth) do
    with :ok <- validate_ast(left, depth + 1),
         :ok <- validate_ast(right, depth + 1) do
      :ok
    end
  end
  
  # Anonymous functions (limited)
  def validate_ast({:fn, _meta, clauses}, depth) when length(clauses) <= 3 do
    Enum.reduce_while(clauses, :ok, fn {:->, _meta2, [params, body]}, :ok ->
      with :ok <- validate_ast(params, depth + 1),
           :ok <- validate_ast(body, depth + 1) do
        {:cont, :ok}
      else
        error -> {:halt, error}
      end
    end)
  end
  
  def validate_ast({:fn, _meta, clauses}, _depth) when length(clauses) > 3 do
    {:error, "Anonymous functions are limited to 3 clauses for simplicity"}
  end
  
  # Everything else is potentially unsafe
  def validate_ast(ast, _depth) do
    {:error, "Construct not allowed: #{inspect(elem(ast, 0), limit: 1)}"}
  end
  
  # Validate Ellex natural language commands
  defp validate_ellex_command(:tell, [message], depth) do
    validate_ast(message, depth + 1)
  end
  
  defp validate_ellex_command(:ask, [question | binding], depth) do
    with :ok <- validate_ast(question, depth + 1) do
      # Validate binding pattern if present
      case binding do
        [] -> :ok
        [bind_pattern] -> validate_binding_pattern(bind_pattern)
        _ -> {:error, "ask can only bind to one variable"}
      end
    end
  end
  
  defp validate_ellex_command(:repeat, [count, {:do, body}], depth) do
    with :ok <- validate_ast(count, depth + 1),
         :ok <- validate_positive_integer(count) do
      validate_ast(body, depth + 1)
    end
  end
  
  defp validate_ellex_command(:when, [condition, {:do, then_clause} | else_clause], depth) do
    with :ok <- validate_ast(condition, depth + 1),
         :ok <- validate_ast(then_clause, depth + 1) do
      case else_clause do
        [] -> :ok
        [{:else, else_body}] -> validate_ast(else_body, depth + 1)
        _ -> {:error, "when can only have one else clause"}
      end
    end
  end
  
  # Turtle graphics commands
  defp validate_ellex_command(turtle_cmd, args, depth) when turtle_cmd in [:move, :turn, :use, :draw] do
    validate_ast(args, depth + 1)
  end
  
  defp validate_ellex_command(cmd, args, _depth) do
    {:error, "Invalid arguments for '#{cmd}': #{inspect(args, limit: 3)}"}
  end
  
  # Validate remote function calls (Module.function)
  defp validate_remote_call({:__aliases__, _meta, modules}, func, args, depth) do
    module = Module.concat(modules)
    validate_module_call(module, func, args, depth)
  end
  
  defp validate_remote_call(module, func, args, depth) when is_atom(module) do
    validate_module_call(module, func, args, depth)
  end
  
  defp validate_remote_call(_module, _func, _args, _depth) do
    {:error, "Invalid module reference"}
  end
  
  # Validate calls to specific modules
  defp validate_module_call(module, func, args, depth) do
    cond do
      module in @blocked_modules ->
        {:error, @error_blocked_module}
      
      module in @allowed_modules ->
        validate_ast(args, depth + 1)
      
      # Special case for IO module - limit to safe functions
      module == IO and func in [:puts, :inspect, :gets] ->
        validate_ast(args, depth + 1)
      
      true ->
        {:error, "Module #{module} is not allowed"}
    end
  end
  
  # Validate control structures with limits
  defp validate_control_structure(:if, [condition, [{:do, then_clause} | rest]], depth) do
    with :ok <- validate_ast(condition, depth + 1),
         :ok <- validate_ast(then_clause, depth + 1) do
      case rest do
        [] -> :ok
        [{:else, else_clause}] -> validate_ast(else_clause, depth + 1)
        _ -> {:error, "if can only have do and else clauses"}
      end
    end
  end
  
  defp validate_control_structure(:case, [expr, [{:do, clauses}]], depth) when length(clauses) <= 5 do
    with :ok <- validate_ast(expr, depth + 1) do
      validate_case_clauses(clauses, depth + 1)
    end
  end
  
  defp validate_control_structure(:case, [_expr, [{:do, clauses}]], _depth) when length(clauses) > 5 do
    {:error, "case statements are limited to 5 clauses for simplicity"}
  end
  
  defp validate_control_structure(structure, _args, _depth) do
    {:error, "Control structure '#{structure}' validation not implemented"}
  end
  
  # Validate case clauses
  defp validate_case_clauses(clauses, depth) do
    Enum.reduce_while(clauses, :ok, fn {:->, _meta, [pattern, body]}, :ok ->
      with {:ok, _bound_vars} <- PatternChecker.extract_bound_vars(pattern),
           :ok <- validate_ast(pattern, depth + 1),
           :ok <- validate_ast(body, depth + 1) do
        {:cont, :ok}
      else
        {:error, reason} -> {:halt, {:error, reason}}
        error -> {:halt, error}
      end
    end)
  end
  
  # Validate binding patterns for ask commands
  defp validate_binding_pattern({:arrow, _meta, [var]}) when is_atom(var) do
    if String.match?(Atom.to_string(var), ~r/^[a-z][a-z0-9_]*$/) do
      :ok
    else
      {:error, "Variable names should start with a lowercase letter and contain only letters, numbers, and underscores"}
    end
  end
  
  defp validate_binding_pattern(_pattern) do
    {:error, "ask commands can only bind to simple variable names"}
  end
  
  # Ensure numeric values are reasonable for kids
  defp validate_positive_integer(ast) do
    case ast do
      n when is_integer(n) and n > 0 and n <= 1000 -> :ok
      n when is_integer(n) and n > 1000 -> {:error, "Numbers should be 1000 or less to keep things simple"}
      n when is_integer(n) and n <= 0 -> {:error, "Count should be a positive number"}
      _ -> :ok  # Will be validated at runtime
    end
  end
  
  # Format errors in a kid-friendly way
  defp format_kid_error(reason) when is_binary(reason) do
    %{
      type: "Safety Check Failed 🛡️",
      message: reason,
      emoji: "🛡️",
      suggestion: "Try using simpler commands that are designed for learning!"
    }
  end
  
  defp format_kid_error(reason) do
    format_kid_error(inspect(reason))
  end
end