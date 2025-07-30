defmodule Ellex.MiniElixir.Validator.PatternChecker do
  @moduledoc """
  Pattern validation for safe pattern matching in kid-friendly code.
  
  Based on Sequin's pattern checker but adapted for educational use.
  """

  defmodule BadPattern do
    @moduledoc false
    defexception [:node]
    
    def message(%{node: node}) do
      "This pattern is too complex for Ellex: #{inspect(node, limit: 3)} 🤔\nTry using simpler patterns!"
    end
  end

  @doc """
  Extract variables bound in a pattern, ensuring they're kid-friendly.
  """
  def extract_bound_vars(pattern) do
    try do
      vars = pattern |> extract_vars([]) |> Enum.uniq() |> validate_variable_names()
      {:ok, vars}
    rescue
      ex in BadPattern ->
        {:error, Exception.message(ex)}
      ex ->
        {:error, "Pattern error: #{Exception.message(ex)}"}
    end
  end

  # Variable extraction with kid-friendly constraints
  defp extract_vars({name, _meta, context}, acc) when is_atom(name) and is_atom(context) do
    var_string = Atom.to_string(name)
    cond do
      # Skip underscore variables (they don't bind)
      String.starts_with?(var_string, "_") -> 
        acc
      
      # Check for kid-friendly variable names
      String.match?(var_string, ~r/^[a-z][a-z0-9_]*$/) ->
        [name | acc]
      
      true ->
        raise BadPattern, node: name
    end
  end

  # Pin operator (^var) - allowed but doesn't bind new variables
  defp extract_vars({:^, _meta, [_pin]}, acc) do
    acc
  end

  # Pattern matching with = operator
  defp extract_vars({:=, _meta, [left, right]}, acc) do
    acc = extract_vars(left, acc)
    extract_vars(right, acc)
  end

  # Struct patterns (limited complexity)
  defp extract_vars({:%, _meta, [struct_name, body]}, acc) do
    # Only allow simple struct patterns
    case body do
      {:%{}, _, pairs} when length(pairs) <= 5 ->
        acc = extract_vars(struct_name, acc)
        Enum.reduce(pairs, acc, fn {_key, value}, inner_acc ->
          extract_vars(value, inner_acc)
        end)
      
      _ ->
        raise BadPattern, node: {:%, [], [struct_name, body]}
    end
  end

  # Map patterns (limited to 5 keys for simplicity)
  defp extract_vars({:%{}, _meta, pairs}, acc) when length(pairs) <= 5 do
    Enum.reduce(pairs, acc, fn {_key, value}, inner_acc ->
      extract_vars(value, inner_acc)
    end)
  end
  
  defp extract_vars({:%{}, _meta, pairs}, _acc) when length(pairs) > 5 do
    raise BadPattern, node: {:map_too_complex, length(pairs)}
  end

  # List patterns (limited to 10 elements for safety)
  defp extract_vars(elements, acc) when is_list(elements) and length(elements) <= 10 do
    Enum.reduce(elements, acc, &extract_vars/2)
  end
  
  defp extract_vars(elements, _acc) when is_list(elements) and length(elements) > 10 do
    raise BadPattern, node: {:list_too_long, length(elements)}
  end

  # List cons pattern [head | tail]
  defp extract_vars({:|, _meta, [head, tail]}, acc) do
    acc = extract_vars(head, acc)
    extract_vars(tail, acc)
  end

  # Tuple patterns (limited to 5 elements)
  defp extract_vars({:{}, _meta, elements}, acc) when length(elements) <= 5 do
    Enum.reduce(elements, acc, &extract_vars/2)
  end
  
  defp extract_vars({:{}, _meta, elements}, _acc) when length(elements) > 5 do
    raise BadPattern, node: {:tuple_too_long, length(elements)}
  end

  # 2-element tuple (special case in Elixir AST)
  defp extract_vars({x, y}, acc) do
    acc = extract_vars(x, acc)
    extract_vars(y, acc)
  end

  # Binary patterns (very limited for kids)
  defp extract_vars({:<<>>, _meta, segments}, acc) when length(segments) <= 3 do
    Enum.reduce(segments, acc, fn
      {:"::", _meta, [var, _type]}, inner_acc -> extract_vars(var, inner_acc)
      segment, inner_acc -> extract_vars(segment, inner_acc)
    end)
  end
  
  defp extract_vars({:<<>>, _meta, segments}, _acc) when length(segments) > 3 do
    raise BadPattern, node: {:binary_too_complex, length(segments)}
  end

  # Guards in patterns (when clause)
  defp extract_vars({:when, _meta, [pattern, _guard]}, acc) do
    # Guards can't bind new variables, so only check the pattern
    extract_vars(pattern, acc)
  end

  # Literals don't bind variables
  defp extract_vars(literal, acc) when is_number(literal) or 
                                       is_binary(literal) or 
                                       is_boolean(literal) or 
                                       is_nil(literal) do
    acc
  end

  # Atoms and module aliases don't bind
  defp extract_vars(atom, acc) when is_atom(atom), do: acc
  defp extract_vars({:__aliases__, _, _}, acc), do: acc

  # Anything else is too complex for kids
  defp extract_vars(other, _acc) do
    raise BadPattern, node: other
  end

  # Validate that variable names are kid-friendly
  defp validate_variable_names(vars) do
    Enum.each(vars, fn var ->
      var_string = Atom.to_string(var)
      
      cond do
        # Too long
        String.length(var_string) > 20 ->
          raise BadPattern, node: {:variable_too_long, var}
        
        # Contains invalid characters
        not String.match?(var_string, ~r/^[a-z][a-z0-9_]*$/) ->
          raise BadPattern, node: {:invalid_variable_name, var}
        
        # Reserved words that might confuse kids
        var in [:def, :defp, :defmodule, :import, :require, :use, :alias] ->
          raise BadPattern, node: {:reserved_word, var}
        
        true ->
          :ok
      end
    end)
    
    vars
  end
end