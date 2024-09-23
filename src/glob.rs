//! Retrieves paths using glob patterns.
use std::{collections::HashSet, path::PathBuf};

use glob::{glob_with, MatchOptions};

/// Retrieves paths using glob patterns.
pub fn retrieve_glob_paths(glob_paths: Vec<String>) -> Vec<PathBuf> {
    let glob_options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    let mut absolute_paths: HashSet<PathBuf> = HashSet::new();

    for path in glob_paths {
        for entry in glob_with(&path, glob_options)
            .unwrap_or_else(|_| panic!("Failed to process glob pattern: {}", path))
        {
            match entry {
                Ok(garbage_path) => {
                    absolute_paths.insert(garbage_path);
                }
                Err(glob_error) => {
                    println!("Failed to process glob pattern {}: {}", path, glob_error);
                }
            }
        }
    }

    absolute_paths.into_iter().collect()
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

        assert!(paths.contains(&retrieve_tests_ilteoood().join("legit.esm.js")));
        assert!(paths.contains(&retrieve_tests_ilteoood().join("legit.js")));
        assert!(paths.contains(&retrieve_tests_ilteoood().join("unlegit.min.js")));
    }
}
