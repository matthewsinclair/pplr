#!/bin/bash

# pplr Setup Script
# This script helps set up pplr on your system

echo "pplr Setup"
echo "=========="
echo

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Default values
DEFAULT_PPLR_ROOT="$SCRIPT_DIR"
DEFAULT_PPLR_DATA="$HOME/Dropbox/Career/People"

# Ask for configuration
echo "Configuration:"
echo

read -p "pplr code directory [$DEFAULT_PPLR_ROOT]: " PPLR_ROOT
PPLR_ROOT="${PPLR_ROOT:-$DEFAULT_PPLR_ROOT}"

read -p "pplr data directory [$DEFAULT_PPLR_DATA]: " PPLR_DATA
PPLR_DATA="${PPLR_DATA:-$DEFAULT_PPLR_DATA}"

echo
echo "Setting up pplr with:"
echo "  Code directory: $PPLR_ROOT"
echo "  Data directory: $PPLR_DATA"
echo

# Create data directory structure if it doesn't exist
if [ ! -d "$PPLR_DATA" ]; then
    echo "Creating data directory structure..."
    mkdir -p "$PPLR_DATA"
    
    # Create A-Z directories
    for letter in {A..Z}; do
        mkdir -p "$PPLR_DATA/$letter"
    done
    
    echo "Data directory created."
else
    echo "Data directory already exists."
fi

# Determine shell config file
SHELL_CONFIG=""
if [ -n "$ZSH_VERSION" ]; then
    SHELL_CONFIG="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ]; then
    if [ -f "$HOME/.bash_profile" ]; then
        SHELL_CONFIG="$HOME/.bash_profile"
    else
        SHELL_CONFIG="$HOME/.bashrc"
    fi
fi

if [ -z "$SHELL_CONFIG" ]; then
    echo "Could not determine shell configuration file."
    echo "Please add the following to your shell configuration manually:"
else
    echo "Adding pplr to your shell configuration ($SHELL_CONFIG)..."
    
    # Check if pplr is already configured
    if grep -q "PPLR_ROOT" "$SHELL_CONFIG" 2>/dev/null; then
        echo "pplr appears to be already configured in $SHELL_CONFIG"
        echo "Please update the configuration manually if needed."
    else
        # Add configuration to shell
        cat >> "$SHELL_CONFIG" << EOF

# pplr - Personal Relationship Manager
export PPLR_ROOT="$PPLR_ROOT"
export PPLR_DATA="$PPLR_DATA"
export PPLR_BIN_DIR="\$PPLR_ROOT/bin"
export PPLR_TEMPLATE_DIR="\$PPLR_ROOT/templates"
export PPLR_DIR="\$PPLR_DATA"  # For backwards compatibility
export PATH="\$PATH:\$PPLR_BIN_DIR"
EOF
        echo "Configuration added to $SHELL_CONFIG"
    fi
fi

echo
echo "Setup complete!"
echo
echo "To start using pplr:"
echo "1. Reload your shell configuration:"
echo "   source $SHELL_CONFIG"
echo
echo "2. Test the installation:"
echo "   pplr help"
echo
echo "3. Create your first person:"
echo "   pplr new \"Surname\" \"Firstname\""
echo
echo "For more information, see:"
echo "   pplr help --details"