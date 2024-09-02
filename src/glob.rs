//! Retrieves paths using glob patterns.
use std::{collections::HashSet, path::PathBuf};

use glob::{glob_with, MatchOptions};

/// Filters duplicated paths from a list of paths.
fn filter_duplicated_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut unique_paths = HashSet::<String>::new();

    paths
        .into_iter()
        .filter(|path| unique_paths.insert(path.display().to_string()))
        .collect()
}

/// Retrieves paths using glob patterns.
pub fn retrieve_glob_paths(glob_paths: Vec<String>) -> Vec<PathBuf> {
    let glob_options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    let mut absolute_paths: Vec<PathBuf> = vec![];

    for path in glob_paths {
        for entry in glob_with(&path, glob_options)
            .unwrap_or_else(|_| panic!("Failed to process glob pattern: {}", path))
        {
            match entry {
                Ok(garbage_path) => absolute_paths.push(garbage_path),
                Err(glob_error) => {
                    println!("Failed to process glob pattern {}: {}", path, glob_error);
                }
            }
        }
    }

    filter_duplicated_paths(absolute_paths)
}

#[cfg(test)]
mod tests_duplicate_paths {
    use super::*;

    #[test]
    fn test_filter_duplicated_paths() {
        let paths = vec![
            PathBuf::from("/a"),
            PathBuf::from("/a"),
            PathBuf::from("/b"),
            PathBuf::from("/b"),
            PathBuf::from("/c"),
            PathBuf::from("/c"),
        ];
        assert_eq!(
            filter_duplicated_paths(paths),
            vec![
                PathBuf::from("/a"),
                PathBuf::from("/b"),
                PathBuf::from("/c")
            ]
        );
    }

    #[test]
    fn test_filter_dont_filter_not_duplicated_paths() {
        let paths = vec![
            PathBuf::from("/a"),
            PathBuf::from("/b"),
            PathBuf::from("/c"),
        ];
        assert_eq!(
            filter_duplicated_paths(paths),
            vec![
                PathBuf::from("/a"),
                PathBuf::from("/b"),
                PathBuf::from("/c")
            ]
        );
    }
}

#[cfg(test)]
mod tests_retrieve_glob_paths {
    use std::env;

    use super::*;

    fn retrieve_tests_ilteoood() -> PathBuf {
        PathBuf::from(env::current_dir().unwrap())
            .join("tests")
            .join("node_modules")
            .join("ilteoood")
    }

    fn retrieve_tests_js_paths() -> Vec<String> {
        vec![retrieve_tests_ilteoood().join("*.js").display().to_string()]
    }

    #[test]
    fn test_retrieve_glob_paths() {
        let js_paths = retrieve_tests_js_paths();
        let paths = retrieve_glob_paths(js_paths);

        assert_eq!(
            paths,
            vec![
                retrieve_tests_ilteoood().join("legit.esm.js"),
                retrieve_tests_ilteoood().join("legit.js"),
                retrieve_tests_ilteoood().join("unlegit.min.js"),
            ]
        );
    }
}
