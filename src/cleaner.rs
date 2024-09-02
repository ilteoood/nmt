//! Cleaner-related code

use std::{fs::metadata, path::PathBuf};

use crate::configurations::CliConfigurations;
use remove_empty_subdirs::remove_empty_subdirs;

use crate::glob::retrieve_glob_paths;

/// List of glob patterns for garbage items to remove
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
    ".bin",
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
    ".airtap.yml",
    "jenkinsfile",
    "makefile",
    ".snyk",
    // generic files
    ".*ignore",
    "*eslint*",
    "*stylelint*",
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
    "gulpfile.*",
    "gruntfile.*",
    ".npm*",
    "yarn*",
];

/// List of glob patterns for garbage items to remove for ESM only
static GARBAGE_ESM_ITEMS: &[&str] = &["esm", "*.esm.js", "*.mjs"];

/// List of glob patterns for garbage items to remove for CJS only
static GARBAGE_CJS_ITEMS: &[&str] = &["cjs", "*.cjs.js", "*.cjs"];

/// Creates a closure that takes a list of garbage items and returns a vector of paths to remove
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

/// Generates a list of paths to remove based on the configuration
fn generate_garbage_paths(configurations: &CliConfigurations) -> Vec<String> {
    let mut garbage_paths: Vec<String> = vec![];

    let mut manage_path_closure = manage_path(&mut garbage_paths, configurations);

    manage_path_closure(GARBAGE_ITEMS);

    if configurations.cjs_only {
        manage_path_closure(GARBAGE_ESM_ITEMS);
    }

    if configurations.esm_only {
        manage_path_closure(GARBAGE_CJS_ITEMS);
    }

    drop(manage_path_closure);

    garbage_paths
}

/// Deletes a path
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

/// Retrieves all garbage items
pub fn retrieve_garbage(configurations: &CliConfigurations) -> Vec<PathBuf> {
    let garbage_paths = generate_garbage_paths(configurations);

    retrieve_glob_paths(garbage_paths)
}

/// Removes empty directories
fn remove_empty_dirs(configurations: &CliConfigurations) {
    match remove_empty_subdirs(&configurations.node_modules_location) {
        Ok(_) => println!("Removed empty directories"),
        Err(_) => println!("Failed to remove empty directories"),
    }
}

/// Deletes pnpm cache
fn delete_pnpm_cache(configurations: &CliConfigurations) {
    delete_path(
        configurations
            .home_location
            .join(".pnpm-state")
            .to_path_buf(),
    );
    delete_path(
        configurations
            .home_location
            .join(".local")
            .join("share")
            .join("pnpm")
            .to_path_buf(),
    );
}

/// Deletes lock files
fn delete_lock_files(configurations: &CliConfigurations) {
    delete_path(
        configurations
            .project_root_location
            .join("package-lock.json")
            .to_path_buf(),
    );
    delete_path(
        configurations
            .project_root_location
            .join("yarn.lock")
            .to_path_buf(),
    );
    delete_path(
        configurations
            .project_root_location
            .join("pnpm-lock.yaml")
            .to_path_buf(),
    );
}

/// Cleans up the node_modules directory
pub fn clean(configurations: &CliConfigurations, garbage: Vec<PathBuf>) {
    for path in garbage {
        delete_path(path);
    }
    remove_empty_dirs(configurations);
    delete_path(configurations.home_location.join(".npm").to_path_buf());
    delete_pnpm_cache(configurations);
    delete_lock_files(configurations);
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::{prelude::*, TempDir};
    use std::env;

    fn base_garbage_structure() -> Vec<String> {
        vec![
            "/node_modules/@types".to_owned(),
            "/node_modules/fastify/README.md".to_owned(),
            "/node_modules/fastify/eslint.config.ts".to_owned(),
            "/node_modules/busboy/.nvmrc".to_owned(),
            "/node_modules/busboy/.eslintrc.json".to_owned(),
            "/node_modules/ilteoood/unlegit.min.js".to_owned(),
            "/node_modules/@types/tsconfig.json".to_owned(),
        ]
    }

    #[test]
    fn test_retrieve_garbage() {
        let configurations = CliConfigurations::new();
        let garbage = retrieve_garbage(&configurations);
        assert!(garbage.is_empty());
    }

    fn retrieve_tests_folders() -> (PathBuf, String, TempDir) {
        let current_dir = env::current_dir().unwrap();
        let tests_dir = current_dir.join("tests");

        let temp = TempDir::new().unwrap().into_persistent();
        temp.copy_from(tests_dir, &["**/*"]).unwrap();

        (temp.join("node_modules"), temp.display().to_string(), temp)
    }

    #[test]
    fn test_retrieve_all_garbage() {
        let (node_modules_location, current_dir, temp) = retrieve_tests_folders();

        let garbage = retrieve_garbage(&CliConfigurations {
            node_modules_location,
            ..Default::default()
        });

        let current_dir = current_dir.as_str();

        let garbage: Vec<String> = garbage
            .iter()
            .map(|path| path.display().to_string().replace(current_dir, ""))
            .collect();

        assert_eq!(garbage, base_garbage_structure());

        temp.close().unwrap();
    }

    #[test]
    fn test_retrieve_all_with_esm_garbage() {
        let (node_modules_location, current_dir, temp) = retrieve_tests_folders();

        let garbage = retrieve_garbage(&CliConfigurations {
            node_modules_location,
            cjs_only: true,
            ..Default::default()
        });

        let current_dir = current_dir.as_str();

        let garbage: Vec<String> = garbage
            .iter()
            .map(|path| path.display().to_string().replace(current_dir, ""))
            .collect();

        let mut expected_garbage = base_garbage_structure();
        expected_garbage.push("/node_modules/ilteoood/legit.esm.js".to_owned());

        assert_eq!(garbage, expected_garbage);

        temp.close().unwrap();
    }

    #[test]
    fn test_remove_empty_dirs() {
        let configurations = CliConfigurations::new();
        remove_empty_dirs(&configurations);
    }

    #[test]
    fn test_clean() {
        let (node_modules_location, _, temp) = retrieve_tests_folders();
        let configurations = &CliConfigurations {
            node_modules_location: node_modules_location.to_path_buf(),
            ..Default::default()
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

        temp.close().unwrap();
    }
}
