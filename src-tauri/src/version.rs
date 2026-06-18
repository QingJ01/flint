use std::cmp::Ordering;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Version(pub u32, pub u32, pub u32);

impl Version {
    pub fn major(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

impl Ord for Version {
    fn cmp(&self, o: &Self) -> Ordering {
        self.0
            .cmp(&o.0)
            .then(self.1.cmp(&o.1))
            .then(self.2.cmp(&o.2))
    }
}
impl PartialOrd for Version {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// 从 `node -v` / `v20.11.0` 这类输出解析版本。
pub fn parse_version(s: &str) -> Result<Version, String> {
    let s = s.trim().trim_start_matches('v');
    let mut parts = s.split('.');
    let maj = parts.next().and_then(|p| p.parse().ok());
    let min = parts.next().and_then(|p| p.parse().ok());
    let pat = parts.next().and_then(|p| p.parse().ok());
    match (maj, min, pat) {
        (Some(a), Some(b), Some(c)) => Ok(Version(a, b, c)),
        _ => Err(format!("invalid version: {s}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_node_version_output() {
        assert_eq!(parse_version("v20.11.0\n").unwrap().to_string(), "20.11.0");
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_version("not a version").is_err());
    }

    #[test]
    fn compares_versions() {
        let a = parse_version("v18.0.0").unwrap();
        let b = parse_version("v20.11.0").unwrap();
        assert!(a < b);
    }
}
