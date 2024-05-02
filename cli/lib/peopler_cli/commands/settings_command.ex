defmodule Peopler.CLI.Commands.SettingsCommand do
  @moduledoc """
  Peopler CLI command to get all settings.
  """

  alias Peopler.CLI

  @doc """
  Get all settings.
  """
  def handle(_args \\ nil, _settings \\ nil) do
    case CLI.load_settings() do
      {:ok, settings} ->
        settings

      {:error, reason} ->
        {:error, reason}

      _ ->
        {:error, "problem with config settings"}
    end
  end
end
