use std::{
    env,
    path::{Path, PathBuf},
};

pub struct Configurations {
    pub node_modules_location: PathBuf,
}

impl Configurations {
    fn retrieve_current_working_directory() -> Result::<String, env::VarError> {
        Ok(env::current_dir().unwrap().to_str().unwrap().to_string())
    }

    pub fn new() -> Configurations {
        let base_directory = env::var("BASE_DIRECTORY")
            .or_else(|_| Configurations::retrieve_current_working_directory())
            .unwrap();

        Configurations {
            node_modules_location: Path::new(&base_directory).join("node_modules"),
        }
    }
}
