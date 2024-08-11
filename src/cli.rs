use std::env;

use configurations::Configurations;

mod cleaner;
mod configurations;

fn main() {
    let base_directory = env::var("BASE_DIRECTORY");

    let configurations = &Configurations::new(base_directory.ok());

    let garbage_paths = cleaner::retrieve_garbage(configurations);

    if !configurations.dry_run {
        cleaner::clean(configurations, garbage_paths);
    } else {
        println!("Dry run. These are the paths that should be removed:");
        garbage_paths.iter().for_each(|path| println!("{}", path.display()));
    }
}
