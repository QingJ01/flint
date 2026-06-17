use crate::error::Result;
use std::process::Command;

/// 返回某工具是否存在。
pub fn is_installed(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

/// 运行 `cmd args`，取 stdout，正则抽版本。None 表示未装/解析不到。
pub fn detect_version(cmd: &str, args: &[String]) -> Result<Option<String>> {
    if !is_installed(cmd) { return Ok(None); }
    let out = Command::new(cmd).args(args).output()?;
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(crate::version::parse_version(&text).ok().map(|v| v.to_string()))
}

/// Node 检测的便捷聚合。
pub fn detect_node() -> Result<Option<String>> {
    detect_version("node", &["--version".into()])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_node_when_present() {
        // 开发机通常有 node；有则解析出版本
        if let Ok(Some(v)) = detect_version("node", &["--version".into()]) {
            assert!(v.starts_with(|c: char| c.is_ascii_digit()));
        }
    }
}
