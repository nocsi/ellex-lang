defmodule Ellex do
  @moduledoc """
  Ellex - A natural language programming environment for kids.
  
  This is the main Elixir backend that provides safe code execution
  using MiniElixir and interfaces with the Rust components via NIFs.
  """

  use Application

  def start(_type, _args) do
    children = [
      {Ellex.MiniElixir, []},
      {Ellex.SafetyMonitor, []},
      {Ellex.TurtleGraphics, []}
    ]

    opts = [strategy: :one_for_one, name: Ellex.Supervisor]
    Supervisor.start_link(children, opts)
  end
  
  @doc """
  Execute Ellex natural language code safely.
  """
  def execute(code, context \\ %{}) do
    Ellex.MiniElixir.execute(code, context)
  end
  
  @doc """
  Parse natural language syntax into Elixir AST.
  """
  def parse_natural_language(code) do
    Ellex.NaturalLanguageParser.parse(code)
  end
end