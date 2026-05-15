pub const DEFAULT_WARN_MS: u32 = 3_000;
pub const DEFAULT_BAD_MS: u32 = 10_000;

/// Health thresholds for the reader status indicator.
/// `warn_ms` is the boundary between Green and Yellow; `bad_ms` between
/// Yellow and Red. Persisted on the server in `ui-settings.json`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct HealthThresholds {
    pub warn_ms: u32,
    pub bad_ms: u32,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            warn_ms: DEFAULT_WARN_MS,
            bad_ms: DEFAULT_BAD_MS,
        }
    }
}

/// Full snapshot of the server-side UI settings. The plaintext MCP
/// write password is never exposed by the server — the UI only
/// learns whether one is configured.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct UiServerSettings {
    pub thresholds: HealthThresholds,
    pub mcp_write_password_set: bool,
}
