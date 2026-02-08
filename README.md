# VSCode Agent Monitor

Monitor AI agent interactions in VSCode and scan for security vulnerabilities.

## Features

- 🔍 **Real-time Monitoring**: Captures document changes and AI agent interactions
- 🔒 **Security Scanning**: Automatically detects API keys, passwords, and secrets
- 📊 **Visual Dashboard**: WebView dashboard showing telemetry and security findings
- 💾 **SQLite Storage**: Persistent storage with BLAKE3 content deduplication
- 🎨 **Dark Theme**: Native VSCode theme integration

## Installation

### Requirements
- VSCode 1.85.0 or higher
- Rust and Cargo ([Install Rust](https://rustup.rs/))

### Install Extension

1. Download `vscode-agent-monitor-0.1.0.vsix`
2. VSCode → Extensions → "..." menu → Install from VSIX
3. Reload VSCode

## Quick Start

1. **Open a project folder** in VSCode
2. Extension activates automatically and creates `agent_telemetry.db`
3. **Start coding** - the extension captures significant changes (>10 chars)
4. **View dashboard**: `Cmd+Shift+P` → `Agent Monitor: Show Dashboard`

## Commands

- `Agent Monitor: Show Dashboard` - View telemetry and security findings
- `Agent Monitor: Show Logs` - See extension activity and debugging info
- `Agent Monitor: Clear Data` - Delete telemetry database

## What Gets Monitored

- Document changes (captures substantial edits, including AI completions)
- File saves
- Context around changes (for security analysis)

## Security Detection

All captured data is automatically scanned for:
- 🔴 **High Severity**: API keys (sk-*), passwords
- 🟠 **Medium Severity**: High-entropy strings (potential secrets)
- 🟡 **Low Severity**: Suspicious patterns

## Usage

### Monitoring

The extension automatically monitors your workspace when active. Check logs to see activity:

```
Cmd+Shift+P → "Agent Monitor: Show Logs"
```

You'll see:
- Database initialization
- Document changes captured
- Capture count

### Dashboard

View captured data and security findings:

```
Cmd+Shift+P → "Agent Monitor: Show Dashboard"
```

The dashboard shows:
- Total events captured
- Unique files (BLAKE3 deduplication)
- Security findings by severity
- Detailed findings table

### Manual Database Initialization

If automatic initialization fails, create the database manually:

```bash
cd /path/to/vscode-agent-monitor
cargo run --example init_db "/path/to/your/project/agent_telemetry.db"
```

Then reload VSCode.

## Development

### Architecture

- **Rust Core**: Security scanner, telemetry collector, storage manager
- **TypeScript Extension**: VSCode integration, UI
- **SQLite**: Persistent storage
- **WebView**: Dashboard visualization

See [ARCHITECTURE.md](ARCHITECTURE.md) for technical details.

### Build Extension

```bash
npm install
npm run compile
vsce package
```

### Run Tests

```bash
cargo test
```

## Privacy & Security

- All data stored locally in your workspace (`agent_telemetry.db`)
- No external servers
- You control when to clear data
- Add `agent_telemetry.db` to `.gitignore`

## License

MIT

## Project Structure

```
vscode-agent-monitor/
├── src/                    # Rust core modules
│   ├── security_scanner.rs
│   ├── telemetry_collector.rs
│   ├── storage_manager.rs
│   └── visualization_panel.rs
├── src-ts/                # TypeScript extension
│   ├── extension.ts
│   └── lspCapture.ts
├── examples/              # Rust CLI examples
├── tests/                # Rust tests
└── out/                  # Compiled TypeScript
```
