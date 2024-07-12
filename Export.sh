#!/bin/bash
# Execute this file with: `source Export.sh`

Build_tool_directory=$(realpath "Build_tool")
Build_tool_executable=$Build_tool_directory/target/release/Build_tool

# Check if the 'Build_tool' directory exists
if [ ! -d "$Build_tool_directory" ]; then
    echo "The 'Build_tool' directory does not exist. Are you in the root of the repository?"
    exit 1
fi


(
# Build the build tool
cd $Build_tool_directory && cargo build --release

if [ $? -ne 0 ]; then
    echo "Failed to build the build tool"
    exit 1
fi
)

clear && echo "Export xila build tool"

alias xila="$Build_tool_executable"

export RUST_BACKTRACE=1
