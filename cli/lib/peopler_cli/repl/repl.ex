defmodule Peopler.CLI.Repl do
  @moduledoc """
  Documentation for `Peopler.CLI.Repl`.
  """

  alias Peopler.CLI
  alias Peopler.CLI.State
  import Peopler.CLI.Utils
  require Logger

  @doc """
  Kicks off the REPL loop.
  """
  def start(args, settings, optimus) do
    Peopler.CLI.intro(args, settings) |> put_lines
    repl(args, settings, optimus)
  end

  # Loop thru read/eval/print loop.
  defp repl(args, settings, optimus) do
    args
    |> read
    |> eval(settings, optimus)
    |> print
    |> case do
      {:ok, :quit} ->
        {:ok, :quit}

      _result ->
        repl(args, settings, optimus)
    end
  end

  # Parse REPL input into dispatchable params
  defp read(_args, prompt \\ repl_prompt()) do
    # Not sure what to do with args here?
    IO.gets(prompt)
  end

  # Handle 'repl' as a special case (do nothing)
  defp eval("repl\n", _settings, _optimus) do
    "The repl is already running."
  end

  # Handle 'q!' as a special case
  defp eval("q!\n", _settings, _optimus) do
    {:ok, :quit}
  end

  # Handle 'quit' as a special case
  defp eval("quit\n", _settings, _optimus) do
    {:ok, :quit}
  end

  # Handle '^d' as a special case
  defp eval(:eof, _settings, _optimus) do
    {:ok, :quit}
  end

  # Handle 'help' as a special case
  defp eval("help\n", settings, optimus) do
    eval(["--help"], settings, optimus)
  end

  # Evaluate params and dispatch to appropriate handler
  defp eval(args, settings, optimus) when is_binary(args) do
    should_push?(args) && Peopler.CLI.State.push_cmd(args)
    Optimus.parse(optimus, String.split(args))
    |> CLI.handle_args(settings, optimus)
  end

  defp eval(args, settings, optimus) when is_list(args) do
    should_push?(args) && Peopler.CLI.State.push_cmd(args)
    Optimus.parse(optimus, args)
    |> CLI.handle_args(settings, optimus)
  end

  # Make sure not to push these commands to the command history
  @non_history_cmds ["history", "redo", "flush", "help"]

  # Returns true iff cmd (as a string) does not exist in @non_history_cmds
  def should_push?(cmd) when is_binary(cmd) do
    (Enum.filter(@non_history_cmds, fn item -> String.trim(cmd) =~ item end) |> length) == 0
  end

  # Returns true iff cmd (as a list) does not exist in @non_history_cmds
  def should_push?(cmd) when is_list(cmd) do
    should_push?(Enum.join(cmd, ""))
  end

  # Allow RedoCommand to replay a command.
  def eval_for_redo({ _history_id, history_cmd}, settings, optimus) when is_binary(history_cmd) do
    eval(history_cmd, settings, optimus)
  end

  # Ignore tuples when printing and return what was sent in (a bit like tee)
  defp print(out) when is_tuple(out), do: out

  # Default print to ANSI and return what was printed (a bit like tee)
  defp print(out) do
    out |> filter_blank_lines |> put_lines
  end

  # Provide the REPL's prompt
  defp repl_prompt do
    "\n🫵 #{State.hlen} > "
  end
end
