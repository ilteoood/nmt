use std::env;

use configurations::Configurations;

mod cleaner;
mod configurations;

fn main() {
    let base_directory = env::var("BASE_DIRECTORY");

    cleaner::clean(&Configurations::new(base_directory.ok()));
}
