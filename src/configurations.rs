use std::{
    env,
    path::{Path, PathBuf},
};

pub struct Configurations {
    pub node_modules_location: PathBuf,
    pub dry_run: bool,
}

impl Configurations {
    fn retrieve_current_working_directory() -> Option<String> {
        Some(env::current_dir().unwrap().to_str().unwrap().to_string())
    }

    pub fn new(base_directory: Option<String>) -> Configurations {
        let base_directory = base_directory
            .or_else(Configurations::retrieve_current_working_directory)
            .unwrap();

        Configurations {
            node_modules_location: Path::new(&base_directory).join("node_modules"),
            dry_run: env::var("DRY_RUN").is_ok(),
        }
    }
}
