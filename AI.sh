#!/usr/bin/env bash

# AI Services Management Script
# Combines functionality from start_claude_code.sh and start_gemini_cli.sh

show_usage() {
    echo "Usage: $0 [claude|gemini]"
    echo ""
    echo "Options:"
    echo "  claude  - Start Claude with choice of API providers (KIMI K2, BIGModel, ANTHROPIC, Qwen)"
    echo "  gemini  - Start Gemini CLI with API key"
    echo ""
    echo "If no option is provided, you'll be prompted to choose."
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
            echo "Starting Claude with KIMI K2..."
            export ANTHROPIC_BASE_URL=https://api.moonshot.cn/v1
            export ANTHROPIC_AUTH_TOKEN=sk-your-kimi-api-key-here
            echo "KIMI K2 environment variables set."
            ;;
        2)
            echo "Starting Claude with BIGModel..."
            export ANTHROPIC_BASE_URL=https://open.bigmodel.cn/api/anthropic
            export ANTHROPIC_AUTH_TOKEN=41345f3d30484b7b9a2687ba4aaecc07.0klx3XqumqfEL434
            echo "BIGModel environment variables set."
            ;;
        3)
            echo "Starting Claude with ANTHROPIC (Official)..."
            export ANTHROPIC_BASE_URL=https://api.anthropic.com
            export ANTHROPIC_AUTH_TOKEN=sk-your-anthropic-api-key-here
            echo "ANTHROPIC official environment variables set."
            ;;
        4)
            echo "Starting Claude with Qwen..."
            export ANTHROPIC_BASE_URL=https://dashscope.aliyuncs.com/compatible-mode/v1
            export ANTHROPIC_AUTH_TOKEN=sk-your-qwen-api-key-here
            echo "Qwen environment variables set."
            ;;
        *)
            echo "Error: Invalid choice '$provider_choice'. Please select 1, 2, 3, or 4."
            exit 1
            ;;
    esac
    
    echo "ANTHROPIC_BASE_URL: $ANTHROPIC_BASE_URL"
}

start_gemini() {
    echo "Starting Gemini CLI..."
    export GEMINI_API_KEY="AIzaSyDVP0QOOZLLlVW7V878Dh-tQnSYSiz3NXo"
    echo "Gemini API key set."
    gemini --telemetry false
}

# Main logic
case "$1" in
    claude)
        start_claude
        ;;
    gemini)
        start_gemini
        ;;
    -h|--help)
        show_usage
        ;;
    "")
        echo "Please choose an AI service:"
        echo "1) Claude"
        echo "2) Gemini"
        read -p "Enter your choice (1-2): " choice
        case $choice in
            1)
                start_claude
                ;;
            2)
                start_gemini
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