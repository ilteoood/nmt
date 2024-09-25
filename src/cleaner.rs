//! Cleaner-related code

use std::{collections::HashSet, fs, path::PathBuf};

use crate::{configurations::CliConfigurations, glob::retrieve_glob_paths};
use remove_empty_subdirs::remove_empty_subdirs;

/// Deletes a path
fn delete_path(path: PathBuf) {
    let path_location = path.display();
    println!("Removing: {}", path_location);
    let metadata = fs::metadata(&path);

    match metadata {
        Ok(metadata) => {
            let remove_result = if metadata.is_dir() {
                fs::remove_dir_all(&path)
            } else {
                fs::remove_file(&path)
            };

            match remove_result {
                Ok(_) => println!("Removed: {}", path_location),
                Err(err) => println!("Failed to remove: {}, {}", path_location, err),
            }
        }
        Err(err) => println!("Failed to remove: {}, {}", path_location, err),
    }
}

/// Removes empty directories
fn remove_empty_dirs(configurations: &CliConfigurations) {
    match remove_empty_subdirs(&configurations.project_root_location) {
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

fn retrieve_garbage(
    configurations: &CliConfigurations,
    module_graph: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    let node_modules_glob = configurations
        .project_root_location
        .join("**")
        .join("node_modules")
        .join("**");

    let package_json_filter = Some("package.json".as_ref());

    retrieve_glob_paths(vec![
        node_modules_glob.join("*").display().to_string(),
        node_modules_glob.join(".*").display().to_string(),
    ])
    .into_iter()
    .filter(|path| path.is_file())
    .filter(|path| !module_graph.contains(path))
    .filter(|path| path.file_name() != package_json_filter)
    .collect()
}

/// Cleans up the node_modules directory
pub fn clean(configurations: &CliConfigurations, module_graph: &HashSet<PathBuf>) {
    let garbage = retrieve_garbage(configurations, module_graph);
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

    fn retrieve_tests_folders() -> (PathBuf, String, TempDir) {
        let current_dir = env::current_dir().unwrap();
        let tests_dir = current_dir.join("tests");

        let temp = TempDir::new().unwrap().into_persistent();
        temp.copy_from(tests_dir, &["**/*"]).unwrap();

        (temp.join("node_modules"), temp.display().to_string(), temp)
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

        clean(
            configurations,
            &HashSet::from([legit_esm_path.clone(), legit_path.clone()]),
        );

        assert_eq!(node_modules_location.join("@types").exists(), false);
        assert_eq!(node_modules_location.join("busboy").exists(), false);
        assert_eq!(node_modules_location.join("fastify").exists(), false);
        assert!(node_modules_location.join("ilteoood").exists());
        assert!(legit_esm_path.exists());
        assert!(legit_path.exists());
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
