#!/usr/bin/bash
set -euo pipefail
url="$1"
output="$2"
args=(-fL --retry 3 --retry-delay 3 -o "$output" "$url")
if [[ "$url" == https://api.github.com/repos/julyx10/lap-binaries/releases/assets/* ]]; then
  args=(-H 'Accept: application/octet-stream' "${args[@]}")
fi
exec curl "${args[@]}"
