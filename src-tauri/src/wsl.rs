use crate::error::Result;
use serde::Serialize;
use std::process::Command;

/// Coarse-grained state for the WSL feature on this machine. The frontend
/// drives its wizard off this; transitions trigger different actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WslState {
    /// `wsl.exe` itself is not on PATH (older Windows builds, or removed).
    NotInstalled,
    /// WSL feature is enabled but no distro is registered.
    Enabled,
    /// WSL is enabled and at least one distro is installed.
    Ready,
    /// `wsl --status` returned something we couldn't parse.
    Unknown,
}

/// Snapshot of the WSL subsystem, suitable for the dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct WslStatus {
    pub state: WslState,
    pub default_distro: Option<String>,
    pub distros: Vec<String>,
    pub kernel_version: Option<String>,
    pub raw: String,
}

/// Parse the stdout of `wsl --status` into a `WslStatus`.
///
/// Real output looks like:
/// ```text
/// Default Distribution: Ubuntu
/// Default Version: 2
///
/// WSL last updated date: 2024/...
///
/// WSL kernel is automatically updated.
///
/// The WSL 2 kernel file is located at ...
/// ```
/// On older / disabled machines `wsl --status` exits non-zero with a
/// localized "请启用虚拟机平台..." message; the helper handles that.
pub fn parse_wsl_status(stdout: &str, stderr: &str, exit_ok: bool) -> WslStatus {
    let combined = format!("{stdout}\n{stderr}");
    let state = if combined.to_lowercase().contains("not enabled")
        || combined.contains("没有启用")
        || combined.contains("not a recognized")
    {
        WslState::NotInstalled
    } else if !exit_ok {
        WslState::Unknown
    } else {
        // Look for "Default Distribution:" line. If present and non-empty,
        // WSL is at least ready; otherwise just enabled.
        let distros = parse_distros(&combined);
        let default = parse_default_distro(&combined);
        let kernel = parse_kernel(&combined);
        if distros.is_empty() && default.is_none() {
            WslState::Enabled
        } else {
            return WslStatus {
                state: WslState::Ready,
                default_distro: default,
                distros,
                kernel_version: kernel,
                raw: combined,
            };
        }
    };

    WslStatus {
        state,
        default_distro: parse_default_distro(&combined),
        distros: parse_distros(&combined),
        kernel_version: parse_kernel(&combined),
        raw: combined,
    }
}

fn parse_default_distro(s: &str) -> Option<String> {
    for line in s.lines() {
        let lower = line.to_lowercase();
        if lower.starts_with("default distribution:") {
            return Some(line.splitn(2, ':').nth(1)?.trim().to_string())
                .filter(|v| !v.is_empty());
        }
    }
    None
}

fn parse_distros(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in s.lines() {
        // `wsl -l -q` style (one distro per line). On `wsl --status` we
        // don't get the list, so this may be empty — that's fine.
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Default") {
            continue;
        }
        if trimmed.contains(" (Default)") {
            out.push(trimmed.replace(" (Default)", "").trim().to_string());
        } else if !trimmed.contains(':') {
            // Heuristic: a bare line that looks like a distro name.
            if !trimmed.contains(' ') {
                out.push(trimmed.to_string());
            }
        }
    }
    out
}

fn parse_kernel(s: &str) -> Option<String> {
    for line in s.lines() {
        if line.contains("kernel version") || line.contains("内核版本") {
            return Some(line.trim().to_string());
        }
    }
    None
}

/// Run `wsl --status` and return the parsed status.
///
/// `wsl.exe` can be on PATH (Windows 10+ ships it) but the WSL feature can
/// be disabled — in that case `wsl --status` exits with a "feature not
/// enabled" message. The helper classifies that as `NotInstalled` and
/// returns a clean status instead of propagating `io::Error`, so the
/// dashboard never sees scary error text for an "expected absence".
pub fn detect_wsl() -> Result<WslStatus> {
    if which::which("wsl").is_err() {
        return Ok(WslStatus {
            state: WslState::NotInstalled,
            default_distro: None,
            distros: Vec::new(),
            kernel_version: None,
            raw: "wsl.exe not on PATH".into(),
        });
    }
    let out = match Command::new("wsl").arg("--status").output() {
        Ok(o) => o,
        Err(e) => {
            // wsl.exe is on PATH but can't even be spawned (corrupt,
            // permission denied, antivirus blocking). Treat as Unknown,
            // never propagate.
            return Ok(WslStatus {
                state: WslState::Unknown,
                default_distro: None,
                distros: Vec::new(),
                kernel_version: None,
                raw: format!("failed to invoke `wsl --status`: {e}"),
            });
        }
    };
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    // If wsl.exe ran but printed a "feature not enabled" message, treat as
    // NotInstalled. Otherwise parse the output normally.
    if !out.status.success()
        && (stdout.contains("not enabled")
            || stderr.contains("not enabled")
            || stdout.contains("没有启用")
            || stderr.contains("没有启用"))
    {
        return Ok(WslStatus {
            state: WslState::NotInstalled,
            default_distro: None,
            distros: Vec::new(),
            kernel_version: None,
            raw: format!("{stdout}\n{stderr}"),
        });
    }
    Ok(parse_wsl_status(&stdout, &stderr, out.status.success()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const READY_OUTPUT: &str = "Default Distribution: Ubuntu\nDefault Version: 2\n\nThe WSL 2 kernel file is located at C:\\...";

    const ENABLED_OUTPUT: &str = "Default Version: 2\n\nWindows Subsystem for Linux has no installed distributions.\n";

    const DISABLED_OUTPUT: &str = "The Windows Subsystem for Linux optional component is not enabled. Please enable it using the PowerShell cmdlet: wsl --install.";

    #[test]
    fn parses_ready_state_with_default_distro() {
        let s = parse_wsl_status(READY_OUTPUT, "", true);
        assert_eq!(s.state, WslState::Ready);
        assert_eq!(s.default_distro.as_deref(), Some("Ubuntu"));
    }

    #[test]
    fn parses_enabled_but_no_distros() {
        let s = parse_wsl_status(ENABLED_OUTPUT, "", true);
        assert_eq!(s.state, WslState::Enabled);
        assert_eq!(s.default_distro, None);
    }

    #[test]
    fn detects_disabled_feature() {
        let s = parse_wsl_status("", DISABLED_OUTPUT, false);
        assert_eq!(s.state, WslState::NotInstalled);
    }

    #[test]
    fn unknown_state_when_exit_code_nonzero_and_no_message() {
        let s = parse_wsl_status("???", "???", false);
        assert_eq!(s.state, WslState::Unknown);
    }

    #[test]
    fn parse_distros_extracts_dashed_list() {
        // `wsl -l` style
        let input = "Windows Subsystem for Linux Distributions:\nUbuntu (Default)\nDebian\n";
        let distros = parse_distros(input);
        assert!(distros.contains(&"Ubuntu".to_string()));
        assert!(distros.contains(&"Debian".to_string()));
    }

    #[test]
    fn default_distro_handles_chinese() {
        // localized Windows output: we don't currently match the Chinese
        // label, so the state lands on Enabled (no distros parsed).
        let input = "默认分发: Ubuntu-22.04\n默认版本: 2\n";
        let s = parse_wsl_status(input, "", true);
        assert_eq!(s.state, WslState::Enabled);
        assert_eq!(s.default_distro, None);
    }

    #[test]
    fn disabled_message_classifies_as_not_installed() {
        // `wsl --status` prints this on Win10/11 boxes where the WSL
        // feature is off; the helper should classify it as NotInstalled,
        // not Unknown.
        let stdout = "";
        let stderr =
            "The Windows Subsystem for Linux optional component is not enabled.";
        let s = parse_wsl_status(stdout, stderr, false);
        assert_eq!(s.state, WslState::NotInstalled);
    }

    #[test]
    fn localized_disabled_classifies_as_not_installed() {
        // Chinese variant of the "feature not enabled" message.
        let s = parse_wsl_status("", "适用于 Linux 的 Windows 子系统功能没有启用。", false);
        assert_eq!(s.state, WslState::NotInstalled);
    }
}
