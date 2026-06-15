#!/usr/bin/env bash
set -euo pipedir
cd "$(git rev-parse --show-toplevel)"
if [ -f lefthook.yml ]; then
  npx lefthook install 2>/dev/null || true
fi
