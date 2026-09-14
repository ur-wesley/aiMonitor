#!/usr/bin/env bash
set -euo pipefail

REPO="ur-wesley/aiMonitor"
TARGET="/mnt/c/Arbeit/aiMonitor"

if ! command -v origin >/dev/null 2>&1; then
  echo "Installing Origin CLI..."
  curl -fsSL https://downloads.cursor.com/origin/install.sh | sh
  export PATH="$HOME/.local/bin:$PATH"
  if ! grep -q '.local/bin' "$HOME/.bashrc" 2>/dev/null; then
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
  fi
fi

export PATH="$HOME/.local/bin:$PATH"

if ! origin auth status 2>/dev/null | grep -q 'Token:.*valid'; then
  echo "Sign in to Origin (browser opens)..."
  origin auth login
fi

origin auth setup-git

sudo mkdir -p /mnt/c/Arbeit
if [ -d "$TARGET/.git" ]; then
  echo "Repository already exists at $TARGET — pulling latest..."
  git -C "$TARGET" pull origin main
else
  echo "Cloning $REPO to $TARGET ..."
  origin repo clone "$REPO" "$TARGET"
fi

echo ""
echo "Done. Project is at: C:\\Arbeit\\aiMonitor"
echo ""
echo "Build on Windows (PowerShell):"
echo "  cd C:\\Arbeit\\aiMonitor"
echo "  dotnet publish -c Release -r win-x64 --self-contained -p:PublishReadyToRun=true"
echo "  .\\bin\\Release\\net10.0\\win-x64\\publish\\aiMonitor.exe"
