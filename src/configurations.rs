use std::{
    env,
    path::{Path, PathBuf},
};

pub struct CliConfigurations {
    pub node_modules_location: PathBuf,
    pub dry_run: bool,
    pub cjs_only: bool,
}

pub struct DockerConfigurations {
    pub cli: CliConfigurations,
    pub source_image: String,
    pub destination_image: String,
}

const BASE_DIR: &str = "BASE_DIR";
const DRY_RUN: &str = "DRY_RUN";
const CJS_ONLY: &str = "CJS_ONLY";

impl CliConfigurations {
    fn retrieve_current_working_directory() -> Option<String> {
        Some(env::current_dir().unwrap().to_str().unwrap().to_string())
    }

    pub fn from_env() -> CliConfigurations {
        let base_directory = env::var(BASE_DIR)
            .ok()
            .or_else(CliConfigurations::retrieve_current_working_directory)
            .unwrap();

        CliConfigurations {
            node_modules_location: Path::new(&base_directory).join("node_modules"),
            dry_run: env::var(DRY_RUN).is_ok(),
            cjs_only: env::var(CJS_ONLY).is_ok(),
        }
    }

    pub fn to_dockerfile_env(&self) -> String {
        let mut env = format!("ENV {}={}", BASE_DIR, self.node_modules_location.display());
        if self.dry_run {
            env += format!(
                "
ENV {}={}",
                DRY_RUN, self.dry_run
            )
            .as_str();
        }

        if self.cjs_only {
            env += format!(
                "
ENV {}={}",
                CJS_ONLY, self.cjs_only
            )
            .as_str();
        }

        env
    }
}

impl DockerConfigurations {
    pub fn from_env() -> DockerConfigurations {
        let source_image = env::var("SOURCE_IMAGE").unwrap_or(String::from("hello-world"));
        let destination_image = env::var("DESTINATION_IMAGE").unwrap_or_else(|_| {
            source_image.split(":").collect::<Vec<&str>>()[0]
                .split("@")
                .collect::<Vec<&str>>()[0]
                .to_string()
                + ":trimmed"
        });

        DockerConfigurations {
            cli: CliConfigurations::from_env(),
            source_image,
            destination_image,
        }
    }
}

#[cfg(test)]
#[serial_test::serial]
mod tests {
    use super::*;
    use std::env;

    fn clean_cli_env() {
        env::remove_var(BASE_DIR);
        env::remove_var(DRY_RUN);
        env::remove_var(CJS_ONLY);
    }

    #[test]
    fn test_cli_default_configurations() {
        clean_cli_env();

        let configurations = CliConfigurations::from_env();
        assert_eq!(
            configurations.node_modules_location,
            PathBuf::from(CliConfigurations::retrieve_current_working_directory().unwrap())
                .join("node_modules")
        );
        assert!(!configurations.dry_run);
        assert!(!configurations.cjs_only);
    }

    #[test]
    fn test_cli_configurations() {
        env::set_var(BASE_DIR, "BASE_DIR");
        env::set_var(DRY_RUN, "true");
        env::set_var(CJS_ONLY, "true");
        let configurations = CliConfigurations::from_env();
        assert_eq!(
            configurations.node_modules_location,
            PathBuf::from("BASE_DIR").join("node_modules")
        );
        assert!(configurations.dry_run);
        assert!(configurations.cjs_only);
    }

    #[test]
    fn test_cli_default_to_docker_env() {
        clean_cli_env();

        let configurations = CliConfigurations::from_env();

        assert_eq!(
            configurations.to_dockerfile_env(),
            format!(
                "ENV BASE_DIR={}/node_modules",
                CliConfigurations::retrieve_current_working_directory().unwrap()
            )
        );
    }

    #[test]
    fn test_cli_to_docker_env() {
        env::set_var(BASE_DIR, "BASE_DIR");
        env::set_var(DRY_RUN, "true");
        env::set_var(CJS_ONLY, "true");
        let configurations = CliConfigurations::from_env();

        assert_eq!(
            configurations.to_dockerfile_env(),
            "ENV BASE_DIR=BASE_DIR/node_modules\nENV DRY_RUN=true\nENV CJS_ONLY=true"
        );
    }

    #[test]
    fn test_docker_default_configurations() {
        env::remove_var("SOURCE_IMAGE");
        env::remove_var("DESTINATION_IMAGE");
        env::remove_var(BASE_DIR);
        env::remove_var(DRY_RUN);
        env::remove_var(CJS_ONLY);
        let configurations = DockerConfigurations::from_env();
        assert_eq!(
            configurations.cli.node_modules_location,
            PathBuf::from(CliConfigurations::retrieve_current_working_directory().unwrap())
                .join("node_modules")
        );
        assert_eq!(configurations.source_image, "hello-world");
        assert_eq!(configurations.destination_image, "hello-world:trimmed");
    }
}
