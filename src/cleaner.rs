//! Cleaner-related code

use std::{collections::HashSet, fs, path::PathBuf};

use crate::{configurations::Cli, glob::retrieve_glob_paths};
use remove_empty_subdirs::remove_empty_subdirs;

/// List of glob patterns for garbage items to remove
static STATIC_GARBAGE_ITEMS: &[&str] = &[
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

pub struct Cleaner<'a> {
    garbage: Vec<PathBuf>,
    configurations: &'a Cli,
}

impl<'a> Cleaner<'a> {
    pub fn from_module_graph(configurations: &'a Cli, module_graph: &HashSet<PathBuf>) -> Self {
        let node_modules_glob = configurations
            .project_root_location
            .join("**")
            .join("node_modules")
            .join("**");

        let package_json_filter = Some("package.json".as_ref());

        let garbage = retrieve_glob_paths(vec![
            node_modules_glob.join("*").display().to_string(),
            node_modules_glob.join(".*").display().to_string(),
        ])
        .into_iter()
        .filter(|path| path.is_file())
        .filter(|path| !module_graph.contains(path))
        .filter(|path| path.file_name() != package_json_filter)
        .collect();

        Cleaner {
            garbage,
            configurations,
        }
    }

    pub fn from_static_garbage(configurations: &'a Cli) -> Self {
        let mut garbage_glob = Vec::new();

        for garbage_item in STATIC_GARBAGE_ITEMS {
            let garbage_path = configurations
                .node_modules_location
                .join("**")
                .join(garbage_item);

            garbage_glob.push(garbage_path.display().to_string());
        }

        Cleaner {
            garbage: retrieve_glob_paths(garbage_glob),
            configurations,
        }
    }

    pub fn retrieve_garbage(&self) -> &Vec<PathBuf> {
        &self.garbage
    }

    /// Deletes a path
    fn delete_path(path: &PathBuf) {
        let path_location = path.display();
        println!("Removing: {path_location}");
        let metadata = fs::metadata(path);

        match metadata {
            Ok(metadata) => {
                let remove_result = if metadata.is_dir() {
                    fs::remove_dir_all(path)
                } else {
                    fs::remove_file(path)
                };

                match remove_result {
                    Ok(()) => println!("Removed: {path_location}"),
                    Err(err) => println!("Failed to remove: {path_location}, {err}"),
                }
            }
            Err(err) => println!("Failed to remove: {path_location}, {err}"),
        }
    }

    /// Removes empty directories
    fn remove_empty_dirs(&self) {
        match remove_empty_subdirs(&self.configurations.project_root_location) {
            Ok(()) => println!("Removed empty directories"),
            Err(_) => println!("Failed to remove empty directories"),
        }
    }

    /// Deletes pnpm cache
    fn delete_pnpm_cache(&self) {
        Self::delete_path(
            &self
                .configurations
                .home_location
                .join(".pnpm-state")
                .clone(),
        );
        Self::delete_path(
            &self
                .configurations
                .home_location
                .join(".local")
                .join("share")
                .join("pnpm")
                .clone(),
        );
    }

    /// Deletes lock files
    fn delete_lock_files(self) {
        Self::delete_path(
            &self
                .configurations
                .project_root_location
                .join("package-lock.json")
                .clone(),
        );
        Self::delete_path(
            &self
                .configurations
                .project_root_location
                .join("yarn.lock")
                .clone(),
        );
        Self::delete_path(
            &self
                .configurations
                .project_root_location
                .join("pnpm-lock.yaml")
                .clone(),
        );
    }

    /// Cleans up the `node_modules` directory
    pub fn clean(self) {
        for path in &self.garbage {
            Self::delete_path(path);
        }
        self.remove_empty_dirs();
        Self::delete_path(&self.configurations.home_location.join(".npm").clone());
        self.delete_pnpm_cache();
        self.delete_lock_files();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::{prelude::*, TempDir};
    use std::env;

    fn retrieve_tests_folders() -> (PathBuf, String, TempDir) {
        let current_dir = env::current_dir().unwrap();
        let tests_dir = current_dir.join("tests");

        let temp = TempDir::new().unwrap().into_persistent();
        temp.copy_from(tests_dir, &["**/*"]).unwrap();

        (temp.join("node_modules"), temp.display().to_string(), temp)
    }

    #[test]
    fn test_remove_empty_dirs() {
        let configurations = &Cli {
            entry_point_location: vec!["tests/index.js".into()],
            ..Default::default()
        };
        Cleaner::from_module_graph(configurations, &HashSet::new()).remove_empty_dirs();
    }

    #[test]
    fn test_clean() {
        let (node_modules_location, _, temp) = retrieve_tests_folders();
        let configurations = &Cli {
            project_root_location: temp.to_path_buf(),
            ..Default::default()
        };

        let legit_esm_path = node_modules_location
            .join("ilteoood")
            .join("legit.esm.js")
            .canonicalize()
            .unwrap();
        let legit_path = node_modules_location
            .join("ilteoood")
            .join("legit.js")
            .canonicalize()
            .unwrap();

        let cleaner = Cleaner::from_module_graph(
            configurations,
            &HashSet::from([legit_esm_path.clone(), legit_path.clone()]),
        );

        cleaner.clean();

        assert!(!node_modules_location.join("@types").exists());
        assert!(!node_modules_location.join("busboy").exists());
        assert!(!node_modules_location.join("fastify").exists());
        assert!(node_modules_location.join("ilteoood").exists());
        assert!(legit_esm_path.exists());
        assert!(legit_path.exists());
        assert!(!node_modules_location
            .join("ilteoood")
            .join("unlegit.min.js")
            .exists(),);

        temp.close().unwrap();
    }
}
