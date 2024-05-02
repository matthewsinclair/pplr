defmodule Peopler.CLI do
  @moduledoc """
  Documentation for `Peopler.CLI`.
  """

  use Application

  require OK
  use OK.Pipe
  import Peopler.CLI.Utils
  alias Peopler.CLI.Config
  alias Peopler.CLI.Repl
  alias Peopler.CLI.Commands.GetCommand
  alias Peopler.CLI.Commands.SettingsCommand
  alias Peopler.CLI.Commands.HistoryCommand
  alias Peopler.CLI.Commands.IndexCommand
  alias Peopler.CLI.Commands.FlushCommand
  alias Peopler.CLI.Commands.StatusCommand
  alias Peopler.CLI.Commands.RedoCommand

  @doc """
  Handle Application functionality to start the Peopler.CLI subsystem.
  """
  @impl true
  def start(_type, _args) do
    Peopler.CLI.State.start_link()
    {:ok, self()}
  end

  @doc """
  Entry point for command line parsing.
  """
  def main(argv) do
    Application.put_env(:elixir, :ansi_enabled, true)
    unless length(argv) != 0, do: intro() |> put_lines

    settings = load_settings() |> with_default(%{})
    optimus = optimus_config()

    Optimus.parse(optimus, argv)
    |> handle_args(settings, optimus)
    |> filter_blank_lines
    |> put_lines
  end

  @doc """
  Provide an Optimus config to drive the CLI.
  """
  def optimus_config do
    Optimus.new!(
      name: name(),
      description: about() <> "\n" <> description(),
      version: version(),
      author: author() <> "\n",
      allow_unknown_args: true,
      parse_double_dash: true,
      subcommands: [
        about: [
          name: "about",
          about: "Info about the Peopler command line interface."
        ],
        flush: [
          name: "flush",
          about: "Flush the command history."
        ],
        get: [
          name: "get",
          about: "Get the value of a setting.",
          args: [
            id: [
              value_name: "SETTING_ID",
              help: "Setting id",
              required: true,
              parser: :string
            ]
          ]
        ],
        history: [
          name: "history",
          about: "Show a history of recent commands."
        ],
        index: [
          name: "index",
          about: "Generate the Peopler index (as .md or .json).",
          args: [
            format: [
              value_name: "MD_OR_JSON",
              help: "Generate index as index.md or index.json",
              required: false,
              parser: :string
            ],
            people_dir: [
              value_name: "PEOPLER_DIR",
              help: "Root directory of Peopler",
              required: false,
              parser: :string
            ]
          ]
        ],
        redo: [
          name: "redo",
          about: "Redo a previous command from the history.",
          multiple: false,
          allow_unknown_args: false,
          args: [
            params: [
              value_name: "CMD",
              help: "ID of previous command",
              required: true
            ]
          ]
        ],
        repl: [
          name: "repl",
          about: "Start the Peopler REPL.",
          multiple: true,
          allow_unknown_args: true,
          args: [
            params: [
              value_name: "PARAMS",
              help: "additional paramaters",
              required: false
            ]
          ]
        ],
        settings: [
          name: "settings",
          about: "Display current configuration settings."
        ],
        status: [
          name: "status",
          about: "Show current CLI state."
        ]
      ]
    )
  end

  @doc """
  Handle the command line arguments
  """
  def handle_args(args, settings, optimus) do
    case args do
      {:ok, [:about], args} ->
        intro(args, settings)

      {:ok, [:repl], args} ->
        Repl.start(args, settings, optimus)
        ""

      {:ok, [:settings], args} ->
        SettingsCommand.handle(args, settings)

      {:ok, [:get], args} ->
        GetCommand.handle(args, settings)

      {:ok, [:status], args} ->
        StatusCommand.handle(args, settings)

      {:ok, [:history], args} ->
        HistoryCommand.handle(args, settings)

      {:ok, [:flush], args} ->
        FlushCommand.handle(args, settings)

      {:ok, [:index], args} ->
        IndexCommand.handle(args, settings)

      {:ok, [:redo], args} ->
        RedoCommand.handle(args, settings, optimus)

      {:ok, msg} when is_binary(msg) ->
        msg

      {:error, cmd, reason} ->
        handle_error(cmd, reason)

      {:error, reason} ->
        handle_error(reason)

      {:help, subcmd} ->
        Optimus.Help.help(optimus, subcmd, 80)

      :help ->
        Optimus.Help.help(optimus, [], 80)

      _other ->
        Optimus.Help.help(optimus, [], 80)
        print_ansi("")
    end
  end

  @doc """
  Handle an error nicely.
  """
  def handle_error(cmd, reason) do
    ("error: " <> Enum.join(cmd, " ") <> ": " <> Enum.join(reason, " "))
    |> String.trim()
  end

  def handle_error(reason) do
    ("error: " <> Enum.join(reason, " "))
    |> String.trim()
  end

  @doc """
  Load settings from JSON config file.
  """
  def load_settings() do
    Config.config_file()
    |> Path.expand()
    |> File.read()
    ~>> Jason.decode()
  end

  @doc """
  Get a setting by its id (and with dot notation).
  """
  def get_setting(id) do
    Config.get(id)
  end

  @doc """
  Save settings to JSON config file.
  """
  def save_settings(new_settings) do
    {:ok, current_settings} = load_settings()
    new_settings = Map.merge(current_settings, new_settings)

    path = Config.config_file() |> Path.expand()
    File.mkdir_p!(Path.dirname(path))

    OK.try do
      json <- Jason.encode(new_settings)
      write_result = File.write(path, Jason.Formatter.pretty_print(json))
    after
      case write_result do
        :ok -> {:ok, nil}
        err -> err
      end
    rescue
      reason ->
        print_ansi([
          :bright,
          :red,
          "ERROR:",
          :reset,
          " Failed to save settings, reason: ",
          Kernel.inspect(reason)
        ])

        reason
    end
  end

  # defp push_cmd(args, state) do
  #   state
  # end

  @doc """
  Print an about message
  """
  def intro(args \\ [], settings \\ nil)

  def intro(_args, _settings) do
    about() <> "\n" <> description() <> "\n" <> url() <> "\n" <> name() <> " " <> version()
  end

  # Accessors for string constants set via config
  def about do
    Application.fetch_env!(:peopler_cli, :about)
  end

  def url do
    Application.fetch_env!(:peopler_cli, :url)
  end

  def name do
    Application.fetch_env!(:peopler_cli, :name)
  end

  def description do
    Application.fetch_env!(:peopler_cli, :description)
  end

  def version do
    Application.fetch_env!(:peopler_cli, :version)
  end

  def author do
    Application.fetch_env!(:peopler_cli, :author)
  end
end
