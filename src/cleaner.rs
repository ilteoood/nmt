use std::{fs::metadata, path::PathBuf};

use glob::{glob_with, MatchOptions};
use nmt::configurations::CliConfigurations;
use remove_empty_subdirs::remove_empty_subdirs;

static GARBAGE_ITEMS: &[&str] = &[
    // folders
    "@types",
    "bench",
    "browser",
    "docs",
    "example",
    "examples",
    "test",
    "tests",
    "benchmark",
    "integration",
    // extensions
    "*.md",
    "*.markdown",
    "*.map",
    "*.*ts",
    // specific files
    "license",
    "contributing",
    ".nycrc",
    "makefile",
    ".DS_Store",
    ".markdownlint-cli2.yaml",
    ".editorconfig",
    ".nvmrc",
    "bower.json",
    // generic files
    ".*ignore",
    ".eslint*",
    "*.min.*",
    "browser.*js",
    ".travis.*",
    ".coveralls.*",
    "tsconfig.*",
    ".prettierrc*",
    "*.bak",
    "karma.conf.*",
    ".git*",
    ".tap*",
    ".c8*",
];

static GARBAGE_ESM_ITEMS: &[&str] = &["esm", "*.esm.js", "*.mjs"];

fn manage_path<'a>(
    garbage_paths: &'a mut Vec<String>,
    configurations: &'a CliConfigurations,
) -> impl FnMut(&[&str]) + 'a {
    move |garbage_items: &[&str]| {
        for garbage_item in garbage_items {
            let garbage_path = configurations
                .node_modules_location
                .join("**")
                .join(garbage_item);

            match garbage_path.to_str() {
                Some(garbage_path) => garbage_paths.push(garbage_path.to_string()),
                None => println!("Failed to process: {}", garbage_item),
            }
        }
    }
}

fn generate_garbage_paths(configurations: &CliConfigurations) -> Vec<String> {
    let mut garbage_paths: Vec<String> = vec![];

    let mut manage_path_closure = manage_path(&mut garbage_paths, configurations);

    manage_path_closure(GARBAGE_ITEMS);

    if configurations.cjs_only {
        manage_path_closure(GARBAGE_ESM_ITEMS);
    }

    drop(manage_path_closure);

    garbage_paths
}

fn delete_path(path: PathBuf) {
    let path_location = path.display();
    println!("Removing: {}", path_location);
    let metadata = metadata(&path);

    match metadata {
        Ok(metadata) => {
            let remove_result = if metadata.is_dir() {
                std::fs::remove_dir_all(&path)
            } else {
                std::fs::remove_file(&path)
            };

            match remove_result {
                Ok(_) => println!("Removed: {}", path_location),
                Err(err) => println!("Failed to remove: {}, {}", path_location, err),
            }
        }
        Err(err) => println!("Failed to remove: {}, {}", path_location, err),
    }
}

pub fn retrieve_garbage(configurations: &CliConfigurations) -> Vec<PathBuf> {
    let glob_options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    let mut garbage_paths: Vec<PathBuf> = vec![];

    for path in generate_garbage_paths(configurations) {
        for entry in glob_with(&path, glob_options)
            .unwrap_or_else(|_| panic!("Failed to process glob pattern: {}", path))
        {
            match entry {
                Ok(garbage_path) => garbage_paths.push(garbage_path),
                Err(glob_error) => {
                    println!("Failed to process glob pattern {}: {}", path, glob_error);
                }
            }
        }
    }

    garbage_paths
}

fn remove_empty_dirs(configurations: &CliConfigurations) {
    match remove_empty_subdirs(&configurations.node_modules_location) {
        Ok(_) => println!("Removed empty directories"),
        Err(_) => println!("Failed to remove empty directories"),
    }
}

pub fn clean(configurations: &CliConfigurations, garbage: Vec<PathBuf>) {
    for path in garbage {
        delete_path(path);
    }
    remove_empty_dirs(configurations);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrieve_garbage() {
        let configurations = CliConfigurations::from_env();
        let garbage = retrieve_garbage(&configurations);
        assert!(garbage.is_empty());
    }
}
