#!/bin/bash

set -e

# Sample banner
cat << "EOF"
▗▄▄▖  ▗▄▖ ▗▖ ▗▖▗▄▄▄▖▗▄▄▄▖▗▄▄▖      ▗▄▖  ▗▄▄▖▗▄▄▄▖▗▖  ▗▖▗▄▄▄▖
▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌  █  ▐▌   ▐▌ ▐▌    ▐▌ ▐▌▐▌   ▐▌   ▐▛▚▖▐▌  █
▐▛▀▚▖▐▌ ▐▌▐▌ ▐▌  █  ▐▛▀▀▘▐▛▀▚▖    ▐▛▀▜▌▐▌▝▜▌▐▛▀▀▘▐▌ ▝▜▌  █
▐▌ ▐▌▝▚▄▞▘▝▚▄▞▘  █  ▐▙▄▄▖▐▌ ▐▌    ▐▌ ▐▌▝▚▄▞▘▐▙▄▄▖▐▌  ▐▌  █
                 By Vihas Methnula :)
EOF

echo ""
echo "Starting installation..."

# ── Preflight: cargo must be installed ───────────────────────────────────────
if ! command -v cargo >/dev/null 2>&1; then
  echo "Cargo not found."
  echo "Please install Rust (https://rustup.rs) before continuing..."
  exit 1
fi

# ── Preflight: git must be installed (we use it for clone/pull) ──────────────
if ! command -v git >/dev/null 2>&1; then
  echo "Git not found."
  echo "Please install git before continuing..."
  exit 1
fi

REPO_URL="https://github.com/VihasMethnula/RouterAgent.git"
REPO_DIR="$HOME/RouterAgent"

# ── Navigate to $HOME so the repo is always installed in a known place ───────
cd "$HOME"

# ── Clone or update the repo ────────────────────────────────────────────────
if [ -d "$REPO_DIR" ]; then
  echo "Existing repository found at $REPO_DIR."
  cd "$REPO_DIR"

  # Verify the existing directory is actually a git repo (not an empty folder
  # or unrelated directory that happens to share the name).
  if [ ! -d ".git" ]; then
    echo "Error: $REPO_DIR exists but is not a git repository."
    echo "Move or remove it, then re-run this script."
    exit 1
  fi

  echo "Pulling the latest changes (fast-forward only)..."
  # --ff-only refuses to merge if the local branch has diverged, so we
  # never silently clobber local commits - the user can resolve manually.
  if ! git pull --ff-only; then
    echo "Error: 'git pull --ff-only' failed."
    echo "Your local branch may have diverged from origin. Resolve manually,"
    echo "then re-run this script."
    exit 1
  fi
else
  echo "Cloning the repository into $REPO_DIR..."
  if ! git clone "$REPO_URL" "$REPO_DIR"; then
    echo "Cloning failed."
    exit 1
  fi
  cd "$REPO_DIR"
fi

# ── Install the binary ──────────────────────────────────────────────────────
# The Cargo project lives in router-agent/ subdirectory of the repo.
echo "Installing the 'router' binary (this may take a few minutes)..."
if cargo install --path router-agent; then
  echo ""
  echo "Installation completed successfully."
  echo "Run 'router' to start the dashboard, or see README.md for the"
  echo "systemd service instructions."
else
  echo "Cargo install failed."
  exit 1
fi
