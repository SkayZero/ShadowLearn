#!/bin/bash
# ShadowLearn CLI Tool
# Command-line interface for managing ShadowLearn

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_NAME="ShadowLearn"
VERSION="1.0.0"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Helper functions
print_header() {
    echo -e "${PURPLE}╔════════════════════════════════════════╗${NC}"
    echo -e "${PURPLE}║${NC}   🌑 ShadowLearn CLI v${VERSION}      ${PURPLE}║${NC}"
    echo -e "${PURPLE}╚════════════════════════════════════════╝${NC}"
    echo ""
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${CYAN}ℹ${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# Command functions
cmd_status() {
    print_info "Checking ShadowLearn status..."

    # Check if process is running
    if pgrep -x "${APP_NAME}" > /dev/null; then
        print_success "ShadowLearn is running"

        # Try to get PID
        PID=$(pgrep -x "${APP_NAME}")
        print_info "PID: $PID"

        # Get memory usage
        if command -v ps &> /dev/null; then
            MEM=$(ps -p $PID -o rss= | awk '{print int($1/1024)" MB"}')
            print_info "Memory: $MEM"
        fi
    else
        print_warning "ShadowLearn is not running"
        return 1
    fi
}

cmd_stats() {
    print_info "Fetching productivity stats..."

    # Check if jq is installed
    if ! command -v jq &> /dev/null; then
        print_warning "jq is not installed. Install jq for formatted output."
        print_info "Stats data stored in: ~/.local/share/ShadowLearn/"
        return 1
    fi

    # Look for stats in data directory
    DATA_DIR="${HOME}/.local/share/ShadowLearn"
    if [ -d "$DATA_DIR" ]; then
        print_success "Data directory found: $DATA_DIR"

        # List databases
        if ls "$DATA_DIR"/*.db &> /dev/null; then
            print_info "Databases:"
            ls -lh "$DATA_DIR"/*.db | awk '{print "  - " $9 " (" $5 ")"}'
        fi
    else
        print_warning "Data directory not found"
    fi
}

cmd_plugins() {
    PLUGIN_DIR="${HOME}/.local/share/ShadowLearn/plugins"

    print_info "Plugin directory: $PLUGIN_DIR"
    echo ""

    if [ ! -d "$PLUGIN_DIR" ]; then
        print_warning "Plugin directory doesn't exist. Creating..."
        mkdir -p "$PLUGIN_DIR"
        print_success "Created: $PLUGIN_DIR"
        return 0
    fi

    # Count plugins
    PLUGIN_COUNT=$(find "$PLUGIN_DIR" -mindepth 1 -maxdepth 1 -type d | wc -l)

    if [ "$PLUGIN_COUNT" -eq 0 ]; then
        print_info "No plugins installed"
        echo ""
        print_info "To create a plugin, run: $0 plugin create <plugin-name>"
    else
        print_success "Found $PLUGIN_COUNT plugin(s):"
        echo ""

        for plugin_dir in "$PLUGIN_DIR"/*/; do
            if [ -d "$plugin_dir" ]; then
                plugin_name=$(basename "$plugin_dir")
                manifest="$plugin_dir/plugin.json"

                if [ -f "$manifest" ]; then
                    if command -v jq &> /dev/null; then
                        name=$(jq -r '.metadata.name // "Unknown"' "$manifest")
                        version=$(jq -r '.metadata.version // "0.0.0"' "$manifest")
                        description=$(jq -r '.metadata.description // "No description"' "$manifest")

                        echo -e "${BLUE}  📦 $name${NC} ${YELLOW}v$version${NC}"
                        echo -e "     $description"
                        echo -e "     ${CYAN}Path:${NC} $plugin_dir"
                    else
                        echo -e "${BLUE}  📦 $plugin_name${NC}"
                        echo -e "     ${CYAN}Path:${NC} $plugin_dir"
                    fi
                else
                    echo -e "${YELLOW}  ⚠ $plugin_name${NC} ${RED}(missing plugin.json)${NC}"
                fi
                echo ""
            fi
        done
    fi
}

cmd_plugin_create() {
    if [ -z "$1" ]; then
        print_error "Plugin name required"
        echo "Usage: $0 plugin create <plugin-name>"
        return 1
    fi

    PLUGIN_NAME="$1"
    PLUGIN_DIR="${HOME}/.local/share/ShadowLearn/plugins/$PLUGIN_NAME"

    if [ -d "$PLUGIN_DIR" ]; then
        print_error "Plugin already exists: $PLUGIN_DIR"
        return 1
    fi

    print_info "Creating plugin: $PLUGIN_NAME"
    mkdir -p "$PLUGIN_DIR"

    # Create plugin.json
    cat > "$PLUGIN_DIR/plugin.json" <<EOF
{
  "metadata": {
    "id": "$PLUGIN_NAME",
    "name": "$PLUGIN_NAME",
    "version": "1.0.0",
    "author": "$USER",
    "description": "A custom ShadowLearn plugin"
  },
  "config": {
    "hooks": [
      {
        "name": "on_suggestion",
        "description": "Triggered when a suggestion is shown",
        "action": {
          "type": "script",
          "command": "on_suggestion.sh",
          "args": []
        }
      }
    ],
    "permissions": ["notifications"]
  }
}
EOF

    # Create example hook script
    cat > "$PLUGIN_DIR/on_suggestion.sh" <<'EOF'
#!/bin/bash
# Example plugin hook
echo "🔌 Plugin triggered!"
echo "Context: $SHADOWLEARN_CONTEXT"
EOF

    chmod +x "$PLUGIN_DIR/on_suggestion.sh"

    print_success "Plugin created: $PLUGIN_DIR"
    echo ""
    print_info "Edit plugin.json to configure your plugin"
    print_info "Edit on_suggestion.sh to add custom logic"
}

cmd_logs() {
    LOG_FILE="/tmp/shadowlearn_dev.log"

    if [ ! -f "$LOG_FILE" ]; then
        print_warning "Log file not found: $LOG_FILE"
        print_info "Make sure ShadowLearn is running in dev mode"
        return 1
    fi

    print_info "Tailing logs from: $LOG_FILE"
    echo ""

    tail -n 50 -f "$LOG_FILE"
}

cmd_clean() {
    print_warning "This will delete all ShadowLearn data!"
    read -p "Are you sure? (y/N) " -n 1 -r
    echo

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Cancelled"
        return 0
    fi

    DATA_DIR="${HOME}/.local/share/ShadowLearn"

    if [ -d "$DATA_DIR" ]; then
        print_info "Removing: $DATA_DIR"
        rm -rf "$DATA_DIR"
        print_success "Data cleaned"
    else
        print_info "No data to clean"
    fi
}

cmd_help() {
    cat <<EOF
${CYAN}ShadowLearn CLI${NC} - Command-line interface for ShadowLearn

${YELLOW}USAGE:${NC}
    shadowlearn [COMMAND] [OPTIONS]

${YELLOW}COMMANDS:${NC}
    ${GREEN}status${NC}              Check if ShadowLearn is running
    ${GREEN}stats${NC}               Show productivity statistics
    ${GREEN}plugins${NC}             List installed plugins
    ${GREEN}plugin create${NC}       Create a new plugin
    ${GREEN}logs${NC}                Tail application logs
    ${GREEN}clean${NC}               Clean all application data
    ${GREEN}help${NC}                Show this help message
    ${GREEN}version${NC}             Show version information

${YELLOW}EXAMPLES:${NC}
    shadowlearn status
    shadowlearn stats
    shadowlearn plugins
    shadowlearn plugin create my-plugin
    shadowlearn logs

${YELLOW}FILES:${NC}
    Data:     ~/.local/share/ShadowLearn/
    Plugins:  ~/.local/share/ShadowLearn/plugins/
    Logs:     /tmp/shadowlearn_dev.log

For more information, visit: https://github.com/shadowlearn
EOF
}

cmd_version() {
    print_header
    echo -e "${CYAN}Version:${NC} $VERSION"
    echo -e "${CYAN}Platform:${NC} $(uname -s) $(uname -m)"
    echo ""
}

# Main command router
main() {
    case "${1:-help}" in
        status)
            cmd_status
            ;;
        stats)
            cmd_stats
            ;;
        plugins)
            cmd_plugins
            ;;
        plugin)
            case "${2:-}" in
                create)
                    cmd_plugin_create "$3"
                    ;;
                *)
                    print_error "Unknown plugin command: ${2:-}"
                    echo "Available: create"
                    exit 1
                    ;;
            esac
            ;;
        logs)
            cmd_logs
            ;;
        clean)
            cmd_clean
            ;;
        version)
            cmd_version
            ;;
        help|--help|-h)
            print_header
            cmd_help
            ;;
        *)
            print_error "Unknown command: $1"
            echo ""
            cmd_help
            exit 1
            ;;
    esac
}

# Run main
main "$@"
