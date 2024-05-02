defmodule Peopler.CLI.Utils do
  use OK.Pipe
  require Jason

  def form_encoded_body(params) do
    params
    |> Enum.map(fn {key, value} -> key <> "=" <> value end)
    |> Enum.join("&")
  end

  def parse_json_body(response) do
    response
    |> Map.fetch(:body)
    ~>> Jason.decode(%{keys: :atoms})
  end

  def return(x) do
    {:ok, x}
  end

  def with_default(x, default) do
    case x do
      {:ok, ok} -> ok
      {:error, _} -> default
    end
  end

  def to_url_link(url) do
    "#{IO.ANSI.light_cyan()}#{IO.ANSI.underline()}\e]8;;#{url}\a#{url}\e]8;;\a#{IO.ANSI.reset()}"
  end

  def print_ansi(to_print) when is_map(to_print) do
    to_print |> to_str |> print_ansi
  end

  def print_ansi(to_print) when is_nil(to_print) do
    # to_print |> to_str |> print_ansi
    "nil" |> print_ansi
  end

  def print_ansi(to_print) when is_atom(to_print) do
    to_print |> to_str |> print_ansi
  end

  def print_ansi(to_print) when is_tuple(to_print) do
    to_print |> to_str |> print_ansi
  end

  def print_ansi(to_print) do
    to_print |> IO.ANSI.format() |> IO.puts()
  end

  def pretty_print(term) do
    IO.puts(to_str(term))
  end

  def put_lines(lines) when is_list(lines) do
    Enum.map(lines, &print_ansi/1)
  end

  def put_lines(map) when is_map(map) do
    # Enum.map(map, &print_ansi/1)
    map |> IO.inspect()
  end

  def put_lines(tpl) when is_tuple(tpl) do
    tpl |> IO.inspect()
  end

  def put_lines(string) when is_binary(string) do
    string |> String.trim() |> print_ansi
  end

  def put_lines(isnil) when is_nil(isnil) do
    "nil" |> print_ansi
  end

  def put_lines(isatom) when is_atom(isatom) do
    unless isatom == :ok, do: to_string(isatom) |> print_ansi
  end

  # defp to_str(term), do: inspect(term, pretty: true, limit: :infinity)
  def to_str(term) when is_atom(term) do
    inspect(term, pretty: true, limit: :infinity)
  end

  def to_str(term) when is_boolean(term) do
    inspect(term, pretty: true, limit: :infinity)
  end

  def to_str(term) when is_function(term) do
    inspect(term, pretty: true, limit: :infinity)
  end

  def to_str(term) when is_list(term) do
    inspect(term, pretty: true, limit: :infinity)
  end

  def to_str(term) when is_map(term) do
    inspect(term, pretty: true, limit: :infinity)
  end

  def to_str(term) when is_nil(term) do
    inspect(term, pretty: true, limit: :infinity)
  end

  def to_str(term) when is_pid(term) do
    inspect(term, pretty: true, limit: :infinity)
  end

  def to_str(term) when is_port(term) do
    inspect(term, pretty: true, limit: :infinity)
  end

  def to_str(term) when is_reference(term) do
    inspect(term, pretty: true, limit: :infinity)
  end

  def to_str(term) when is_tuple(term) do
    inspect(term, pretty: true, limit: :infinity)
  end

  def to_str(term) when is_binary(term) do
    inspect(term, pretty: true, limit: :infinity)
  end

  def to_str(term) when is_bitstring(term) do
    inspect(term, pretty: true, limit: :infinity)
  end

  def to_str(term) when is_integer(term) do
    inspect(term, pretty: true, limit: :infinity)
  end

  def to_str(term) when is_float(term) do
    inspect(term, pretty: true, limit: :infinity)
  end

  def to_str(term) when is_number(term) do
    inspect(term, pretty: true, limit: :infinity)
  end

  def type_of(term) do
    cond do
      is_atom(term) -> :atom
      is_boolean(term) -> :boolean
      is_function(term) -> :function
      is_list(term) -> :list
      is_map(term) -> :map
      is_nil(term) -> nil
      is_pid(term) -> :pid
      is_port(term) -> :port
      is_reference(term) -> :reference
      is_tuple(term) -> :tuple
      is_binary(term) -> :binary
      is_bitstring(term) -> :bitstring
      is_integer(term) -> :integer
      is_float(term) -> :float
      is_number(term) -> :number
      true -> :error
    end
  end

  def is_blank?(data) when is_binary(data), do: is_nil(data) || Regex.match?(~r/\A\s*\z/, data)
  def is_blank?(list) when is_list(list), do: is_nil(list) || length(list) > 0
  def is_blank?(tuple) when is_tuple(tuple), do: is_nil(tuple) || tuple_size(tuple) > 0
  def is_blank?(map) when is_map(map), do: is_nil(map) || map |> Map.keys() |> length > 0
  def is_blank?(thing), do: is_nil(thing)

  @doc """
  Removes blank lines from the end of a list where the list is of the form: ["Thing", "Other Thing", "", ""] which would result in the list ["Thing", "Other Thing"]. Blanks in the middle of the list are left alone. Works for Lists, Tuples, and Maps (by running to_list/1 on them).

  If the passed in value is a string, it will remove any double-newlines ("\n\n") from the end of the string.
  """
  def filter_blank_lines(lines)

  def filter_blank_lines(lines) when is_list(lines) do
    lines |> Enum.reverse() |> Enum.drop_while(fn line -> is_blank?(line) end) |> Enum.reverse()
  end

  def filter_blank_lines(lines) when is_tuple(lines) do
    lines |> Tuple.to_list() |> filter_blank_lines
  end

  def filter_blank_lines(map) when is_map(map), do: map

  def filter_blank_lines(atom) when is_atom(atom), do: atom

  def filter_blank_lines(string) when is_binary(string) do
    newstr = String.replace(string, ~r/\n\n$/, "\n")

    case newstr == string do
      true -> newstr
      false -> filter_blank_lines(newstr)
    end
  end
end
