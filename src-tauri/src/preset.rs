use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Preset {
    pub meta: PresetMeta,
    pub tools: PresetTools,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct PresetMeta {
    pub id: String,
    pub display_name: String,
    pub description: String,
    #[serde(default)]
    pub emoji: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct PresetTools {
    /// Tool ids (matching recipe `meta.id`) to install in this preset.
    /// Order is the install order.
    pub ids: Vec<String>,
    /// Optional default parameter values for tools in this preset. The
    /// frontend may still override these; the preset is just a starting
    /// point.
    #[serde(default)]
    pub params: HashMap<String, HashMap<String, String>>,
}

impl Preset {
    pub fn load(id: &str) -> Result<Self, String> {
        Self::load_from(Path::new("resources/presets"), id)
    }

    pub fn load_optional(id: &str) -> Option<Self> {
        Self::load(id).ok()
    }

    pub fn load_from(dir: &Path, id: &str) -> Result<Self, String> {
        let path = dir.join(format!("{id}.toml"));
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        toml::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
    }

    /// Convenience: list every preset at CWD-relative `resources/presets/`.
    pub fn list_available() -> Vec<PresetMeta> {
        Self::list_available_from(Path::new("resources/presets"))
    }

    /// Scan `dir` for `<id>.toml` files and return each preset's `Meta`.
    /// Skips files that fail to parse (stderr) so one bad preset doesn't
    /// crash the dashboard.
    pub fn list_available_from(dir: &Path) -> Vec<PresetMeta> {
        let entries = match std::fs::read_dir(dir) {
            Ok(it) => it,
            Err(_) => return Vec::new(),
        };
        let mut metas = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            match Self::load_from(dir, id) {
                Ok(p) => metas.push(p.meta),
                Err(e) => eprintln!("[flint] skipping {id}.toml: {e}"),
            }
        }
        metas.sort_by(|a, b| a.id.cmp(&b.id));
        metas
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const VIBECODER_TOML: &str = r#"
[meta]
id = "vibecoder-stack"
display_name = "Vibecoder 全家桶"
description = "Node + Bun + Git + GitHub CLI + Claude Code + OpenCode + Cursor + pnpm"
emoji = "🤖"

[tools]
ids = ["node", "bun", "git", "github-cli", "claude-code", "opencode", "cursor", "pnpm"]

[tools.params]
node = { node_version = "lts-latest" }
"#;

    fn tempdir(label: &str) -> PathBuf {
        let d = std::env::temp_dir().join(label);
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn parses_preset_toml() {
        let p: Preset = toml::from_str(VIBECODER_TOML).unwrap();
        assert_eq!(p.meta.id, "vibecoder-stack");
        assert_eq!(p.meta.emoji, "🤖");
        assert_eq!(p.tools.ids.len(), 8);
        assert!(p.tools.ids.contains(&"claude-code".into()));
        assert_eq!(
            p.tools.params.get("node").unwrap().get("node_version").unwrap(),
            "lts-latest"
        );
    }

    #[test]
    fn list_available_finds_all_presets() {
        let dir = tempdir("flint_preset_list");
        std::fs::write(dir.join("alpha.toml"), preset_toml_with_id("alpha")).unwrap();
        std::fs::write(dir.join("beta.toml"), preset_toml_with_id("beta")).unwrap();
        std::fs::write(dir.join("not_a_preset.txt"), "ignored").unwrap();

        let metas = Preset::list_available_from(&dir);
        let ids: Vec<_> = metas.iter().map(|m| m.id.clone()).collect();
        assert_eq!(ids, vec!["alpha", "beta"]);
    }

    #[test]
    fn list_available_skips_malformed_preset() {
        let dir = tempdir("flint_preset_malformed");
        std::fs::write(dir.join("good.toml"), preset_toml_with_id("good")).unwrap();
        std::fs::write(dir.join("bad.toml"), "this is = not valid toml ===").unwrap();
        let metas = Preset::list_available_from(&dir);
        assert_eq!(metas.len(), 1);
        assert_eq!(metas[0].id, "good");
    }

    fn preset_toml_with_id(id: &str) -> String {
        format!(
            r#"
[meta]
id = "{id}"
display_name = "Test {id}"
description = "test"
emoji = "🧪"

[tools]
ids = ["node"]
"#
        )
    }

    #[test]
    fn list_available_returns_empty_for_missing_dir() {
        let dir = PathBuf::from("Z:/this/should/not/exist");
        let metas = Preset::list_available_from(&dir);
        assert!(metas.is_empty());
    }

    #[test]
    fn shipped_presets_parse_and_have_tools() {
        let dir = std::path::Path::new("resources/presets");
        if !dir.exists() {
            return;
        }
        let metas = Preset::list_available_from(dir);
        assert!(metas.len() >= 5, "expected 5 shipped presets, found {}", metas.len());
        for m in &metas {
            let p = Preset::load_from(dir, &m.id).unwrap();
            assert!(!p.tools.ids.is_empty(), "preset {} has empty tools.ids", m.id);
        }
    }
}
