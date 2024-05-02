defmodule Peopler.CLI.Commands.StatusCommand do
  @moduledoc """
  Peopler CLI command to show current status of everything.
  """

  require Logger

  @doc """
  Show current status of everything.
  """
  def handle(_args \\ nil, _settings \\ nil) do
    # TODO: Redo to print the status in a more user-friendly way
    inspect(Peopler.CLI.State.state())
  end
end
