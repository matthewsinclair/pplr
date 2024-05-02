defmodule Peopler.CLI.Commands.GetCommand do
  @moduledoc """
  Peopler CLI command to get a property from settings.
  """

  alias Peopler.CLI, as: CLI

  @doc """
  Get a settings value from settings by its id (with dot notation)
  """
  def handle(args, _settings \\ nil) do
    CLI.get_setting(args.args.id)
  end
end
