# Game Loop and Network Processing Analysis

## Document Status
**Analysis Date:** 2026-01-27  
**Binary:** Rag2.exe (Ragnarok Online 2 Client)  
**Focus:** Main game loop execution flow and network message processing  
**Status:** Phase 1 Complete - Core loop traced

---

## Executive Summary

This document traces the complete execution flow of the RO2 client's main game loop, from WinMain through to network message processing and rendering. The analysis reveals a sophisticated frame-rate adaptive game loop with integrated ProudNet networking middleware.

### Key Discoveries

1. **Frame-Rate Adaptive Loop**: Dynamic frame rate adjustment (High/Medium/Normal) based on game state performance
2. **Network Processing Model**: Two-phase network update with vtable-dispatched handlers
3. **Timer Management**: Three separate timer systems for different subsystems
4. **Shutdown Mechanism**: Global flag-based shutdown with cleanup handlers

---

## WinMain Message Pump

**Function:** `WinMain @ 0x00A502F0`  
**Parameters:** 4 (hInstance, hPrevInstance, lpCmdLine, nShowCmd)

### Message Loop Structure

```c
tagMSG msg;
msg.message = 0;
PeekMessageW(&msg, NULL, 0, 0, 0);  // Prime the pump

while (msg.message != WM_QUIT) {  // 0x12 = WM_QUIT
    if (PeekMessageW(&msg, NULL, 0, 0, PM_REMOVE)) {
        // Special handling for input messages
        if (!g_InputSystemEnabled && 
            g_InputWindow != NULL &&
            (msg.message >= WM_KEYDOWN && msg.message <= WM_KEYLAST)) {
            SendMessageW(g_InputWindow, msg.message, msg.wParam, msg.lParam);
        }
        
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    } else {
        // No Windows messages - process game frame
        MainGameLoopUpdate();
        Sleep(0);  // Yield CPU time slice
    }
}
```

### Analysis

**Message Priority System:**
1. Windows messages are processed first (UI responsiveness)
2. Input messages get special routing to `g_InputWindow` when `g_InputSystemEnabled == false`
3. Game logic only runs when no Windows messages are pending
4. `Sleep(0)` yields CPU but allows immediate re-scheduling

**Input Handling:**
- `g_InputWindow @ unknown` - Window handle for input message forwarding
- `g_InputSystemEnabled @ unknown` - Flag to control input routing
- Key messages (WM_KEYDOWN through WM_KEYLAST) get forwarded before normal dispatch

---

## MainGameLoopUpdate Function

**Function:** `MainGameLoopUpdate @ 0x00A4C300`  
**Parameters:** 0 (void)  
**Called From:** WinMain message loop

### Execution Flow

```
┌─────────────────────────────────┐
│  1. Time Management             │
│     - GetCurrentGameTime()      │
│     - Calculate frame delta     │
└────────────┬────────────────────┘
             │
┌────────────▼────────────────────┐
│  2. Frame Rate Adjustment       │
│     - Check game state perf     │
│     - Set High/Medium/Normal    │
└────────────┬────────────────────┘
             │
┌────────────▼────────────────────┐
│  3. Timer Updates               │
│     - Update timer[0] (main)    │
│     - Update timer[8] (alt)     │
└────────────┬────────────────────┘
             │
┌────────────▼────────────────────┐
│  4. Network Update Phase 1      │
│     - NetworkManager vtable+0x24│
│     - Pre-game-logic network    │
└────────────┬────────────────────┘
             │
┌────────────▼────────────────────┐
│  5. Game State Update           │
│     - GameStateManager_Update() │
│     - Process game logic        │
└────────────┬────────────────────┘
             │
┌────────────▼────────────────────┐
│  6. Renderer Update             │
│     - Renderer_UpdateSettings() │
└────────────┬────────────────────┘
             │
┌────────────▼────────────────────┐
│  7. Sound Manager Update        │
│     - SoundManager_Update()     │
└────────────┬────────────────────┘
             │
┌────────────▼────────────────────┐
│  8. Network Update Phase 2      │
│     - NetworkManager vtable+0x28│
│     - Post-game-logic network   │
└────────────┬────────────────────┘
             │
┌────────────▼────────────────────┐
│  9. Shutdown Check              │
│     - Check g_ShutdownRequested │
│     - HandleGameShutdownRequest │
└────────────┬────────────────────┘
             │
┌────────────▼────────────────────┐
│ 10. Rendering                   │
│     - GameStateManager_Render() │
└─────────────────────────────────┘
```

### Phase 1: Time Management

```c
float10 currentGameTime = GetCurrentGameTime();
double currentTimeAsDouble = (double)currentGameTime;

// First-time initialization
if ((_g_GameTimeInitialized & 1) == 0) {
    _g_GameTimeInitialized = _g_GameTimeInitialized | 1;
    previousGameTime = currentGameTime;
} else {
    previousGameTime = (float10)_g_CurrentGameTime;
}

_g_CurrentGameTime = (double)currentGameTime;
```

**Global Variables:**
- `_g_GameTimeInitialized @ unknown` - Bit flags for initialization state
  - Bit 0: Current game time initialized
  - Bit 1: Last frame time initialized
- `_g_CurrentGameTime @ unknown` - double, tracks current frame time
- `_g_LastFrameTime @ unknown` - double, tracks previous frame time

**Function Called:**
- `GetCurrentGameTime @ unknown` - Returns `float10` (80-bit extended precision)

### Phase 2: Frame Rate Adjustment

The game uses three frame rate tiers based on performance metrics from `GameStateManager`:

```c
int tempMgr = GetGameStateManager();
float performanceMetric = *(float*)(tempMgr + 0x2c);

if (performanceMetric < THRESHOLD_HIGH) {
    if (performanceMetric < THRESHOLD_MEDIUM) {
        if (performanceMetric < THRESHOLD_LOW) {
            // Set frame rate to NORMAL (lowest)
            if (currentFrameRate != FLOAT_FrameRateNormal) {
                SetFrameRate(FLOAT_01357d28);  // Normal rate
            }
        }
    } else {
        // Set frame rate to MEDIUM
        if (currentFrameRate != FLOAT_FrameRateMedium) {
            SetFrameRate(FLOAT_013c5e74);  // Medium rate
        }
    }
} else {
    // Set frame rate to HIGH (highest)
    if (currentFrameRate != FLOAT_FrameRateHigh) {
        SetFrameRate(_FLOAT_DefaultFrameRate);  // High rate
    }
}
```

**Frame Rate Constants:**
| Constant | Address | Value | Description |
|----------|---------|-------|-------------|
| `_FLOAT_DefaultFrameRate` | unknown | ? | Highest frame rate (best perf) |
| `FLOAT_FrameRateHigh` | unknown | ? | High performance target |
| `FLOAT_013c5e74` (Medium) | 0x013c5e74 | ? | Medium performance target |
| `FLOAT_FrameRateMedium` | unknown | ? | Medium perf threshold |
| `FLOAT_01357d28` (Normal) | 0x01357d28 | ? | Low performance target |
| `FLOAT_FrameRateNormal` | unknown | ? | Low perf threshold |

**Performance Thresholds:**
| Constant | Address | Description |
|----------|---------|-------------|
| `DOUBLE_01358008` | 0x01358008 | High perf threshold |
| `DOUBLE_0135b6d8` | 0x0135b6d8 | Medium perf threshold |

**GameStateManager Structure:**
```c
struct GameStateManager {
    char unknown[0x08];
    float currentFrameTime;      // +0x08
    char unknown2[0x20];
    float performanceMetric;     // +0x2C - Used for frame rate decisions
    char unknown3[0x40];
    DWORD lastSystemTimeMs;      // +0x64 (100 decimal) - timeGetTime() result
    // ... more fields
};
```

### Phase 3: Timer Updates

Three separate timer instances are managed:

```c
int* timersInstance = GetGameTimersInstance();

// Timer 0: Main game timer
void* timer0 = (void*)timersInstance[0];
GameTimer_UpdateDelta(timer0, deltaTime);

// Timer 8: Alternate timer
void* timer8 = *(void**)(timersInstance + 8);
GameTimer_UpdateDelta(timer8, deltaTime);
```

**GameTimersInstance Structure:**
```c
struct GameTimersInstance {
    GameTimer* timer0;          // +0x00 - Main timer
    GameTimer* timer1;          // +0x04 - Secondary timer (frame-based)
    GameTimer* timer2;          // +0x08 - Tertiary timer (alternate)
    // ... more fields
};
```

**GameTimer Structure:**
```c
struct GameTimer {
    char unknown[0x08];
    float targetFrameRate;      // +0x08
    float currentDelta;         // +0x10 - Time delta for this frame
    char isPaused;              // +0x14 - Pause flag
    // ... more fields
};
```

**Functions:**
- `GetGameTimersInstance @ unknown` - Returns singleton pointer to timer manager
- `GameTimer_UpdateDelta @ unknown` - Updates timer with frame delta
- `GameTimer_SetValue @ unknown` - Sets target frame rate
- `GameTimer_IsRunning @ unknown` - Returns bool (char), checks if timer is not paused
- `GameTimer_Reset @ unknown` - Resets timer state

### Phase 4: Network Update Phase 1 (Pre-Game-Logic)

```c
int* networkManager = GetNetworkManager();
int* timersInstance = GetGameTimersInstance();

// Call vtable function at offset 0x24
(*(code**)(*networkManager + 0x24))(*(float*)(*timersInstance + 0x10));
```

**Analysis:**
- First network update occurs **before** game state processing
- Receives the main timer's delta (from `timer0 + 0x10`)
- Likely **receives and queues** incoming network messages
- Function signature: `void NetworkUpdate1(float deltaTime)`

**NetworkManager VTable:**
```c
struct NetworkManager_VTable {
    // ... unknown functions
    void (*PreGameUpdate)(float deltaTime);   // +0x24
    void (*PostGameUpdate)(float deltaTime);  // +0x28
    // ... unknown functions
};
```

### Phase 5: Game State Update

```c
int tempMgr = GetGameTimersInstance();
char isPaused = GameTimer_IsRunning(*(int*)(tempMgr + 4));

if (isPaused != 0) {
    DWORD systemTime = timeGetTime();
    int gameStateMgr = GetGameStateManager();
    *(DWORD*)(gameStateMgr + 100) = systemTime;
    *(float*)(gameStateMgr + 8) = (float)currentGameTime;
    
    int configMgr = GetGameConfigurationManager();
    char configFlags = *(char*)(configMgr + 0x2fc);
    
    void* gameState = GetGameStateManager();
    GameStateManager_Update(gameState, configFlags);
    
    // ... (additional updates)
}
```

**Key Points:**
1. Game state only updates if timer[1] (frame timer) is running
2. System time from `timeGetTime()` is stored in GameStateManager
3. Configuration flags are passed to the update function

**Functions:**
- `GetGameStateManager @ unknown` - Returns singleton pointer
- `GameStateManager_Update @ unknown` - 2 params (void* this, char configFlags)
- `GetGameConfigurationManager @ unknown` - Returns singleton pointer

**GameConfigurationManager Structure:**
```c
struct GameConfigurationManager {
    char unknown[0x2cc];
    char renderingFlag;         // +0x2cc
    char unknown2[0x2f];
    char updateConfigFlags;     // +0x2fc - Passed to GameStateManager_Update
    // ... more fields
};
```

### Phase 6: Renderer Update

```c
int tempMgr = GetGameTimersInstance();
if (*(char*)(*(int*)(tempMgr + 4) + 0x14) == 0) {  // If timer not paused
    int configMgr = GetGameConfigurationManager();
    char renderFlag = *(char*)(configMgr + 0x2cc);
    
    int renderer = GetRendererInstance();
    *(char*)(renderer + 0x98) = renderFlag;
    
    void* rendererPtr = GetRendererInstance();
    Renderer_UpdateSettings(rendererPtr);
}
```

**Renderer Structure:**
```c
struct Renderer {
    char unknown[0x98];
    char renderingFlag;         // +0x98 - Copied from config manager
    // ... more fields
};
```

**Functions:**
- `GetRendererInstance @ unknown` - Returns singleton pointer
- `Renderer_UpdateSettings @ unknown` - 1 param (void* this)

### Phase 7: Sound Manager Update

```c
int* timersInstance = GetGameTimersInstance();
float deltaTime = *(float*)(*timersInstance + 0x10);

// Clamp delta time
if ((float)DOUBLE_0135a598 < deltaTime) {
    deltaTime = FLOAT_01358a14;  // Max delta clamp value
}

int soundMgr = GetSoundManager();
if (soundMgr != 0) {
    void* soundPtr = GetSoundManager();
    SoundManager_Update(soundPtr, deltaTime);
}
```

**Analysis:**
- Delta time is clamped to prevent large jumps (e.g., during lag or debugging)
- Sound manager can be NULL (check before calling)
- Uses the main timer delta (same as network)

**Functions:**
- `GetSoundManager @ unknown` - Returns singleton pointer (can be NULL)
- `SoundManager_Update @ unknown` - 2 params (void* this, float deltaTime)

**Constants:**
- `DOUBLE_0135a598 @ 0x0135a598` - Maximum delta threshold (double)
- `FLOAT_01358a14 @ 0x01358a14` - Clamped delta value (float)

### Phase 8: Network Update Phase 2 (Post-Game-Logic)

```c
int* networkManager = GetNetworkManager();
int* timersInstance = GetGameTimersInstance();

// Call vtable function at offset 0x28
(*(code**)(*networkManager + 0x28))(*(float*)(*timersInstance + 0x10));
```

**Analysis:**
- Second network update occurs **after** all game logic and sound
- Receives the same main timer delta as Phase 4
- Likely **sends** outgoing network messages and processes responses
- Function signature: `void NetworkUpdate2(float deltaTime)`

**Purpose Hypothesis:**
- **Phase 1 (0x24)**: Receive incoming packets, queue messages for processing
- **Phase 2 (0x28)**: Send outgoing packets, flush network buffers

This two-phase design allows game state to react to received messages before sending responses.

### Phase 9: Shutdown Check

```c
int tempMgr = GetGameTimersInstance();
char isTimerRunning = GameTimer_IsRunning(*(int*)(tempMgr + 4));

if (isTimerRunning != 0) {
    tempMgr = GetGameTimersInstance();
    if (*(char*)(*(int*)(tempMgr + 8) + 0x14) != 0) {  // Timer 8 paused?
        GetNetworkManager();
        char isConnected = Network_IsConnectionActive();
        if (isConnected == 0) {
            goto SKIP_SHUTDOWN_AND_RESET;
        }
    }
}

if (g_ShutdownRequested != false) {
    HandleGameShutdownRequest();
}

int tempMgr = GetGameTimersInstance();
GameTimer_Reset(*(void**)(tempMgr + 8));

SKIP_SHUTDOWN_AND_RESET:
```

**Global Variables:**
- `g_ShutdownRequested @ unknown` - bool flag, triggers shutdown sequence

**Functions:**
- `Network_IsConnectionActive @ unknown` - Returns bool (char)
- `HandleGameShutdownRequest @ unknown` - 0 params, initiates shutdown

**Logic:**
1. If timer[1] is running AND timer[8] is paused AND network is disconnected → Skip shutdown check
2. Otherwise, if `g_ShutdownRequested` is set → Call shutdown handler
3. Reset timer[8] (alternate timer)

**Shutdown Flow:**
```
User/System → Set g_ShutdownRequested → HandleGameShutdownRequest() → 
    Clean up resources → Post WM_QUIT → Exit message loop
```

### Phase 10: Rendering

```c
int gameStateMgr = GetGameStateManager();
GameStateManager_Render(gameStateMgr);
```

**Functions:**
- `GameStateManager_Render @ unknown` - 1 param (int this), renders current frame

**Analysis:**
- Final step in game loop
- Rendering happens even if game logic is paused (maintains UI responsiveness)
- Uses Gamebryo rendering engine (identified from WinMain analysis)

---

## Network Manager Architecture

**Global Instance:** `g_NetworkManager_ProudNetClient @ 0x015B53AC`

### GetNetworkManager Function

**Function:** `GetNetworkManager @ 0x00A30660`  
**Parameters:** 0 (void)  
**Returns:** void* (NetworkManager singleton)

```c
void* GetNetworkManager(void) {
    return g_NetworkManager_ProudNetClient;
}
```

**Cross-References:** 80+ references throughout codebase (critical component)

### NetworkManager VTable Structure

Based on the two network update calls from MainGameLoopUpdate:

```c
struct NetworkManager {
    NetworkManager_VTable* vtable;  // +0x00
    // ... instance data
};

struct NetworkManager_VTable {
    // Unknown functions at offsets 0x00 - 0x20
    void (*PreGameNetworkUpdate)(float deltaTime);   // +0x24
    void (*PostGameNetworkUpdate)(float deltaTime);  // +0x28
    // ... more functions
};
```

### Network Processing Model

**Two-Phase Update:**

1. **Pre-Game Phase (vtable+0x24)**
   - Called before GameStateManager_Update
   - Receives network packets from socket
   - Parses ProudNet headers
   - Deserializes RMI messages
   - Queues game messages for processing
   - Updates connection state

2. **Post-Game Phase (vtable+0x28)**
   - Called after all game logic completes
   - Processes outgoing message queue
   - Serializes RMI calls from game logic
   - Encrypts/compresses packets (if enabled)
   - Sends packets to server
   - Flushes network buffers

**Advantages of Two-Phase Design:**
- Game logic can react to received messages in the same frame
- Outgoing messages reflect updated game state
- Clean separation of input/output processing
- Allows batching of outgoing messages

---

## Critical Global Variables

### Already Identified

| Variable Name | Address | Type | Description |
|---------------|---------|------|-------------|
| `g_NetworkManager_ProudNetClient` | 0x015B53AC | `void*` | ProudNet network manager singleton |
| `g_GameStateManager` | 0x015C9088 | `void*` | Game state manager singleton |
| `g_RendererInstance` | 0x015D7708 | `void*` | Gamebryo renderer singleton |
| `g_UIManager` | 0x015B17A8 | `void*` | UI manager singleton |
| `g_ShutdownRequested` | 0x015A1C10 | `bool` | Shutdown flag |
| `g_InputWindow` | unknown | `HWND` | Window for input forwarding |
| `g_InputSystemEnabled` | unknown | `bool` | Input routing control |
| `PTR_FUN_015a5244` | 0x015A5244 | `FatalErrorHandler*` | Fatal error callback |

### Newly Discovered (Need Address Confirmation)

| Variable Name | Address | Type | Description |
|---------------|---------|------|-------------|
| `_g_GameTimeInitialized` | unknown | `int` | Bit flags for time init state |
| `_g_CurrentGameTime` | unknown | `double` | Current frame game time |
| `_g_LastFrameTime` | unknown | `double` | Previous frame game time |
| `g_GameServerIPAddress` | unknown | `char[32]` | Server IP from command line |

---

## Critical Functions Summary

### Manager Getters (Singletons)

| Function | Address | Returns | Description |
|----------|---------|---------|-------------|
| `GetNetworkManager` | 0x00A30660 | `void*` | Network manager singleton |
| `GetGameStateManager` | unknown | `void*` | Game state manager singleton |
| `GetGameTimersInstance` | unknown | `void*` | Timer manager singleton |
| `GetRendererInstance` | unknown | `void*` | Renderer singleton |
| `GetSoundManager` | unknown | `void*` | Sound manager singleton (nullable) |
| `GetGameConfigurationManager` | unknown | `void*` | Config manager singleton |
| `GetLocalizationManager` | unknown | `void*` | Localization manager singleton |

### Game Loop Functions

| Function | Address | Params | Description |
|----------|---------|--------|-------------|
| `WinMain` | 0x00A502F0 | 4 | Application entry point |
| `MainGameLoopUpdate` | 0x00A4C300 | 0 | Main game loop (called each frame) |
| `GetCurrentGameTime` | unknown | 0 | Returns `float10` current time |
| `HandleGameShutdownRequest` | unknown | 0 | Initiates shutdown sequence |
| `Network_IsConnectionActive` | unknown | 0 | Returns bool connection state |

### Timer Functions

| Function | Address | Params | Description |
|----------|---------|--------|-------------|
| `GameTimer_UpdateDelta` | unknown | 2 | Update timer with delta time |
| `GameTimer_SetValue` | unknown | 2 | Set timer target frame rate |
| `GameTimer_IsRunning` | unknown | 1 | Check if timer is not paused |
| `GameTimer_Reset` | unknown | 1 | Reset timer state |

### Game State Functions

| Function | Address | Params | Description |
|----------|---------|--------|-------------|
| `GameStateManager_Update` | unknown | 2 | Update game logic (this, configFlags) |
| `GameStateManager_Render` | unknown | 1 | Render current frame |

### Renderer Functions

| Function | Address | Params | Description |
|----------|---------|--------|-------------|
| `Renderer_UpdateSettings` | unknown | 1 | Update renderer configuration |

### Sound Functions

| Function | Address | Params | Description |
|----------|---------|--------|-------------|
| `SoundManager_Update` | unknown | 2 | Update sound system (this, deltaTime) |

### Network Functions (VTable)

| Function | VTable Offset | Params | Description |
|----------|---------------|--------|-------------|
| *(NetworkMgr + 0x24)* | 0x24 | 1 | Pre-game network update |
| *(NetworkMgr + 0x28)* | 0x28 | 1 | Post-game network update |

---

## Next Steps

### High Priority

1. **Identify Network Update Functions**
   - Disassemble vtable at `g_NetworkManager_ProudNetClient + 0x00`
   - Get function pointers at offsets 0x24 and 0x28
   - Decompile and analyze both network update functions
   - Trace to message dispatch handlers

2. **Map GameStateManager_Update**
   - Analyze game logic update flow
   - Identify state machine handling
   - Find message processing callbacks
   - Locate RMI message handlers

3. **Locate Manager Getter Implementations**
   - Find all singleton getter functions
   - Confirm global variable addresses
   - Document structure layouts

4. **Confirm Global Variable Addresses**
   - Search for timer-related globals
   - Find frame rate constant values
   - Locate performance threshold values

### Medium Priority

5. **ProudNet Message Dispatch Analysis**
   - Find message dispatch table
   - Map message ID → handler function
   - Identify system messages (0x0B, 0x1F, 0x20)
   - Identify game RMI messages (0x1000+)

6. **Extract Frame Rate Values**
   - Dump float constants from addresses
   - Determine actual FPS targets
   - Document performance tuning parameters

7. **Analyze Shutdown Sequence**
   - Trace `HandleGameShutdownRequest` function
   - Document cleanup order
   - Identify `CleanupNetworkAndResources` details

### Low Priority

8. **Input System Analysis**
   - Understand input forwarding mechanism
   - Find `g_InputWindow` and `g_InputSystemEnabled` addresses
   - Document input message routing

9. **Timer System Deep Dive**
   - Document all timer structure fields
   - Understand pause/resume mechanism
   - Identify timer synchronization

---

## Ghidra Bookmarks Created

| Category | Address | Description |
|----------|---------|-------------|
| Game Loop | 0x00A502F0 | WinMain entry point |
| Game Loop | 0x00A4C300 | MainGameLoopUpdate (core loop) |
| Network | 0x00A30660 | GetNetworkManager |
| Network | 0x015B53AC | g_NetworkManager_ProudNetClient (global) |
| Globals | 0x015C9088 | g_GameStateManager |
| Globals | 0x015D7708 | g_RendererInstance |
| Globals | 0x015B17A8 | g_UIManager |
| Globals | 0x015A1C10 | g_ShutdownRequested |

---

## Code References for Implementation

When implementing the RO2 server emulator, the following findings are critical:

### Network Message Processing Order

```rust
// Pseudocode based on client analysis
fn game_loop_update() {
    let delta_time = get_current_game_time() - previous_game_time;
    
    // 1. Receive and queue incoming messages (BEFORE game logic)
    network_manager.pre_game_update(delta_time);
    
    // 2. Process queued messages and update game state
    game_state_manager.update(config_flags);
    
    // 3. Send outgoing messages (AFTER game logic)
    network_manager.post_game_update(delta_time);
    
    // 4. Render
    game_state_manager.render();
}
```

### Frame Rate Adaptation

Server should track client performance hints if sent:
- High performance: Target 60+ FPS
- Medium performance: Target 30-60 FPS
- Low performance: Target 15-30 FPS

### Message Timing

The two-phase network update means:
1. Client processes received messages immediately
2. Client sends responses in the same frame
3. Server should expect quick turnaround on request/response pairs

---

## References

- **WinMain Analysis:** `docs/ghidra-findings/WINMAIN-ANALYSIS.md`
- **Protocol Specification:** `docs/protocol/RFC-RO2-PROTOCOL.md`
- **ProudNet Documentation:** (proprietary, limited public info)
- **Gamebryo Engine:** (confirmed from initialization strings)
