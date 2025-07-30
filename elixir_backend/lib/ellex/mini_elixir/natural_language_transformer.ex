defmodule Ellex.MiniElixir.NaturalLanguageTransformer do
  @moduledoc """
  Transforms Ellex natural language syntax into executable Elixir AST.
  
  Supports kid-friendly commands like:
  - tell "Hello!" → IO.puts("Hello!")
  - ask "Name?" → name → name = IO.gets("Name?") |> String.trim()
  - repeat 5 times: ... → Enum.each(1..5, fn _ -> ... end)
  - when x > 5: ... → if x > 5, do: ...
  - move forward 10 → Ellex.TurtleGraphics.move_forward(10)
  """
  
  require Logger

  @doc """
  Transform natural language Ellex code to Elixir AST.
  """
  def transform(code) when is_binary(code) do
    try do
      # First try to parse as regular Elixir (for mixed syntax)
      case Code.string_to_quoted(code) do
        {:ok, ast} ->
          # Transform any natural language constructs within the AST
          transformed_ast = transform_ast(ast)
          {:ok, transformed_ast}
        
        {:error, _} ->
          # Try parsing as pure natural language
          parse_natural_language(code)
      end
    rescue
      error ->
        {:error, "Could not understand: #{Exception.message(error)} 🤔"}
    end
  end

  @doc """
  Parse pure natural language Ellex commands.
  """
  def parse_natural_language(code) do
    code
    |> String.trim()
    |> parse_command()
  end

  # Transform existing AST to replace natural language constructs
  defp transform_ast(ast) do
    Macro.prewalk(ast, &transform_node/1)
  end

  # Transform individual AST nodes
  defp transform_node({:tell, meta, [message]}) do
    # tell "Hello!" → IO.puts("Hello!")
    {{:., [], [IO, :puts]}, meta, [message]}
  end

  defp transform_node({:ask, meta, [question, {:arrow, _, [var]}]}) when is_atom(var) do
    # ask "Name?" → name → name = IO.gets("Name?") |> String.trim()
    {:=, meta, [
      {var, [], nil},
      {:|>, [], [
        {{:., [], [IO, :gets]}, [], [question]},
        {{:., [], [String, :trim]}, [], []}
      ]}
    ]}
  end

  defp transform_node({:repeat, meta, [count, {:times, _, _}, {:do, body}]}) do
    # repeat 5 times: body → Enum.each(1..5, fn _ -> body end)
    {{:., [], [Enum, :each]}, meta, [
      {:.., [], [1, count]},
      {:fn, [], [{:->, [], [[{:_, [], nil}], body]}]}
    ]}
  end

  defp transform_node({:when, meta, [condition, {:do, then_clause}]}) do
    # when condition: then_clause → if condition, do: then_clause
    {:if, meta, [condition, [do: then_clause]]}
  end

  defp transform_node({:when, meta, [condition, {:do, then_clause}, {:else, else_clause}]}) do
    # when condition: then_clause else: else_clause
    {:if, meta, [condition, [do: then_clause, else: else_clause]]}
  end

  # Turtle graphics commands
  defp transform_node({:move, meta, [:forward, distance]}) do
    {{:., [], [Ellex.TurtleGraphics, :move_forward]}, meta, [distance]}
  end

  defp transform_node({:move, meta, [:backward, distance]}) do
    {{:., [], [Ellex.TurtleGraphics, :move_backward]}, meta, [distance]}
  end

  defp transform_node({:turn, meta, [:left, degrees]}) do
    {{:., [], [Ellex.TurtleGraphics, :turn_left]}, meta, [degrees]}
  end

  defp transform_node({:turn, meta, [:right, degrees]}) do
    {{:., [], [Ellex.TurtleGraphics, :turn_right]}, meta, [degrees]}
  end

  defp transform_node({:use, meta, [:color, color]}) do
    {{:., [], [Ellex.TurtleGraphics, :set_color]}, meta, [color]}
  end

  defp transform_node({:draw, meta, [:circle, {:with, _, [{:radius, _, [radius]}]}]}) do
    {{:., [], [Ellex.TurtleGraphics, :draw_circle]}, meta, [radius]}
  end

  # Keep other nodes unchanged
  defp transform_node(node), do: node

  # Parse individual natural language commands
  defp parse_command(code) do
    case code do
      # tell "message"
      tell_pattern when is_binary(tell_pattern) ->
        case Regex.run(~r/^tell\s+"([^"]+)"/, tell_pattern) do
          [_, message] ->
            {:ok, {{:., [], [IO, :puts]}, [], [message]}}
          nil ->
            try_other_patterns(code)
        end

      # ask "question?" → variable
      ask_pattern when is_binary(ask_pattern) ->
        case Regex.run(~r/^ask\s+"([^"]+)"\s*→\s*(\w+)/, ask_pattern) do
          [_, question, var] ->
            var_atom = String.to_atom(var)
            {:ok, {:=, [], [
              {var_atom, [], nil},
              {:|>, [], [
                {{:., [], [IO, :gets]}, [], [question]},
                {{:., [], [String, :trim]}, [], []}
              ]}
            ]}}
          nil ->
            try_other_patterns(code)
        end

      # repeat N times: body
      repeat_pattern when is_binary(repeat_pattern) ->
        case Regex.run(~r/^repeat\s+(\d+)\s+times:\s*(.+)$/s, repeat_pattern) do
          [_, count_str, body_str] ->
            count = String.to_integer(count_str)
            case parse_command(String.trim(body_str)) do
              {:ok, body_ast} ->
                {:ok, {{:., [], [Enum, :each]}, [], [
                  {:.., [], [1, count]},
                  {:fn, [], [{:->, [], [[{:_, [], nil}], body_ast]}]}
                ]}}
              error ->
                error
            end
          nil ->
            try_other_patterns(code)
        end

      # Move commands
      "move forward " <> rest ->
        case Integer.parse(rest) do
          {distance, ""} ->
            {:ok, {{:., [], [Ellex.TurtleGraphics, :move_forward]}, [], [distance]}}
          _ ->
            {:error, "Expected a number after 'move forward'"}
        end

      "move backward " <> rest ->
        case Integer.parse(rest) do
          {distance, ""} ->
            {:ok, {{:., [], [Ellex.TurtleGraphics, :move_backward]}, [], [distance]}}
          _ ->
            {:error, "Expected a number after 'move backward'"}
        end

      # Turn commands
      "turn left " <> rest ->
        case Integer.parse(rest) do
          {degrees, ""} ->
            {:ok, {{:., [], [Ellex.TurtleGraphics, :turn_left]}, [], [degrees]}}
          _ ->
            {:error, "Expected a number after 'turn left'"}
        end

      "turn right " <> rest ->
        case Integer.parse(rest) do
          {degrees, ""} ->
            {:ok, {{:., [], [Ellex.TurtleGraphics, :turn_right]}, [], [degrees]}}
          _ ->
            {:error, "Expected a number after 'turn right'"}
        end

      # Color commands
      "use color " <> color ->
        color_atom = String.to_atom(String.trim(color))
        {:ok, {{:., [], [Ellex.TurtleGraphics, :set_color]}, [], [color_atom]}}

      # Unknown command
      unknown ->
        suggest_command(unknown)
    end
  end

  # Try other parsing patterns if first ones fail
  defp try_other_patterns(code) do
    cond do
      # Simple expressions that might be valid Elixir
      String.contains?(code, ["+", "-", "*", "/", "==", "!="]) ->
        case Code.string_to_quoted(code) do
          {:ok, ast} -> {:ok, ast}
          {:error, _} -> suggest_command(code)
        end

      # Variable assignment
      String.contains?(code, "=") and not String.contains?(code, ["==", "!="]) ->
        case Code.string_to_quoted(code) do
          {:ok, ast} -> {:ok, ast}
          {:error, _} -> suggest_command(code)
        end

      true ->
        suggest_command(code)
    end
  end

  # Provide helpful suggestions for unrecognized commands
  defp suggest_command(code) do
    suggestions = []

    suggestions = 
      if String.contains?(String.downcase(code), ["hello", "hi", "say"]) do
        ["Try: tell \"Hello, world!\"" | suggestions]
      else
        suggestions
      end

    suggestions = 
      if String.contains?(String.downcase(code), ["name", "ask", "input"]) do
        ["Try: ask \"What's your name?\" → name" | suggestions]
      else
        suggestions
      end

    suggestions = 
      if String.contains?(String.downcase(code), ["loop", "repeat", "times"]) do
        ["Try: repeat 5 times: tell \"Hello!\"" | suggestions]
      else
        suggestions
      end

    suggestions = 
      if String.contains?(String.downcase(code), ["move", "forward", "turtle"]) do
        ["Try: move forward 50" | suggestions]
      else
        suggestions
      end

    suggestions = 
      if String.contains?(String.downcase(code), ["turn", "left", "right"]) do
        ["Try: turn left 90" | suggestions]
      else
        suggestions
      end

    error_message = if Enum.empty?(suggestions) do
      "I don't understand '#{code}' 🤔\n\nHere are some things you can try:\n" <>
      "• tell \"your message\"\n" <>
      "• ask \"your question?\" → variable_name\n" <>
      "• repeat 5 times: your_command\n" <>
      "• move forward 50\n" <>
      "• turn left 90\n" <>
      "• use color red"
    else
      "I don't understand '#{code}' 🤔\n\nDid you mean:\n" <>
      Enum.join(suggestions, "\n")
    end

    {:error, error_message}
  end

  @doc """
  Get available natural language commands for help.
  """
  def available_commands do
    %{
      "Communication" => [
        ~s{tell "message" - Display a message},
        ~s{ask "question?" → variable - Ask user for input}
      ],
      "Control Flow" => [
        "repeat N times: command - Repeat a command",
        "when condition: command - Do something if condition is true"
      ],
      "Turtle Graphics" => [
        "move forward N - Move turtle forward",
        "move backward N - Move turtle backward", 
        "turn left N - Turn turtle left by N degrees",
        "turn right N - Turn turtle right by N degrees",
        "use color name - Change turtle color",
        "draw circle with radius N - Draw a circle"
      ],
      "Variables and Math" => [
        "variable = value - Store a value",
        "variable + number - Add numbers",
        "variable > number - Compare values"
      ]
    }
  end

  @doc """
  Format help text for natural language commands.
  """
  def help_text do
    commands = available_commands()
    
    text = "🌟 Ellex Natural Language Commands 🌟\n\n"
    
    Enum.reduce(commands, text, fn {category, cmds}, acc ->
      acc <> "#{category}:\n" <>
      Enum.map_join(cmds, "\n", fn cmd -> "  • #{cmd}" end) <>
      "\n\n"
    end) <>
    "💡 Tip: You can combine commands and use regular Elixir too!\n" <>
    "🚀 Try: tell \"Hello!\" then ask \"What's your name?\" → name"
  end
end