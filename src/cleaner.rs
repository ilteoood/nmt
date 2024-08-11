use std::{fs::metadata, path::PathBuf};

use crate::configurations::Configurations;
use glob::{glob_with, MatchOptions};
use remove_empty_subdirs::remove_empty_subdirs;

static DEFAULT_PATHS: &[&str] = &[
    // folders
    "@types",
    ".github",
    "bench",
    "browser",
    "docs",
    "example",
    "examples",
    "test",
    "tests",
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
    ".DS_Store",
];

fn generate_default_paths(configurations: &Configurations) -> Vec<String> {
    let mut paths: Vec<String> = vec![];

    for default_path in DEFAULT_PATHS {
        let join = configurations
            .node_modules_location
            .join("**")
            .join(default_path);
        paths.push(join.to_str().unwrap().to_string());
    }

    paths
}

fn delete_path(path: PathBuf) {
    let path_location = path.display();
    println!("Removing: {}", path_location);
    let metadata = metadata(&path).unwrap();

    let remove_result = if metadata.is_dir() {
        std::fs::remove_dir_all(&path)
    } else {
        std::fs::remove_file(&path)
    };

    match remove_result {
        Ok(_) => println!("Removed: {}", path_location),
        Err(_) => println!("Failed to remove: {}", path_location),
    }
}

pub fn retrieve_garbage(configurations: &Configurations) -> Vec<PathBuf> {
    let glob_options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    let mut garbage_paths: Vec<PathBuf> = vec![];

    for path in generate_default_paths(configurations) {
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

fn remove_empty_dirs(configurations: &Configurations) {
    match remove_empty_subdirs(&configurations.node_modules_location) {
        Ok(_) => println!("Removed empty directories"),
        Err(_) => println!("Failed to remove empty directories"),
    }
}

pub fn clean(configurations: &Configurations, garbage: Vec<PathBuf>) {
    for path in garbage {
        delete_path(path);
    }
    remove_empty_dirs(configurations);
}
