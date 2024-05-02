defmodule Peopler.CLI.Commands.RedoCommand do
  @moduledoc """
  Peopler CLI command to redo a previous command from the command history.
  """

  require Logger

  @doc """
  Peopler CLI command to redo a previous command from the command history.
  """
  def handle(args, settings, optimus) do
    inspect(Peopler.CLI.State.history())
    inspect(args)

    Peopler.CLI.State.history()
    |> List.to_tuple()
    |> elem(String.to_integer(args.args.params))
    |> Peopler.CLI.Repl.eval_for_redo(settings, optimus)
  end
end
