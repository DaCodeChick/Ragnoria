# Launcher Testing Guide

**Date:** 2026-01-28  
**Status:** Ready for Testing  
**Critical Fix:** `-FromLauncher` flag implemented

---

## Quick Start (Windows)

### 1. Build the Launcher

```powershell
cd C:\path\to\Ragnoria
cargo build --bin launcher --release
```

The binary will be at: `target\release\launcher.exe`

### 2. Run the Launcher (Default Mode)

```powershell
# Option 1: Run directly
cargo run --bin launcher --release

# Option 2: Run the binary
.\target\release\launcher.exe
```

**What to expect:**
- GUI window opens with server configuration
- Default: `127.0.0.1:7101`
- Game path auto-detected or browse for `Rag2.exe`
- Click "Launch Game"
- **No Updater error dialogs should appear!** ✅

### 3. Testing Different Launch Options

```powershell
# Option 0 (default): Just -FromLauncher flag
cargo run --bin launcher --release

# Option 1: -FromLauncher + IP and PORT as separate args
$env:LAUNCH_OPTION="1"; cargo run --bin launcher --release

# Option 2: -FromLauncher + /IP=x.x.x.x /PORT=xxxx
$env:LAUNCH_OPTION="2"; cargo run --bin launcher --release

# Option 3: -FromLauncher + IP:PORT combined
$env:LAUNCH_OPTION="3"; cargo run --bin launcher --release

# Option 4: NO PARAMETERS (ERROR TEST - should show Updater errors)
$env:LAUNCH_OPTION="4"; cargo run --bin launcher --release
```

---

## What Was Fixed

### Before Fix
- Launcher passed various parameter combinations WITHOUT `-FromLauncher`
- Game showed two error dialogs:
  1. "Updater를 통해 다시 실행 하겠습니다." (It will be run again through the Updater)
  2. "Updater.exe 파일이 존재하지 않습니다." (The Updater.exe file does not exist)
- Game exited immediately

### After Fix
- Launcher now ALWAYS includes `-FromLauncher` as first argument (except test option 4)
- Game bypasses `ValidateGameLaunchParameters()` check
- No Updater error dialogs
- Game proceeds to initialization

---

## Expected Behavior

### ✅ Success Indicators

1. **No error dialogs** about Updater.exe
2. **Game window opens** (may be black/loading screen)
3. **Process stays alive** (doesn't exit immediately)
4. **Terminal output shows:**
   ```
   Using launch option 0: ["-FromLauncher"]
   Platform: Windows (native execution)
   ✓ Process spawned successfully! PID: 12345
   ✓ SUCCESS: Game launched! Connecting to 127.0.0.1:7101
   ```

### ❌ Failure Indicators

1. **Two error dialogs appear** with garbled Korean text
2. **Game exits immediately** after clicking OK
3. **This means:** `-FromLauncher` flag is missing or malformed

---

## Next Steps After Successful Launch

Once the game launches without Updater errors, you may encounter:

### Possible Scenario 1: Game Connects to Server
- Check test server logs for connection attempts
- Game may show login screen or error about server response
- **Next task:** Implement login protocol handlers

### Possible Scenario 2: Game Shows Different Error
- Take screenshot of new error
- Use Ghidra to analyze error source
- **Next task:** Fix new validation/initialization issue

### Possible Scenario 3: Black Screen / Hangs
- Game may be trying to connect to hardcoded server
- Check network traffic with Wireshark
- May need to modify hosts file or patch executable
- **Next task:** Redirect network connections

### Possible Scenario 4: Missing Resources
- Game may look for data files, textures, etc.
- Check game directory structure
- **Next task:** Ensure all game files are present

---

## Testing with Test Server

### Start Test Server

```bash
# On Linux (or Windows with WSL)
cd /home/admin/Documents/GitHub/Ragnoria
cargo run --bin test_server --release
```

**Expected output:**
```
Starting ProudNet test server on 0.0.0.0:7101
Server is ready to accept connections
RSA-1024 key pair generated
AES-128 encryption configured
```

### Launch Game and Monitor

**Terminal 1 (Server):**
```bash
cargo run --bin test_server --release
# Watch for connection messages
```

**Terminal 2 (Launcher on Windows):**
```powershell
cargo run --bin launcher --release
# Click "Launch Game"
```

**Look for:**
- Server logs showing: "New connection from 127.0.0.1:xxxxx"
- Packet decryption logs
- ProudNet handshake messages

---

## Troubleshooting

### Issue: Still Getting Updater Errors

**Check:**
1. Did you rebuild the launcher after pulling latest code?
   ```powershell
   cargo clean
   cargo build --bin launcher --release
   ```

2. Are you running the correct binary?
   ```powershell
   # Check version
   git log -1 --oneline crates/launcher/
   # Should show: "Fix launcher: Add mandatory -FromLauncher flag"
   ```

3. Is `LAUNCH_OPTION=4` set? (This deliberately omits the flag for testing)
   ```powershell
   # Clear environment variable
   Remove-Item Env:\LAUNCH_OPTION
   ```

### Issue: Game Launches But Immediately Exits (No Errors)

**Possible causes:**
1. Missing DLLs
   - Launcher sets working directory to game root (where DLLs are)
   - Check terminal output for "Working directory will be set to: ..."

2. Missing data files
   - Check for `dict.lex` in game root
   - Check for `Data/` folder with game assets

3. Anti-cheat software (HackShield)
   - May block execution if it detects modifications
   - Try disabling or removing HackShield

### Issue: Can't Find Rag2.exe

**Game path should point to:**
```
C:\Gravity\Ragnarok Online 2\SHIPPING\Rag2.exe
```

**NOT:**
- `RO2Client.exe` (launcher executable)
- `Launcher2.exe` (old launcher)
- `WPLauncher.exe` (WarpPortal launcher)

Use the "Browse" button in the GUI to locate it.

### Issue: Launcher GUI Doesn't Open

**Check dependencies:**
```powershell
# On Windows, you may need Visual C++ Redistributable
# Download from: https://aka.ms/vs/17/release/vc_redist.x64.exe

# Verify Rust/Cargo installation
cargo --version
rustc --version
```

---

## Debug Output

The launcher prints detailed debug information. Look for:

```
===========================================
    Ragnoria Launcher v0.1.0
    RO2 Custom Server Launcher
===========================================

Loading configuration...
Config loaded:
  Server IP: 127.0.0.1
  Server Port: 7101
  Game Path: C:\...\SHIPPING\Rag2.exe

Game executable: "C:\\...\\SHIPPING\\Rag2.exe"
SHIPPING directory: "C:\\...\\SHIPPING"
Game root directory (with DLLs): "C:\\..."
Working directory will be set to: "C:\\..."

NOTE: Based on Ghidra analysis, the game calls:
  - ParseGameLaunchArguments() to parse command line
  - GetGameServerIPFromCommandLine() to extract server IP
  - ValidateGameLaunchParameters() to check for '-FromLauncher' flag

CRITICAL: Game REQUIRES '-FromLauncher' flag in command line!
Without it, shows Korean error dialogs about Updater.exe

Using launch option 0: ["-FromLauncher"]

Platform: Windows (native execution)
Spawning command: "C:\...\Rag2.exe"
With args: ["-FromLauncher"]
Working directory: "C:\..."

✓ Process spawned successfully! PID: 12345
✓ SUCCESS: Game launched! Connecting to 127.0.0.1:7101
```

---

## Success Criteria

### Minimum Success (Phase 1) ✅ **TARGET FOR THIS TEST**
- [x] Launcher GUI opens
- [x] Game path detected/selectable
- [ ] **Game launches WITHOUT Updater error dialogs** ⬅️ **YOU ARE HERE**
- [ ] Game process stays alive (doesn't immediately exit)

### Phase 2 Success (Next Step)
- [ ] Game shows loading screen or game window
- [ ] Test server receives connection attempt
- [ ] ProudNet handshake initiated

### Phase 3 Success (Future)
- [ ] Login screen appears
- [ ] Can enter credentials
- [ ] Server authentication works
- [ ] Character selection screen

### Phase 4 Success (Goal)
- [ ] Character loads into game world
- [ ] Can see game environment
- [ ] Basic movement works
- [ ] Server protocol fully implemented

---

## Command Reference

```powershell
# Build launcher
cargo build --bin launcher --release

# Run launcher (default)
cargo run --bin launcher --release

# Run with specific launch option
$env:LAUNCH_OPTION="1"; cargo run --bin launcher --release

# Check git history
git log --oneline crates/launcher/

# View Ghidra findings
cat docs/ghidra-findings/VALIDATE-LAUNCH-PARAMS-ANALYSIS.md

# Clean build
cargo clean

# Rebuild from scratch
cargo clean && cargo build --bin launcher --release
```

---

## Files Modified in This Fix

1. **`crates/launcher/src/main.rs`**
   - Added `-FromLauncher` to all command-line argument vectors
   - Updated default to Option 0 (just the flag)
   - Added detailed comments about Ghidra discovery

2. **`crates/launcher/README.md`**
   - Documented `-FromLauncher` requirement
   - Added parameter option table
   - Updated troubleshooting section

3. **`docs/ghidra-findings/VALIDATE-LAUNCH-PARAMS-ANALYSIS.md`** (NEW)
   - Full reverse engineering analysis
   - Decompiled C code
   - Assembly listing
   - Error string translations
   - Testing results

---

## Contact & Support

If you encounter issues:

1. **Check terminal output** - detailed debug info is printed
2. **Check Ghidra analysis** - review `VALIDATE-LAUNCH-PARAMS-ANALYSIS.md`
3. **Test with option 4** - deliberately causes Updater error to verify detection
4. **Capture screenshots** - especially any new error dialogs

---

## Summary

**The fix is simple but critical:**
```rust
// Before (WRONG - shows Updater errors)
vec![]

// After (CORRECT - bypasses validation)
vec![String::from("-FromLauncher")]
```

This single command-line flag bypasses the `ValidateGameLaunchParameters()` check in Rag2.exe, discovered through Ghidra reverse engineering at address `0x00a501d0`.

**Good luck with testing! 🚀**
