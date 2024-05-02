defmodule Mix.Tasks.Peopler.CLI do
  @moduledoc "Custom mix tasks for Peopler CLI: mix peopler.cli"
  use Mix.Task
  alias Peopler.CLI, as: CLI

  @impl Mix.Task
  @requirements ["app.config", "app.start"]
  @shortdoc "Runs the Peopler CLI"
  @doc "Invokes the Peopler CLI and passes it the supplied command line params."
  def run(args) do
    CLI.main(args)
  end
end
