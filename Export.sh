#!/bin/bash
# Execute this file with: `source Export.sh`

Build_tool_directory=$(realpath "Build_tool")
Build_tool_executable=$Build_tool_directory/target/release/Build_tool

if [ ! -d "$Build_tool_directory" ]; then
    echo "The 'Build_tool' directory does not exist. Are you in the root of the repository?"
    exit 1
fi

if [ ! -f "$Build_tool_executable" ]; then
    echo "The 'Build_tool' executable does not exist. Did you build the project? (Run 'cargo build --release')"
    exit 1
fi

clear && echo "Export xila build tool"

alias xila="$Build_tool_executable"

export RUST_BACKTRACE=1
