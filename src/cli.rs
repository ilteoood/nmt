// use nmt::{cleaner, configurations::CliConfigurations, minifier};
use nmt::{configurations::CliConfigurations, module_graph::Visitor};

fn main() {
    let configurations = &CliConfigurations::new();

    let module_graph = Visitor::new(configurations).run();

    // let garbage_paths = cleaner::retrieve_garbage(configurations);

    if !configurations.dry_run {
        // cleaner::clean(configurations, garbage_paths);
    } else {
        println!("Dry run. These are the paths that should be removed:");
        /*garbage_paths
            .iter()
            .for_each(|path| println!("{}", path.display()));
        */
    }

    if configurations.minify {
        // minifier::minify_js(configurations);
    } else {
        println!("Minification skipped");
    }
}
