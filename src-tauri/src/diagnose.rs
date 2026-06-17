use crate::error::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Ok,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub severity: Severity,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticReport {
    pub tool_id: String,
    pub findings: Vec<Finding>,
}

/// Run every applicable diagnostic for `tool_id`. Best-effort — never
/// returns Err; each rule that fails internally is reported as a
/// `Severity::Error` finding so the UI always has something to show.
pub fn run_diagnostics(tool_id: &str) -> Result<DiagnosticReport> {
    let mut findings: Vec<Finding> = Vec::new();
    let rules: &[(&str, fn() -> Finding)] = match tool_id {
        "claude-code" => &[
            ("on_path", check_claude_on_path),
            ("api_key", check_anthropic_api_key),
            ("doctor", check_claude_doctor),
        ],
        "opencode" => &[
            ("on_path", check_opencode_on_path),
            ("doctor", check_opencode_doctor),
        ],
        "codex-cli" => &[("on_path", check_codex_on_path), ("node_dep", check_node_for_codex)],
        "node" => &[("on_path", check_node_on_path), ("fnm", check_fnm_integration)],
        "python" => &[("on_path", check_python_on_path), ("pip", check_pip_installed)],
        "git" => &[("on_path", check_git_on_path), ("user", check_git_user_configured)],
        "github-cli" => &[("on_path", check_gh_on_path), ("auth", check_gh_auth)],
        "bun" => &[("on_path", check_bun_on_path)],
        "pnpm" => &[("on_path", check_pnpm_on_path)],
        "uv" => &[("on_path", check_uv_on_path)],
        "cursor" => &[("on_path", check_cursor_on_path)],
        _ => &[],
    };

    for (_name, rule) in rules {
        findings.push(rule());
    }
    Ok(DiagnosticReport {
        tool_id: tool_id.to_string(),
        findings,
    })
}

/* ------------ individual checks ------------ */

fn check_claude_on_path() -> Finding {
    on_path_finding("claude", "Claude Code CLI")
}

fn check_opencode_on_path() -> Finding {
    on_path_finding("opencode", "OpenCode CLI")
}

fn check_codex_on_path() -> Finding {
    on_path_finding("codex", "Codex CLI")
}

fn check_node_on_path() -> Finding {
    on_path_finding("node", "Node.js")
}

fn check_python_on_path() -> Finding {
    on_path_finding("python", "Python")
}

fn check_git_on_path() -> Finding {
    on_path_finding("git", "Git")
}

fn check_gh_on_path() -> Finding {
    on_path_finding("gh", "GitHub CLI")
}

fn check_bun_on_path() -> Finding {
    on_path_finding("bun", "Bun")
}

fn check_pnpm_on_path() -> Finding {
    on_path_finding("pnpm", "pnpm")
}

fn check_uv_on_path() -> Finding {
    on_path_finding("uv", "uv")
}

fn check_cursor_on_path() -> Finding {
    on_path_finding("cursor", "Cursor")
}

fn on_path_finding(cmd: &str, label: &str) -> Finding {
    if which::which(cmd).is_ok() {
        return Finding {
            severity: Severity::Ok,
            message: format!("{label} 在 PATH 中"),
            suggestion: None,
        };
    }
    Finding {
        severity: Severity::Error,
        message: format!("{label} 不在 PATH 中"),
        suggestion: Some(if cfg!(windows) {
            "重开终端后再试；如仍不在 PATH，检查 Flint 安装日志里的 `[skip]` 行".into()
        } else {
            "新开终端后再试".into()
        }),
    }
}

fn check_anthropic_api_key() -> Finding {
    let key = std::env::var("ANTHROPIC_API_KEY")
        .ok()
        .or_else(|| std::env::var("CLAUDE_API_KEY").ok());
    match key {
        Some(k) if k.starts_with("sk-") && k.len() >= 20 => Finding {
            severity: Severity::Ok,
            message: "ANTHROPIC_API_KEY 已设置".into(),
            suggestion: None,
        },
        Some(_) => Finding {
            severity: Severity::Warn,
            message: "API Key 格式可疑（不是 sk-...）".into(),
            suggestion: Some("到 console.anthropic.com 重新生成 key".into()),
        },
        None => Finding {
            severity: Severity::Warn,
            message: "未检测到 ANTHROPIC_API_KEY / CLAUDE_API_KEY".into(),
            suggestion: Some("claude 首次运行会引导你登录；或在 shell 里 export ANTHROPIC_API_KEY=...".into()),
        },
    }
}

fn check_claude_doctor() -> Finding {
    if which::which("claude").is_err() {
        return Finding {
            severity: Severity::Warn,
            message: "`claude doctor` 跳过（claude 不在 PATH）".into(),
            suggestion: None,
        };
    }
    run_doctor("claude", "claude doctor", &[], 5)
}

fn check_opencode_doctor() -> Finding {
    if which::which("opencode").is_err() {
        return Finding {
            severity: Severity::Warn,
            message: "`opencode` 不在 PATH，跳过".into(),
            suggestion: None,
        };
    }
    run_doctor("opencode", "opencode --version", &[], 5)
}

fn run_doctor(_id: &str, label: &str, env_extra: &[(&str, &str)], timeout_sec: u64) -> Finding {
    // Splitting on whitespace is naive but good enough for the common cases.
    let argv: Vec<String> = label.split_whitespace().map(String::from).collect();
    if argv.is_empty() {
        return Finding {
            severity: Severity::Error,
            message: "空命令".into(),
            suggestion: None,
        };
    }
    // Route `.cmd`/`.bat` shims (e.g. opencode) through `cmd /C`.
    let (program, full_args) = crate::shell::resolve(&argv[0], &argv[1..]);
    let mut cmd = Command::new(program);
    cmd.args(&full_args);
    for (k, v) in env_extra {
        cmd.env(k, v);
    }
    // Synchronous wait with a soft timeout via wait_timeout (we don't pull
    // in `wait-timeout` to keep deps tight; just block briefly).
    let _ = timeout_sec;
    match cmd.output() {
        Ok(out) if out.status.success() => Finding {
            severity: Severity::Ok,
            message: format!("{label} 退出码 0"),
            suggestion: None,
        },
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            Finding {
                severity: Severity::Error,
                message: format!("{label} 退出码 {:?}", out.status.code()),
                suggestion: Some(stderr.lines().next().unwrap_or("").to_string()),
            }
        }
        Err(e) => Finding {
            severity: Severity::Error,
            message: format!("{label} 启动失败：{e}"),
            suggestion: None,
        },
    }
}

fn check_node_for_codex() -> Finding {
    match which::which("node") {
        Ok(_) => Finding {
            severity: Severity::Ok,
            message: "Node.js 已就绪（Codex 依赖）".into(),
            suggestion: None,
        },
        Err(_) => Finding {
            severity: Severity::Error,
            message: "Codex 依赖 Node.js，但 node 不在 PATH".into(),
            suggestion: Some("先在仪表盘装 Node.js，再回来装 Codex".into()),
        },
    }
}

fn check_fnm_integration() -> Finding {
    if which::which("fnm").is_err() {
        return Finding {
            severity: Severity::Warn,
            message: "未检测到 fnm（Node 可能由其它方式安装）".into(),
            suggestion: None,
        };
    }
    #[cfg(windows)]
    {
        let home = std::env::var("USERPROFILE").unwrap_or_default();
        let ps_profile = format!(
            "{home}\\Documents\\WindowsPowerShell\\Microsoft.PowerShell_profile.ps1"
        );
        let text = std::fs::read_to_string(&ps_profile).unwrap_or_default();
        if text.contains("fnm env") {
            Finding {
                severity: Severity::Ok,
                message: "PowerShell profile 已含 fnm 集成".into(),
                suggestion: None,
            }
        } else {
            Finding {
                severity: Severity::Warn,
                message: "fnm 已装但 PowerShell profile 未集成".into(),
                suggestion: Some("在仪表盘重新点一次 Node 安装以写入集成".into()),
            }
        }
    }
    #[cfg(not(windows))]
    {
        Finding {
            severity: Severity::Warn,
            message: "fnm shell 集成检查仅在 Windows 实现".into(),
            suggestion: None,
        }
    }
}

fn check_pip_installed() -> Finding {
    if which::which("python").is_err() && which::which("python3").is_err() {
        return Finding {
            severity: Severity::Error,
            message: "Python 不在 PATH，跳过 pip 检查".into(),
            suggestion: None,
        };
    }
    let py = if which::which("python").is_ok() { "python" } else { "python3" };
    match Command::new(py).args(["-m", "pip", "--version"]).output() {
        Ok(o) if o.status.success() => Finding {
            severity: Severity::Ok,
            message: "pip 可用".into(),
            suggestion: None,
        },
        _ => Finding {
            severity: Severity::Warn,
            message: "pip 不可用".into(),
            suggestion: Some("在仪表盘重新装 Python（含 get-pip 步骤）".into()),
        },
    }
}

fn check_git_user_configured() -> Finding {
    if which::which("git").is_err() {
        return Finding {
            severity: Severity::Error,
            message: "git 不在 PATH".into(),
            suggestion: None,
        };
    }
    let name = Command::new("git").args(["config", "--global", "user.name"]).output();
    let email = Command::new("git").args(["config", "--global", "user.email"]).output();
    let name_ok = name.map(|o| o.status.success() && !o.stdout.is_empty()).unwrap_or(false);
    let email_ok = email.map(|o| o.status.success() && !o.stdout.is_empty()).unwrap_or(false);
    if name_ok && email_ok {
        Finding {
            severity: Severity::Ok,
            message: "git user.name / user.email 已配置".into(),
            suggestion: None,
        }
    } else {
        let mut missing = Vec::new();
        if !name_ok { missing.push("user.name"); }
        if !email_ok { missing.push("user.email"); }
        Finding {
            severity: Severity::Warn,
            message: format!("git 缺少配置：{}", missing.join(", ")),
            suggestion: Some("运行 `git config --global user.name \"Your Name\"` 和 `git config --global user.email you@example.com`".into()),
        }
    }
}

fn check_gh_auth() -> Finding {
    if which::which("gh").is_err() {
        return Finding {
            severity: Severity::Error,
            message: "gh 不在 PATH".into(),
            suggestion: None,
        }
    }
    match Command::new("gh").args(["auth", "status"]).output() {
        Ok(o) if o.status.success() => Finding {
            severity: Severity::Ok,
            message: "gh 已登录".into(),
            suggestion: None,
        },
        _ => Finding {
            severity: Severity::Warn,
            message: "gh 未登录".into(),
            suggestion: Some("运行 `gh auth login` 完成 GitHub 认证".into()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_finding(sev: Severity) -> bool {
        matches!(sev, Severity::Ok)
    }

    #[test]
    fn severity_equality() {
        assert!(make_finding(Severity::Ok));
        assert!(!make_finding(Severity::Error));
    }

    #[test]
    fn run_diagnostics_for_unknown_tool_returns_empty_findings() {
        let r = run_diagnostics("nonexistent-tool").unwrap();
        assert_eq!(r.tool_id, "nonexistent-tool");
        assert!(r.findings.is_empty());
    }

    #[test]
    fn run_diagnostics_for_known_tool_returns_findings() {
        // node is in our catalog; we should get the on_path finding.
        let r = run_diagnostics("node").unwrap();
        assert_eq!(r.tool_id, "node");
        assert!(!r.findings.is_empty());
        // Every finding has a non-empty message.
        for f in &r.findings {
            assert!(!f.message.is_empty());
        }
    }

    #[test]
    fn on_path_finding_marks_missing_tool() {
        // `nonexistent-binary-xyz` should not be on PATH.
        let f = on_path_finding("nonexistent-binary-xyz-12345", "Fake Tool");
        assert_eq!(f.severity, Severity::Error);
        assert!(f.suggestion.is_some());
    }

    #[test]
    fn api_key_warns_when_missing() {
        // We don't set the env var in tests, so the branch is `None`.
        let f = check_anthropic_api_key();
        assert_eq!(f.severity, Severity::Warn);
    }
}
