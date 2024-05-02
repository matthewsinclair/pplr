defmodule Peopler.CLI.Commands.IndexCommand do
  @moduledoc """
  Peopler CLI command to generate the Peopler index as either .md or .json.
  """

  require Logger

  @doc """
  Peopler CLI command to generate the Peopler index as either .md or .json.
  """
  def handle(args, _settings, _optimus \\ nil) do
    dbg(args)
  end
end
