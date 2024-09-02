//! Configuration-related code

use std::path::{Path, PathBuf};

use clap::{command, Parser};
use dirs;

/// Configuration for the CLI

const PROJECT_ROOT_LOCATION: &str = "PROJECT_ROOT_LOCATION";
const HOME_LOCATION: &str = "HOME_LOCATION";
const DRY_RUN: &str = "DRY_RUN";
const CJS_ONLY: &str = "CJS_ONLY";
const ESM_ONLY: &str = "ESM_ONLY";
const SOURCE_IMAGE: &str = "SOURCE_IMAGE";
const DESTINATION_IMAGE: &str = "DESTINATION_IMAGE";
const MINIFY: &str = "MINIFY";
const DEFAULT_IMAGE_NAME: &str = "hello-world";
const DEFAULT_HOME_DIR: &str = "~";
const DEFAULT_ROOT_LOCATION: &str = ".";

#[derive(Debug, Parser, Default)]
#[command(version, about, long_about)]
pub struct CliConfigurations {
    /// Path to the project root
    #[arg(short, long, default_value = DEFAULT_ROOT_LOCATION, env = PROJECT_ROOT_LOCATION)]
    pub project_root_location: PathBuf,
    /// Path to the node_modules directory
    #[arg(skip)]
    pub node_modules_location: PathBuf,
    /// Path to the home directory
    #[arg(short = 'H', long, default_value = DEFAULT_HOME_DIR, env = HOME_LOCATION)]
    pub home_location: PathBuf,
    /// Whether to perform a dry run
    #[arg(short, long, default_value_t = false, env = DRY_RUN)]
    pub dry_run: bool,
    /// Whether to remove all ESM files
    #[arg(short, long, default_value_t = false, env = CJS_ONLY)]
    pub cjs_only: bool,
    /// Whether to remove all CJS files
    #[arg(short, long, default_value_t = false, env = ESM_ONLY)]
    pub esm_only: bool,
    /// Whether to minify JS files
    #[arg(short, long, default_value_t = false, env = MINIFY)]
    pub minify: bool,
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

        self.node_modules_location = self.project_root_location.join("node_modules");
    }

    /// Converts the configuration to a Dockerfile
    pub fn to_dockerfile_env(&self) -> String {
        let mut env = format!(
            "ENV {}={}",
            PROJECT_ROOT_LOCATION,
            self.project_root_location.display()
        );

        [
            (DRY_RUN, self.dry_run),
            (CJS_ONLY, self.cjs_only),
            (ESM_ONLY, self.esm_only),
            (MINIFY, self.minify),
        ]
        .iter()
        .filter(|(_, value)| *value)
        .for_each(|(env_name, value)| {
            env += format!(
                "
ENV {}={}",
                env_name, value
            )
            .as_str();
        });

        env
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
        docker_configurations.cli.post_parse();

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
        env::remove_var(CJS_ONLY);
    }

    fn clean_docker_env() {
        clean_cli_env();

        env::remove_var(SOURCE_IMAGE);
        env::remove_var(DESTINATION_IMAGE);
    }

    #[test]
    fn test_cli_default_configurations() {
        clean_cli_env();

        let configurations = CliConfigurations::new();
        assert_eq!(configurations.project_root_location, PathBuf::from("."));
        assert_eq!(
            configurations.node_modules_location,
            PathBuf::from("./node_modules")
        );
        assert!(!configurations.dry_run);
        assert!(!configurations.cjs_only);
    }

    #[test]
    fn test_cli_configurations() {
        clean_cli_env();
        env::set_var(PROJECT_ROOT_LOCATION, "PROJECT_ROOT_LOCATION");
        env::set_var(DRY_RUN, "true");
        env::set_var(CJS_ONLY, "true");
        let configurations = CliConfigurations::new();
        assert_eq!(
            configurations.project_root_location,
            PathBuf::from("PROJECT_ROOT_LOCATION")
        );
        assert_eq!(
            configurations.node_modules_location,
            PathBuf::from("PROJECT_ROOT_LOCATION/node_modules")
        );
        assert!(configurations.dry_run);
        assert!(configurations.cjs_only);
    }

    #[test]
    fn test_cli_default_to_docker_env() {
        clean_cli_env();

        let configurations = CliConfigurations::parse();

        assert_eq!(
            configurations.to_dockerfile_env(),
            "ENV PROJECT_ROOT_LOCATION=."
        );
    }

    #[test]
    fn test_cli_to_docker_env() {
        clean_docker_env();
        env::set_var(PROJECT_ROOT_LOCATION, "PROJECT_ROOT_LOCATION");
        env::set_var(DRY_RUN, "true");
        env::set_var(CJS_ONLY, "true");
        let configurations = CliConfigurations::parse();

        assert_eq!(
            configurations.to_dockerfile_env(),
            "ENV PROJECT_ROOT_LOCATION=PROJECT_ROOT_LOCATION\nENV DRY_RUN=true\nENV CJS_ONLY=true"
        );
    }

    #[test]
    fn test_docker_tag_configurations() {
        clean_docker_env();

        let source_image = format!("{}:foo", DEFAULT_IMAGE_NAME);
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
        assert_eq!(
            configurations.cli.node_modules_location,
            PathBuf::from("./node_modules")
        );
        assert_eq!(configurations.source_image, DEFAULT_IMAGE_NAME);
        assert_eq!(
            configurations.destination_image,
            format!("{DEFAULT_IMAGE_NAME}:trimmed")
        );
    }
}
