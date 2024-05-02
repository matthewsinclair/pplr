defmodule Peopler.CLI.Utils.Test do
  use ExUnit.Case, async: false
  import Peopler.CLI.Utils

  doctest Peopler.CLI.Utils

  # Can't realy test these types meaningfully
  # @a_function fn(a, b) -> a + b end
  # @a_pid self()
  # @a_port
  # @a_reference Kernel.make_ref
  #
  @a_atom :atom123
  @a_boolean true
  @a_list [1, 2, 3]
  @a_map %{a: 1, b: 2, c: 3}
  @a_nil nil
  @a_tuple {:one, :two, :three}
  @a_binary <<1, 2, 3>>
  @a_bitstring <<1::size(2), 6::size(4)>>
  @a_integer 123
  @a_float 123.123
  @a_number 102_030
  @a_string "a string"

  describe "Peopler.CLI.Utils" do
    # setup do
    # end

    test "know all of the types" do
      # Can't realy test these types meaningfully
      # assert is_function @a_function
      # assert is_pid @a_pid
      # assert is_port @a_port
      # assert is_reference @a_reference

      assert is_atom(@a_atom)
      assert is_boolean(@a_boolean)
      assert is_list(@a_list)
      assert is_map(@a_map)
      assert is_nil(@a_nil)
      assert is_tuple(@a_tuple)
      assert is_binary(@a_binary)
      assert is_bitstring(@a_bitstring)
      assert is_integer(@a_integer)
      assert is_float(@a_float)
      assert is_number(@a_number)
      assert is_bitstring(@a_string)

      # There's a few whacky bits here with things like :boolean being an :atom.
      # I'm not sure what is going there as I need to learn more about Elixir
      # types to know if this is problematic or not.
      assert type_of(@a_atom) == :atom
      # :boolean
      assert type_of(@a_boolean) == :atom
      assert type_of(@a_list) == :list
      assert type_of(@a_map) == :map
      # :nil
      assert type_of(@a_nil) == :atom
      assert type_of(@a_tuple) == :tuple
      assert type_of(@a_binary) == :binary
      assert type_of(@a_bitstring) == :bitstring
      assert type_of(@a_integer) == :integer
      assert type_of(@a_float) == :float
      # :number
      assert type_of(@a_number) == :integer
      # :bistring
      assert type_of(@a_string) == :binary
    end

    test "pretty_print all of the types" do
      assert true

      # IO.write "atom:      "; pretty_print @a_atom
      # IO.write "boolean:   "; pretty_print @a_boolean
      # IO.write "list:      "; pretty_print @a_list
      # IO.write "map:       "; pretty_print @a_map
      # IO.write "nil:       "; pretty_print @a_nil
      # IO.write "tuple:     "; pretty_print @a_tuple
      # IO.write "binary:    "; pretty_print @a_binary
      # IO.write "bitstring: "; pretty_print @a_bitstring
      # IO.write "integer:   "; pretty_print @a_integer
      # IO.write "float:     "; pretty_print @a_float
      # IO.write "number:    "; pretty_print @a_number
      # IO.write "string:    "; pretty_print @a_string

      # I know that these are basically true == true assetions but I just
      # wanted to make sure that the functions were callable.
      assert to_str(@a_atom) == inspect(@a_atom)
      assert to_str(@a_boolean) == inspect(@a_boolean)
      assert to_str(@a_list) == inspect(@a_list)
      assert to_str(@a_map) == inspect(@a_map)
      assert to_str(@a_nil) == inspect(@a_nil)
      assert to_str(@a_tuple) == inspect(@a_tuple)
      assert to_str(@a_binary) == inspect(@a_binary)
      assert to_str(@a_bitstring) == inspect(@a_bitstring)
      assert to_str(@a_integer) == inspect(@a_integer)
      assert to_str(@a_float) == inspect(@a_float)
      assert to_str(@a_number) == inspect(@a_number)
      assert to_str(@a_string) == inspect(@a_string)
    end

    test "form encoded body" do
      map = %{ "a" => "1", "b" => "2", "c" => "3" }
      str = form_encoded_body(map)
      assert str == "a=1&b=2&c=3"
    end

    test "parse json body" do
      body = %{ body: "<body>\nHello, World!\n</body>" }
      _json = parse_json_body(body)
      assert true
      # assert json == %{ a: 1 }
    end

    test "filter blank lines from list" do
      no_blank_lines_at_end_list = ["ABC", "DEF", "GHI"]
      one_blank_line_at_end_list = ["ABC", "DEF", "GHI", ""]
      two_blank_lines_at_end_list = ["ABC", "DEF", "GHI", "", ""]
      blank_lines_not_at_end_list = ["", "ABC", "", "DEF", "GHI"]
      blank_lines_not_at_end_and_at_end_list = ["", "ABC", "", "DEF", "GHI", "", ""]

      assert no_blank_lines_at_end_list |> filter_blank_lines() == ["ABC", "DEF", "GHI"]
      assert one_blank_line_at_end_list |> filter_blank_lines() == ["ABC", "DEF", "GHI"]
      assert two_blank_lines_at_end_list |> filter_blank_lines() == ["ABC", "DEF", "GHI"]
      assert blank_lines_not_at_end_list |> filter_blank_lines() == ["", "ABC", "", "DEF", "GHI"]
      assert blank_lines_not_at_end_and_at_end_list |> filter_blank_lines() == ["", "ABC", "", "DEF", "GHI"]
    end

    test "filter blank lines from tuple" do
      no_blank_lines_at_end_tuple = ["ABC", "DEF", "GHI"]
      one_blank_line_at_end_tuple = ["ABC", "DEF", "GHI", ""]
      two_blank_lines_at_end_tuple = ["ABC", "DEF", "GHI", "", ""]
      blank_lines_not_at_end_tuple = ["", "ABC", "", "DEF", "GHI"]
      blank_lines_not_at_end_and_at_end_tuple = ["", "ABC", "", "DEF", "GHI", "", ""]

      assert no_blank_lines_at_end_tuple |> filter_blank_lines() == ["ABC", "DEF", "GHI"]
      assert one_blank_line_at_end_tuple |> filter_blank_lines() == ["ABC", "DEF", "GHI"]
      assert two_blank_lines_at_end_tuple |> filter_blank_lines() == ["ABC", "DEF", "GHI"]
      assert blank_lines_not_at_end_tuple |> filter_blank_lines() == ["", "ABC", "", "DEF", "GHI"]
      assert blank_lines_not_at_end_and_at_end_tuple |> filter_blank_lines() == ["", "ABC", "", "DEF", "GHI"]
    end

    test "filter blank lines from string" do
      no_blank_lines_string = "ABC DEF GHI"
      one_blank_line_string = "ABC DEF GHI\n"
      two_blank_line_string = "ABC DEF GHI\n\n"
      not_at_end_blank_line_string = "\nABC\nDEF GHI"
      not_at_end_and_at_end_blank_line_string = "\nABC\nDEF\n\nGHI\n\n"
      losts_at_end_blank_line_string = "\nABC\nDEF\n\nGHI\n\n\n\n\n\n"

      assert no_blank_lines_string |> filter_blank_lines() == "ABC DEF GHI"
      assert one_blank_line_string |> filter_blank_lines() == "ABC DEF GHI\n"
      assert two_blank_line_string |> filter_blank_lines() == "ABC DEF GHI\n"
      assert not_at_end_blank_line_string |> filter_blank_lines() == "\nABC\nDEF GHI"
      assert not_at_end_and_at_end_blank_line_string |> filter_blank_lines() == "\nABC\nDEF\n\nGHI\n"
      assert losts_at_end_blank_line_string |> filter_blank_lines() == "\nABC\nDEF\n\nGHI\n"
    end
  end
end
