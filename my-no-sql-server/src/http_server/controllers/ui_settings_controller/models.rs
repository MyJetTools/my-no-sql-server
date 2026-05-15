use my_http_server::macros::*;
use my_http_server::types::RawDataTyped;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

const DEFAULT_WARN_MS: u32 = 3_000;
const DEFAULT_BAD_MS: u32 = 10_000;

/// On-disk shape (`ui-settings.json`). Salt + hash for the MCP write
/// password are stored here; the plaintext value is never persisted
/// and never returned by the GET endpoint. See `SettingsPublicModel`
/// for the wire shape returned to UI clients.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiSettingsModel {
    #[serde(rename = "warnMs")]
    pub warn_ms: u32,
    #[serde(rename = "badMs")]
    pub bad_ms: u32,
    #[serde(
        rename = "mcpWritePasswordSalt",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mcp_write_password_salt: Option<String>,
    #[serde(
        rename = "mcpWritePasswordHash",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mcp_write_password_hash: Option<String>,
}

impl Default for UiSettingsModel {
    fn default() -> Self {
        Self {
            warn_ms: DEFAULT_WARN_MS,
            bad_ms: DEFAULT_BAD_MS,
            mcp_write_password_salt: None,
            mcp_write_password_hash: None,
        }
    }
}

impl UiSettingsModel {
    pub fn sanitized(mut self) -> Self {
        if self.warn_ms > self.bad_ms {
            std::mem::swap(&mut self.warn_ms, &mut self.bad_ms);
        }
        self
    }

    pub fn has_mcp_write_password(&self) -> bool {
        self.mcp_write_password_hash.is_some() && self.mcp_write_password_salt.is_some()
    }

    /// `Some(non-empty)` → generates fresh salt + hash; `Some("")` or
    /// `None` → clears both fields. Trims surrounding whitespace.
    pub fn set_mcp_write_password(&mut self, value: Option<&str>) {
        match value.map(|v| v.trim()) {
            Some(plain) if !plain.is_empty() => {
                let salt = generate_salt_hex();
                let hash = hash_password(&salt, plain);
                self.mcp_write_password_salt = Some(salt);
                self.mcp_write_password_hash = Some(hash);
            }
            _ => {
                self.mcp_write_password_salt = None;
                self.mcp_write_password_hash = None;
            }
        }
    }

    /// Constant-time check against the stored salt+hash. Returns false
    /// if no password is configured.
    pub fn verify_mcp_write_password(&self, password: &str) -> bool {
        let (Some(salt), Some(hash)) = (
            self.mcp_write_password_salt.as_deref(),
            self.mcp_write_password_hash.as_deref(),
        ) else {
            return false;
        };

        let candidate = hash_password(salt, password);
        candidate.as_bytes().ct_eq(hash.as_bytes()).into()
    }
}

fn generate_salt_hex() -> String {
    // uuid v4 uses a cryptographically secure RNG (`getrandom`).
    // 16 bytes of entropy is plenty for a per-password salt.
    let bytes = uuid::Uuid::new_v4().into_bytes();
    let mut out = String::with_capacity(32);
    for b in bytes.iter() {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

fn hash_password(salt_hex: &str, password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt_hex.as_bytes());
    hasher.update(password.as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for b in digest.iter() {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

/// Wire shape returned by GET `/api/Settings`. Hides salt/hash —
/// callers only learn whether a password is set. `Deserialize` is
/// derived only to satisfy `RawDataTyped`'s bound; the actual POST
/// body is deserialized into `UiSettingsPatchBody`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, MyHttpObjectStructure)]
pub struct SettingsPublicModel {
    #[serde(rename = "warnMs")]
    pub warn_ms: u32,
    #[serde(rename = "badMs")]
    pub bad_ms: u32,
    #[serde(rename = "mcpWritePasswordSet")]
    pub mcp_write_password_set: bool,
}

impl From<&UiSettingsModel> for SettingsPublicModel {
    fn from(m: &UiSettingsModel) -> Self {
        Self {
            warn_ms: m.warn_ms,
            bad_ms: m.bad_ms,
            mcp_write_password_set: m.has_mcp_write_password(),
        }
    }
}

/// Partial-update payload for POST `/api/Settings`. Fields omitted
/// from the body are left as-is on the server.
#[derive(Serialize, Deserialize, Debug, Default, Clone, MyHttpObjectStructure)]
pub struct SettingsPatchBody {
    #[serde(rename = "warnMs", default, skip_serializing_if = "Option::is_none")]
    pub warn_ms: Option<u32>,
    #[serde(rename = "badMs", default, skip_serializing_if = "Option::is_none")]
    pub bad_ms: Option<u32>,
}

#[derive(MyHttpInput)]
pub struct SettingsUpdateInput {
    #[http_body_raw(
        description = "Settings JSON. All fields optional: warnMs, badMs. Omitted fields are left unchanged."
    )]
    pub body: RawDataTyped<SettingsPatchBody>,
}

/// Body for POST `/api/Settings/McpWritePassword`. Empty string in
/// `password` clears the configured value.
#[derive(Serialize, Deserialize, Debug, Default, Clone, MyHttpObjectStructure)]
pub struct McpWritePasswordBody {
    #[serde(rename = "password")]
    pub password: String,
}

#[derive(MyHttpInput)]
pub struct McpWritePasswordInput {
    #[http_body_raw(
        description = "JSON body { password }. Pass an empty string to clear the password."
    )]
    pub body: RawDataTyped<McpWritePasswordBody>,
}
