defmodule Peopler.CLI.Commands.HistoryCommand do
  @moduledoc """
  Show history of previous commands.
  """

  require Logger

  @doc """
  Show history of previous commands.
  """
  def handle(_args \\ nil, _settings \\ nil) do
    Peopler.CLI.State.history()
    |> format_history_list()
  end

  defp format_history_list(history_list)

  defp format_history_list([{history_id, history_cmd} | tail]) do
    String.pad_leading(Integer.to_string(history_id), signif_digits(history_id), "0") <>
      ": #{history_cmd}\n" <> format_history_list(tail)
  end

  defp format_history_list([]), do: ""

  defp signif_digits(number) do
    number |> Integer.digits() |> length
  end
end
