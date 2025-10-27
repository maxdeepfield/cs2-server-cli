# CS2 Server CLI

A command-line tool for managing Counter-Strike 2 dedicated servers. This tool simplifies the installation, configuration, and management of CS2 servers across different platforms.

## Features

- **Easy Installation**: Download and set up CS2 server files with a single command
- **Multi-Server Support**: Manage multiple server instances on the same machine
- **Configuration Management**: Generate and modify server configurations
- **Server Control**: Start, stop, and monitor server status
- **Plugin Framework**: Install and manage plugins with recommendations
- **Backup & Restore**: Create backups of server configurations
- **Cross-Platform**: Works on Windows, Linux, and macOS

## Installation

### Prerequisites

- Rust (latest stable version)
- SteamCMD installed and available in PATH
- Sufficient disk space (CS2 server files ~30GB)

### Building from Source

```bash
git clone <repository-url>
cd cs2-server-cli
cargo build --release
```

## Usage

### Basic Commands

```bash
# Install a new CS2 server
cs2-server-cli install my-server

# Start the server
cs2-server-cli start my-server

# Check server status
cs2-server-cli status my-server

# Stop the server
cs2-server-cli stop my-server

# List all servers
cs2-server-cli list
```

### Configuration

```bash
# Set server configuration
cs2-server-cli config my-server hostname "My CS2 Server"
cs2-server-cli config my-server maxplayers 16
cs2-server-cli config my-server map de_dust2
```

### Plugins

```bash
# Show recommended plugins
cs2-server-cli plugin recommended

# Install a plugin
cs2-server-cli plugin install my-server sourcemod

# List installed plugins
cs2-server-cli plugin list my-server
```

### Backup and Restore

```bash
# Create a backup
cs2-server-cli backup my-server backup-2024-01

# Restore from backup
cs2-server-cli restore my-server backup-2024-01
```

### Updates

```bash
# Update server files
cs2-server-cli update my-server
```

## Configuration Files

The tool stores configuration in:
- `~/.config/cs2-server-cli/config.toml` - Tool configuration and server registry
- `servers/{server-name}/server.cfg` - Individual server configurations

## Directory Structure

```
servers/
├── my-server/
│   ├── game/           # CS2 game files
│   ├── server.cfg      # Server configuration
│   ├── backups/        # Configuration backups
│   └── logs/           # Server logs (future)
```

## Steam Authentication

For downloading CS2 server files, the tool uses SteamCMD. You may need to provide Steam credentials for authenticated downloads. The tool will prompt for credentials when required.

## Supported Platforms

- **Windows**: Full support with Windows-specific optimizations
- **Linux**: Primary target platform for CS2 servers
- **macOS**: Experimental support

## Troubleshooting

### Common Issues

1. **SteamCMD not found**: Ensure SteamCMD is installed and in your PATH
2. **Permission errors**: Run with appropriate permissions for server directories
3. **Port conflicts**: Ensure server ports are available (default: 27015)

### Logs

Server logs are stored in the server's `logs/` directory. Check these for detailed error information.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Disclaimer

This tool is not officially affiliated with Valve Corporation or Counter-Strike 2. Use at your own risk.