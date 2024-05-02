defmodule Peopler.CLI.Config.Test do
  use ExUnit.Case, async: false
  alias Peopler.CLI.Config, as: Config
  alias Peopler.CLI.Test.Support, as: Support
  doctest Peopler.CLI.Config

  describe "Peopler.CLI.Config" do
    setup do
      # Get previous env var for config path and file names
      previous_env = System.get_env()

      # Set up to load the local .peopler/config.json file
      System.put_env("PEOPLER_CONFIG_PATH", "./.peopler")
      System.put_env("PEOPLER_CONFIG_FILE", "config.json")

      # Write a known config file to a known location
      Support.write_default_config_file(
        System.get_env("PEOPLER_CONFIG_FILE"),
        System.get_env("PEOPLER_CONFIG_PATH"))

      # Put things back how we gound them
      on_exit(fn -> System.put_env(previous_env) end)
    end

    test "config file path and name" do
      config_pathname = Config.config_pathname()
      config_filename = Config.config_filename()
      config_file = Config.config_file()

      assert config_pathname != nil
      assert config_filename != nil
      assert config_file != nil
      assert config_file === Path.join(config_pathname, config_filename)
    end

    test "config file path and name via env var" do
      # Jam something into the env vars
      System.put_env("PEOPLER_CONFIG_PATH", "/tmp/")
      System.put_env("PEOPLER_CONFIG_FILE", "bozo.json")

      # Test that they are equal to what LLConfig thinks they should be
      assert System.get_env("PEOPLER_CONFIG_PATH") == Config.config_pathname()
      assert System.get_env("PEOPLER_CONFIG_FILE") == Config.config_filename()
    end

    test "load valid configuration file (and succeed)" do
      # Check that we can load the default configuration file
      case Config.load() do
        {:ok, config} ->
          # will exec
          assert config != nil
          assert config["id"] == "DOT_SLASH_DOT_LL_SLASH_CONFIG_DOT_JSON"

        {:error, reason} ->
          # won't exec
          assert reason == nil
      end
    end

    test "load invalid configuration file (and fail)" do
      # Check that if we set up a bad file that load() will fail
      # Jam some rubbish into the env vars
      System.put_env("PEOPLER_CONFIG_PATH", "/tmp/")
      System.put_env("PEOPLER_CONFIG_FILE", "bozo.json")

      # Check that we can load the default configuration file
      case Peopler.CLI.Config.load() do
        # won't exec
        {:ok, config} -> assert config == nil
        # will exec
        {:error, reason} -> assert reason != nil
      end
    end

    test "inspect config property" do
      {:ok, id_from_string} = Config.inspect_property("id")
      assert id_from_string == "DOT_SLASH_DOT_LL_SLASH_CONFIG_DOT_JSON"
    end

    test "get config property" do
      id_from_string = Config.get("id")
      id_from_atom = Config.get(:id)
      assert id_from_string == "DOT_SLASH_DOT_LL_SLASH_CONFIG_DOT_JSON"
      assert id_from_atom == "DOT_SLASH_DOT_LL_SLASH_CONFIG_DOT_JSON"
    end

    test "put config property" do
      # Use a simple config file with just a timestamp attribute
      System.put_env("PEOPLER_CONFIG_PATH", "./.peopler")
      System.put_env("PEOPLER_CONFIG_FILE", "timestamp.json")

      # Make sure file exists (and empty)
      Config.config_file()
      |> Path.expand()
      |> File.write("{}")

      timestamp_in = DateTime.to_string(DateTime.utc_now())

      # Put a new value into the config and ensure that works
      case Config.put(:timestamp, timestamp_in) do
        :ok -> assert :ok
        {:error, reason} -> assert reason == false
      end

      # Grab that value back from the config and ensure that works
      case Config.get(:timestamp) do
        timestamp_out -> assert timestamp_out == timestamp_in
      end

      # Remove the file now we're done with it
      Config.config_file()
      |> File.rm()
    end
  end
end
