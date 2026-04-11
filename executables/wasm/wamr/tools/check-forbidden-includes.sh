#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PATTERN='#include\s*<((sys/[^>]+)|(pthread\.h)|(unistd\.h)|(arpa/inet\.h)|(poll\.h)|(netinet/[^>]+)|(syscall\.h)|(fcntl\.h))>'

if grep -R -n -E --include='*.h' --include='*.c' --include='*.cpp' "$PATTERN" "$ROOT_DIR/include" "$ROOT_DIR/src"; then
  printf '%s\n' "ERROR: forbidden host OS headers detected in wamr platform sources"
  exit 1
fi

printf '%s\n' "OK: no forbidden host OS headers detected"
