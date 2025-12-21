use std::path::PathBuf;

/// Find the root directory of the Git repository by traversing parent directories
/// from the current directory.
///
/// # Returns
/// - `Some(PathBuf)`: The root directory of the Git repository
/// - `None`: If no `.git` directory is found
pub fn find_git_root() -> Option<PathBuf> {
    find_git_root_from(std::env::current_dir().ok()?)
}

/// Find the root directory of the Git repository by traversing parent directories
/// from the specified starting directory.
///
/// This function is exposed for testing purposes.
pub fn find_git_root_from(start_dir: PathBuf) -> Option<PathBuf> {
    let mut current = start_dir;

    loop {
        if current.join(".git").exists() {
            return Some(current);
        }

        if !current.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_find_git_root_at_root() {
        let temp = tempdir().unwrap();
        let git_dir = temp.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        let result = find_git_root_from(temp.path().to_path_buf());
        assert_eq!(result, Some(temp.path().to_path_buf()));
    }

    #[test]
    fn test_find_git_root_in_subdir() {
        let temp = tempdir().unwrap();
        let git_dir = temp.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        let subdir = temp.path().join("src").join("lib");
        fs::create_dir_all(&subdir).unwrap();

        let result = find_git_root_from(subdir);
        assert_eq!(result, Some(temp.path().to_path_buf()));
    }

    #[test]
    fn test_find_git_root_no_git() {
        let temp = tempdir().unwrap();
        let subdir = temp.path().join("some").join("dir");
        fs::create_dir_all(&subdir).unwrap();

        let result = find_git_root_from(subdir);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_git_root_nested_git() {
        // When there are nested .git directories, return the closest one
        let temp = tempdir().unwrap();

        // Parent .git
        let parent_git = temp.path().join(".git");
        fs::create_dir(&parent_git).unwrap();

        // Child .git (like a submodule)
        let child_dir = temp.path().join("submodule");
        fs::create_dir_all(&child_dir).unwrap();
        let child_git = child_dir.join(".git");
        fs::create_dir(&child_git).unwrap();

        let result = find_git_root_from(child_dir.clone());
        assert_eq!(result, Some(child_dir));
    }
}
