defmodule Peopler.CLI.Commands.FlushCommand do
  @moduledoc """
  Flush the history of previous commands.
  """

  require Logger

  @doc """
  Flush the history of previous commands.
  """
  def handle(_args \\ nil, _settings \\ nil) do
    Peopler.CLI.State.flush_history()
    "ok"
  end
end
