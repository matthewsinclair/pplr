defmodule Peopler.CLI.Db.JsonIndexer do
  @moduledoc """
  Generate a JSON index of the Peopler db
  """
  @ignore_patterns ~r/^_Templates$|^bin$|^index\.md$|^index\.json$|^\..*$|^Icon.?/

  def index(base_dir \\ ".") do
    people_data =
      File.ls!(base_dir)
      |> Enum.filter(&valid_directory_or_file?/1)
      |> Enum.sort()
      |> Enum.map(&process_directory(&1, base_dir))
      |> build_json()
      |> Jason.encode!()

    IO.puts(people_data)
  end

  defp valid_directory_or_file?(name) do
    not Regex.match?(@ignore_patterns, name)
  end

  defp process_directory(letter, base_dir) do
    letter_path = Path.join([base_dir, letter])

    entries =
      File.ls!(letter_path)
      |> Enum.filter(&valid_directory_or_file?/1)
      |> Enum.sort()
      |> Enum.map(&format_person(&1, letter_path))

    {letter, entries}
  end

  defp format_person(person, letter_path) do
    person_path = Path.join([letter_path, person])
    about_path = Path.join([person_path, "About"])
    meetings_path = Path.join([person_path, "Meetings"])

    %{
      name: person,
      about: get_about_details(about_path, person),
      meetings: get_meeting_details(meetings_path)
    }
  end

  defp get_about_details(about_path, person_name) do
    %{
      about: Path.join([about_path, "#{person_name} (About).md"]),
      linkedin: Path.join([about_path, "#{person_name} (LinkedIn).webloc"]),
      picture: Path.join([about_path, "#{person_name} (Picture).png"]),
      profile: Path.join([about_path, "#{person_name} (Profile).pdf"])
    }
  end

  defp get_meeting_details(meetings_path) do
    File.ls!(meetings_path)
    |> Enum.filter(&valid_directory_or_file?/1)
    |> Enum.map(&format_meeting(&1, meetings_path))
  end

  defp format_meeting(meeting, meetings_path) do
    %{
      id: meeting,
      details: Path.join([meetings_path, meeting, "#{meeting}.md"])
    }
  end

  defp build_json(data) do
    %{"people" => data}
  end
end
