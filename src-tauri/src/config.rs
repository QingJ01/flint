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

/// Append `dir` to the user's persistent PATH (Windows registry HKCU\Environment\Path).
/// Returns `true` if the PATH was modified, `false` if `dir` was already present.
/// No-op / returns error on non-Windows for now.
pub fn add_to_user_path(dir: &Path) -> Result<bool> {
    #[cfg(windows)]
    {
        let dir_str = dir.to_string_lossy().to_string();
        let current = read_user_path()?;
        if let Some(new_path) = append_path_dir(&current, &dir_str) {
            write_user_path(&new_path)?;
            return Ok(true);
        }
        Ok(false)
    }
    #[cfg(not(windows))]
    {
        let _ = dir;
        Err(FlintError::Other(
            "add_to_user_path is only implemented on Windows in this slice".into(),
        ))
    }
}

/// Pure helper: given the current PATH string and a dir to add, return
/// `Some(new_path)` if `dir` was appended, or `None` if it was already present
/// (case-insensitive on Windows, which is what the PATH comparison is in practice).
/// Empty current PATH => just `dir`.
pub fn append_path_dir(current: &str, dir: &str) -> Option<String> {
    if current.split(';').any(|seg| seg.eq_ignore_ascii_case(dir)) {
        return None;
    }
    if current.is_empty() {
        Some(dir.to_string())
    } else {
        Some(format!("{current};{dir}"))
    }
}

#[cfg(windows)]
fn read_user_path() -> Result<String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env: RegKey = hkcu
        .open_subkey("Environment")
        .map_err(|e| FlintError::Other(format!("open HKCU\\Environment: {e}")))?;
    let path: String = env
        .get_value("Path")
        .map_err(|e| FlintError::Other(format!("read Path: {e}")))?;
    Ok(path)
}

#[cfg(windows)]
fn write_user_path(new_path: &str) -> Result<()> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", winreg::enums::KEY_SET_VALUE)
        .map_err(|e| FlintError::Other(format!("open HKCU\\Environment for write: {e}")))?;
    env.set_value("Path", &new_path)
        .map_err(|e| FlintError::Other(format!("write Path: {e}")))?;
    Ok(())
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

    #[test]
    fn append_path_dir_adds_when_absent() {
        let result = append_path_dir("C:\\Windows;C:\\Program Files", "C:\\Python313");
        assert_eq!(result.as_deref(), Some("C:\\Windows;C:\\Program Files;C:\\Python313"));
    }

    #[test]
    fn append_path_dir_noop_when_present() {
        let result = append_path_dir("C:\\Windows;C:\\Python313", "C:\\Python313");
        assert_eq!(result, None);
    }

    #[test]
    fn append_path_dir_case_insensitive_match() {
        let result = append_path_dir("C:\\Windows;c:\\python313", "C:\\Python313");
        assert_eq!(result, None);
    }

    #[test]
    fn append_path_dir_handles_empty_current() {
        let result = append_path_dir("", "C:\\Python313");
        assert_eq!(result.as_deref(), Some("C:\\Python313"));
    }
}
