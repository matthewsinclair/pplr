defmodule Peopler.CLI.Test do
  use ExUnit.Case
  import ExUnit.CaptureIO
  alias Peopler.CLI, as: CLI
  alias Peopler.CLI.Test.Support, as: Support
  doctest Peopler.CLI

  # @image0001 "test/data/images/0001.png"

  @cli_commands [
    ["about"],
    ["get", "id"],
    ["help", "settings"],
    ["history"],
    ["redo 0"],
    ["flush"],
    # ["repl"], # doesn't work as a test as it is interactive
    ["settings"],
    ["status"],
    ["--version"],
    ["--help"]
  ]

  describe "Peopler.CLI" do
    setup do
      # Get previous env var for config path and file names
      previous_env = System.get_env()

      # Set up to load the local .peopler/config.json file
      System.put_env("PEOPLER_CONFIG_PATH", "./.peopler")
      System.put_env("PEOPLER_CONFIG_FILE", "config.json")

      # Write a known config file to a known location
      Support.write_default_config_file(
        System.get_env("PEOPLER_CONFIG_FILE"),
        System.get_env("PEOPLER_CONFIG_PATH")
      )

      # Put things back how we gound them
      on_exit(fn -> System.put_env(previous_env) end)

      # Make sure that the CLI State process is running
      # {:ok, _pid} = Peopler.CLI.State.start_link()
      :ok
    end

    test "cli commands smoke test" do
      # Run thru each command (except 'repl') and smoke test each one
      @cli_commands
      |> Enum.all?(fn cmd ->
        IO.puts("\nSmoke testing command: #{Enum.join(cmd, " ")}")

        try do
          CLI.main(cmd)
          assert true
        rescue
          e in RuntimeError ->
            IO.puts("error: " <> e.message)
            assert false
        end
      end)
    end

    test "about" do
      assert capture_io(fn ->
               Peopler.CLI.main(["about"])
             end)
             |> String.trim() ==
               """
               🫵 Peopler CLI\nA simple CLI-based CRM that uses plain text files and directories\nhttps://peopler.io\npeopler_cli 0.1.0
               """
               |> String.trim()
    end

    test "settings" do
      assert capture_io(fn ->
               Peopler.CLI.main(["settings"])
             end)
             |> String.trim() ==
               """
               %{"id" => "DOT_SLASH_DOT_LL_SLASH_CONFIG_DOT_JSON"}
               """
               |> String.trim()
    end

    test "get" do
      assert capture_io(fn ->
               Peopler.CLI.main(["get"])
             end)
             |> String.trim() ==
               """
               error: get: missing required arguments: SETTING_ID
               """
               |> String.trim()
    end

    test "get id" do
      assert capture_io(fn ->
               Peopler.CLI.main(["get", "id"])
             end)
             |> String.trim() ==
               """
               DOT_SLASH_DOT_LL_SLASH_CONFIG_DOT_JSON
               """
               |> String.trim()
    end

    test "help" do
      assert capture_io(fn ->
               Peopler.CLI.main(["help"])
             end)
             |> String.trim() ==
               """
               error: invalid subcommand:
               """
               |> String.trim()
    end

    test "help settings" do
      assert capture_io(fn ->
               Peopler.CLI.main(["help", "settings"])
             end)
             |> String.trim() ==
               """
               🫵 Peopler CLI
               A simple CLI-based CRM that uses plain text files and directories 0.1.0
               hello@peopler.io

               Display current configuration settings.

               USAGE:
                   peopler_cli settings
               """
               |> String.trim()
    end

    test "--help" do
      assert capture_io(fn ->
               Peopler.CLI.main(["--help"])
             end)
             |> String.trim() ==
               """
               🫵 Peopler CLI\nA simple CLI-based CRM that uses plain text files and directories 0.1.0\nhello@peopler.io\n\n\nUSAGE:\n    peopler_cli ...\n    peopler_cli --version\n    peopler_cli --help\n    peopler_cli help subcommand\n\nSUBCOMMANDS:\n\n    about           Info about the Peopler command line interface.              \n    flush           Flush the command history.                                  \n    get             Get the value of a setting.                                 \n    history         Show a history of recent commands.                          \n    index           Generate the Peopler index (as .md or .json).               \n    redo            Redo a previous command from the history.                   \n    repl            Start the Peopler REPL.                                     \n    settings        Display current configuration settings.                     \n    status          Show current CLI state.
               """
               |> String.trim()
    end
  end
end
