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

/// One entry from the GitHub `/releases` API (only the fields we need).
#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    draft: bool,
}

/// Parse a GitHub `/releases` JSON body into version options, newest first.
/// Skips drafts and pre-releases. `strip_prefix` is removed from each tag to
/// yield the bare version (e.g. "v" → "2.95.0", "bun-v" → "1.3.14"); tags
/// that don't start with the prefix, or don't look like X.Y.Z afterwards, are
/// dropped (filters out `untagged-*` and odd tags). Caps the list length.
pub fn parse_github_tags(json: &str, strip_prefix: &str, cap: usize) -> Vec<ParameterOption> {
    let releases: Vec<GithubRelease> = match serde_json::from_str(json) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    releases
        .into_iter()
        .filter(|r| !r.prerelease && !r.draft)
        .filter_map(|r| {
            let ver = r.tag_name.strip_prefix(strip_prefix)?;
            // Keep only clean X.Y.Z (drops rc/untagged/odd tags).
            if ver.split('.').count() == 3
                && ver
                    .split('.')
                    .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
            {
                Some(ParameterOption {
                    value: ver.to_string(),
                    label: ver.to_string(),
                })
            } else {
                None
            }
        })
        .take(cap)
        .collect()
}

/// Fetch versions from a GitHub repo's releases (newest first). `repo` is
/// "owner/name", `strip_prefix` turns a tag into a bare version. Returns
/// `None` on any network/parse failure — caller falls back to static options.
/// GitHub's API requires a User-Agent header.
pub async fn fetch_github_release_versions(
    repo: &str,
    strip_prefix: &str,
    cap: usize,
) -> Option<Vec<ParameterOption>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(6))
        .build()
        .ok()?;
    let url = format!("https://api.github.com/repos/{repo}/releases?per_page=30");
    let body = client
        .get(&url)
        .header("User-Agent", "flint-dev-launcher")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    let opts = parse_github_tags(&body, strip_prefix, cap);
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

    // ---- parse_github_tags (bun / gh version lists) ----

    const GH_RELEASES_SAMPLE: &str = r#"[
        {"tag_name":"v2.95.0","prerelease":false,"draft":false},
        {"tag_name":"v2.94.0","prerelease":false,"draft":false},
        {"tag_name":"v2.96.0-rc1","prerelease":true,"draft":false},
        {"tag_name":"untagged-abc123","prerelease":false,"draft":false},
        {"tag_name":"v2.93.0","prerelease":false,"draft":true}
    ]"#;

    #[test]
    fn parse_github_tags_strips_prefix_and_keeps_clean_versions() {
        let opts = parse_github_tags(GH_RELEASES_SAMPLE, "v", 10);
        // Only the two clean, non-prerelease, non-draft releases survive.
        assert_eq!(opts.len(), 2);
        assert_eq!(opts[0].value, "2.95.0");
        assert_eq!(opts[1].value, "2.94.0");
    }

    #[test]
    fn parse_github_tags_drops_prerelease_draft_and_untagged() {
        let opts = parse_github_tags(GH_RELEASES_SAMPLE, "v", 10);
        assert!(!opts.iter().any(|o| o.value.contains("rc")));
        assert!(!opts.iter().any(|o| o.value.contains("untagged")));
        assert!(!opts.iter().any(|o| o.value == "2.93.0")); // draft
    }

    #[test]
    fn parse_github_tags_handles_bun_prefix() {
        let bun = r#"[
            {"tag_name":"bun-v1.3.14","prerelease":false,"draft":false},
            {"tag_name":"bun-v1.3.13","prerelease":false,"draft":false}
        ]"#;
        let opts = parse_github_tags(bun, "bun-v", 10);
        assert_eq!(opts.len(), 2);
        assert_eq!(opts[0].value, "1.3.14");
    }

    #[test]
    fn parse_github_tags_respects_cap() {
        let opts = parse_github_tags(GH_RELEASES_SAMPLE, "v", 1);
        assert_eq!(opts.len(), 1);
    }

    #[test]
    fn parse_github_tags_handles_garbage() {
        assert!(parse_github_tags("not json", "v", 10).is_empty());
    }
}
