#!/usr/bin/env bats

# pplr's project commands (ST0002 AC-08.1): every script that was in tools/ is
# a devbin project command in bin/.devbin/cmd/, reached through bin/devbin,
# and tools/ is gone. The commands are read from the directory, so one added
# there is held to the same rule without this file changing.

load test_helper

@test "AT-08.1 every project command in bin/.devbin/cmd resolves through bin/devbin from its handler, and tools/ is gone" {
    [ ! -e "$PPLR_ROOT/tools" ] || { echo "tools/ is still there" >&2; return 1; }

    run bash -c "cd '$PPLR_ROOT' && bin/devbin help --why"
    [ "$status" -eq 0 ] || { echo "bin/devbin help --why exited $status: $output" >&2; return 1; }

    local handler name checked=0
    for handler in "$PPLR_ROOT"/bin/.devbin/cmd/*; do
        # A <command>.d directory holds sub-commands, not a command
        [ -f "$handler" ] || continue
        name=${handler##*/}
        [ -x "$handler" ] || { echo "bin/.devbin/cmd/$name is not executable, so nothing resolves to it" >&2; return 1; }
        # The whole line, so one command's handler never answers for another's
        printf '%s\n' "$output" | sed 's/^[[:space:]]*//' | grep -Fqx -- "from: project handler bin/.devbin/cmd/$name" ||
            { echo "bin/devbin $name does not resolve from its handler" >&2; return 1; }
        checked=$((checked + 1))
    done
    [ "$checked" -gt 0 ] || { echo "bin/.devbin/cmd/ holds no command to check" >&2; return 1; }
}
