# ShadowLearn CLI

Command-line interface for managing and interacting with ShadowLearn.

## Installation

```bash
# Add to PATH
ln -s $(pwd)/cli/shadowlearn-cli.sh /usr/local/bin/shadowlearn

# Or create alias
echo 'alias shadowlearn="$HOME/path/to/ShadowLearn/cli/shadowlearn-cli.sh"' >> ~/.bashrc
source ~/.bashrc
```

## Commands

### Status & Monitoring

```bash
# Check if ShadowLearn is running
shadowlearn status

# Show productivity statistics
shadowlearn stats

# Tail application logs
shadowlearn logs
```

### Plugin Management

```bash
# List all installed plugins
shadowlearn plugins

# Create a new plugin
shadowlearn plugin create my-awesome-plugin
```

### Maintenance

```bash
# Clean all application data
shadowlearn clean

# Show version information
shadowlearn version

# Show help
shadowlearn help
```

## Plugin Development

Create a new plugin:

```bash
shadowlearn plugin create weather-notifier
```

This creates a plugin directory at `~/.local/share/ShadowLearn/plugins/weather-notifier/` with:

- `plugin.json` - Plugin manifest
- `on_suggestion.sh` - Example hook script

### Plugin Structure

```
~/.local/share/ShadowLearn/plugins/
└── my-plugin/
    ├── plugin.json        # Plugin manifest
    ├── on_suggestion.sh   # Hook scripts
    └── README.md          # Optional documentation
```

### Example Plugin Manifest

```json
{
  "metadata": {
    "id": "weather-notifier",
    "name": "Weather Notifier",
    "version": "1.0.0",
    "author": "Your Name",
    "description": "Shows weather in suggestions"
  },
  "config": {
    "hooks": [
      {
        "name": "on_suggestion",
        "description": "Add weather info to suggestions",
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
```

### Available Hooks

- `on_suggestion` - Triggered when a suggestion is shown
- `on_flow_detected` - Triggered when flow state is detected
- `on_pattern_learned` - Triggered when a new pattern is learned
- `on_daily_digest` - Triggered when generating daily digest

## Environment Variables

Hook scripts have access to these environment variables:

- `SHADOWLEARN_CONTEXT` - Current context data (JSON)
- `SHADOWLEARN_PLUGIN_PATH` - Path to the plugin directory

## Data Locations

- **Data directory**: `~/.local/share/ShadowLearn/`
- **Plugins**: `~/.local/share/ShadowLearn/plugins/`
- **Database**: `~/.local/share/ShadowLearn/*.db`
- **Logs**: `/tmp/shadowlearn_dev.log`

## Examples

### Check System Status

```bash
$ shadowlearn status
ℹ Checking ShadowLearn status...
✓ ShadowLearn is running
ℹ PID: 12345
ℹ Memory: 145 MB
```

### List Plugins

```bash
$ shadowlearn plugins
ℹ Plugin directory: /Users/you/.local/share/ShadowLearn/plugins

✓ Found 2 plugin(s):

  📦 Weather Notifier v1.0.0
     Shows weather information in suggestions
     Path: /Users/you/.local/share/ShadowLearn/plugins/weather-notifier/

  📦 GitHub Integration v2.1.0
     Integrates with GitHub API
     Path: /Users/you/.local/share/ShadowLearn/plugins/github-integration/
```

## Requirements

- Bash 4.0+
- Optional: `jq` for JSON parsing (prettier output)

## License

MIT
