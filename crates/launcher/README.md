# Ragnoria Launcher

Custom launcher GUI for connecting the Ragnarok Online 2 client to private servers.

## Features

- ✅ Server IP and port configuration
- ✅ Game path selection with file browser
- ✅ Settings persistence
- ✅ Cross-platform (Windows native, Linux with Wine)

## Usage

### Quick Start

```bash
cargo run --bin launcher --release
```

### Configuration

The launcher provides input fields for:

1. **Server IP**: Custom server address (default: `127.0.0.1`)
2. **Server Port**: Server port (default: `7101`)
3. **Game Path**: Path to `Rag2.exe` in the SHIPPING folder

### Launching

1. Enter your server details
2. Browse for `Rag2.exe` (usually in `SHIPPING/` folder)
3. Click "Launch Game"

Settings are automatically saved to:
- **Linux**: `~/.config/ragnoria/launcher.toml`
- **Windows**: `%APPDATA%\ragnoria\launcher.toml`

## How It Works

### Command Line Parameters

The launcher passes parameters discovered through reverse engineering:

```bash
Rag2.exe /FROM=-FromLauncher /IP=127.0.0.1
```

**Parameter Format:**
- Game parses command line by tokenizing on space and `/`
- Each token is split by `=` into key-value pairs
- Parameters stored in `g_LaunchParametersMap`

**Validation:**
- Game checks `map["FROM"] == "-FromLauncher"`
- Without this, shows Korean error dialogs about "Updater.exe"
- Game extracts server IP from `map["IP"]`

### Directory Structure

```
Game Root/              ← Working directory (contains DLLs)
├── *.dll
├── RO2Client.exe
└── SHIPPING/
    └── Rag2.exe        ← Game executable
```

Launcher sets working directory to game root so DLLs can be loaded.

## Testing with Test Server

```bash
# Terminal 1: Start login server
cargo run -p ro2-login --release

# Terminal 2: Launch game
cargo run --bin launcher --release
```

## Troubleshooting

### Korean Error Dialogs about "Updater"

**Cause:** Parameter validation failed  
**Solution:** Use the launcher - it includes the required `/FROM=-FromLauncher` parameter

### Game Not Found

Ensure path points to `Rag2.exe` in the `SHIPPING` folder, not:
- `RO2Client.exe` (update downloader)
- `Launcher2.exe` (old launcher)
- `WPLauncher.exe` (WarpPortal launcher)

### Missing DLL Errors

Verify:
- Game installation is complete
- DLLs exist in game root directory
- Launcher sets working directory correctly

### Linux: Blank Window

Install required libraries:
```bash
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

## Technical Details

- **GUI Framework**: Iced 0.13
- **Config Format**: TOML (serde)
- **Launch Method**: Spawns game process with custom working directory
- **Parameter Discovery**: Ghidra reverse engineering of `Rag2.exe`

### Reverse Engineering Notes

Key functions analyzed:
- `ParseGameLaunchArguments()` (0xa4e9d0) - Tokenizes command line
- `ValidateGameLaunchParameters()` (0xa501d0) - Checks FROM parameter
- `GetGameServerIPFromCommandLine()` (0xa4f880) - Extracts IP from map

See `docs/ghidra-findings/` for detailed analysis.
