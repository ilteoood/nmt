//! Configuration-related code

use std::path::{Path, PathBuf};

use clap::{command, Parser};
use dirs;

use crate::glob::retrieve_glob_paths;

const PROJECT_ROOT_LOCATION: &str = "PROJECT_ROOT_LOCATION";
const ENTRY_POINT_LOCATION: &str = "ENTRY_POINT_LOCATION";
const KEEP: &str = "KEEP";
const HOME_LOCATION: &str = "HOME_LOCATION";
const DRY_RUN: &str = "DRY_RUN";
const SOURCE_IMAGE: &str = "SOURCE_IMAGE";
const DESTINATION_IMAGE: &str = "DESTINATION_IMAGE";
const MINIFY: &str = "MINIFY";
const DEFAULT_IMAGE_NAME: &str = "hello-world";
const DEFAULT_HOME_DIR: &str = "~";
const DEFAULT_ROOT_LOCATION: &str = ".";
const DEFAULT_ENTRY_POINT_LOCATION: &str = "dist/index.js";

/// Configuration for the CLI
#[derive(Debug, Parser, Default)]
#[command(version, about, long_about)]
pub struct CliConfigurations {
    /// Path to the project root
    #[arg(short, long, default_value = DEFAULT_ROOT_LOCATION, env = PROJECT_ROOT_LOCATION)]
    pub project_root_location: PathBuf,
    /// Path to the application's entry point
    #[arg(short, long, default_value = DEFAULT_ENTRY_POINT_LOCATION, env = ENTRY_POINT_LOCATION)]
    pub entry_point_location: PathBuf,
    /// Path to the home directory
    #[arg(short = 'H', long, default_value = DEFAULT_HOME_DIR, env = HOME_LOCATION)]
    pub home_location: PathBuf,
    /// Whether to perform a dry run
    #[arg(short, long, default_value_t = false, env = DRY_RUN)]
    pub dry_run: bool,
    /// Whether to minify JS files
    #[arg(short, long, default_value_t = false, env = MINIFY)]
    pub minify: bool,
    /// A list of files to ignore
    #[arg(short, long, env = KEEP, value_delimiter = ',')]
    pub keep: Option<Vec<String>>,
}

/// Configuration for the Docker image
#[derive(Debug, Parser, Default)]
#[command(version, about, long_about)]
pub struct DockerConfigurations {
    #[command(flatten)]
    pub cli: CliConfigurations,
    /// The source image
    #[arg(short, long, default_value = DEFAULT_IMAGE_NAME, env = SOURCE_IMAGE)]
    pub source_image: String,
    /// The destination image
    #[arg(short = 'D', long, default_value = "", env = DESTINATION_IMAGE)]
    pub destination_image: String,
}

impl CliConfigurations {
    /// Returns a new configuration
    pub fn new() -> Self {
        let mut parsed = Self::parse();

        parsed.post_parse();

        parsed
    }

    /// Performs post-parsing work
    pub fn post_parse(&mut self) {
        if self.home_location.display().to_string() == DEFAULT_HOME_DIR {
            self.home_location =
                dirs::home_dir().unwrap_or(Path::new(DEFAULT_ROOT_LOCATION).to_path_buf())
        }

        self.entry_point_location = self
            .project_root_location
            .join(&self.entry_point_location)
            .canonicalize()
            .expect("Failed to canonicalize entry point location");
    }

    pub fn keep_files(&self) -> Vec<PathBuf> {
        let globs: Vec<String> = self
            .keep
            .as_ref()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|keep_pattern| {
                self.project_root_location
                    .join(keep_pattern)
                    .display()
                    .to_string()
            })
            .collect();

        retrieve_glob_paths(globs)
            .into_iter()
            .filter(|path| path.is_file())
            .collect()
    }

    /// Converts the configuration to a Dockerfile
    pub fn to_dockerfile_env(&self) -> String {
        let mut env = "".to_owned();

        [
            (PROJECT_ROOT_LOCATION, self.project_root_location.display()),
            (ENTRY_POINT_LOCATION, self.entry_point_location.display()),
            (HOME_LOCATION, self.home_location.display()),
        ]
        .iter()
        .for_each(|(env_name, value)| {
            env += format!(
                "ENV {}={}
",
                env_name, value
            )
            .as_str();
        });

        [(DRY_RUN, self.dry_run), (MINIFY, self.minify)]
            .iter()
            .filter(|(_, value)| *value)
            .for_each(|(env_name, value)| {
                env += format!(
                    "ENV {}={}
",
                    env_name, value
                )
                .as_str();
            });

        if let Some(keep) = &self.keep {
            env += format!("ENV {}={}", KEEP, keep.join(",")).as_str();
        }

        env.trim_end().to_owned()
    }
}

impl DockerConfigurations {
    /// Sets the default destination image
    pub fn default_destination_image(&mut self) {
        if self.destination_image.is_empty() {
            self.destination_image = self.source_image.split(":").collect::<Vec<&str>>()[0]
                .split("@")
                .collect::<Vec<&str>>()[0]
                .to_string()
                + ":trimmed";
        }
    }

    /// Returns a new configuration
    pub fn new() -> Self {
        let mut docker_configurations = Self::parse();

        docker_configurations.default_destination_image();

        docker_configurations
    }
}

#[cfg(test)]
#[serial_test::serial]
mod tests {
    use super::*;
    use std::env;

    fn clean_cli_env() {
        env::remove_var(PROJECT_ROOT_LOCATION);
        env::remove_var(DRY_RUN);
        env::remove_var(ENTRY_POINT_LOCATION);
        env::remove_var(KEEP);
    }

    fn clean_docker_env() {
        clean_cli_env();

        env::remove_var(SOURCE_IMAGE);
        env::remove_var(DESTINATION_IMAGE);
    }

    #[test]
    fn test_cli_configurations() {
        clean_cli_env();
        env::set_var(DRY_RUN, "true");
        env::set_var(ENTRY_POINT_LOCATION, "tests/index.js");
        let configurations = CliConfigurations::new();
        assert_eq!(configurations.project_root_location, PathBuf::from("."));
        assert_eq!(
            configurations.entry_point_location,
            PathBuf::from("./tests/index.js").canonicalize().unwrap()
        );
        assert!(configurations.dry_run);
    }

    #[test]
    fn test_cli_default_to_docker_env() {
        clean_cli_env();

        let configurations = CliConfigurations::parse();

        assert_eq!(
            configurations.to_dockerfile_env(),
            "ENV PROJECT_ROOT_LOCATION=.\nENV ENTRY_POINT_LOCATION=dist/index.js\nENV HOME_LOCATION=~"
        );
    }

    #[test]
    fn test_cli_to_docker_env() {
        clean_cli_env();
        env::set_var(PROJECT_ROOT_LOCATION, "PROJECT_ROOT_LOCATION");
        env::set_var(DRY_RUN, "true");
        env::set_var(KEEP, "path/1,path/2");
        let configurations = CliConfigurations::parse();

        assert_eq!(
            configurations.to_dockerfile_env(),
            "ENV PROJECT_ROOT_LOCATION=PROJECT_ROOT_LOCATION\nENV ENTRY_POINT_LOCATION=dist/index.js\nENV HOME_LOCATION=~\nENV DRY_RUN=true\nENV KEEP=path/1,path/2"
        );
    }

    #[test]
    fn test_cli_keep() {
        clean_cli_env();
        env::set_var(KEEP, "path/1,path/2");
        let configurations = CliConfigurations::parse();

        assert_eq!(
            configurations.keep,
            Some(vec!["path/1".to_owned(), "path/2".to_owned()])
        );
    }

    #[test]
    fn test_docker_tag_configurations() {
        clean_docker_env();

        let source_image = format!("{}:foo", DEFAULT_IMAGE_NAME);
        let source_image = source_image.as_str();

        env::set_var(SOURCE_IMAGE, source_image);
        env::set_var(ENTRY_POINT_LOCATION, "tests/index.js");

        let configurations = DockerConfigurations::new();

        assert_eq!(configurations.source_image, source_image);
        assert_eq!(
            configurations.destination_image,
            format!("{DEFAULT_IMAGE_NAME}:trimmed")
        );
    }

    #[test]
    fn test_docker_sha_configurations() {
        clean_docker_env();

        let source_image = format!(
            "{}:@sha256:c34ce3c1fcc0c7431e1392cc3abd0dfe2192ffea1898d5250f199d3ac8d8720f",
            DEFAULT_IMAGE_NAME
        );
        let source_image = source_image.as_str();

        env::set_var(SOURCE_IMAGE, source_image);

        let configurations = DockerConfigurations::new();

        assert_eq!(configurations.source_image, source_image);
        assert_eq!(
            configurations.destination_image,
            format!("{DEFAULT_IMAGE_NAME}:trimmed")
        );
    }

    #[test]
    fn test_docker_default_configurations() {
        clean_docker_env();

        let configurations = DockerConfigurations::new();
        assert_eq!(configurations.cli.project_root_location, PathBuf::from("."));
        assert_eq!(
            configurations.cli.entry_point_location,
            PathBuf::from("dist/index.js")
        );
        assert_eq!(configurations.source_image, DEFAULT_IMAGE_NAME);
        assert_eq!(
            configurations.destination_image,
            format!("{DEFAULT_IMAGE_NAME}:trimmed")
        );
    }
}
