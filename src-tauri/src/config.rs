use crate::error::{FlintError, Result};
use std::path::Path;

const FNM_PROFILE_LINE: &str =
    "fnm env --use-on-cd --shell powershell | Out-String | Invoke-Expression";

/// Ensure `line` is present in `path`. Creates the file (and parent dir) if absent.
/// Appends `line` if missing. Returns `true` if the file was modified.
pub fn ensure_line_in_file(path: &Path, line: &str) -> Result<bool> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    if existing.contains(line) {
        return Ok(false);
    }
    let mut content = existing;
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(line);
    content.push('\n');
    std::fs::write(path, content)?;
    Ok(true)
}

/// Write the fnm shell-integration snippet into the user's PowerShell profiles
/// (PS5.1 and PS7+) so a NEW terminal resolves `node`. User-scope, no admin.
/// Returns the list of profile paths that were created/modified.
pub fn ensure_fnm_in_powershell_profiles() -> Result<Vec<String>> {
    let home = std::env::var("USERPROFILE")
        .map_err(|e| FlintError::Other(format!("USERPROFILE not set: {e}")))?;
    let candidates = [
        format!("{home}\\Documents\\WindowsPowerShell\\Microsoft.PowerShell_profile.ps1"),
        format!("{home}\\Documents\\PowerShell\\Microsoft.PowerShell_profile.ps1"),
    ];
    let mut changed = Vec::new();
    for c in candidates {
        let path = std::path::PathBuf::from(&c);
        if ensure_line_in_file(&path, FNM_PROFILE_LINE)? {
            changed.push(c);
        }
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_line_when_missing() {
        let dir = std::env::temp_dir().join("flint_cfg_test_append");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("profile.ps1");
        std::fs::write(&f, "existing line\n").unwrap();
        let changed = ensure_line_in_file(&f, "FNM SNIPPET").unwrap();
        assert!(changed, "should have appended");
        let after = std::fs::read_to_string(&f).unwrap();
        assert!(after.contains("existing line"));
        assert!(after.contains("FNM SNIPPET"));
    }

    #[test]
    fn is_idempotent_when_line_present() {
        let dir = std::env::temp_dir().join("flint_cfg_test_idem");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("profile.ps1");
        std::fs::write(&f, "FNM SNIPPET\n").unwrap();
        let changed = ensure_line_in_file(&f, "FNM SNIPPET").unwrap();
        assert!(!changed, "should not re-append");
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "FNM SNIPPET\n");
    }

    #[test]
    fn creates_file_if_absent() {
        let dir = std::env::temp_dir().join("flint_cfg_test_create");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("profile.ps1");
        let changed = ensure_line_in_file(&f, "FNM SNIPPET").unwrap();
        assert!(changed);
        assert!(std::fs::read_to_string(&f).unwrap().contains("FNM SNIPPET"));
    }
}
