# The Swift engine behind pplr sync, resolve, links and open (src/pplr-contacts),
# built into .build/ on first use and again whenever its source changes.
# Source this after PPLR_ROOT is set.

engine_src="$PPLR_ROOT/src/pplr-contacts/main.swift"
engine="$PPLR_ROOT/.build/pplr-contacts"

build_engine() {
  if [ ! -x "$engine" ] || [ "$engine_src" -nt "$engine" ]; then
    command -v swiftc >/dev/null 2>&1 || { echo "Error: swiftc not found. Install the Xcode command line tools: xcode-select --install" >&2; exit 1; }
    mkdir -p "$(dirname "$engine")"
    echo "Building the Contacts engine..." >&2
    swiftc -O -swift-version 5 -o "$engine" "$engine_src" -framework Contacts >&2 || { echo "Error: could not build $engine_src" >&2; exit 1; }
  fi
}
