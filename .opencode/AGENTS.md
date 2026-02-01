# Agent Instructions for Ragnoria Project

## CRITICAL: Function Naming Protocol (DICK - Document, Identify, Categorize, Know)

When working with Ghidra analysis of the RO2 client, you MUST follow the DICK protocol:

### 1. Document
Every time you encounter an unnamed function (e.g., `FUN_00xxxxxx`), immediately document it in your analysis.

### 2. Identify
Use Ghidra tools to understand what the function does:
- Read the decompiled code
- Check cross-references (callers and callees)
- Examine the assembly if needed
- Look at function parameters and return type

### 3. Categorize
Based on the function's behavior, determine its category:
- Network/Protocol functions
- UI/GameUI functions
- Protection/Security functions
- Parameter/Config functions
- Manager/System functions

### 4. Know (Rename in Ghidra)
**IMMEDIATELY** use `ghidra_rename_symbol` to give it a proper name:
```
ghidra_rename_symbol(
    identifier="FUN_00xxxxxx",
    new_name="DescriptiveFunctionName",
    target_type="function"
)
```

## Naming Conventions

### Function Prefixes by Category
- **Network**: `Send*`, `Recv*`, `Handle*Packet`, `Network*`
- **UI**: `GameUI_*`, `ShowDialog*`, `Update*UI`
- **Protection**: `Check*Protection`, `Verify*`, `Initialize*Guard`
- **Parameters**: `Get*Parameter`, `Parse*Param`, `Validate*`
- **Managers**: `*Manager_*`, `Get*Manager`
- **Stage/State**: `Stage*_Enter`, `Stage*_Update`, `Stage*_Handler`

### Examples of Good Names
- `FUN_00a4ff60` → `IsStarterModeEnabled`
- `FUN_00a4fe00` → `GetStarterParameterValue`
- `FUN_00636830` → `StageLogin_HandleUIEvent`
- `FUN_00a6e200` → `GameUI_SendStarterModeLogin`

## When NOT to Rename
- Functions you haven't fully analyzed yet
- Functions with unclear purpose (mark with `TODO_` prefix)
- Auto-generated thunks or wrappers (unless they're important)

## Workflow for Ghidra Analysis

1. **Encounter unnamed function** → Stop and DICK it
2. **Read decompiled code** → Understand what it does
3. **Check xrefs** → See who calls it and why
4. **Name it immediately** → Don't leave it as FUN_*
5. **Document in findings** → Add to analysis docs
6. **Continue work** → Now with properly named function

## Documentation Requirements

After renaming, update:
- `/docs/ghidra-findings/FUNCTION-RENAMING-PROGRESS.md`
- Any analysis documents referencing the function
- Code comments in the Rust server if relevant

## Why This Matters

Leaving functions unnamed is **technical debt** that:
- Makes future analysis harder
- Wastes time re-analyzing the same function
- Creates confusion in documentation
- Slows down debugging and implementation

**Every unnamed function is a missed opportunity to understand the codebase better.**

## Enforcement

If you encounter an unnamed function and don't DICK it:
1. You'll be reminded (rudely)
2. You'll have to go back and do it anyway
3. You'll waste everyone's time

**NO EXCEPTIONS. DICK EVERY FUNCTION YOU TOUCH.**
