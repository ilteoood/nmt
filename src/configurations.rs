use std::path::{Path, PathBuf};

use clap::{command, Parser};
use dirs;

const NODE_MODULES_LOCATION: &str = "NODE_MODULES_LOCATION";
const NPM_CACHE_LOCATION: &str = "NPM_CACHE_LOCATION";
const DRY_RUN: &str = "DRY_RUN";
const CJS_ONLY: &str = "CJS_ONLY";
const ESM_ONLY: &str = "ESM_ONLY";
const SOURCE_IMAGE: &str = "SOURCE_IMAGE";
const DESTINATION_IMAGE: &str = "DESTINATION_IMAGE";
const MINIFY: &str = "MINIFY";
const DEFAULT_IMAGE_NAME: &str = "hello-world";
const DEFAULT_CACHE_DIR: &str = "~/.npm";

#[derive(Debug, Parser, Default)]
#[command(version, about, long_about)]
pub struct CliConfigurations {
    /// Path to node_modules
    #[arg(
        short,
        long,
        default_value = "./node_modules",
        env = NODE_MODULES_LOCATION
    )]
    pub node_modules_location: PathBuf,
    #[arg(
        short='N',
        long,
        default_value = DEFAULT_CACHE_DIR,
        env = NPM_CACHE_LOCATION
    )]
    pub npm_cache_location: PathBuf,
    /// Dry run, will not remove files but will print them
    #[arg(short, long, default_value_t = false, env = DRY_RUN)]
    pub dry_run: bool,
    /// Removes every ESM file
    #[arg(short, long, default_value_t = false, env = CJS_ONLY)]
    pub cjs_only: bool,
    /// Removes every CJS file
    #[arg(short, long, default_value_t = false, env = ESM_ONLY)]
    pub esm_only: bool,
    /// Minify JS files
    #[arg(short, long, default_value_t = false, env = MINIFY)]
    pub minify: bool,
}

#[derive(Debug, Parser, Default)]
#[command(version, about, long_about)]
pub struct DockerConfigurations {
    #[command(flatten)]
    pub cli: CliConfigurations,
    #[arg(short, long, default_value_t = DEFAULT_IMAGE_NAME.to_string(), env = SOURCE_IMAGE)]
    pub source_image: String,
    #[arg(short='D', long, default_value = "", env = DESTINATION_IMAGE)]
    pub destination_image: String,
}

impl CliConfigurations {
    pub fn new() -> Self {
        let mut parsed = Self::parse();

        if parsed.npm_cache_location.display().to_string() == DEFAULT_CACHE_DIR {
            parsed.npm_cache_location = dirs::home_dir()
                .or(Some(Path::new("./").to_path_buf()))
                .unwrap()
                .join(".npm")
        }

        parsed
    }

    pub fn to_dockerfile_env(&self) -> String {
        let mut env = format!(
            "ENV {}={}",
            NODE_MODULES_LOCATION,
            self.node_modules_location.display()
        );

        vec![
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
    pub fn default_destination_image(&mut self) {
        if self.destination_image.is_empty() {
            self.destination_image = self.source_image.split(":").collect::<Vec<&str>>()[0]
                .split("@")
                .collect::<Vec<&str>>()[0]
                .to_string()
                + ":trimmed";
        }
    }

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
        env::remove_var(NODE_MODULES_LOCATION);
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

        let configurations = CliConfigurations::parse();
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
        env::set_var(NODE_MODULES_LOCATION, "NODE_MODULES_LOCATION");
        env::set_var(DRY_RUN, "true");
        env::set_var(CJS_ONLY, "true");
        let configurations = CliConfigurations::parse();
        assert_eq!(
            configurations.node_modules_location,
            PathBuf::from("NODE_MODULES_LOCATION")
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
            "ENV NODE_MODULES_LOCATION=./node_modules"
        );
    }

    #[test]
    fn test_cli_to_docker_env() {
        clean_docker_env();
        env::set_var(NODE_MODULES_LOCATION, "NODE_MODULES_LOCATION");
        env::set_var(DRY_RUN, "true");
        env::set_var(CJS_ONLY, "true");
        let configurations = CliConfigurations::parse();

        assert_eq!(
            configurations.to_dockerfile_env(),
            "ENV NODE_MODULES_LOCATION=NODE_MODULES_LOCATION\nENV DRY_RUN=true\nENV CJS_ONLY=true"
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
