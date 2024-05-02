defmodule Peopler.CLI.MixProject do
  use Mix.Project

  def project do
    [
      app: :peopler_cli,
      version: "0.1.0",
      elixir: "~> 1.16",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      escript: [main_module: Peopler.CLI, path: "_build/escript/peopler_cli", name: "peopler_cli"],
      mix_tasks: [
        peopler_cli: Mix.Tasks.Peopler.CLI,
        comment: "🫵 Peopler CLI"
      ]
    ]
  end

  def application do
    [
      mod: {Peopler.CLI, []},
      ansi_enabled: true
    ]
  end

  defp deps do
    [
      {:ok, "~> 2.3"},
      {:httpoison, "~> 2.1"},
      {:optimus, "~> 0.2"},
      {:castore, "~> 0.1.0"},
      {:jason, "~> 1.4"},
      {:tesla, "~> 1.5.1"},
      {:certifi, "~> 2.9"},
      {:ex_doc, "~> 0.29.1"},
      {:owl, "~> 0.6.1"},
      {:pathex, "~> 2.5.1"},
      {:table_rex, "~> 4.0.0"},
      {:elixir_uuid, "~> 1.2"}
    ]
  end
end
