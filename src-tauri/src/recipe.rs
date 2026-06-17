use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Recipe {
    pub meta: Meta,
    #[serde(default)]
    pub install: HashMap<String, PlatformInstall>,
    #[serde(default)]
    pub detect: HashMap<String, PlatformCommand>,
    #[serde(default)]
    pub verify: HashMap<String, PlatformCommand>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Meta {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub category: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlatformInstall {
    #[serde(default)]
    pub requires_elevation: bool,
    pub steps: Vec<Step>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Step {
    pub cmd: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlatformCommand {
    pub cmd: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub version_regex: Option<String>,
}

impl Recipe {
    pub fn load(id: &str) -> Result<Self, String> {
        let path = std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(format!("resources/recipes/{id}.toml"));
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        toml::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_node_recipe() {
        let toml = r#"
[meta]
id = "node"
display_name = "Node.js"
category = "runtime"
[install.windows]
requires_elevation = false
steps = [ { cmd = "fnm", args = ["install", "--lts"] } ]
"#;
        let r: Recipe = toml::from_str(toml).unwrap();
        assert_eq!(r.meta.id, "node");
        let win = r.install.get("windows").unwrap();
        assert_eq!(win.steps.len(), 1);
        assert_eq!(win.steps[0].cmd, "fnm");
        assert!(!win.requires_elevation);
    }
}
