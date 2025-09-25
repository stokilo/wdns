# macOS Network Connection Monitor

A real-time network connection monitoring application for macOS built with Rust and egui.

## Features

- **Real-time monitoring**: Shows all active network connections on the host
- **Protocol support**: Displays both TCP and UDP connections
- **Process information**: Shows which process owns each connection
- **Filtering and sorting**: Filter by address, process, protocol, or state
- **Statistics**: Live statistics about connection counts
- **Modern UI**: Clean, responsive interface built with egui

## Screenshots

The application displays:
- Connection statistics (total, TCP, UDP, listening ports, established connections)
- Sortable table of all network connections
- Filter options for local/remote connections
- Real-time updates every 1-10 seconds (configurable)
- Detailed connection information on selection

## Installation

### Prerequisites

- Rust 1.70+ 
- macOS (uses `netstat` and `lsof` commands)

### Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd wdns/macos-listener

# Build the application
cargo build --release

# Or use the convenient build script
./build.sh

# Run the application
cargo run

# Or use the convenient run script
./run.sh
```

## Usage

### Running the Application

```bash
# Development mode
cargo run

# Release mode (optimized)
cargo run --release

# Using the run script (includes system checks)
./run.sh
```

### Testing Network Monitoring

```bash
# Create test network activity
./test-network.sh

# This will:
# - Make HTTP requests to external sites
# - Start a local HTTP server
# - Show you what connections to look for in the monitor
```

### Features

1. **Connection Monitoring**: Automatically scans and displays all network connections
2. **Filtering**: 
   - Filter by text (address, process name, protocol, state)
   - Show only local connections
   - Show only remote connections
3. **Sorting**: Sort by any column (address, process, protocol, state, bytes)
4. **Real-time Updates**: Configurable update interval (1-10 seconds)
5. **Connection Details**: Click any connection to see detailed information

### Keyboard Shortcuts

- **Ctrl+Q**: Quit application
- **Ctrl+R**: Refresh connections
- **F5**: Toggle auto-refresh

## Technical Details

### Architecture

- **Frontend**: egui for the user interface
- **Network Monitoring**: Uses `netstat` and `lsof` system commands
- **Data Processing**: Real-time parsing of network connection data
- **Threading**: Async updates to prevent UI blocking

### Dependencies

- `eframe` - Application framework
- `egui` - Immediate mode GUI
- `tokio` - Async runtime
- `tracing` - Logging
- `serde` - Serialization
- `chrono` - Time handling

### System Requirements

- macOS 10.15+ (uses modern system commands)
- Network access for connection monitoring
- Sufficient permissions to read process information

## Development

### Project Structure

```
macos-listener/
├── src/
│   └── main.rs          # Main application logic
├── Cargo.toml          # Dependencies and metadata
└── README.md           # This file
```

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Check code
cargo check
```

### Adding Features

The application is designed to be easily extensible:

1. **New connection types**: Add parsing logic in `get_*_connections()` methods
2. **Additional filters**: Extend the filtering logic in `render_connections_table()`
3. **New statistics**: Add fields to `NetworkStats` struct
4. **UI improvements**: Modify the `render_ui()` method

## Troubleshooting

### Common Issues

1. **No connections shown**: Ensure the application has network permissions
2. **Slow updates**: Reduce the update interval or check system performance
3. **Missing process names**: Some connections may not have associated process information

### Performance

- The application uses system commands (`netstat`, `lsof`) which may be slow on systems with many connections
- Consider increasing the update interval for better performance
- The UI is designed to handle thousands of connections efficiently

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Changelog

### v0.1.0
- Initial release
- Basic network connection monitoring
- TCP and UDP support
- Filtering and sorting
- Real-time updates
- Process information display
