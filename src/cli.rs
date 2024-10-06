use nmt::module_graph::Visitor;
use nmt::{
    cleaner::Cleaner,
    configurations::{CliConfigurations, Strategy},
    minifier,
};

fn main() {
    let configurations = &CliConfigurations::new();

    let cleaner = match configurations.strategy {
        Strategy::Ast => {
            let module_graph = Visitor::new(configurations).run();

            Cleaner::from_module_graph(configurations, &module_graph)
        }
        Strategy::Static => Cleaner::from_static_garbage(configurations),
    };

    if configurations.dry_run {
        println!("Dry run. These are the paths that would be removed:");
        cleaner
            .retrieve_garbage()
            .iter()
            .for_each(|path| println!("{}", path.display()));
    } else {
        cleaner.clean();
    }

    if configurations.minify {
        minifier::minify(configurations);
    } else {
        println!("Minification skipped");
    }
}
