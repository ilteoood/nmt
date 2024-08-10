use std::path::Path;

use glob::glob;
use remove_empty_subdirs::remove_empty_subdirs;

static DEFAULT_PATHS: &'static [&str] = &[
    // folders
    "@types",
    ".github",
    "bench",
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
];

fn generate_default_paths(node_modules_location: &str) -> Vec<String> {
    let mut paths: Vec<String> = vec![];

    for default_path in DEFAULT_PATHS {
        let join = Path::new(node_modules_location).join(default_path);
        paths.push(join.to_str().unwrap().to_string());
    }

    paths
}

fn clean_content (node_modules_location: &str) {
    for path in generate_default_paths(node_modules_location) {
        for entry in glob(&path).expect(&format!("Failed to clean glob pattern: {}", path)) {
            match entry {
                Ok(path) => {
                    println!("Removing: {}", path.display());
                    std::fs::remove_file(path).unwrap();
                }
                Err(globError) => {
                    println!("Failed to clean glob pattern {}: {}", path, globError.to_string());
                }
            }
        }
    }
}

fn remove_empty_dirs(node_modules_location: &str) {
    let path = Path::new(node_modules_location);
    match remove_empty_subdirs(path) {
        Ok(_) => println!("Removed empty directories"),
        Err(_) => println!("Failed to remove empty directories"),
    }
}

pub fn clean(node_modules_location: &str) {
    clean_content(node_modules_location);
    remove_empty_dirs(node_modules_location);
}
