use std::{collections::HashSet, fs::metadata, path::PathBuf};

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
    "*.ts",
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
    "*eslint*",
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

fn filter_duplicated_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut unique_paths = HashSet::<String>::new();

    paths
        .into_iter()
        .filter(|path| unique_paths.insert(path.display().to_string()))
        .collect()
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

    filter_duplicated_paths(garbage_paths)
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
    use std::env;

    use super::*;

    fn base_garbage_structure() -> Vec<String> {
        vec![
            "/tests/node_modules/@types".to_owned(),
            "/tests/node_modules/fastify/README.md".to_owned(),
            "/tests/node_modules/fastify/eslint.config.ts".to_owned(),
            "/tests/node_modules/busboy/.nvmrc".to_owned(),
            "/tests/node_modules/busboy/.eslintrc.json".to_owned(),
            "/tests/node_modules/ilteoood/unlegit.min.js".to_owned(),
            "/tests/node_modules/@types/tsconfig.json".to_owned(),
        ]
    }

    #[test]
    fn test_retrieve_garbage() {
        let configurations = CliConfigurations::new();
        let garbage = retrieve_garbage(&configurations);
        assert!(garbage.is_empty());
    }

    fn retrieve_tests_folders() -> (PathBuf, String) {
        let current_dir = env::current_dir().unwrap();

        (
            current_dir.join("tests").join("node_modules"),
            current_dir.display().to_string(),
        )
    }

    #[test]
    fn test_retrieve_all_garbage() {
        let (node_modules_location, current_dir) = retrieve_tests_folders();

        let garbage = retrieve_garbage(&CliConfigurations {
            node_modules_location,
            cjs_only: false,
            dry_run: true,
        });

        let current_dir = current_dir.as_str();

        let garbage: Vec<String> = garbage
            .iter()
            .map(|path| path.display().to_string().replace(current_dir, ""))
            .collect();

        assert_eq!(garbage, base_garbage_structure());
    }

    #[test]
    fn test_retrieve_all_with_esm_garbage() {
        let (node_modules_location, current_dir) = retrieve_tests_folders();

        let garbage = retrieve_garbage(&CliConfigurations {
            node_modules_location,
            cjs_only: true,
            dry_run: true,
        });

        let current_dir = current_dir.as_str();

        let garbage: Vec<String> = garbage
            .iter()
            .map(|path| path.display().to_string().replace(current_dir, ""))
            .collect();

        let mut expected_garbage = base_garbage_structure();
        expected_garbage.push("/tests/node_modules/ilteoood/legit.esm.js".to_owned());

        assert_eq!(garbage, expected_garbage);
    }

    #[test]
    fn test_remove_empty_dirs() {
        let configurations = CliConfigurations::new();
        remove_empty_dirs(&configurations);
    }

    #[test]
    fn test_clean() {
        let (node_modules_location, _) = retrieve_tests_folders();
        let configurations = &CliConfigurations {
            node_modules_location: node_modules_location.clone(),
            cjs_only: false,
            dry_run: true,
        };

        let garbage = retrieve_garbage(configurations);

        clean(configurations, garbage);

        assert_eq!(node_modules_location.join("@types").exists(), false);
        assert_eq!(node_modules_location.join("busboy").exists(), false);
        assert_eq!(node_modules_location.join("fastify").exists(), false);
        assert!(node_modules_location.join("ilteoood").exists());
        assert!(node_modules_location
            .join("ilteoood")
            .join("legit.esm.js")
            .exists());
        assert!(node_modules_location
            .join("ilteoood")
            .join("legit.js")
            .exists());
        assert_eq!(
            node_modules_location
                .join("ilteoood")
                .join("unlegit.min.js")
                .exists(),
            false
        );
    }
}
