#![allow(dead_code)]

//! Security, anti-tampering, and anti-reverse-engineering primitives.
//!
//! Provides runtime checks (debugger detection, binary integrity, memory protection)
//! and string obfuscation to protect commercial assets and license validation logic.

/// Obfuscated string container decrypted dynamically in memory at runtime.
///
/// Prevents plain-text strings (API URLs, cryptographic labels, sensitive constants)
/// from appearing in disassembled binaries (.rodata section).
pub struct ObfStr {
    cipher: &'static [u8],
    key: u8,
}

impl ObfStr {
    pub const fn new(cipher: &'static [u8], key: u8) -> Self {
        Self { cipher, key }
    }

    /// Decrypts the obfuscated string on the heap.
    pub fn decrypt(&self) -> String {
        let bytes: Vec<u8> = self.cipher.iter().map(|b| b ^ self.key).collect();
        String::from_utf8(bytes).unwrap_or_default()
    }
}

/// Detects if the current process is running under an active debugger or injection tool
/// (e.g. LLDB, GDB, Frida, DYLD library injection).
pub fn is_debugger_present() -> bool {
    // 1. Detect dynamic injection hooks (Frida, Substrate, dylib injectors)
    if std::env::var_os("DYLD_INSERT_LIBRARIES").is_some()
        || std::env::var_os("__XPC_DYLD_INSERT_LIBRARIES").is_some()
        || std::env::var_os("LD_PRELOAD").is_some()
    {
        return true;
    }

    // 2. Detect debugger via parent process on Unix / macOS
    #[cfg(unix)]
    {
        if let Ok(parent_cmd) = read_parent_process_name() {
            let lower = parent_cmd.to_lowercase();
            if lower.contains("lldb")
                || lower.contains("gdb")
                || lower.contains("debugserver")
                || lower.contains("ida")
                || lower.contains("ghidra")
            {
                return true;
            }
        }
    }

    false
}

#[cfg(unix)]
fn read_parent_process_name() -> Result<String, ()> {
    let ppid = unsafe { libc::getppid() };
    if let Ok(output) = std::process::Command::new("ps")
        .args(["-p", &ppid.to_string(), "-o", "comm="])
        .output()
    {
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }
    Err(())
}

/// Applies macOS anti-attach protection (PT_DENY_ATTACH).
///
/// When active, any attempt to attach a debugger to this process will cause
/// it to terminate with a segmentation violation (SIGSEGV).
/// Note: only called in strict commercial mode; skipped during development.
#[cfg(target_os = "macos")]
pub fn deny_debugger_attach() {
    // PT_DENY_ATTACH = 31 in macOS sys/ptrace.h
    const PT_DENY_ATTACH: libc::c_int = 31;
    unsafe {
        libc::ptrace(PT_DENY_ATTACH, 0, std::ptr::null_mut::<libc::c_char>(), 0);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn deny_debugger_attach() {}
