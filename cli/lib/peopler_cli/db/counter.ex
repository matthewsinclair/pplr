defmodule Peopler.CLI.Db.Counter do
  @moduledoc """
  Count the number of People in the Peopler db
  """

  @ignore_dirs ~r/^_Templates$|^bin$|^index\.md$|^index\.json$/

  def count(base_dir \\ ".") do
    count =
      File.ls!(base_dir)
      |> Enum.filter(&valid_directory?(&1))
      |> Enum.map(&count_people_in_directory(&1, base_dir))
      |> Enum.sum()

    IO.puts(count)
  end

  defp valid_directory?(dir_name) do
    not Regex.match?(@ignore_dirs, dir_name)
  end

  defp count_people_in_directory(letter, base_dir) do
    letter_path = Path.join([base_dir, letter])

    case File.ls(letter_path) do
      {:ok, people} -> Enum.count(people)
      _ -> 0
    end
  end
end
