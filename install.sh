#!/usr/bin/env bash

# Setup script to install git-triage utility and configure the Git alias.
# Installs to default user binary path ~/.local/bin

set -euo pipefail

BIN_DIR="$HOME/.local/bin"
TARGET_NAME="git-triage"
SCRIPT_PATH="$(pwd)/triage_diff.py"

# Ensure bin directory exists
mkdir -p "${BIN_DIR}"

# Copy script to the destination
cp "${SCRIPT_PATH}" "${BIN_DIR}/${TARGET_NAME}"
chmod +x "${BIN_DIR}/${TARGET_NAME}"

# Configure git alias globally pointing to the user binary path
git config --global alias.triage "!${BIN_DIR}/${TARGET_NAME}"

echo -e "\033[0;32m[SUCCESS] Installed git-triage to ${BIN_DIR}/${TARGET_NAME}\033[0m"
echo -e "\033[0;32m[SUCCESS] Configured global Git alias 'git triage' -> !${BIN_DIR}/${TARGET_NAME}\033[0m"

# Check if BIN_DIR is in PATH
if [[ ":$PATH:" != *":${BIN_DIR}:"* ]]; then
    echo -e "\033[0;33m[WARNING] ${BIN_DIR} is not in your \$PATH.\033[0m"
    echo "To add it, append the following line to your ~/.bashrc or ~/.profile:"
    echo -e "  \033[1;37mexport PATH=\"\$HOME/.local/bin:\$PATH\"\033[0m"
fi

echo -e "\nUsage: \033[1;36mgit triage [target_branch] [source_branch]\033[0m"
echo "If no arguments are provided, it will launch the interactive selection menu."
