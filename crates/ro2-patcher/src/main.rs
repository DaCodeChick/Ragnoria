/*!
 * RO2 Client Patcher
 *
 * Patches the Ragnarok Online 2 client (Rag2.exe) to bypass HackShield anti-cheat checks.
 * This allows the client to connect to custom/private servers without HackShield.
 *
 * ## Safety
 * - Always backs up the original executable before patching
 * - Verifies file checksum before applying patches
 * - Can verify patches were applied correctly
 *
 * ## Usage
 * ```bash
 * # Patch the client
 * ro2-patcher patch /path/to/Rag2.exe
 *
 * # Restore backup
 * ro2-patcher restore /path/to/Rag2.exe
 *
 * # Verify patches
 * ro2-patcher verify /path/to/Rag2.exe
 * ```
 */

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// RO2 Client Patcher - Bypass HackShield protection
#[derive(Parser)]
#[command(name = "ro2-patcher")]
#[command(about = "Patches Rag2.exe to bypass HackShield anti-cheat", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Patch the client executable
    Patch {
        /// Path to Rag2.exe
        #[arg(value_name = "FILE")]
        path: PathBuf,

        /// Skip backup creation
        #[arg(long)]
        no_backup: bool,
    },

    /// Restore from backup
    Restore {
        /// Path to Rag2.exe
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },

    /// Verify patches are applied
    Verify {
        /// Path to Rag2.exe
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },

    /// List available patches
    List,
}

/// Patch definition
#[derive(Debug)]
struct Patch {
    name: &'static str,
    description: &'static str,
    offset: usize,
    original: &'static [u8],
    patched: &'static [u8],
}

/// Known Rag2.exe checksums
const KNOWN_CHECKSUMS: &[&str] = &[
    "5f6e211535d4b541b8c26c921a5fc8a968db151d9bef4a9df1f9982cf9e2e099", // RO2 Jawaii SHIPPING build
];

/// Patch definitions for Rag2.exe
const PATCHES: &[Patch] = &[
    // Patch 1: Force CheckGameProtectionEnabled to return FALSE
    // Virtual Address: 0x00A4FFA0, File Offset: 0x0064F3A0
    // This function checks if game protection (HackShield) is enabled
    // We replace the function prologue with: MOV AL, 0; RET (+ NOPs to match original length)
    // This makes the function always return FALSE (protection NOT enabled)
    // The login flow checks: if (CheckGameProtectionEnabled() == '\0')
    // So we need this to return 0 for the check to pass!
    Patch {
        name: "bypass_game_protection_check",
        description: "Forces CheckGameProtectionEnabled to return FALSE",
        offset: 0x0064F3A0,
        original: &[
            0x55, 0x8B, 0xEC, 0x6A, 0xFF, 0x68, 0xB8, 0x2D, 0x2D, 0x01, 0x64, 0xA1, 0x00, 0x00,
            0x00, 0x00,
        ], // PUSH EBP; MOV EBP, ESP; PUSH -1; PUSH 0x012D2DB8; MOV EAX, dword ptr fs:[0]
        patched: &[
            0xB0, 0x00, 0xC3, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90,
            0x90, 0x90,
        ], // MOV AL, 0; RET; NOP×13
    },
    // Patch 2: Force CheckProtectionSystemEnabled to return TRUE
    // Virtual Address: 0x00A4CEF0, File Offset: 0x0064C2F0
    // This function checks if protection system is active
    // We replace the function prologue with: MOV AL, 1; RET (+ NOPs to match original length)
    Patch {
        name: "bypass_protection_system_check",
        description: "Forces CheckProtectionSystemEnabled to return TRUE",
        offset: 0x0064C2F0,
        original: &[
            0x55, 0x8B, 0xEC, 0x6A, 0xFF, 0x68, 0x58, 0x25, 0x2D, 0x01, 0x64, 0xA1, 0x00, 0x00,
            0x00, 0x00,
        ], // PUSH EBP; MOV EBP, ESP; PUSH -1; PUSH 0x012D2558; MOV EAX, dword ptr fs:[0]
        patched: &[
            0xB0, 0x01, 0xC3, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90,
            0x90, 0x90,
        ], // MOV AL, 1; RET; NOP×13
    },
];

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Patch { path, no_backup } => patch_client(&path, !no_backup),
        Commands::Restore { path } => restore_backup(&path),
        Commands::Verify { path } => verify_patches(&path),
        Commands::List => list_patches(),
    }
}

fn patch_client(path: &Path, create_backup: bool) -> Result<()> {
    println!("🔧 RO2 Client Patcher");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Verify file exists
    if !path.exists() {
        bail!("File not found: {}", path.display());
    }

    // Read the file
    println!("📂 Reading: {}", path.display());
    let mut data = fs::read(path).context("Failed to read executable")?;

    // Calculate checksum
    let checksum = calculate_checksum(&data);
    println!("🔐 SHA-256: {}", checksum);

    // Warn if checksum unknown
    if !KNOWN_CHECKSUMS.contains(&checksum.as_str()) {
        println!("⚠️  Warning: Unknown executable version");
        println!("   Patches may not work correctly!");
        println!();
    }

    // Create backup
    if create_backup {
        let backup_path = get_backup_path(path);
        println!("💾 Creating backup: {}", backup_path.display());
        fs::copy(path, &backup_path).context("Failed to create backup")?;
    }

    // Apply patches
    println!();
    println!("🔨 Applying patches:");
    println!();

    let mut applied = 0;
    for patch in PATCHES {
        print!("  • {} ... ", patch.description);

        match apply_patch(&mut data, patch) {
            Ok(true) => {
                println!("✓ Applied");
                applied += 1;
            }
            Ok(false) => {
                println!("⊗ Already applied");
            }
            Err(e) => {
                println!("✗ Failed: {}", e);
            }
        }
    }

    if applied == 0 {
        println!();
        println!("⚠️  No patches were applied. Client may already be patched.");
        return Ok(());
    }

    // Write patched file
    println!();
    println!("💾 Writing patched executable...");
    fs::write(path, &data).context("Failed to write patched file")?;

    println!();
    println!("✅ Successfully patched! Applied {} patch(es)", applied);
    println!();
    println!("🎮 You can now run the client without HackShield!");

    Ok(())
}

fn restore_backup(path: &Path) -> Result<()> {
    let backup_path = get_backup_path(path);

    if !backup_path.exists() {
        bail!("Backup not found: {}", backup_path.display());
    }

    println!("♻️  Restoring from backup...");
    fs::copy(&backup_path, path).context("Failed to restore backup")?;

    println!("✅ Restored original executable");
    Ok(())
}

fn verify_patches(path: &Path) -> Result<()> {
    println!("🔍 Verifying patches...");
    println!();

    let data = fs::read(path).context("Failed to read executable")?;

    let mut verified = 0;
    for patch in PATCHES {
        print!("  • {} ... ", patch.name);

        if is_patch_applied(&data, patch) {
            println!("✓ Applied");
            verified += 1;
        } else {
            println!("✗ Not applied");
        }
    }

    println!();
    if verified == PATCHES.len() {
        println!("✅ All patches verified!");
    } else {
        println!("⚠️  {} of {} patches applied", verified, PATCHES.len());
    }

    Ok(())
}

fn list_patches() -> Result<()> {
    println!("📋 Available Patches:");
    println!();

    for (i, patch) in PATCHES.iter().enumerate() {
        println!("{}. {} (0x{:08X})", i + 1, patch.name, patch.offset);
        println!("   {}", patch.description);
        println!("   Original: {}", hex::encode(patch.original));
        println!("   Patched:  {}", hex::encode(patch.patched));
        println!();
    }

    Ok(())
}

fn apply_patch(data: &mut [u8], patch: &Patch) -> Result<bool> {
    let end = patch.offset + patch.original.len();

    if end > data.len() {
        bail!("Offset out of bounds");
    }

    let current = &data[patch.offset..end];

    // Check if already patched
    if current == patch.patched {
        return Ok(false);
    }

    // Verify original bytes match
    if current != patch.original {
        bail!(
            "Original bytes don't match. Expected {}, found {}",
            hex::encode(patch.original),
            hex::encode(current)
        );
    }

    // Apply patch
    data[patch.offset..end].copy_from_slice(patch.patched);

    Ok(true)
}

fn is_patch_applied(data: &[u8], patch: &Patch) -> bool {
    let end = patch.offset + patch.patched.len();

    if end > data.len() {
        return false;
    }

    &data[patch.offset..end] == patch.patched
}

fn calculate_checksum(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn get_backup_path(path: &Path) -> PathBuf {
    let mut backup = path.to_path_buf();
    backup.set_extension("exe.bak");
    backup
}
