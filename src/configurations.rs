use std::{
    env,
    path::{Path, PathBuf},
};

pub struct CliConfigurations {
    pub node_modules_location: PathBuf,
    pub dry_run: bool,
    pub cjs_only: bool,
}

impl CliConfigurations {
    fn retrieve_current_working_directory() -> Option<String> {
        Some(env::current_dir().unwrap().to_str().unwrap().to_string())
    }

    pub fn new(base_directory: Option<String>) -> CliConfigurations {
        let base_directory = base_directory
            .or_else(CliConfigurations::retrieve_current_working_directory)
            .unwrap();

        CliConfigurations {
            node_modules_location: Path::new(&base_directory).join("node_modules"),
            dry_run: env::var("DRY_RUN").is_ok(),
            cjs_only: env::var("CJS_ONLY").is_ok(),
        }
    }
}
