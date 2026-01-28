# Ragnoria Launcher

Custom launcher GUI for connecting the Ragnarok Online 2 client to private servers.

## Features

- ✅ Server IP and port configuration
- ✅ Game path selection
- ✅ Settings persistence (saves to `~/.config/ragnoria/launcher.toml`)
- ✅ Auto-detection of game installation path
- ✅ Cross-platform (Windows native, Linux with Wine)

## Usage

### Running the Launcher

```bash
cd /home/admin/Documents/GitHub/Ragnoria
cargo run --bin launcher
```

### Configuration

The launcher will display input fields for:

1. **Server IP**: The IP address of your custom server (default: `127.0.0.1`)
2. **Server Port**: The port your server listens on (default: `7101`)
3. **Game Path**: Full path to `Rag2.exe` (auto-detected if possible)

Example game path:
```
/run/media/admin/FE6407F46407AE89/Gravity/Ragnarok Online 2 - Jawaii/SHIPPING/Rag2.exe
```

### Launching the Game

1. Fill in the server details
2. Verify the game path is correct
3. Click "Launch Game"
4. The launcher will save your settings and start Rag2.exe with custom parameters

## How It Works

The launcher passes command-line parameters to Rag2.exe based on reverse engineering of RO2Client.exe:

```bash
Rag2.exe /FROM=-Ragnoria /STARTER=2 /IP=127.0.0.1 /PORT=7101
```

### Parameter Formats Tested

The launcher currently tests these parameter formats (in order):

1. **Option 1** (Default): `/FROM=-Ragnoria /STARTER=2 /IP=x.x.x.x /PORT=xxxx`
2. **Option 2**: `/FROM=-Ragnoria /STARTER=2 /SERVER=x.x.x.x:xxxx`
3. **Option 3**: `/FROM=-Ragnoria /STARTER=2 /LOGINSERVER=x.x.x.x:xxxx`

> **Note**: The exact parameter format accepted by Rag2.exe needs to be verified through testing. Option 1 is based on RO2Client.exe analysis.

## Configuration File

Settings are saved to:
- **Linux**: `~/.config/ragnoria/launcher.toml`
- **Windows**: `%APPDATA%\ragnoria\launcher.toml`

Example `launcher.toml`:
```toml
game_path = "/path/to/SHIPPING/Rag2.exe"

[server]
ip = "127.0.0.1"
port = 7101
```

## Testing with Test Server

To test the launcher with the ProudNet test server:

```bash
# Terminal 1: Start test server
cargo run --bin test_server

# Terminal 2: Launch the launcher
cargo run --bin launcher
# Enter server IP: 127.0.0.1
# Enter server port: 7101
# Enter game path (or let it auto-detect)
# Click "Launch Game"
```

## Troubleshooting

### "Game executable not found"

Verify the game path points to `Rag2.exe` in the `SHIPPING` folder, not `RO2Client.exe` or `Launcher2.exe`.

### Game doesn't connect

The parameter format may need adjustment. Check the launcher output and test server logs to see if the client attempts to connect.

### Blank window on Linux

Make sure you have the required graphics libraries:
```bash
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

## Implementation Details

- **GUI Framework**: iced 0.13
- **Config Format**: TOML
- **Platform**: Cross-platform (uses Wine on Linux)
- **Launch Method**: Spawns Rag2.exe as child process with custom parameters

## Next Steps

- [ ] Add file browser dialog for game path selection
- [ ] Add multiple server profiles
- [ ] Add game version detection
- [ ] Add update checker
- [ ] Add news/announcement display
