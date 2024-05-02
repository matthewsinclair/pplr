# General application configuration
import Config

# Configure Peopler.CLI
config :peopler_cli,
  env: config_env(),
  default_config_path: "~/.peopler/",
  default_config_file: "config.json",
  config_path: System.get_env("PEOPLER_CONFIG_PATH", "~/.peopler/"),
  config_file: System.get_env("PEOPLER_CONFIG_FILE", "config.json"),
  name: "peopler_cli",
  about: "🫵 Peopler CLI",
  description: "A simple CLI-based CRM that uses plain text files and directories",
  version: "0.1.0",
  author: "hello@peopler.io",
  url: "https://peopler.io"

# Configures Elixir's Logger
config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

# Import environment specific config. This must remain at the bottom
# of this file so it overrides the configuration defined above.
import_config "#{config_env()}.exs"
