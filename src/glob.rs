use std::{collections::HashSet, path::PathBuf};

use glob::{glob_with, MatchOptions};

fn filter_duplicated_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut unique_paths = HashSet::<String>::new();

    paths
        .into_iter()
        .filter(|path| unique_paths.insert(path.display().to_string()))
        .collect()
}

pub fn retrieve_glob_paths(paths: Vec<String>) -> Vec<PathBuf> {
    let glob_options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    let mut absolute_paths: Vec<PathBuf> = vec![];

    for path in paths {
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
