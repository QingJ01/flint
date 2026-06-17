//! How Flint spawns external processes.
//!
//! Windows quirk: `CreateProcessW` (what `Command::new` uses under the hood)
//! does **not** consult `PATHEXT`. So `Command::new("pnpm")` fails with
//! "program not found" even though `pnpm.cmd` is on PATH — there is no
//! `pnpm.exe`. Every npm-global tool (pnpm, npm, opencode, codex, …) ships
//! as a `.cmd`/`.bat` batch shim, so they all hit this.
//!
//! [`resolve`] is the single chokepoint that fixes it: resolve the real path
//! via `which`, and for batch shims route through `cmd /C <name>` (letting
//! cmd do its own PATHEXT search, which sidesteps any spaces in the path).
//! Real executables get their resolved full path passed straight through.

use std::path::Path;

/// Is `resolved` a `.cmd` / `.bat` batch shim (case-insensitive extension)?
fn is_batch(resolved: &Path) -> bool {
    resolved
        .extension()
        .map(|e| e.eq_ignore_ascii_case("cmd") || e.eq_ignore_ascii_case("bat"))
        .unwrap_or(false)
}

/// Translate a `(program, args)` pair into the form `Command::new(..)` should
/// actually spawn on this platform. See the module docs for the Windows
/// `.cmd`/`.bat` rationale.
pub fn resolve(program: &str, args: &[String]) -> (String, Vec<String>) {
    match which::which(program) {
        // Batch shim — CreateProcessW can't run it directly; hand it to cmd
        // so PATHEXT resolves `pnpm` → `pnpm.cmd`. We pass the bare name
        // (not the resolved path) so a space in the shim's directory can't
        // trip cmd's quoting rules.
        Ok(path) if cfg!(windows) && is_batch(&path) => {
            let mut full = Vec::with_capacity(args.len() + 2);
            full.push("/C".to_string());
            full.push(program.to_string());
            full.extend(args.iter().cloned());
            ("cmd".to_string(), full)
        }
        // Real executable — pass the resolved full path (more robust than
        // re-searching PATH, and avoids the CreateProcessW bare-name trap).
        Ok(path) => (path.to_string_lossy().into_owned(), args.to_vec()),
        // Not on PATH — return the bare name so the caller gets the usual
        // NotFound io error (callers are expected to be best-effort).
        Err(_) => (program.to_string(), args.to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_cmd_and_bat_extensions_case_insensitively() {
        assert!(is_batch(Path::new(r"C:\Users\X\npm\pnpm.cmd")));
        assert!(is_batch(Path::new("pnpm.CMD")));
        assert!(is_batch(Path::new("tool.bat")));
        assert!(is_batch(Path::new("tool.BAT")));
        assert!(!is_batch(Path::new(r"D:\nodejs\node.exe")));
        assert!(!is_batch(Path::new("node"))); // no extension
        assert!(!is_batch(Path::new("opencode"))); // no extension
    }

    #[test]
    #[cfg(windows)]
    fn resolve_routes_batch_shim_through_cmd_exe() {
        // pnpm is an npm-global `.cmd` shim on a typical Windows dev box.
        // Skip cleanly if this machine doesn't have it rather than fail.
        if which::which("pnpm").is_err() {
            eprintln!("[test] pnpm not on PATH; skipping cmd-shim routing test");
            return;
        }
        let (prog, args) = resolve("pnpm", &["--version".to_string()]);
        assert_eq!(prog, "cmd", "pnpm.cmd must route through cmd /C");
        assert_eq!(args[0], "/C");
        assert_eq!(
            args[1], "pnpm",
            "should pass the bare name (not the resolved path)"
        );
        assert_eq!(args[2], "--version");
    }
}
