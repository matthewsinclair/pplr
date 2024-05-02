defmodule Peopler.CLI.State.Test do
  use ExUnit.Case, async: false
  alias Peopler.CLI.State, as: State
  alias Peopler.CLI.Test.Support, as: Support
  doctest Peopler.CLI.State

  describe "Peopler.CLI.State" do
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

      :ok
    end

    test "history" do
      {_worked, _pid} = State.start_link()

      current_history = State.history()
      assert current_history == []

      State.push_cmd("one")
      State.push_cmd("two")
      State.push_cmd("three")
      new_history = State.history()
      assert new_history == [{0, "one"}, {1, "two"}, {2, "three"}]

      State.flush_history()
      flushed_history = State.history()
      assert flushed_history == []
    end
  end
end
