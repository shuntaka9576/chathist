use std::path::{Path, PathBuf};

pub fn expand_tilde(path: &str) -> PathBuf {
    if !path.starts_with('~') {
        return PathBuf::from(path);
    }

    if let Some(home) = dirs::home_dir() {
        if path == "~" {
            return home;
        }

        if let Some(rest) = path.strip_prefix("~/") {
            return home.join(rest);
        }
    }

    PathBuf::from(path)
}

/// Encode a path for use as a directory name (Claude Code compatible)
/// Replaces '/' and '.' with '-'
pub fn encode_path_for_dirname(path: &Path) -> String {
    path.to_string_lossy().replace(['/', '.'], "-")
}

pub fn contract_tilde(path: &Path) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        let path_str = path.to_string_lossy();

        if path_str.starts_with(home_str.as_ref()) {
            let rest = &path_str[home_str.len()..];

            if rest.is_empty() {
                return "~".to_string();
            } else if rest.starts_with('/') {
                return format!("~{rest}");
            }
        }
        path_str.into_owned()
    } else {
        path.to_string_lossy().into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde_no_tilde() {
        let path = "/usr/bin";
        let expanded = expand_tilde(path);
        assert_eq!(expanded, PathBuf::from("/usr/bin"));
    }

    #[test]
    fn test_expand_tilde_home_only() {
        let expanded = expand_tilde("~");
        let home = dirs::home_dir().unwrap();
        assert_eq!(expanded, home);
    }

    #[test]
    fn test_expand_tilde_home_slash() {
        let expanded = expand_tilde("~/Documents");
        let home = dirs::home_dir().unwrap();
        assert_eq!(expanded, home.join("Documents"));
    }

    #[test]
    fn test_contract_tilde_outside_home() {
        let path = Path::new("/var/log");
        let contracted = contract_tilde(path);
        assert_eq!(contracted, "/var/log");
    }

    #[test]
    fn test_contract_tilde_exact_home() {
        let home = dirs::home_dir().unwrap();
        let contracted = contract_tilde(&home);
        assert_eq!(contracted, "~");
    }

    #[test]
    fn test_contract_tilde_home_subdir() {
        let home = dirs::home_dir().unwrap();
        let sub_path = home.join("Pictures");
        let contracted = contract_tilde(&sub_path);
        assert_eq!(contracted, "~/Pictures");
    }

    #[test]
    fn test_encode_path_for_dirname() {
        let path = Path::new("/Users/test/repos/project");
        assert_eq!(encode_path_for_dirname(path), "-Users-test-repos-project");
    }

    #[test]
    fn test_encode_path_for_dirname_with_dots() {
        let path = Path::new("/home/user/.config/app");
        assert_eq!(encode_path_for_dirname(path), "-home-user--config-app");
    }
}
