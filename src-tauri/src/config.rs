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

/* ------------------------------------------------------------------ */
/* Mirror / proxy configuration (Slice 4)                              */
/* ------------------------------------------------------------------ */

/// Well-known registry URLs. `Custom` lets the user paste their own.
pub const NPM_REGISTRIES: &[(&str, &str)] = &[
    ("https://registry.npmjs.org/", "官方源 (npmjs.org)"),
    ("https://registry.npmmirror.com/", "淘宝镜像 (npmmirror.com)"),
    ("https://mirrors.huaweicloud.com/repository/npm/", "华为云镜像"),
    ("https://mirrors.cloud.tencent.com/npm/", "腾讯云镜像"),
];

pub const PIP_REGISTRIES: &[(&str, &str)] = &[
    ("https://pypi.org/simple", "官方源 (PyPI)"),
    ("https://pypi.tuna.tsinghua.edu.cn/simple", "清华源"),
    ("https://mirrors.aliyun.com/pypi/simple", "阿里源"),
    ("https://mirrors.huaweicloud.com/repository/pypi/simple", "华为云镜像"),
    ("https://mirrors.cloud.tencent.com/pypi/simple", "腾讯云镜像"),
];

pub const GITHUB_MIRRORS: &[(&str, &str)] = &[
    ("https://github.com", "官方 (github.com)"),
    ("https://gh-proxy.com", "gh-proxy.com (Web 代理)"),
    ("https://ghps.cc", "ghps.cc (克隆加速)"),
];

/// The home directory for the current user (USERPROFILE on Windows, HOME
/// elsewhere). Centralized so tests can stub it.
pub fn user_home() -> Result<std::path::PathBuf> {
    #[cfg(windows)]
    {
        let h = std::env::var("USERPROFILE")
            .map_err(|e| FlintError::Other(format!("USERPROFILE: {e}")))?;
        Ok(std::path::PathBuf::from(h))
    }
    #[cfg(not(windows))]
    {
        let h = std::env::var("HOME")
            .map_err(|e| FlintError::Other(format!("HOME: {e}")))?;
        Ok(std::path::PathBuf::from(h))
    }
}

/// Pure helper: build the contents of a `~/.npmrc` file that pins the
/// given registry. Existing unrelated keys (e.g. `save-exact`) are
/// preserved when `existing` is supplied.
pub fn build_npmrc(registry_url: &str, existing: &str) -> String {
    let mut out = String::new();
    let mut registry_seen = false;
    for line in existing.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("registry") && !trimmed.starts_with("//") {
            registry_seen = true;
            out.push_str(&format!("registry={registry_url}\n"));
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !registry_seen {
        out.push_str(&format!("registry={registry_url}\n"));
    }
    out
}

/// Pure helper: build the contents of a `pip.ini` (Windows) / `pip.conf`
/// (POSIX) pinning the given index URL.
pub fn build_pip_conf(index_url: &str, existing: &str) -> String {
    let mut out = String::new();
    let mut in_global = false;
    let mut seen_global = false;
    let mut seen_index = false;
    for line in existing.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') {
            in_global = trimmed.starts_with("[global]");
            if in_global {
                seen_global = true;
            }
            out.push_str(line);
            out.push('\n');
        } else if in_global && trimmed.starts_with("index-url") {
            seen_index = true;
            out.push_str(&format!("index-url = {index_url}\n"));
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !seen_global {
        out.push_str("[global]\n");
    }
    if !seen_index {
        out.push_str(&format!("index-url = {index_url}\n"));
    }
    out
}

/// Apply an npm registry by writing to `~/.npmrc`. Creates the file
/// (USERPROFILE dir) if absent. Returns `true` if the file was modified.
pub fn apply_npm_registry(registry_url: &str) -> Result<bool> {
    let home = user_home()?;
    let path = home.join(".npmrc");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let new_content = build_npmrc(registry_url, &existing);
    if new_content == format!("{existing}\n") || new_content == existing {
        return Ok(false);
    }
    std::fs::write(&path, new_content)?;
    Ok(true)
}

/// Apply a pip index URL by writing to `%APPDATA%\pip\pip.ini` on Windows
/// or `~/.config/pip/pip.conf` on POSIX.
pub fn apply_pip_registry(index_url: &str) -> Result<bool> {
    let path = pip_config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let new_content = build_pip_conf(index_url, &existing);
    if new_content == existing {
        return Ok(false);
    }
    std::fs::write(&path, new_content)?;
    Ok(true)
}

#[cfg(windows)]
fn pip_config_path() -> Result<std::path::PathBuf> {
    let appdata = std::env::var("APPDATA")
        .map_err(|e| FlintError::Other(format!("APPDATA: {e}")))?;
    Ok(std::path::PathBuf::from(appdata).join("pip").join("pip.ini"))
}

#[cfg(not(windows))]
fn pip_config_path() -> Result<std::path::PathBuf> {
    let home = user_home()?;
    Ok(home.join(".config").join("pip").join("pip.conf"))
}

/// Read the current npm registry from `~/.npmrc` (if set).
pub fn current_npm_registry() -> Result<Option<String>> {
    let home = user_home()?;
    let path = home.join(".npmrc");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    Ok(extract_equals_value(&text, "registry"))
}

/// Read the current pip index-url from the pip config file.
pub fn current_pip_registry() -> Result<Option<String>> {
    let path = pip_config_path()?;
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let mut in_global = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') {
            in_global = trimmed.starts_with("[global]");
            continue;
        }
        if in_global && trimmed.starts_with("index-url") {
            if let Some((_, v)) = trimmed.split_once('=') {
                return Ok(Some(v.trim().to_string()));
            }
        }
    }
    Ok(None)
}

fn extract_equals_value(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix(key) {
            let rest = rest.trim_start();
            if let Some(v) = rest.strip_prefix('=') {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod mirror_tests {
    use super::*;

    #[test]
    fn npmrc_replaces_existing_registry() {
        let existing = "save-exact=true\nregistry=https://old.example/\n";
        let out = build_npmrc("https://new.example/", existing);
        assert!(out.contains("save-exact=true"));
        assert!(out.contains("registry=https://new.example/"));
        assert!(!out.contains("old.example"));
    }

    #[test]
    fn npmrc_appends_when_missing() {
        let existing = "save-exact=true\n";
        let out = build_npmrc("https://new.example/", existing);
        assert!(out.contains("save-exact=true"));
        assert!(out.contains("registry=https://new.example/"));
    }

    #[test]
    fn npmrc_preserves_commented_registry() {
        // A line starting with `//` (e.g. "//registry=") is a comment;
        // it's preserved as-is, and the active registry is added fresh.
        let existing = "//registry=https://commented.example/\n";
        let out = build_npmrc("https://new.example/", existing);
        assert!(out.contains("//registry=https://commented.example/"));
        assert!(out.contains("registry=https://new.example/"));
    }

    #[test]
    fn pip_conf_creates_global_section_when_missing() {
        let out = build_pip_conf("https://new.example/simple", "");
        assert!(out.contains("[global]"));
        assert!(out.contains("index-url = https://new.example/simple"));
    }

    #[test]
    fn pip_conf_replaces_existing_index_in_global() {
        let existing = "[global]\nindex-url = https://old.example/simple\n";
        let out = build_pip_conf("https://new.example/simple", existing);
        assert!(out.contains("index-url = https://new.example/simple"));
        assert!(!out.contains("old.example"));
    }

    #[test]
    fn pip_conf_keeps_index_outside_global_section() {
        // If a [install] section has index-url, we leave it alone — we
        // only manage the [global] index-url.
        let existing = "[global]\n[install]\nindex-url = https://keep.example/simple\n";
        let out = build_pip_conf("https://new.example/simple", existing);
        assert!(out.contains("index-url = https://keep.example/simple"));
        // The global index-url should now be the new one, not duplicated.
        let global_count = out
            .lines()
            .filter(|l| l.trim_start().starts_with("index-url"))
            .count();
        assert_eq!(global_count, 2, "should keep [install] index-url + add [global] one");
    }

    #[test]
    fn extract_equals_value_finds_key() {
        let text = "foo=bar\nregistry=https://x.example/\n";
        assert_eq!(extract_equals_value(text, "registry").as_deref(), Some("https://x.example/"));
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
