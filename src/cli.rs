use std::env;

use configurations::Configurations;

mod cleaner;
mod configurations;

fn main() {
    let base_directory = env::var("BASE_DIRECTORY");

    let configurations = &Configurations::new(base_directory.ok());

    let garbage_paths = cleaner::retrieve_garbage(configurations);

    cleaner::clean(configurations, garbage_paths);
}
