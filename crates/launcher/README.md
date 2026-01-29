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

### Critical Discovery: The `-FromLauncher` Flag

Through Ghidra reverse engineering, we discovered that **Rag2.exe REQUIRES the `-FromLauncher` flag** in the command line:

```c
// Decompiled from ValidateGameLaunchParameters() at 0x00a501d0
if (!strcmp(commandLine, "-FromLauncher")) {
    return; // Success! Continue to game initialization
}

// If -FromLauncher is NOT found:
// 1. Show Korean error: "Updater를 통해서만 실행할 수 있습니다."
//    (Translation: "Can only be executed through Updater")
// 2. Check for ../Updater.exe file
// 3. Show second error if Updater.exe doesn't exist
```

**Without this flag, the game shows error dialogs and refuses to launch!**

### Launch Parameters

The launcher passes these parameters to Rag2.exe:

```bash
Rag2.exe -FromLauncher [optional: IP PORT or /IP=x.x.x.x /PORT=xxxx]
```

### Parameter Options

The launcher supports multiple parameter formats via `LAUNCH_OPTION` environment variable:

| Option | Parameters | Description |
|--------|-----------|-------------|
| **0** (default) | `-FromLauncher` | Just the required flag (RECOMMENDED) |
| 1 | `-FromLauncher IP PORT` | With server IP and port |
| 2 | `-FromLauncher /IP=x.x.x.x /PORT=xxxx` | With named parameters |
| 3 | `-FromLauncher IP:PORT` | With combined address |
| 4 | *(none)* | No parameters (ERROR TEST - shows Updater error) |

To test different options:
```bash
LAUNCH_OPTION=1 cargo run --bin launcher
```

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

### "Updater" Error Dialogs (Korean Text)

If you see error dialogs with garbled Korean text about "Updater":
- **Cause**: The `-FromLauncher` flag is missing from command line
- **Solution**: Use the launcher (it includes this flag automatically) or add `-FromLauncher` manually

### "Game executable not found"

Verify the game path points to `Rag2.exe` in the `SHIPPING` folder, not `RO2Client.exe` or `Launcher2.exe`.

### Game doesn't connect

The parameter format may need adjustment. Try different `LAUNCH_OPTION` values (see above).

### Missing DLL Errors

The launcher sets the working directory to the game root (parent of SHIPPING) where all DLLs are located. If you still get DLL errors:
- Verify your game installation is complete
- Check that DLLs exist in the root game directory

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
