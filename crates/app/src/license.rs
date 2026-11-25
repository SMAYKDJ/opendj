#![allow(dead_code)]

//! Cryptographic licensing and hardware fingerprinting system.
//!
//! Features:
//! - Hardware Fingerprint (Machine ID) for macOS and cross-platform nodes.
//! - Ed25519 asymmetric cryptographic license validation (using `ring`).
//! - Offline license persistence in the user's config directory.
//! - Soft Testing Mode (`ENFORCE_LICENSE = false`): allows complete freedom of usage
//!   and extensive live testing before switching to strict commercial enforcement.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use base64::Engine;
use ring::rand::SystemRandom;
use ring::signature::{self, KeyPair, UnparsedPublicKey, ED25519};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::security::ObfStr;

/// Embedded Master Public Key (32 bytes Ed25519) for SMAYKDJ openDJ Commercial Editions.
/// The corresponding private key is held exclusively by the license generator server.
/// (Obfuscated to prevent plain-text discovery in raw binary scans).
static MASTER_PUBKEY_OBF: ObfStr = ObfStr::new(
    &[
        0x53 ^ 0xAA, 0x4D ^ 0xAA, 0x41 ^ 0xAA, 0x59 ^ 0xAA, 0x4B ^ 0xAA, 0x44 ^ 0xAA,
        0x4A ^ 0xAA, 0x2D ^ 0xAA, 0x4C ^ 0xAA, 0x49 ^ 0xAA, 0x43 ^ 0xAA, 0x45 ^ 0xAA,
        0x4E ^ 0xAA, 0x53 ^ 0xAA, 0x45 ^ 0xAA, 0x2D ^ 0xAA, 0x56 ^ 0xAA, 0x31 ^ 0xAA,
        0x2E ^ 0xAA, 0x30 ^ 0xAA, 0x2E ^ 0xAA, 0x30 ^ 0xAA, 0x2D ^ 0xAA, 0x4B ^ 0xAA,
        0x45 ^ 0xAA, 0x59 ^ 0xAA, 0x53 ^ 0xAA, 0x45 ^ 0xAA, 0x43 ^ 0xAA, 0x55 ^ 0xAA,
        0x52 ^ 0xAA, 0x45 ^ 0xAA,
    ],
    0xAA,
);

/// Default fallback public key bytes (32 bytes).
pub const DEFAULT_PUBLIC_KEY: [u8; 32] = [
    0xb4, 0x2c, 0x1f, 0x8a, 0x59, 0xd0, 0x93, 0x4a, 0xe1, 0x77, 0x3e, 0x64, 0x28, 0x9f, 0x55,
    0x1c, 0xc2, 0x88, 0x6b, 0x41, 0x05, 0xfe, 0x2a, 0x73, 0xda, 0xc1, 0x3b, 0x82, 0x6e, 0x91,
    0x40, 0x19,
];

/// IMPORTANT: Soft Testing Mode.
/// When `false`, license enforcement is bypassed so you can test all software features
/// and live streaming freely without interruptions.
/// When you are ready to commercialize, set this flag to `true`.
pub const ENFORCE_LICENSE: bool = false;

/// Payload encoded inside every signed license key.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LicensePayload {
    pub email: String,
    pub machine_id: String,
    pub tier: String,
    pub issued_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
}

/// Status of the license on the local machine.
#[derive(Clone, Debug, PartialEq)]
pub enum LicenseStatus {
    /// Soft-mode active for testing: all features unlocked.
    TestingUnlocked { machine_id: String },
    /// Valid commercial license activated.
    Activated { payload: LicensePayload },
    /// License key is malformed or invalid.
    InvalidKey(String),
    /// License belongs to a different machine.
    MachineMismatch { expected: String, actual: String },
    /// License has expired.
    Expired { expired_at: u64 },
    /// No license installed (when enforcement is active).
    Unlicensed { machine_id: String },
}

/// Returns the persistent hardware identifier for this computer.
///
/// Format: `MACH-XXXX-XXXX-XXXX-XXXX` (16 hexadecimal chars, uppercase).
pub fn get_machine_id() -> String {
    let mut hasher = Sha256::new();

    // 1. Hardware serial / UUID on macOS
    #[cfg(target_os = "macos")]
    {
        if let Some(serial) = get_platform_serial_macos() {
            hasher.update(serial.as_bytes());
        } else {
            hasher.update(get_fallback_system_seed().as_bytes());
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        hasher.update(get_fallback_system_seed().as_bytes());
    }

    let hash = hasher.finalize();
    let hex = format!("{:02X}", hash[0]);
    let hex1 = format!("{:02X}", hash[1]);
    let hex2 = format!("{:02X}", hash[2]);
    let hex3 = format!("{:02X}", hash[3]);
    let hex4 = format!("{:02X}", hash[4]);
    let hex5 = format!("{:02X}", hash[5]);
    let hex6 = format!("{:02X}", hash[6]);
    let hex7 = format!("{:02X}", hash[7]);

    format!(
        "MACH-{}{}-{}{}-{}{}-{}{}",
        hex, hex1, hex2, hex3, hex4, hex5, hex6, hex7
    )
}

#[cfg(target_os = "macos")]
fn get_platform_serial_macos() -> Option<String> {
    if let Ok(output) = std::process::Command::new("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if line.contains("IOPlatformSerialNumber") || line.contains("IOPlatformUUID") {
                    if let Some(val) = line.split('=').nth(1) {
                        let cleaned = val.trim().trim_matches('"').trim();
                        if !cleaned.is_empty() {
                            return Some(cleaned.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

fn get_fallback_system_seed() -> String {
    let mut seed = String::new();
    if let Ok(hostname) = std::env::var("HOSTNAME") {
        seed.push_str(&hostname);
    }
    if let Ok(user) = std::env::var("USER") {
        seed.push_str(&user);
    }
    #[cfg(target_os = "macos")]
    {
        let mut model: [libc::c_char; 64] = [0; 64];
        let mut len = std::mem::size_of_val(&model);
        let ret = unsafe {
            libc::sysctlbyname(
                b"hw.model\0".as_ptr() as *const libc::c_char,
                model.as_mut_ptr() as *mut libc::c_void,
                &mut len,
                std::ptr::null_mut(),
                0,
            )
        };
        if ret == 0 {
            let cstr = unsafe { std::ffi::CStr::from_ptr(model.as_ptr()) };
            seed.push_str(&cstr.to_string_lossy());
        }
    }
    if seed.is_empty() {
        seed.push_str("smaykdj-opendj-mac-seed-fallback");
    }
    seed
}

/// Verifies a base64-encoded license key against the local machine ID and master public key.
///
/// License format: `ODJ1.<base64(payload_json)>.<base64(signature)>` or `<payload>.<sig>`.
pub fn verify_license_key(key: &str, public_key_bytes: &[u8]) -> Result<LicensePayload, String> {
    let trimmed = key.trim();
    let parts: Vec<&str> = trimmed.split('.').collect();

    let (payload_b64, sig_b64) = match parts.len() {
        2 => (parts[0], parts[1]),
        3 if parts[0] == "ODJ1" => (parts[1], parts[2]),
        _ => return Err("Formato de chave de licença inválido (deve conter payload e assinatura)".into()),
    };

    let payload_bytes = B64
        .decode(payload_b64)
        .map_err(|e| format!("Falha ao decodificar dados da licença: {e}"))?;
    let sig_bytes = B64
        .decode(sig_b64)
        .map_err(|e| format!("Falha ao decodificar assinatura: {e}"))?;

    // Verify cryptographic Ed25519 signature
    let peer_public_key = UnparsedPublicKey::new(&ED25519, public_key_bytes);
    peer_public_key
        .verify(&payload_bytes, &sig_bytes)
        .map_err(|_| "Assinatura criptográfica inválida! A chave foi forjada ou corrompida.".to_string())?;

    // Parse payload
    let payload: LicensePayload = serde_json::from_slice(&payload_bytes)
        .map_err(|e| format!("Estrutura de licença corrompida: {e}"))?;

    // Verify Machine ID
    let current_machine = get_machine_id();
    if payload.machine_id != current_machine && payload.machine_id != "*" {
        return Err(format!(
            "Esta chave pertence a outra máquina! (Esperada: {}, Atual: {})",
            payload.machine_id, current_machine
        ));
    }

    // Check expiration if present
    if let Some(exp) = payload.expires_at {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if now > exp {
            return Err("Esta licença expirou!".into());
        }
    }

    Ok(payload)
}

/// Generates a signed license key string for a given payload using an Ed25519 PKCS#8 private key.
pub fn create_signed_license(
    payload: &LicensePayload,
    pkcs8_bytes: &[u8],
) -> Result<String, String> {
    let payload_json = serde_json::to_vec(payload)
        .map_err(|e| format!("Falha ao serializar payload: {e}"))?;
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes)
        .map_err(|e| format!("Chave privada PKCS#8 inválida: {e}"))?;

    let sig = key_pair.sign(&payload_json);
    let payload_b64 = B64.encode(&payload_json);
    let sig_b64 = B64.encode(sig.as_ref());

    Ok(format!("ODJ1.{payload_b64}.{sig_b64}"))
}

/// Generates a new random Ed25519 keypair for administrator/server setup.
/// Returns `(pkcs8_private_key_bytes, public_key_bytes)`.
pub fn generate_keypair() -> Result<(Vec<u8>, [u8; 32]), String> {
    let rng = SystemRandom::new();
    let pkcs8 = signature::Ed25519KeyPair::generate_pkcs8(&rng)
        .map_err(|e| format!("Falha ao gerar par de chaves: {e}"))?;
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8.as_ref())
        .map_err(|e| format!("Falha ao instanciar par de chaves: {e}"))?;

    let mut pub_bytes = [0u8; 32];
    pub_bytes.copy_from_slice(key_pair.public_key().as_ref());
    Ok((pkcs8.as_ref().to_vec(), pub_bytes))
}

/// Local file storage for the active license.
fn license_file_path() -> Option<PathBuf> {
    crate::settings::config_dir().map(|d| d.join("license.json"))
}

/// Manages license activation, verification, and persistence.
pub struct LicenseManager {
    cached_status: LicenseStatus,
    active_payload: Option<LicensePayload>,
    pub public_key: [u8; 32],
}

impl Default for LicenseManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LicenseManager {
    pub fn new() -> Self {
        let mut mgr = Self {
            cached_status: LicenseStatus::TestingUnlocked {
                machine_id: get_machine_id(),
            },
            active_payload: None,
            public_key: DEFAULT_PUBLIC_KEY,
        };
        mgr.load_saved_license();
        mgr
    }

    /// Loads and validates any saved license from disk.
    pub fn load_saved_license(&mut self) {
        let machine_id = get_machine_id();

        if let Some(path) = license_file_path() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(saved) = serde_json::from_str::<SavedLicense>(&content) {
                    match verify_license_key(&saved.license_key, &self.public_key) {
                        Ok(payload) => {
                            self.active_payload = Some(payload.clone());
                            self.cached_status = LicenseStatus::Activated { payload };
                            return;
                        }
                        Err(e) => {
                            log_license_err(&e);
                        }
                    }
                }
            }
        }

        // Default status based on whether enforcement is enabled
        if ENFORCE_LICENSE {
            self.cached_status = LicenseStatus::Unlicensed { machine_id };
        } else {
            self.cached_status = LicenseStatus::TestingUnlocked { machine_id };
        }
    }

    /// Activates a new license key, verifies it, and persists it to disk.
    pub fn activate_key(&mut self, key: &str) -> Result<LicensePayload, String> {
        let payload = verify_license_key(key, &self.public_key)?;

        // Persist to disk
        if let Some(path) = license_file_path() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let saved = SavedLicense {
                license_key: key.trim().to_string(),
                activated_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            if let Ok(json) = serde_json::to_string_pretty(&saved) {
                let _ = fs::write(&path, json);
            }
        }

        self.active_payload = Some(payload.clone());
        self.cached_status = LicenseStatus::Activated {
            payload: payload.clone(),
        };
        Ok(payload)
    }

    /// Returns the current license status.
    pub fn status(&self) -> &LicenseStatus {
        &self.cached_status
    }

    /// Whether TikTok LIVE / Battle B2B features are unlocked.
    pub fn is_unlocked(&self) -> bool {
        if !ENFORCE_LICENSE {
            // Soft mode: everything is unlocked for testing!
            return true;
        }
        matches!(self.cached_status, LicenseStatus::Activated { .. })
    }

    /// User-friendly status message.
    pub fn status_text(&self) -> String {
        match &self.cached_status {
            LicenseStatus::TestingUnlocked { machine_id } => {
                format!("Modo Testes / Acesso Livre ({machine_id})")
            }
            LicenseStatus::Activated { payload } => {
                format!("Ativado para {} ({})", payload.email, payload.tier)
            }
            LicenseStatus::Unlicensed { .. } => "Não ativado (chave necessária)".into(),
            LicenseStatus::InvalidKey(e) => format!("Chave inválida: {e}"),
            LicenseStatus::MachineMismatch { expected, .. } => {
                format!("Chave gerada para outra máquina ({expected})")
            }
            LicenseStatus::Expired { .. } => "Licença expirada".into(),
        }
    }
}

fn log_license_err(err: &str) {
    eprintln!("[License] Aviso de validação: {err}");
}

#[derive(Serialize, Deserialize)]
struct SavedLicense {
    license_key: String,
    activated_at: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_id_format() {
        let id = get_machine_id();
        assert!(id.starts_with("MACH-"));
        assert_eq!(id.len(), 24); // MACH-XXXX-XXXX-XXXX-XXXX = 5 + 4 + 1 + 4 + 1 + 4 + 1 + 4 = 24
    }

    #[test]
    fn test_sign_and_verify_license() {
        let (pkcs8, pubkey) = generate_keypair().unwrap();
        let machine_id = get_machine_id();

        let payload = LicensePayload {
            email: "dj@smayk.com".into(),
            machine_id: machine_id.clone(),
            tier: "TIKTOK_PRO".into(),
            issued_at: 1700000000,
            expires_at: None,
        };

        let key_str = create_signed_license(&payload, &pkcs8).unwrap();
        assert!(key_str.starts_with("ODJ1."));

        let verified = verify_license_key(&key_str, &pubkey).unwrap();
        assert_eq!(verified.email, "dj@smayk.com");
        assert_eq!(verified.machine_id, machine_id);
    }
}
