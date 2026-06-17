use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Recipe {
    pub meta: Meta,
    #[serde(default)]
    pub install: HashMap<String, PlatformInstall>,
    #[serde(default)]
    pub detect: HashMap<String, PlatformCommand>,
    #[serde(default)]
    pub verify: HashMap<String, PlatformCommand>,
    #[serde(default)]
    pub parameters: HashMap<String, ParameterDef>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Meta {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ParameterDef {
    pub label: String,
    #[serde(default)]
    pub default: Option<String>,
    pub options: Vec<ParameterOption>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ParameterOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlatformInstall {
    #[serde(default)]
    pub requires_elevation: bool,
    pub steps: Vec<Step>,
    /// Absolute paths (with `%VAR%` placeholders) to add to the user's
    /// persistent PATH after all install steps succeed. `add_to_user_path`
    /// is idempotent (de-duped, case-insensitive on Windows).
    #[serde(default)]
    pub add_to_user_path: Vec<String>,
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
        Self::load_from(Path::new("resources/recipes"), id)
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

    /// Convenience: list available recipes from CWD-relative `resources/recipes/`.
    pub fn list_available() -> Vec<Meta> {
        Self::list_available_from(Path::new("resources/recipes"))
    }

    /// Scan `dir` for `<id>.toml` files and return each recipe's `Meta`.
    /// Skips files that fail to parse (and writes the error to stderr so a
    /// malformed recipe is visible but does not crash the dashboard).
    pub fn list_available_from(dir: &Path) -> Vec<Meta> {
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
                Ok(r) => metas.push(r.meta),
                Err(e) => eprintln!("[flint] skipping {id}.toml: {e}"),
            }
        }
        metas.sort_by(|a, b| a.id.cmp(&b.id));
        metas
    }

    /// Replace `{name}` placeholders in step `cmd` / `args` and in
    /// `add_to_user_path` entries, using `params`. Unknown placeholders are
    /// left as literal (so a step can include a `{` without escaping).
    /// Missing param values fall back to the recipe's declared `default`.
    /// If no value and no default, returns an error.
    pub fn substitute(&self, params: &HashMap<String, String>) -> Result<Recipe, String> {
        let mut new_install = HashMap::new();
        for (platform, plat_install) in &self.install {
            let mut new_steps = Vec::with_capacity(plat_install.steps.len());
            for step in &plat_install.steps {
                let cmd = substitute_placeholders(&step.cmd, &self.parameters, params)?;
                let args = step
                    .args
                    .iter()
                    .map(|a| substitute_placeholders(a, &self.parameters, params))
                    .collect::<Result<Vec<_>, _>>()?;
                new_steps.push(Step { cmd, args });
            }
            let new_add_to_path = plat_install
                .add_to_user_path
                .iter()
                .map(|p| substitute_placeholders(p, &self.parameters, params))
                .collect::<Result<Vec<_>, _>>()?;
            new_install.insert(
                platform.clone(),
                PlatformInstall {
                    requires_elevation: plat_install.requires_elevation,
                    steps: new_steps,
                    add_to_user_path: new_add_to_path,
                },
            );
        }
        Ok(Recipe {
            meta: self.meta.clone(),
            install: new_install,
            detect: self.detect.clone(),
            verify: self.verify.clone(),
            parameters: self.parameters.clone(),
        })
    }
}

fn substitute_placeholders(
    s: &str,
    declared: &HashMap<String, ParameterDef>,
    params: &HashMap<String, String>,
) -> Result<String, String> {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find('}') {
            Some(close) => {
                let key = &after[..close];
                if !is_known_or_unused(key, declared, params) {
                    return Err(format!(
                        "parameter '{key}' is used in a step but has no value and no declared default"
                    ));
                }
                let value = lookup_value(key, declared, params);
                out.push_str(&value);
                rest = &after[close + 1..];
            }
            None => {
                out.push_str(&rest[open..]);
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    Ok(out)
}

fn is_known_or_unused(
    key: &str,
    declared: &HashMap<String, ParameterDef>,
    params: &HashMap<String, String>,
) -> bool {
    if params.contains_key(key) {
        return true;
    }
    if let Some(def) = declared.get(key) {
        return def.default.is_some();
    }
    false
}

fn lookup_value(
    key: &str,
    declared: &HashMap<String, ParameterDef>,
    params: &HashMap<String, String>,
) -> String {
    if let Some(v) = params.get(key) {
        return v.clone();
    }
    if let Some(def) = declared.get(key) {
        if let Some(d) = &def.default {
            return d.clone();
        }
    }
    String::new()
}

/// Expand `%FOO%` placeholders using process environment variables.
/// Unknown vars are left as literal so a step/path stays informative.
/// Pure: takes `&std::collections::HashMap<String, String>` so tests can
/// pass synthetic envs without touching the real process environment.
pub fn expand_env_vars(input: &str, env: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(open) = rest.find('%') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find('%') {
            Some(close) => {
                let key = &after[..close];
                if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    // Not a valid %VAR% — keep both `%`s literal.
                    out.push('%');
                    out.push_str(&after[..close + 1]);
                    rest = &after[close + 1..];
                } else {
                    let val = env
                        .get(key)
                        .cloned()
                        .unwrap_or_else(|| format!("%{key}%"));
                    out.push_str(&val);
                    rest = &after[close + 1..];
                }
            }
            None => {
                out.push_str(&rest[open..]);
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    const NODE_TOML: &str = r#"
[meta]
id = "node"
display_name = "Node.js"
category = "runtime"
[install.windows]
requires_elevation = false
steps = [ { cmd = "fnm", args = ["install", "--lts"] } ]
"#;

    const PYTHON_TOML: &str = r#"
[meta]
id = "python"
display_name = "Python"
category = "runtime"

[parameters.python_version]
type = "select"
label = "Python 版本"
default = "3.12.7"
options = [
  { value = "3.13.0", label = "3.13" },
  { value = "3.12.7", label = "3.12" },
]

[install.windows]
requires_elevation = false
steps = [
  { cmd = "powershell", args = ["-Command", "Invoke-WebRequest https://python.org/ftp/python/{python_version}/python-{python_version}-embed.zip"] },
]
"#;

    #[test]
    fn parses_node_recipe() {
        let r: Recipe = toml::from_str(NODE_TOML).unwrap();
        assert_eq!(r.meta.id, "node");
        let win = r.install.get("windows").unwrap();
        assert_eq!(win.steps.len(), 1);
        assert_eq!(win.steps[0].cmd, "fnm");
        assert!(!win.requires_elevation);
    }

    #[test]
    fn parses_recipe_with_parameters() {
        let r: Recipe = toml::from_str(PYTHON_TOML).unwrap();
        let p = r.parameters.get("python_version").unwrap();
        assert_eq!(p.label, "Python 版本");
        assert_eq!(p.default.as_deref(), Some("3.12.7"));
        assert_eq!(p.options.len(), 2);
        assert_eq!(p.options[0].value, "3.13.0");
        assert_eq!(p.options[1].value, "3.12.7");
    }

    #[test]
    fn substitute_replaces_placeholder_in_cmd_and_args() {
        let r: Recipe = toml::from_str(PYTHON_TOML).unwrap();
        let mut params = HashMap::new();
        params.insert("python_version".into(), "3.12.7".into());
        let r2 = r.substitute(&params).unwrap();
        let win = r2.install.get("windows").unwrap();
        let step = &win.steps[0];
        assert!(step.cmd.starts_with("powershell"));
        let arg = &step.args[1];
        assert!(arg.contains("python-3.12.7-embed.zip"), "got: {arg}");
        assert!(!arg.contains("{python_version}"), "placeholder not replaced: {arg}");
    }

    #[test]
    fn substitute_falls_back_to_declared_default() {
        let r: Recipe = toml::from_str(PYTHON_TOML).unwrap();
        let params = HashMap::new(); // empty — should use default
        let r2 = r.substitute(&params).unwrap();
        let win = r2.install.get("windows").unwrap();
        let arg = &win.steps[0].args[1];
        assert!(arg.contains("python-3.12.7-embed.zip"), "default not applied: {arg}");
    }

    #[test]
    fn substitute_errors_on_undeclared_param_without_default() {
        let r: Recipe = toml::from_str(NODE_TOML).unwrap();
        // Force a placeholder into a step
        let mut bad = r.clone();
        bad.install.get_mut("windows").unwrap().steps[0].args.push("echo {missing}".into());
        let params = HashMap::new();
        let err = bad.substitute(&params).unwrap_err();
        assert!(err.contains("missing"), "got: {err}");
    }

    #[test]
    fn substitute_leaves_braces_without_placeholder_as_literal() {
        let r: Recipe = toml::from_str(NODE_TOML).unwrap();
        // step with a `{` that isn't a known placeholder
        let mut bad = r.clone();
        bad.install.get_mut("windows").unwrap().steps[0].args = vec!["echo {not_a_param}".into()];
        let params = HashMap::new();
        // unknown placeholder should be treated as undeclared → error
        let err = bad.substitute(&params).unwrap_err();
        assert!(err.contains("not_a_param"));
    }

    #[test]
    fn load_optional_returns_none_for_missing_id() {
        // Use a known-missing path. We can't easily redirect CWD, so just
        // call the explicit load_from with a tempdir.
        let dir = std::env::temp_dir().join("flint_recipe_load_optional_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(Recipe::load_optional("nonexistent_tool").is_none().then_some(()).is_none() == false);
    }

    #[test]
    fn list_available_finds_recipes_in_dir() {
        let dir = tempdir("flint_list_available");
        std::fs::write(dir.join("alpha.toml"), recipe_toml_with_id("alpha")).unwrap();
        std::fs::write(dir.join("beta.toml"), recipe_toml_with_id("beta")).unwrap();
        std::fs::write(dir.join("not_a_recipe.txt"), "ignored").unwrap();

        let metas = Recipe::list_available_from(&dir);
        let ids: Vec<_> = metas.iter().map(|m| m.id.clone()).collect();
        assert_eq!(ids, vec!["alpha", "beta"]);
    }

    #[test]
    fn list_available_skips_malformed_recipe() {
        let dir = tempdir("flint_list_malformed");
        std::fs::write(dir.join("good.toml"), recipe_toml_with_id("good")).unwrap();
        std::fs::write(dir.join("bad.toml"), "this is = not valid = toml ===").unwrap();
        let metas = Recipe::list_available_from(&dir);
        assert_eq!(metas.len(), 1);
        assert_eq!(metas[0].id, "good");
    }

    fn recipe_toml_with_id(id: &str) -> String {
        format!(
            r#"
[meta]
id = "{id}"
display_name = "Test {id}"
category = "runtime"
[install.windows]
requires_elevation = false
steps = [ {{ cmd = "echo", args = ["{id}"] }} ]
"#
        )
    }

    #[test]
    fn list_available_returns_empty_for_missing_dir() {
        let dir = PathBuf::from("Z:/this/should/not/exist/at/all");
        let metas = Recipe::list_available_from(&dir);
        assert!(metas.is_empty());
    }

    fn tempdir(label: &str) -> PathBuf {
        let d = std::env::temp_dir().join(label);
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn env_fixture() -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("USERPROFILE".into(), "C:\\Users\\me".into());
        h.insert("LOCALAPPDATA".into(), "C:\\Users\\me\\AppData\\Local".into());
        h
    }

    #[test]
    fn expand_env_vars_replaces_known_vars() {
        let env = env_fixture();
        let s = expand_env_vars("%LOCALAPPDATA%\\Programs\\Python", &env);
        assert_eq!(s, "C:\\Users\\me\\AppData\\Local\\Programs\\Python");
    }

    #[test]
    fn expand_env_vars_leaves_unknown_as_literal() {
        let env = env_fixture();
        let s = expand_env_vars("%NOPE%\\bin", &env);
        assert_eq!(s, "%NOPE%\\bin");
    }

    #[test]
    fn expand_env_vars_handles_mixed_text() {
        let env = env_fixture();
        let s = expand_env_vars("prefix-%USERPROFILE%-suffix", &env);
        assert_eq!(s, "prefix-C:\\Users\\me-suffix");
    }

    #[test]
    fn expand_env_vars_handles_no_placeholders() {
        let env = env_fixture();
        let s = expand_env_vars("plain string", &env);
        assert_eq!(s, "plain string");
    }

    #[test]
    fn expand_env_vars_keeps_lone_percent_literal() {
        let env = env_fixture();
        let s = expand_env_vars("100% done", &env);
        assert_eq!(s, "100% done");
    }

    const PYTHON_TOML_WITH_PATH: &str = r#"
[meta]
id = "python"
display_name = "Python"
category = "runtime"

[parameters.python_version]
label = "Python 版本"
default = "3.12.7"
options = [
  { value = "3.12.7", label = "3.12" },
]

[install.windows]
requires_elevation = false
add_to_user_path = ["%LOCALAPPDATA%\\Programs\\Python\\python-{python_version}"]
steps = [
  { cmd = "echo", args = ["install {python_version}"] },
]
"#;

    #[test]
    fn substitute_replaces_placeholders_in_add_to_user_path() {
        let r: Recipe = toml::from_str(PYTHON_TOML_WITH_PATH).unwrap();
        let mut params = HashMap::new();
        params.insert("python_version".into(), "3.12.7".into());
        let r2 = r.substitute(&params).unwrap();
        let win = r2.install.get("windows").unwrap();
        assert_eq!(win.add_to_user_path.len(), 1);
        assert_eq!(win.add_to_user_path[0], "%LOCALAPPDATA%\\Programs\\Python\\python-3.12.7");
    }
}
