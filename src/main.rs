use std::{env, path::Path};

use configurations::Configurations;

mod cleaner;
mod configurations;

fn main() {
    let configurations = Configurations {
        node_modules_location: Path::new(&env::current_dir().unwrap()).join("node_modules"),
    };

    cleaner::clean(&configurations);
}
