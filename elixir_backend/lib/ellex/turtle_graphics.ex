defmodule Ellex.TurtleGraphics do
  @moduledoc """
  Turtle graphics system for visual programming with Ellex.
  
  Provides kid-friendly turtle graphics commands with state management
  and visual output capabilities.
  """
  
  use GenServer
  
  require Logger
  
  defstruct [
    :x, :y,           # Position
    :angle,           # Direction in degrees
    :pen_down,        # Whether pen is down
    :color,           # Current pen color
    :width,           # Line width
    :canvas_width,    # Canvas dimensions
    :canvas_height,
    :commands,        # History of drawing commands
    :visible          # Whether turtle is visible
  ]
  
  def start_link(opts) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end
  
  @impl true
  def init(_opts) do
    {:ok, default_state()}
  end
  
  @doc """
  Get the initial turtle state.
  """
  def initial_state, do: default_state()
  
  defp default_state do
    %__MODULE__{
      x: 250.0,
      y: 250.0,
      angle: 0.0,
      pen_down: true,
      color: :black,
      width: 2,
      canvas_width: 500,
      canvas_height: 500,
      commands: [],
      visible: true
    }
  end
  
  @doc """
  Reset turtle to initial position and state.
  """
  def reset do
    GenServer.call(__MODULE__, :reset)
  end
  
  @doc """
  Move turtle forward by specified distance.
  """
  def move_forward(distance) when is_number(distance) do
    GenServer.call(__MODULE__, {:move_forward, distance})
  end
  
  @doc """
  Move turtle backward by specified distance.
  """
  def move_backward(distance) when is_number(distance) do
    GenServer.call(__MODULE__, {:move_backward, distance})
  end
  
  @doc """
  Turn turtle left by specified degrees.
  """
  def turn_left(degrees) when is_number(degrees) do
    GenServer.call(__MODULE__, {:turn_left, degrees})
  end
  
  @doc """
  Turn turtle right by specified degrees.
  """
  def turn_right(degrees) when is_number(degrees) do
    GenServer.call(__MODULE__, {:turn_right, degrees})
  end
  
  @doc """
  Set turtle pen color.
  """
  def set_color(color) when is_atom(color) or is_binary(color) do
    GenServer.call(__MODULE__, {:set_color, color})
  end
  
  @doc """
  Lift pen up (stop drawing).
  """
  def pen_up do
    GenServer.call(__MODULE__, :pen_up)
  end
  
  @doc """
  Put pen down (start drawing).
  """
  def pen_down do
    GenServer.call(__MODULE__, :pen_down)
  end
  
  @doc """
  Set pen width.
  """
  def set_width(width) when is_number(width) and width > 0 do
    GenServer.call(__MODULE__, {:set_width, width})
  end
  
  @doc """
  Draw a circle with specified radius.
  """
  def draw_circle(radius) when is_number(radius) and radius > 0 do
    GenServer.call(__MODULE__, {:draw_circle, radius})
  end
  
  @doc """
  Draw a square with specified side length.
  """
  def draw_square(side_length) when is_number(side_length) and side_length > 0 do
    GenServer.call(__MODULE__, {:draw_square, side_length})
  end
  
  @doc """
  Get current turtle state.
  """
  def get_state do
    GenServer.call(__MODULE__, :get_state)
  end
  
  @doc """
  Get drawing commands for rendering.
  """
  def get_commands do
    GenServer.call(__MODULE__, :get_commands)
  end
  
  @doc """
  Clear all drawing commands and reset canvas.
  """
  def clear do
    GenServer.call(__MODULE__, :clear)
  end
  
  # GenServer Callbacks
  
  @impl true
  def handle_call(:reset, _from, _state) do
    new_state = %{default_state() | commands: []}
    Logger.debug("[TurtleGraphics] Reset to initial state")
    {:reply, :ok, new_state}
  end
  
  @impl true
  def handle_call({:move_forward, distance}, _from, state) do
    {new_state, result} = move_turtle(state, distance)
    {:reply, result, new_state}
  end
  
  @impl true
  def handle_call({:move_backward, distance}, _from, state) do
    {new_state, result} = move_turtle(state, -distance)
    {:reply, result, new_state}
  end
  
  @impl true
  def handle_call({:turn_left, degrees}, _from, state) do
    new_angle = normalize_angle(state.angle - degrees)
    new_state = %{state | angle: new_angle}
    
    command = {:turn, state.angle, new_angle}
    new_state = add_command(new_state, command)
    
    Logger.debug("[TurtleGraphics] Turned left #{degrees}° to #{new_angle}°")
    {:reply, :ok, new_state}
  end
  
  @impl true
  def handle_call({:turn_right, degrees}, _from, state) do
    new_angle = normalize_angle(state.angle + degrees)
    new_state = %{state | angle: new_angle}
    
    command = {:turn, state.angle, new_angle}
    new_state = add_command(new_state, command)
    
    Logger.debug("[TurtleGraphics] Turned right #{degrees}° to #{new_angle}°")
    {:reply, :ok, new_state}
  end
  
  @impl true
  def handle_call({:set_color, color}, _from, state) do
    normalized_color = normalize_color(color)
    new_state = %{state | color: normalized_color}
    
    command = {:color, normalized_color}
    new_state = add_command(new_state, command)
    
    Logger.debug("[TurtleGraphics] Set color to #{normalized_color}")
    {:reply, :ok, new_state}
  end
  
  @impl true
  def handle_call(:pen_up, _from, state) do
    new_state = %{state | pen_down: false}
    command = {:pen, :up}
    new_state = add_command(new_state, command)
    
    Logger.debug("[TurtleGraphics] Pen up")
    {:reply, :ok, new_state}
  end
  
  @impl true
  def handle_call(:pen_down, _from, state) do
    new_state = %{state | pen_down: true}
    command = {:pen, :down}
    new_state = add_command(new_state, command)
    
    Logger.debug("[TurtleGraphics] Pen down")
    {:reply, :ok, new_state}
  end
  
  @impl true
  def handle_call({:set_width, width}, _from, state) do
    new_state = %{state | width: width}
    command = {:width, width}
    new_state = add_command(new_state, command)
    
    Logger.debug("[TurtleGraphics] Set width to #{width}")
    {:reply, :ok, new_state}
  end
  
  @impl true
  def handle_call({:draw_circle, radius}, _from, state) do
    if state.pen_down do
      command = {:circle, state.x, state.y, radius, state.color, state.width}
      new_state = add_command(state, command)
      
      Logger.debug("[TurtleGraphics] Drew circle at (#{state.x}, #{state.y}) with radius #{radius}")
      {:reply, :ok, new_state}
    else
      Logger.debug("[TurtleGraphics] Circle not drawn - pen is up")
      {:reply, :ok, state}
    end
  end
  
  @impl true
  def handle_call({:draw_square, side_length}, _from, state) do
    # Draw square by moving forward and turning right 4 times
    {new_state, _} = 
      Enum.reduce(1..4, {state, :ok}, fn _, {current_state, _} ->
        {state_after_move, _} = move_turtle(current_state, side_length)
        new_angle = normalize_angle(state_after_move.angle + 90)
        state_after_turn = %{state_after_move | angle: new_angle}
        
        turn_command = {:turn, state_after_move.angle, new_angle}
        final_state = add_command(state_after_turn, turn_command)
        
        {final_state, :ok}
      end)
    
    Logger.debug("[TurtleGraphics] Drew square with side length #{side_length}")
    {:reply, :ok, new_state}
  end
  
  @impl true
  def handle_call(:get_state, _from, state) do
    state_map = %{
      x: state.x,
      y: state.y,
      angle: state.angle,
      pen_down: state.pen_down,
      color: state.color,
      width: state.width,
      visible: state.visible
    }
    {:reply, state_map, state}
  end
  
  @impl true
  def handle_call(:get_commands, _from, state) do
    {:reply, Enum.reverse(state.commands), state}
  end
  
  @impl true
  def handle_call(:clear, _from, state) do
    new_state = %{state | commands: []}
    command = {:clear}
    new_state = add_command(new_state, command)
    
    Logger.debug("[TurtleGraphics] Cleared canvas")
    {:reply, :ok, new_state}
  end
  
  # Helper Functions
  
  defp move_turtle(state, distance) do
    # Calculate new position based on current angle and distance
    angle_rad = state.angle * :math.pi() / 180.0
    new_x = state.x + distance * :math.cos(angle_rad)
    new_y = state.y + distance * :math.sin(angle_rad)
    
    # Keep turtle within canvas bounds
    bounded_x = max(0, min(state.canvas_width, new_x))
    bounded_y = max(0, min(state.canvas_height, new_y))
    
    new_state = %{state | x: bounded_x, y: bounded_y}
    
    # Add drawing command if pen is down
    new_state = if state.pen_down do
      command = {:line, state.x, state.y, bounded_x, bounded_y, state.color, state.width}
      add_command(new_state, command)
    else
      command = {:move, state.x, state.y, bounded_x, bounded_y}
      add_command(new_state, command)
    end
    
    result = if new_x != bounded_x or new_y != bounded_y do
      {:warning, "Turtle reached canvas edge!"}
    else
      :ok
    end
    
    Logger.debug("[TurtleGraphics] Moved from (#{state.x}, #{state.y}) to (#{bounded_x}, #{bounded_y})")
    {new_state, result}
  end
  
  defp add_command(state, command) do
    %{state | commands: [command | state.commands]}
  end
  
  defp normalize_angle(angle) when angle >= 360 do
    normalize_angle(angle - 360)
  end
  
  defp normalize_angle(angle) when angle < 0 do
    normalize_angle(angle + 360)
  end
  
  defp normalize_angle(angle), do: angle
  
  defp normalize_color(color) when is_binary(color) do
    String.to_atom(String.downcase(color))
  end
  
  defp normalize_color(color) when is_atom(color) do
    case color do
      c when c in [:red, :green, :blue, :yellow, :orange, :purple, :pink, :brown, :black, :white, :gray] ->
        c
      _ ->
        :black  # Default to black for unknown colors
    end
  end
  
  @doc """
  Get available turtle commands for help.
  """
  def available_commands do
    %{
      "Movement" => [
        "move forward N - Move forward N pixels",
        "move backward N - Move backward N pixels"
      ],
      "Turning" => [
        "turn left N - Turn left N degrees",
        "turn right N - Turn right N degrees"
      ],
      "Drawing" => [
        "pen up - Stop drawing while moving",
        "pen down - Start drawing while moving",
        "use color name - Change pen color",
        "set width N - Change line thickness"
      ],
      "Shapes" => [
        "draw circle with radius N - Draw a circle",
        "draw square with side N - Draw a square"
      ],
      "Canvas" => [
        "clear - Clear the drawing",
        "reset - Reset turtle to center"
      ]
    }
  end
end