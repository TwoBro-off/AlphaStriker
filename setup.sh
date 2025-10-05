#!/bin/bash
# This script acts as a simple launcher for the main AI setup agent.

# Find the directory where the script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

PYTHON_SCRIPT_PATH="$SCRIPT_DIR/solana-ai-trading-bot/auto_setup.py"

# Check if the python script exists
if [ ! -f "$PYTHON_SCRIPT_PATH" ]; then
    echo -e "\033[91mERROR: Main setup script not found at $PYTHON_SCRIPT_PATH\033[0m"
    exit 1
fi

# Ensure python3 is available
if ! command -v python3 &> /dev/null
then
    echo -e "\033[91mERROR: python3 could not be found. Please install Python 3.9 or higher.\033[0m"
    exit 1
fi

# Run the main setup script
echo -e "\033[95m--- Launching AlphaStriker AI Setup Agent --- \033[0m"
python3 "$PYTHON_SCRIPT_PATH"
