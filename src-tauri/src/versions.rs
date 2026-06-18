//! Dynamic version lists for installable tools. The recipe TOMLs ship a small
//! static `[parameters.*_version]` option list, but those go stale and are
//! tiny. This module fetches the *real* available versions at request time:
//! Node via `fnm list-remote`, Python via the endoflife.date API. Everything
//! is best-effort — on any failure the caller falls back to the recipe's
//! static options, so the UI never blocks or errors.

use crate::recipe::ParameterOption;

/// Parse the stdout of `fnm list-remote` into version options, newest first.
///
/// `fnm list-remote` prints one version per line, oldest→newest, e.g.:
/// ```text
/// v18.20.4
/// v20.17.0
/// v22.9.0 (Latest LTS: Jod)
/// ```
/// We keep only clean `vX.Y.Z` releases, drop pre-releases, reverse to
/// newest-first, and cap the list so the dropdown stays usable. `lts-latest`
/// and `latest` (the fnm aliases the recipe defaults to) are pinned on top so
/// the common case is one click.
pub fn parse_fnm_list_remote(stdout: &str, cap: usize) -> Vec<ParameterOption> {
    let mut versions: Vec<(String, bool)> = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        // First whitespace-delimited token is the version; the rest may be an
        // "(Latest LTS: …)" annotation.
        let Some(tok) = line.split_whitespace().next() else {
            continue;
        };
        let ver = tok.strip_prefix('v').unwrap_or(tok);
        // Keep only X.Y.Z numeric releases (skip headers / pre-releases).
        if ver.split('.').count() == 3
            && ver.split('.').all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
        {
            let is_lts = line.to_lowercase().contains("lts");
            versions.push((ver.to_string(), is_lts));
        }
    }
    versions.reverse(); // newest first

    let mut out = vec![
        ParameterOption { value: "lts-latest".into(), label: "LTS (推荐)".into() },
        ParameterOption { value: "latest".into(), label: "Latest".into() },
    ];
    for (ver, is_lts) in versions.into_iter().take(cap) {
        let label = if is_lts {
            format!("{ver} (LTS)")
        } else {
            ver.clone()
        };
        out.push(ParameterOption { value: ver, label });
    }
    out
}

/// One release line from the endoflife.date Python API.
#[derive(serde::Deserialize)]
struct PythonEolEntry {
    cycle: String,
    latest: String,
}

/// Parse the endoflife.date `python.json` body into version options.
/// Each entry's `latest` is a full patch version (e.g. "3.13.14") — exactly
/// the form the Python embeddable-zip URL needs.
///
/// endoflife returns every release line back to Python 1.x. The Windows
/// embeddable zip only exists for 3.x, so we keep only `3.y` lines (dropping
/// 2.x/1.x that would just 404 the download), and sort newest cycle first so
/// 3.14 / 3.13 lead the dropdown.
pub fn parse_python_eol(json: &str) -> Vec<ParameterOption> {
    let entries: Vec<PythonEolEntry> = match serde_json::from_str(json) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut lines: Vec<(u32, String)> = entries
        .into_iter()
        .filter(|e| !e.latest.is_empty())
        .filter_map(|e| {
            // cycle is "3.13"; keep major==3, parse minor for sorting.
            let mut parts = e.cycle.split('.');
            let major = parts.next()?;
            let minor: u32 = parts.next()?.parse().ok()?;
            if major != "3" {
                return None;
            }
            Some((minor, e.latest))
        })
        .collect();
    lines.sort_by(|a, b| b.0.cmp(&a.0)); // newest minor first
    lines
        .into_iter()
        .map(|(_minor, latest)| ParameterOption {
            value: latest.clone(),
            label: latest,
        })
        .collect()
}

/// Fetch Node versions via `fnm list-remote`. Returns `None` if fnm isn't on
/// PATH (Node not installed yet) or the command fails — caller falls back to
/// the recipe's static options.
pub async fn fetch_node_versions(cap: usize) -> Option<Vec<ParameterOption>> {
    if which::which("fnm").is_err() {
        return None;
    }
    let (program, args) = crate::shell::resolve("fnm", &["list-remote".to_string()]);
    let out = tokio::process::Command::new(program)
        .args(&args)
        .output()
        .await
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let opts = parse_fnm_list_remote(&stdout, cap);
    // Only the two aliases means we parsed nothing useful.
    if opts.len() <= 2 {
        None
    } else {
        Some(opts)
    }
}

/// Fetch Python versions from endoflife.date. Returns `None` on any network
/// or parse failure — caller falls back to static options.
pub async fn fetch_python_versions() -> Option<Vec<ParameterOption>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(6))
        .build()
        .ok()?;
    let body = client
        .get("https://endoflife.date/api/python.json")
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    let opts = parse_python_eol(&body);
    if opts.is_empty() {
        None
    } else {
        Some(opts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FNM_SAMPLE: &str = "\
v16.20.2
v18.20.4
v20.17.0
v22.9.0 (Latest LTS: Jod)
v23.0.0
";

    #[test]
    fn parse_fnm_keeps_numeric_releases_newest_first() {
        let opts = parse_fnm_list_remote(FNM_SAMPLE, 10);
        // First two are the pinned aliases.
        assert_eq!(opts[0].value, "lts-latest");
        assert_eq!(opts[1].value, "latest");
        // Then real versions, newest first.
        assert_eq!(opts[2].value, "23.0.0");
        assert_eq!(opts[3].value, "22.9.0");
        // The LTS annotation is reflected in the label.
        assert!(opts[3].label.contains("LTS"), "got: {}", opts[3].label);
    }

    #[test]
    fn parse_fnm_respects_cap() {
        let opts = parse_fnm_list_remote(FNM_SAMPLE, 2);
        // 2 aliases + 2 capped versions.
        assert_eq!(opts.len(), 4);
    }

    #[test]
    fn parse_fnm_skips_garbage_lines() {
        let opts = parse_fnm_list_remote("not a version\nv20.1.0\n\n", 10);
        assert_eq!(opts.len(), 3); // 2 aliases + 1 real
        assert_eq!(opts[2].value, "20.1.0");
    }

    const PYTHON_EOL_SAMPLE: &str = r#"[
        {"cycle":"3.9","latest":"3.9.20","eol":"2025-10-31"},
        {"cycle":"3.14","latest":"3.14.6","eol":false},
        {"cycle":"3.13","latest":"3.13.14","eol":false},
        {"cycle":"2.7","latest":"2.7.18","eol":"2020-01-01"},
        {"cycle":"1.6","latest":"1.6.1","eol":true}
    ]"#;

    #[test]
    fn parse_python_eol_maps_latest_patch_newest_first() {
        let opts = parse_python_eol(PYTHON_EOL_SAMPLE);
        // 3.14 → 3.13 → 3.9 (sorted by minor desc), 2.x/1.x dropped.
        assert_eq!(opts.len(), 3);
        assert_eq!(opts[0].value, "3.14.6");
        assert_eq!(opts[1].value, "3.13.14");
        assert_eq!(opts[2].value, "3.9.20");
    }

    #[test]
    fn parse_python_eol_drops_pre_3x_versions() {
        let opts = parse_python_eol(PYTHON_EOL_SAMPLE);
        assert!(
            !opts.iter().any(|o| o.value.starts_with("2.") || o.value.starts_with("1.")),
            "must drop Python 2.x / 1.x (no embeddable zip): {opts:?}"
        );
    }

    #[test]
    fn parse_python_eol_label_has_no_eol_marker() {
        let opts = parse_python_eol(PYTHON_EOL_SAMPLE);
        assert!(
            opts.iter().all(|o| !o.label.contains("停止维护")),
            "labels must not carry EOL markers: {opts:?}"
        );
    }

    #[test]
    fn parse_python_eol_handles_garbage() {
        assert!(parse_python_eol("not json").is_empty());
    }
}
