#!/usr/bin/env bash
# Install gstack skills for Claude Code (run once per machine)
set -euo pipefail

GSTACK_DIR="$HOME/.claude/skills/gstack"

# Install bun if missing
if ! command -v bun &>/dev/null; then
  echo "Installing bun..."
  BUN_VERSION="1.3.10"
  tmpfile=$(mktemp)
  curl -fsSL "https://bun.sh/install" -o "$tmpfile"
  BUN_VERSION="$BUN_VERSION" bash "$tmpfile"
  rm "$tmpfile"
  export PATH="$HOME/.bun/bin:$PATH"
fi

# Clone or update gstack
if [ -d "$GSTACK_DIR" ]; then
  echo "Updating gstack..."
  git -C "$GSTACK_DIR" pull --ff-only
else
  echo "Cloning gstack..."
  git clone --single-branch --depth 1 https://github.com/garrytan/gstack.git "$GSTACK_DIR"
fi

cd "$GSTACK_DIR" && ./setup

echo ""
echo "gstack installed. Reload Claude Code to activate skills."
