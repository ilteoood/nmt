use clap::Parser;
use nmt::configurations::CliConfigurations;

mod cleaner;

fn main() {
    let configurations = &CliConfigurations::parse();

    let garbage_paths = cleaner::retrieve_garbage(configurations);

    if !configurations.dry_run {
        cleaner::clean(configurations, garbage_paths);
    } else {
        println!("Dry run. These are the paths that should be removed:");
        garbage_paths
            .iter()
            .for_each(|path| println!("{}", path.display()));
    }
}
