defmodule Ellex.Application do
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      {Ellex.MiniElixir, []},
      {Ellex.SafetyMonitor, []},
      {Ellex.TurtleGraphics, []}
    ]

    opts = [strategy: :one_for_one, name: Ellex.Supervisor]
    Supervisor.start_link(children, opts)
  end
end