use nmt::module_graph::Visitor;
use nmt::{cleaner, configurations::CliConfigurations, minifier};

fn main() {
    let configurations = &CliConfigurations::new();

    let module_graph = Visitor::new(configurations).run();

    if !configurations.dry_run {
        cleaner::clean(configurations, &module_graph);
    } else {
        println!("Dry run. These are the paths that should be kept:");
        module_graph
            .iter()
            .for_each(|path| println!("{}", path.display()));
    }

    if configurations.minify {
        minifier::minify_js(&module_graph);
    } else {
        println!("Minification skipped");
    }
}
