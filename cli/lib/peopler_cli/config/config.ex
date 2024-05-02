defmodule Peopler.CLI.Config do
  @moduledoc """
  Provides a simple programmatic API to a set of configuration properties held in a JSON config file.
  """

  @config_path_env_var "PEOPLER_CONFIG_PATH"
  @config_file_env_var "PEOPLER_CONFIG_FILE"

  @doc """
  Loads configuration file defaults to "config.json"
  """
  def load(config_file \\ config_file()) do
    config_file
    |> Path.expand()
    |> File.read()
    |> case do
      {:ok, content} ->
        case content do
          "" -> "{}"
          _ -> content
        end
        |> Jason.decode()
        |> case do
          {:ok, result} ->
            {:ok, result}

          {:error, %{position: position, token: token, data: _data}} ->
            {:error,
             "Error parsing config: file: #{config_file} at position: #{position}, token: '#{token}'"}
        end

      {:error, reason} ->
        {:error, "Failed to load config file: #{reason}"}
    end
  end

  @doc """
  Get fully-qualified path name of config file (as string)
  """
  def config_file do
    Path.join(config_pathname(), config_filename())
  end

  @doc """
  Get path for config file (as string), try to pull from env: PEOPLER_CONFIG_PATH
  """
  def config_pathname do
    System.get_env(@config_path_env_var) ||
      Application.get_env(:peopler_cli, :config_path) ||
      default_config_path()
  end

  @doc """
  Get path for data files (as string).
  """
  def config_data_pathname do
    Enum.join([config_pathname(), "data", "links"], "/")
  end

  @doc """
  Get name of config file (as string), try to pull from env: PEOPLER_CONFIG_FILE
  """
  def config_filename do
    System.get_env(@config_file_env_var) ||
      Application.get_env(:peopler_cli, :config_file) ||
      default_config_file()
  end

  @doc """
  Inspects a configuration property by name.b
  """
  def inspect_property(name) do
    load()
    |> case do
      {:ok, config} ->
        case Map.get(config, name) do
          nil ->
            {:error, "No such property: #{inspect(name)}"}

          value ->
            {:ok, value}
        end

      {:error, reason} ->
        {:error, reason}
    end
  end

  @doc """
  Provides a map-style accessor to the configuration properties.
  """
  def get(key) do
    load()
    |> case do
      {:ok, config} ->
        String.split(to_string(key), ".")
        |> List.foldl(config, &Map.get(&2, &1))

      {:error, reason} ->
        {:error, reason}

      nil ->
        {:error, "'#{key}' not found"}
    end
  end

  @doc """
  Update a value in the config and save the config file.
  """
  def put(key, value) do
    load()
    |> case do
      {:ok, config} ->
        keys = String.split(to_string(key), ".")
        {parent_keys, [last_key]} = Enum.split(keys, -1)

        parent =
          Enum.reduce(parent_keys, config, fn k, acc ->
            Map.get(acc, k, %{})
          end)

        updated_parent = Map.put(parent, last_key, value)

        updated_config =
          Enum.reverse(parent_keys)
          |> Enum.reduce(updated_parent, fn k, acc ->
            Map.put(%{}, k, acc)
          end)

        write_config(updated_config)

      {:error, reason} ->
        {:error, reason}
    end
  end

  # Write the config to disk using default config path (if not specified).
  defp write_config(config, config_file \\ config_file()) do
    config_file
    |> Path.expand()
    |> File.write(Jason.encode!(config, pretty: true))
    |> case do
      :ok -> :ok
      {:error, reason} -> {:error, reason}
    end
  end

  defp default_config_path do
    Application.get_env(:peopler_cli, :default_config_path, "~/.peopler/")
  end

  defp default_config_file do
    Application.get_env(:peopler_cli, :default_config_file, "config.json")
  end

  @doc """
  What OpenAI model to use?
  """
  def openai_gpt_model do
    Application.get_env(:peopler_cli, :openai_gpt_model, "gpt-4-0125-preview")
  end
end
