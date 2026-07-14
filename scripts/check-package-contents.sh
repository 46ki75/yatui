#!/usr/bin/env bash
set -euo pipefail

packages=(
  yatui-core
  yatui-text
  yatui-layout
  yatui-render
  yatui-terminal
  yatui-ui
  yatui-backend-crossterm
  yatui-runtime
  yatui-widgets
  yatui-test
  yatui
)

for package in "${packages[@]}"; do
  contents=$'\n'$(cargo package -p "$package" --list --locked --allow-dirty)$'\n'
  for required in Cargo.toml README.md LICENSE-MIT LICENSE-APACHE src/lib.rs; do
    if [[ "$contents" != *$'\n'"$required"$'\n'* ]]; then
      printf '%s\n' "$package package is missing $required" >&2
      exit 1
    fi
  done
done
