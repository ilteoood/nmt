use nmt::module_graph::Visitor;
use nmt::{cleaner, configurations::CliConfigurations, minifier};

fn main() {
    let configurations = &CliConfigurations::new();

    let module_graph = Visitor::new(configurations).run();

    let cleaner = cleaner::Cleaner::from_module_graph(configurations, &module_graph);

    if !configurations.dry_run {
        cleaner.clean();
    } else {
        println!("Dry run. These are the paths that should be kept:");
        module_graph
            .iter()
            .for_each(|path| println!("{}", path.display()));
    }

    if configurations.minify {
        minifier::minify(configurations);
    } else {
        println!("Minification skipped");
    }
}
