defmodule Peopler.CLI.State do
  @moduledoc """
  Documentation for `Peopler.CLI.State`.
  """

  use Agent

  # Hold state for the cli
  defmodule PeoplerCLIState do
    defstruct(
      history: []
    )
  end

  @doc """
  Start the State agent
  """
  def start_link(state \\ %PeoplerCLIState{}) do
    Agent.start_link(fn -> state end, name: __MODULE__)
  end

  @doc """
  Get the State struct.
  """
  def state do
    Agent.get(__MODULE__, & &1)
  end

  # Update the State struct.
  defp update(new_state) do
    Agent.update(__MODULE__, fn _ -> new_state end)
  end

  @doc """
  Push a new command onto the front of the command history.
  """
  def push_cmd(cmd) when cmd == "history", do: state().history

  def push_cmd(cmd) when is_binary(cmd) do
    state = state()
    history_length = length(state.history)
    new_history = [{history_length, String.trim(cmd)} | state.history]
    new_state = Map.put(state, :history, new_history)
    update(new_state)
    new_history
  end

  def push_cmd(cmd), do: push_cmd(inspect(cmd))

  @doc """
  Get the list of recent commands. Note that the commands come out in reverse order to being pushed. The logic here is that when displaying them in the CLI, you'd want to see the newest (highest numbered) cmd at the end, closest to where the the user and curser are currently sitting.
  """
  def history do
    state().history
    |> Enum.reverse()
  end

  @doc """
  Get the current history length (for use in the prompt.
  """
  def hlen do
    state().history
    |> length
  end

  @doc """
  Flush the history completely.
  """
  def flush_history() do
    state = state()
    new_history = []
    new_state = Map.put(state, :history, new_history)
    update(new_state)
    new_history
  end
end
