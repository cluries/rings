#!/usr/bin/env bash

# AI Services Management Script
# Combines functionality from start_claude_code.sh and start_gemini_cli.sh

# Configuration file path
CONFIG_FILE="$HOME/.ai_config"

show_usage() {
    echo "Usage: $0 [claude|gemini|check|update|config] [provider]"
    echo ""
    echo "Options:"
    echo "  claude [provider]  - Start Claude with optional provider specification"
    echo "                      Providers: kimi, bigmodel, anthropic, qwen"
    echo "  gemini            - Start Gemini CLI with API key"
    echo "  check             - Check installation status of Claude Code and Gemini CLI"
    echo "  update            - Update Claude Code and Gemini CLI to latest versions"
    echo "  config            - Configure API keys and settings"
    echo ""
    echo "Examples:"
    echo "  $0 claude kimi      # Start Claude with KIMI K2"
    echo "  $0 claude bigmodel  # Start Claude with BIGModel"
    echo "  $0 claude anthropic # Start Claude with ANTHROPIC"
    echo "  $0 claude qwen      # Start Claude with Qwen"
    echo "  $0 claude           # Interactive provider selection"
    echo "  $0 config           # Configure API keys"
    echo ""
    echo "Configuration:"
    echo "  API keys are stored in: $CONFIG_FILE"
    echo "  You can also set environment variables:"
    echo "    KIMI_API_KEY, BIGMODEL_API_KEY, ANTHROPIC_API_KEY, QWEN_API_KEY, GEMINI_API_KEY"
    echo ""
    echo "If no option is provided, you'll be prompted to choose."
}

# Load configuration from file
load_config() {
    if [ -f "$CONFIG_FILE" ]; then
        source "$CONFIG_FILE"
    fi
}

# Get API key for a provider
get_api_key() {
    local provider="$1"
    local key=""
    
    case "$provider" in
        kimi)
            key="${KIMI_API_KEY:-}"
            ;;
        bigmodel)
            key="${BIGMODEL_API_KEY:-}"
            ;;
        anthropic)
            key="${ANTHROPIC_API_KEY:-}"
            ;;
        qwen)
            key="${QWEN_API_KEY:-}"
            ;;
        gemini)
            key="${GEMINI_API_KEY:-}"
            ;;
    esac
    
    echo "$key"
}

# Configure API keys
configure_api_keys() {
    echo "=== API Key Configuration ==="
    echo "Configure your API keys. Press Enter to skip a provider."
    echo ""
    
    # Create or backup existing config
    if [ -f "$CONFIG_FILE" ]; then
        cp "$CONFIG_FILE" "${CONFIG_FILE}.backup"
        echo "Existing config backed up to ${CONFIG_FILE}.backup"
    fi
    
    # Configure each provider
    echo "# AI Services API Configuration" > "$CONFIG_FILE"
    echo "# Generated on $(date)" >> "$CONFIG_FILE"
    echo "" >> "$CONFIG_FILE"
    
    read -p "KIMI K2 API Key: " kimi_key
    if [ -n "$kimi_key" ]; then
        echo "export KIMI_API_KEY=\"$kimi_key\"" >> "$CONFIG_FILE"
    fi
    
    read -p "BIGModel API Key: " bigmodel_key
    if [ -n "$bigmodel_key" ]; then
        echo "export BIGMODEL_API_KEY=\"$bigmodel_key\"" >> "$CONFIG_FILE"
    fi
    
    read -p "ANTHROPIC API Key: " anthropic_key
    if [ -n "$anthropic_key" ]; then
        echo "export ANTHROPIC_API_KEY=\"$anthropic_key\"" >> "$CONFIG_FILE"
    fi
    
    read -p "Qwen API Key: " qwen_key
    if [ -n "$qwen_key" ]; then
        echo "export QWEN_API_KEY=\"$qwen_key\"" >> "$CONFIG_FILE"
    fi
    
    read -p "Gemini API Key: " gemini_key
    if [ -n "$gemini_key" ]; then
        echo "export GEMINI_API_KEY=\"$gemini_key\"" >> "$CONFIG_FILE"
    fi
    
    echo ""
    echo "Configuration saved to: $CONFIG_FILE"
    echo "You can edit this file manually or run '$0 config' again to reconfigure."
}

start_claude_with_provider() {
    local provider="$1"
    local api_key=""
    
    # Load configuration
    load_config
    
    case "$provider" in
        kimi|k2)
            echo "Starting Claude with KIMI K2..."
            export ANTHROPIC_BASE_URL=https://api.moonshot.cn/v1
            api_key=$(get_api_key "kimi")
            if [ -z "$api_key" ]; then
                echo "Error: KIMI API key not found!"
                echo "Please run '$0 config' to configure your API keys or set KIMI_API_KEY environment variable."
                exit 1
            fi
            export ANTHROPIC_AUTH_TOKEN="$api_key"
            echo "KIMI K2 environment variables set."
            ;;
        bigmodel|big)
            echo "Starting Claude with BIGModel..."
            export ANTHROPIC_BASE_URL=https://open.bigmodel.cn/api/anthropic
            api_key=$(get_api_key "bigmodel")
            if [ -z "$api_key" ]; then
                echo "Error: BIGModel API key not found!"
                echo "Please run '$0 config' to configure your API keys or set BIGMODEL_API_KEY environment variable."
                exit 1
            fi
            export ANTHROPIC_AUTH_TOKEN="$api_key"
            echo "BIGModel environment variables set."
            ;;
        anthropic|official)
            echo "Starting Claude with ANTHROPIC (Official)..."
            export ANTHROPIC_BASE_URL=https://api.anthropic.com
            api_key=$(get_api_key "anthropic")
            if [ -z "$api_key" ]; then
                echo "Error: ANTHROPIC API key not found!"
                echo "Please run '$0 config' to configure your API keys or set ANTHROPIC_API_KEY environment variable."
                exit 1
            fi
            export ANTHROPIC_AUTH_TOKEN="$api_key"
            echo "ANTHROPIC official environment variables set."
            ;;
        qwen)
            echo "Starting Claude with Qwen..."
            export ANTHROPIC_BASE_URL=https://dashscope.aliyuncs.com/compatible-mode/v1
            api_key=$(get_api_key "qwen")
            if [ -z "$api_key" ]; then
                echo "Error: Qwen API key not found!"
                echo "Please run '$0 config' to configure your API keys or set QWEN_API_KEY environment variable."
                exit 1
            fi
            export ANTHROPIC_AUTH_TOKEN="$api_key"
            echo "Qwen environment variables set."
            ;;
        *)
            echo "Error: Unknown provider '$provider'"
            echo "Available providers: kimi, bigmodel, anthropic, qwen"
            exit 1
            ;;
    esac
    
    echo "ANTHROPIC_BASE_URL: $ANTHROPIC_BASE_URL"
    echo "API key loaded successfully."
}

start_claude() {
    echo "Choose Claude API provider:"
    echo "1) KIMI K2"
    echo "2) BIGModel"
    echo "3) ANTHROPIC (Official)"
    echo "4) Qwen"
    read -p "Enter your choice (1-4): " provider_choice
    
    case $provider_choice in
        1)
            start_claude_with_provider "kimi"
            ;;
        2)
            start_claude_with_provider "bigmodel"
            ;;
        3)
            start_claude_with_provider "anthropic"
            ;;
        4)
            start_claude_with_provider "qwen"
            ;;
        *)
            echo "Error: Invalid choice '$provider_choice'. Please select 1, 2, 3, or 4."
            exit 1
            ;;
    esac
}

start_gemini() {
    echo "Starting Gemini CLI..."
    
    # Load configuration
    load_config
    
    local api_key=$(get_api_key "gemini")
    if [ -z "$api_key" ]; then
        echo "Error: Gemini API key not found!"
        echo "Please run '$0 config' to configure your API keys or set GEMINI_API_KEY environment variable."
        exit 1
    fi
    
    export GEMINI_API_KEY="$api_key"
    echo "Gemini API key loaded successfully."
    gemini --telemetry false
}

check_installations() {
    echo "Checking AI tools installation status..."
    echo ""
    
    # Check Claude Code
    echo "=== Claude Code ==="
    if command -v claude &> /dev/null; then
        echo "✓ Claude Code is installed"
        claude --version 2>/dev/null || echo "  Version: Unable to determine"
    else
        echo "✗ Claude Code is not installed"
        echo "  Install with: npm install -g @anthropic-ai/claude-3-cli"
    fi
    echo ""
    
    # Check Gemini CLI
    echo "=== Gemini CLI ==="
    if command -v gemini &> /dev/null; then
        echo "✓ Gemini CLI is installed"
        gemini --version 2>/dev/null || echo "  Version: Unable to determine"
    else
        echo "✗ Gemini CLI is not installed"
        echo "  Install with: npm install -g @google/generative-ai-cli"
    fi
    echo ""
    
    # Check Node.js (required for both)
    echo "=== Node.js ==="
    if command -v node &> /dev/null; then
        echo "✓ Node.js is installed"
        echo "  Version: $(node --version)"
    else
        echo "✗ Node.js is not installed (required for both tools)"
        echo "  Install from: https://nodejs.org/"
    fi
    echo ""
    
    # Check npm
    echo "=== npm ==="
    if command -v npm &> /dev/null; then
        echo "✓ npm is installed"
        echo "  Version: $(npm --version)"
    else
        echo "✗ npm is not installed (required for installation)"
    fi
}

update_tools() {
    echo "Updating AI tools..."
    echo ""
    
    # Check if npm is available
    if ! command -v npm &> /dev/null; then
        echo "Error: npm is not installed. Please install Node.js and npm first."
        exit 1
    fi
    
    # Update Claude Code
    echo "=== Updating Claude Code ==="
    if command -v claude &> /dev/null; then
        echo "Updating existing Claude Code installation..."
        npm update -g @anthropic-ai/claude-3-cli
    else
        echo "Claude Code not found. Installing..."
        npm install -g @anthropic-ai/claude-3-cli
    fi
    echo ""
    
    # Update Gemini CLI
    echo "=== Updating Gemini CLI ==="
    if command -v gemini &> /dev/null; then
        echo "Updating existing Gemini CLI installation..."
        npm update -g @google/generative-ai-cli
    else
        echo "Gemini CLI not found. Installing..."
        npm install -g @google/generative-ai-cli
    fi
    echo ""
    
    echo "Update completed! Run '$0 check' to verify installations."
}

# Main logic
case "$1" in
    claude)
        if [ -n "$2" ]; then
            # Direct provider specification
            start_claude_with_provider "$2"
        else
            # Interactive selection
            start_claude
        fi
        ;;
    gemini)
        start_gemini
        ;;
    check)
        check_installations
        ;;
    update)
        update_tools
        ;;
    config)
        configure_api_keys
        ;;
    -h|--help)
        show_usage
        ;;
    "")
        echo "Please choose an option:"
        echo "1) Claude"
        echo "2) Gemini"
        echo "3) Check installations"
        echo "4) Update tools"
        echo "5) Configure API keys"
        read -p "Enter your choice (1-5): " choice
        case $choice in
            1)
                start_claude
                ;;
            2)
                start_gemini
                ;;
            3)
                check_installations
                ;;
            4)
                update_tools
                ;;
            5)
                configure_api_keys
                ;;
            *)
                echo "Invalid choice. Please run with --help for usage information."
                exit 1
                ;;
        esac
        ;;
    *)
        echo "Unknown option: $1"
        show_usage
        exit 1
        ;;
esac