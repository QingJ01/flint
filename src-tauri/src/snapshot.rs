//! Environment snapshot: capture the current dev environment as a portable
//! JSON file, and restore it on another machine (install missing tools +
//! apply mirrors). Export is exhaustive; restore is "smart" — it only fills
//! gaps (skips already-installed tools) and never uninstalls or rewrites
//! PATH. See `docs/plans` for the design rationale.

use crate::config;
use crate::detector::{self, ToolStatus};
use crate::error::{FlintError, Result};
use crate::wsl::{self, WslStatus};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Bumped if the on-disk shape changes incompatibly. `load` accepts any
/// `schema` for now (the struct is additive); a future breaking change
/// would gate on this.
pub const SCHEMA_VERSION: u32 = 1;

/// The portable description of a dev environment. Every field reuses an
/// existing serializable backend type so the snapshot stays in lockstep with
/// what the dashboard already shows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snapshot {
    pub schema: u32,
    pub tools: Vec<ToolStatus>,
    pub npm_registry: Option<String>,
    pub pip_registry: Option<String>,
    pub wsl: Option<WslStatus>,
}

/// Capture the live environment. Best-effort per field: a failing probe
/// (e.g. no `~/.npmrc`) contributes `None`/empty rather than aborting.
pub fn build() -> Result<Snapshot> {
    let tools = detector::detect_environment()?;
    let npm_registry = config::current_npm_registry().ok().flatten();
    let pip_registry = config::current_pip_registry().ok().flatten();
    let wsl = wsl::detect_wsl().ok();
    Ok(Snapshot {
        schema: SCHEMA_VERSION,
        tools,
        npm_registry,
        pip_registry,
        wsl,
    })
}

/// Serialize a snapshot to pretty JSON and write it to `path`.
pub fn export_to(path: &Path, snap: &Snapshot) -> Result<()> {
    let json = serde_json::to_string_pretty(snap)
        .map_err(|e| FlintError::Other(format!("serialize snapshot: {e}")))?;
    std::fs::write(path, json)?; // io::Error auto-converts via FlintError::Io
    Ok(())
}

/// Read a snapshot file back into a `Snapshot`.
pub fn load(path: &Path) -> Result<Snapshot> {
    let text = std::fs::read_to_string(path)?;
    serde_json::from_str(&text)
        .map_err(|e| FlintError::Parse(format!("parse snapshot {}: {e}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detector::ToolCategory;
    use crate::wsl::WslState;

    fn sample() -> Snapshot {
        Snapshot {
            schema: SCHEMA_VERSION,
            tools: vec![
                ToolStatus {
                    id: "node".into(),
                    display_name: "Node.js".into(),
                    category: ToolCategory::Runtime,
                    installed: true,
                    version: Some("22.1.0".into()),
                },
                ToolStatus {
                    id: "uv".into(),
                    display_name: "uv".into(),
                    category: ToolCategory::Runtime,
                    installed: false,
                    version: None,
                },
            ],
            npm_registry: Some("https://registry.npmmirror.com/".into()),
            pip_registry: None,
            wsl: Some(WslStatus {
                state: WslState::Ready,
                default_distro: Some("Ubuntu".into()),
                distros: vec!["Ubuntu".into()],
                kernel_version: None,
                raw: "Default Distribution: Ubuntu".into(),
            }),
        }
    }

    #[test]
    fn snapshot_round_trips_through_json() {
        let snap = sample();
        let json = serde_json::to_string_pretty(&snap).unwrap();
        let back: Snapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snap, back, "snapshot must survive serialize → deserialize");
    }

    #[test]
    fn export_then_load_is_identity() {
        let snap = sample();
        let dir = std::env::temp_dir().join("flint_snapshot_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("flint-snapshot.json");
        export_to(&path, &snap).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(snap, loaded);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_rejects_malformed_json() {
        let dir = std::env::temp_dir().join("flint_snapshot_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("bad.json");
        std::fs::write(&path, "{ not valid json").unwrap();
        assert!(load(&path).is_err());
        let _ = std::fs::remove_file(&path);
    }
}
