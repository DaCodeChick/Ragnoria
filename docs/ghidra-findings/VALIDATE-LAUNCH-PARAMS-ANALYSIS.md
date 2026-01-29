# ValidateGameLaunchParameters() Analysis

**File:** Rag2.exe  
**Function Address:** 0x00a501d0  
**Function Name:** `ValidateGameLaunchParameters` (named based on behavior)  
**Date:** 2026-01-28  

## Summary

The game requires the `-FromLauncher` command-line flag to bypass validation checks. Without this flag, the game displays error dialogs and refuses to launch.

## Critical Discovery

**The `-FromLauncher` flag is MANDATORY** - without it, the game:
1. Shows Korean error dialog about "Updater" 
2. Checks for `../Updater.exe` file
3. Shows second error if file doesn't exist
4. Exits without launching

## Decompiled Code

```c
void ValidateGameLaunchParameters(void)
{
  byte bVar1;
  char cVar2;
  byte *pbVar3;
  int iVar4;
  char *pcVar5;
  bool bVar6;
  char local_210 [260];
  char local_10c [260];
  uint local_8;
  
  local_8 = g_StackCanary ^ (uint)&stack0xfffffffc;
  
  // Get command line arguments
  pbVar3 = (byte *)FUN_00a4f9e0();
  
  // Compare command line with "-FromLauncher"
  pcVar5 = "-FromLauncher";
  do {
    bVar1 = *pcVar5;
    bVar6 = bVar1 < *pbVar3;
    if (bVar1 != *pbVar3) {
LAB_00a50210:
      iVar4 = (1 - (uint)bVar6) - (uint)(bVar6 != 0);
      goto LAB_00a50215;
    }
    if (bVar1 == 0) break;
    bVar1 = ((byte *)pcVar5)[1];
    bVar6 = bVar1 < pbVar3[1];
    if (bVar1 != pbVar3[1]) goto LAB_00a50210;
    pcVar5 = (char *)((byte *)pcVar5 + 2);
    pbVar3 = pbVar3 + 2;
  } while (bVar1 != 0);
  iVar4 = 0;
  
LAB_00a50215:
  // If "-FromLauncher" was found (iVar4 == 0), return successfully
  if (iVar4 == 0) {
    __security_check_cookie(local_8 ^ (uint)&stack0xfffffffc);
    return;  // SUCCESS - continue to game initialization
  }
  
  // ERROR PATH: -FromLauncher NOT found
  
  // Build error message about "Updater"
  strcpy_s(local_210, 0x104, "Updater");
  FUN_00518fa0(local_10c, s__s___013dfcc4, local_210);
  
  // Display first error dialog
  // Korean: "Updater를 통해 다시 실행 하겠습니다."
  // English: "It will be run again through the Updater."
  if (PTR_FUN_015a5244 != (undefined *)0x0) {
    (*(code *)PTR_FUN_015a5244)(local_10c, "ERROR", 0);
  }
  
  // Check if ../Updater.exe exists
  FUN_00518fa0(local_10c, "../%s.exe", local_210);
  cVar2 = FUN_00a4b390(local_10c);  // File exists check
  
  // If Updater.exe doesn't exist, show second error
  // Korean: "Updater.exe 파일이 존재하지 않습니다."
  // English: "The Updater.exe file does not exist."
  if (cVar2 == '\0') {
    FUN_00518fa0(local_10c, s__s_exe___013dfc98, local_210);
    if (PTR_FUN_015a5244 != (undefined *)0x0) {
      (*(code *)PTR_FUN_015a5244)(local_10c, "Error", 0);
    }
  }
  
  __security_check_cookie(local_8 ^ (uint)&stack0xfffffffc);
  return;
}
```

## Assembly Listing

```asm
; Function start
00a501d0:  push   ebp
00a501d1:  mov    ebp, esp
00a501d3:  sub    esp, 0x320
00a501d9:  mov    eax, [g_StackCanary]
00a501de:  xor    eax, ebp
00a501e0:  mov    [ebp-0x4], eax

; Get command line
00a501e3:  call   FUN_00a4f9e0

; String comparison with "-FromLauncher"
00a501e8:  mov    esi, eax
00a501ea:  mov    edi, offset s_-FromLauncher
00a501ef:  movsx  eax, byte ptr [esi]
00a501f2:  movsx  ecx, byte ptr [edi]
00a501f5:  sub    eax, ecx
00a501f7:  jne    LAB_00a50210
...

; Success path - return
00a50215:  test   eax, eax
00a50217:  jne    ERROR_PATH
00a50219:  mov    al, 0x1
00a5021b:  mov    ecx, [ebp-0x4]
00a5021e:  xor    ecx, ebp
00a50220:  call   __security_check_cookie
00a50225:  mov    esp, ebp
00a50227:  pop    ebp
00a50228:  ret

; Error path - show Updater errors
00a50229:  push   0x13dfce4  ; "Updater"
00a5022e:  lea    eax, [ebp-0x20c]
00a50234:  push   0x104
00a50239:  push   eax
00a5023a:  call   strcpy_s
...
```

## Error Strings

**String at 0x013dfce4:** "Updater"

**First Error Dialog at 0x013dfcc4:**  
```
Korean: Updater를 통해 다시 실행 하겠습니다.
English: "It will be run again through the Updater."
Garbled display: "Updater를 통해 다시 실행 하겠습니다." (EUC-KR interpreted as Windows-1252)
```

**Second Error Dialog at 0x013dfc98:**  
```
Korean: Updater.exe 파일이 존재하지 않습니다.
English: "The Updater.exe file does not exist."
Garbled display: "Updater.exe파일이 존재하지 않습니다." (EUC-KR interpreted as Windows-1252)
```

**Note:** Error text appears garbled on non-Korean Windows systems because the game uses EUC-KR encoding but the system interprets it as Windows-1252/CP1252.

### Error Dialog Screenshots

When launching without `-FromLauncher` flag, users see two sequential error dialogs:

**Dialog 1 (Screenshot 2026-01-28 154714.png):**
```
Title: ERROR
Garbled Text: Updater를 통해 다시 실행 하겠습니다.
Actual Korean: Updater를 통해 다시 실행 하겠습니다.
English: "It will be run again through the Updater."
```

**Dialog 2 (Screenshot 2026-01-28 154728.png):**
```
Title: Error
Garbled Text: Updater.exe파일이 존재하지 않습니다.
Actual Korean: Updater.exe 파일이 존재하지 않습니다.
English: "The Updater.exe file does not exist."
```

Both dialogs are **misleading** - the real issue is not a missing Updater.exe file, but the missing `-FromLauncher` command-line flag!

## String References

```
Address    String             Usage
-------    ------             -----
013dfce4   "Updater"          Copied to buffer for error message
013dfcc4   "%s를..."          First error dialog format
013dfc98   "%s.exe..."        Second error dialog format
```

## Cross-References

**Function called from:**
- WinMain (0x00a4f000) during game initialization
- Called after `ParseGameLaunchArguments()`
- Called before main game loop initialization

**Functions called:**
- `FUN_00a4f9e0()` - Get command line string
- `strcpy_s()` - String operations
- `FUN_00518fa0()` - String formatting (sprintf equivalent)
- `PTR_FUN_015a5244()` - Error dialog display function (MessageBox wrapper)
- `FUN_00a4b390()` - File existence check

## Solution

To launch Rag2.exe successfully, **ALWAYS include `-FromLauncher` in command line**:

### Option 1: Minimum (Recommended)
```bash
Rag2.exe -FromLauncher
```

### Option 2: With server parameters
```bash
Rag2.exe -FromLauncher 127.0.0.1 7101
```

### Option 3: With named parameters
```bash
Rag2.exe -FromLauncher /IP=127.0.0.1 /PORT=7101
```

## Notes

1. **No Updater.exe exists** in RO2 installation - the check is legacy/deprecated
2. **Error text is Korean (EUC-KR encoding)** - appears garbled on non-Korean Windows systems:
   - First error: "Updater를 통해 다시 실행 하겠습니다." appears as "Updater를 통해 다시 실행 하겠습니다."
   - Second error: "Updater.exe 파일이 존재하지 않습니다." appears as "Updater.exe파일이 존재하지 않습니다."
3. **String comparison is case-sensitive** - must be exactly `-FromLauncher`
4. **Dash prefix is required** - not slash (`-FromLauncher` not `/FromLauncher`)
5. **RO2Client.exe uses `-FromUpdater`** - but Rag2.exe checks for `-FromLauncher`!
6. **Error messages are misleading** - they don't indicate the real problem (missing flag)

## Related Functions

- `ParseGameLaunchArguments()` - Converts Unicode command line to ANSI
- `GetGameServerIPFromCommandLine()` - Extracts server IP from arguments
- `ValidateGameLaunchParameters()` - **This function** - checks for `-FromLauncher`

## Testing Results

| Command Line | Result |
|-------------|--------|
| *(none)* | ❌ Shows Updater error dialogs |
| `-FromUpdater` | ❌ Wrong flag, shows errors |
| `/FromLauncher` | ❌ Wrong prefix (slash), shows errors |
| `-FromLauncher` | ✅ Success! Game launches |
| `-FromLauncher 127.0.0.1 7101` | ✅ Success + custom server |

## Implementation

The custom launcher now includes `-FromLauncher` by default:

```rust
// crates/launcher/src/main.rs
let args = vec![
    String::from("-FromLauncher"),
    // ... optional: IP, PORT parameters
];

Command::new(&game_path)
    .args(&args)
    .current_dir(&game_root_dir)
    .spawn()?;
```

## Conclusion

**The `-FromLauncher` flag is mandatory for Rag2.exe to launch.** This was discovered through Ghidra decompilation of the `ValidateGameLaunchParameters()` function. The error dialogs about "Updater.exe" are misleading - they simply indicate the missing flag, not an actual missing file.
