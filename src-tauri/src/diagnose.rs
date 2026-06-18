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
pub async fn run_diagnostics(tool_id: &str) -> Result<DiagnosticReport> {
    let mut findings: Vec<Finding> = Vec::new();
    match tool_id {
        "claude-code" => {
            findings.push(check_claude_on_path());
            findings.push(check_anthropic_api_key());
            findings.push(check_claude_doctor().await);
            findings.push(check_reachability("Anthropic API", "https://api.anthropic.com").await);
        }
        "opencode" => {
            findings.push(check_opencode_on_path());
            findings.push(check_opencode_doctor().await);
        }
        "codex-cli" => {
            findings.push(check_codex_on_path());
            findings.push(check_node_for_codex());
            findings.push(check_reachability("npm registry", "https://registry.npmjs.org").await);
        }
        "node" => {
            findings.push(check_node_on_path());
            findings.push(check_fnm_integration());
            findings.push(check_reachability("npm registry", "https://registry.npmjs.org").await);
        }
        "python" => {
            findings.push(check_python_on_path());
            findings.push(check_pip_installed());
            findings.push(check_reachability("PyPI", "https://pypi.org").await);
        }
        "git" => {
            findings.push(check_git_on_path());
            findings.push(check_git_user_configured());
            findings.push(check_reachability("GitHub", "https://github.com").await);
        }
        "github-cli" => {
            findings.push(check_gh_on_path());
            findings.push(check_gh_auth());
            findings.push(check_reachability("GitHub", "https://github.com").await);
        }
        "bun" => {
            findings.push(check_bun_on_path());
            findings.push(check_reachability("npm registry", "https://registry.npmjs.org").await);
        }
        "pnpm" => {
            findings.push(check_pnpm_on_path());
            findings.push(check_reachability("npm registry", "https://registry.npmjs.org").await);
        }
        "uv" => {
            findings.push(check_uv_on_path());
            findings.push(check_reachability("PyPI", "https://pypi.org").await);
        }
        "cursor" => {
            findings.push(check_cursor_on_path());
        }
        _ => {}
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
            suggestion: Some(
                "claude 首次运行会引导你登录；或在 shell 里 export ANTHROPIC_API_KEY=...".into(),
            ),
        },
    }
}

async fn check_claude_doctor() -> Finding {
    if which::which("claude").is_err() {
        return Finding {
            severity: Severity::Warn,
            message: "`claude doctor` 跳过（claude 不在 PATH）".into(),
            suggestion: None,
        };
    }
    run_doctor("claude", "claude doctor", &[], 15).await
}

async fn check_opencode_doctor() -> Finding {
    if which::which("opencode").is_err() {
        return Finding {
            severity: Severity::Warn,
            message: "`opencode` 不在 PATH，跳过".into(),
            suggestion: None,
        };
    }
    run_doctor("opencode", "opencode --version", &[], 5).await
}

async fn run_doctor(
    _id: &str,
    label: &str,
    env_extra: &[(&str, &str)],
    timeout_sec: u64,
) -> Finding {
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
    let mut cmd = tokio::process::Command::new(program);
    cmd.args(&full_args)
        // Stdio::null() 防止 claude doctor 这类命令卡在交互式提示上把整个
        // 诊断（进而 Tauri 异步执行器）挂住。
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    for (k, v) in env_extra {
        cmd.env(k, v);
    }
    let Ok(child) = cmd.spawn() else {
        return Finding {
            severity: Severity::Error,
            message: format!("{label} 启动失败"),
            suggestion: None,
        };
    };
    // 真超时：到点杀子进程（kill_on_drop），降级为 Warn 而非无限阻塞。
    match tokio::time::timeout(
        std::time::Duration::from_secs(timeout_sec),
        child.wait_with_output(),
    )
    .await
    {
        Ok(Ok(out)) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            Finding {
                severity: Severity::Ok,
                message: format!("{label} 退出码 0"),
                // 把首行输出（版本/健康摘要）当证据展示，便于排查。
                suggestion: stdout.lines().next().map(|s| s.to_string()),
            }
        }
        Ok(Ok(out)) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let stdout = String::from_utf8_lossy(&out.stdout);
            Finding {
                severity: Severity::Error,
                message: format!("{label} 退出码 {:?}", out.status.code()),
                suggestion: Some(
                    stderr
                        .lines()
                        .next()
                        .or_else(|| stdout.lines().next())
                        .unwrap_or("")
                        .to_string(),
                ),
            }
        }
        Ok(Err(_)) => Finding {
            severity: Severity::Error,
            message: format!("{label} 启动失败"),
            suggestion: None,
        },
        Err(_) => Finding {
            severity: Severity::Warn,
            message: format!("{label} 超时（{timeout_sec}s），已中止"),
            suggestion: None,
        },
    }
}

/// Best-effort 网络可达性探测。任何 HTTP 响应（含 4xx/5xx）都算「可达」
/// ——只有连接失败/超时才算「不可达」，且降级为 Warn（一次网络抖动不算
/// 工具本身的问题）。短超时，离线机器不会拖慢诊断。
async fn check_reachability(label: &str, url: &str) -> Finding {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .build();
    let Ok(client) = client else {
        return Finding {
            severity: Severity::Warn,
            message: format!("{label} 探测器构建失败"),
            suggestion: None,
        };
    };
    match client.get(url).send().await {
        Ok(resp) => Finding {
            severity: Severity::Ok,
            message: format!("{label} 可达（HTTP {}）", resp.status().as_u16()),
            suggestion: None,
        },
        Err(e) => Finding {
            severity: Severity::Warn,
            message: format!("{label} 不可达：{e}"),
            suggestion: Some("检查网络 / 代理设置".into()),
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
        let ps_profile =
            format!("{home}\\Documents\\WindowsPowerShell\\Microsoft.PowerShell_profile.ps1");
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
    let py = if which::which("python").is_ok() {
        "python"
    } else {
        "python3"
    };
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
    let name = Command::new("git")
        .args(["config", "--global", "user.name"])
        .output();
    let email = Command::new("git")
        .args(["config", "--global", "user.email"])
        .output();
    let name_ok = name
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false);
    let email_ok = email
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false);
    if name_ok && email_ok {
        Finding {
            severity: Severity::Ok,
            message: "git user.name / user.email 已配置".into(),
            suggestion: None,
        }
    } else {
        let mut missing = Vec::new();
        if !name_ok {
            missing.push("user.name");
        }
        if !email_ok {
            missing.push("user.email");
        }
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
        };
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

/// Classify an Anthropic API-key check into a `Finding`. Pure + deterministic
/// (no network) so it's unit-testable; [`verify_anthropic_key`] does the
/// actual HTTP call and feeds the result here.
///
/// `has_key=false` → no key in env (Warn). `http_status=None` → could not
/// reach the API (Warn, *not* Error — a network blip isn't a key problem).
/// 200 → valid (Ok), 401 → invalid (Error), 403 → authenticated but no
/// permission (Warn).
pub fn classify_key_check(has_key: bool, http_status: Option<u16>) -> Finding {
    if !has_key {
        return Finding {
            severity: Severity::Warn,
            message: "未检测到 ANTHROPIC_API_KEY / CLAUDE_API_KEY".into(),
            suggestion: Some(
                "在 shell 里 export ANTHROPIC_API_KEY=...（console.anthropic.com 生成）".into(),
            ),
        };
    }
    match http_status {
        Some(200) => Finding {
            severity: Severity::Ok,
            message: "API key 有效（GET /v1/models 返回 200）".into(),
            suggestion: None,
        },
        Some(401) => Finding {
            severity: Severity::Error,
            message: "API key 无效或已失效（401）".into(),
            suggestion: Some("到 console.anthropic.com 重新生成 key".into()),
        },
        Some(403) => Finding {
            severity: Severity::Warn,
            message: "API key 认证通过但权限不足（403）".into(),
            suggestion: Some("检查 key 所属组织 / 权限范围".into()),
        },
        Some(code) => Finding {
            severity: Severity::Warn,
            message: format!("Anthropic 返回意外状态码：HTTP {code}"),
            suggestion: None,
        },
        None => Finding {
            severity: Severity::Warn,
            message: "无法连接 api.anthropic.com（网络不可达或超时）".into(),
            suggestion: Some("检查网络 / 代理设置".into()),
        },
    }
}

/// Opt-in: validate the Anthropic API key by calling the (free)
/// `GET https://api.anthropic.com/v1/models` endpoint — not an inference
/// call, so it costs nothing. Never run automatically; only when the user
/// clicks the button. Endpoint + headers verified against the Anthropic API
/// reference (x-api-key + anthropic-version: 2023-06-01).
pub async fn verify_anthropic_key() -> Finding {
    let key = std::env::var("ANTHROPIC_API_KEY")
        .ok()
        .or_else(|| std::env::var("CLAUDE_API_KEY").ok());
    let Some(key) = key else {
        return classify_key_check(false, None);
    };
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build();
    let Ok(client) = client else {
        return classify_key_check(true, None);
    };
    let status = client
        .get("https://api.anthropic.com/v1/models")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await
        .ok()
        .map(|r| r.status().as_u16());
    classify_key_check(true, status)
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

    #[tokio::test]
    async fn run_diagnostics_for_unknown_tool_returns_empty_findings() {
        let r = run_diagnostics("nonexistent-tool").await.unwrap();
        assert_eq!(r.tool_id, "nonexistent-tool");
        assert!(r.findings.is_empty());
    }

    #[tokio::test]
    async fn run_diagnostics_for_known_tool_returns_findings() {
        // "cursor" 只跑 on_path 检查（which::which）——不 spawn、不联网，
        // 测试保持快且离线。
        let r = run_diagnostics("cursor").await.unwrap();
        assert_eq!(r.tool_id, "cursor");
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

    #[test]
    fn classify_key_check_ok_on_200() {
        assert_eq!(classify_key_check(true, Some(200)).severity, Severity::Ok);
    }

    #[test]
    fn classify_key_check_error_on_401() {
        assert_eq!(
            classify_key_check(true, Some(401)).severity,
            Severity::Error
        );
    }

    #[test]
    fn classify_key_check_warn_on_403() {
        assert_eq!(classify_key_check(true, Some(403)).severity, Severity::Warn);
    }

    #[test]
    fn classify_key_check_warn_when_no_key() {
        assert_eq!(classify_key_check(false, None).severity, Severity::Warn);
    }

    #[test]
    fn classify_key_check_warn_on_network_failure() {
        // has a key but the request failed → None status → Warn, not Error
        assert_eq!(classify_key_check(true, None).severity, Severity::Warn);
    }
}
