use crate::error::Result;
use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolCategory {
    Runtime,
    AiTool,
}

#[derive(Debug, Clone, Copy)]
pub struct ToolSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub category: ToolCategory,
    pub cmd: &'static str,
    pub args: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToolStatus {
    pub id: String,
    pub display_name: String,
    pub category: ToolCategory,
    pub installed: bool,
    pub version: Option<String>,
}

pub fn tool_catalog() -> &'static [ToolSpec] {
    &[
        ToolSpec {
            id: "node",
            display_name: "Node.js",
            category: ToolCategory::Runtime,
            cmd: "node",
            args: &["--version"],
        },
        ToolSpec {
            id: "bun",
            display_name: "Bun",
            category: ToolCategory::Runtime,
            cmd: "bun",
            args: &["--version"],
        },
        ToolSpec {
            id: "python",
            display_name: "Python",
            category: ToolCategory::Runtime,
            cmd: "python",
            args: &["--version"],
        },
        ToolSpec {
            id: "git",
            display_name: "Git",
            category: ToolCategory::Runtime,
            cmd: "git",
            args: &["--version"],
        },
        ToolSpec {
            id: "github-cli",
            display_name: "GitHub CLI",
            category: ToolCategory::Runtime,
            cmd: "gh",
            args: &["--version"],
        },
        ToolSpec {
            id: "claude-code",
            display_name: "Claude Code",
            category: ToolCategory::AiTool,
            cmd: "claude",
            args: &["--version"],
        },
        ToolSpec {
            id: "opencode",
            display_name: "OpenCode",
            category: ToolCategory::AiTool,
            cmd: "opencode",
            args: &["--version"],
        },
        ToolSpec {
            id: "codex-cli",
            display_name: "Codex CLI",
            category: ToolCategory::AiTool,
            cmd: "codex",
            args: &["--version"],
        },
        ToolSpec {
            id: "cursor",
            display_name: "Cursor",
            category: ToolCategory::AiTool,
            cmd: "cursor",
            args: &["--version"],
        },
        ToolSpec {
            id: "pnpm",
            display_name: "pnpm",
            category: ToolCategory::Runtime,
            cmd: "pnpm",
            args: &["--version"],
        },
        ToolSpec {
            id: "uv",
            display_name: "uv",
            category: ToolCategory::Runtime,
            cmd: "uv",
            args: &["--version"],
        },
    ]
}

/// Return whether a command can be found on PATH.
pub fn is_installed(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

/// Run `cmd args`, read stdout, and parse a SemVer-ish version.
pub fn detect_version(cmd: &str, args: &[String]) -> Result<Option<String>> {
    if !is_installed(cmd) {
        return Ok(None);
    }
    let out = Command::new(cmd).args(args).output()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    Ok(extract_version(&stdout).or_else(|| extract_version(&stderr)))
}

/// Convenience detector for Slice 0 Node status.
pub fn detect_node() -> Result<Option<String>> {
    detect_version("node", &["--version".into()])
}

pub fn detect_environment() -> Result<Vec<ToolStatus>> {
    detect_tools_with(tool_catalog(), detect_version_from_spec)
}

pub fn detect_tools_with<F>(specs: &[ToolSpec], mut probe: F) -> Result<Vec<ToolStatus>>
where
    F: FnMut(&str, &[String]) -> Result<Option<String>>,
{
    let mut statuses = Vec::with_capacity(specs.len());
    for spec in specs {
        let args: Vec<String> = spec.args.iter().map(|arg| (*arg).to_string()).collect();
        let version = probe(spec.cmd, &args)?;
        statuses.push(ToolStatus {
            id: spec.id.to_string(),
            display_name: spec.display_name.to_string(),
            category: spec.category,
            installed: version.is_some(),
            version,
        });
    }
    Ok(statuses)
}

fn detect_version_from_spec(cmd: &str, args: &[String]) -> Result<Option<String>> {
    detect_version(cmd, args)
}

fn extract_version(text: &str) -> Option<String> {
    let re = regex::Regex::new(r"\d+\.\d+\.\d+").expect("valid version regex");
    re.find(text).map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_node_when_present() {
        if let Ok(Some(v)) = detect_version("node", &["--version".into()]) {
            assert!(v.starts_with(|c: char| c.is_ascii_digit()));
        }
    }

    #[test]
    fn extracts_semver_from_prefixed_output() {
        assert_eq!(extract_version("git version 2.45.1\n").as_deref(), Some("2.45.1"));
    }

    #[test]
    fn tool_catalog_covers_slice_1_command_line_targets() {
        let ids: Vec<_> = tool_catalog().iter().map(|tool| tool.id).collect();
        for expected in [
            "node",
            "bun",
            "python",
            "git",
            "github-cli",
            "claude-code",
            "opencode",
            "codex-cli",
            "cursor",
            "pnpm",
            "uv",
        ] {
            assert!(ids.contains(&expected), "missing {expected} in tool catalog");
        }
    }

    #[test]
    fn detect_tools_marks_missing_targets() {
        let specs = [ToolSpec {
            id: "node",
            display_name: "Node.js",
            category: ToolCategory::Runtime,
            cmd: "node",
            args: &["--version"],
        }];

        let statuses = detect_tools_with(&specs, |_cmd, _args| Ok(None)).unwrap();

        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].id, "node");
        assert!(!statuses[0].installed);
        assert_eq!(statuses[0].version, None);
    }

    #[test]
    fn detect_tools_returns_detected_versions() {
        let specs = [ToolSpec {
            id: "git",
            display_name: "Git",
            category: ToolCategory::Runtime,
            cmd: "git",
            args: &["--version"],
        }];

        let statuses = detect_tools_with(&specs, |_cmd, _args| Ok(Some("2.45.1".into()))).unwrap();

        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].id, "git");
        assert!(statuses[0].installed);
        assert_eq!(statuses[0].version.as_deref(), Some("2.45.1"));
    }
}
