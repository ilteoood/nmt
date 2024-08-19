use std::{fs, path::PathBuf, sync::Arc};

use nmt::configurations::CliConfigurations;
use swc::{config, try_with_handler, BoolConfig, BoolOrDataConfig};
use swc_common::{SourceMap, GLOBALS};
use swc_ecma_ast::EsVersion;

use crate::glob::retrieve_glob_paths;

fn retrieve_js_files(configurations: &CliConfigurations) -> Vec<PathBuf> {
    let js_glob_path = configurations.node_modules_location.join("**").join("*js");
    let js_glob_path = js_glob_path.display();

    retrieve_glob_paths(vec![js_glob_path.to_string()])
}

fn compile(path: &PathBuf, cm: &Arc<SourceMap>, opts: &config::Options) -> String {
    let c = swc::Compiler::new(cm.clone());
    let output = GLOBALS
        .set(&Default::default(), || {
            try_with_handler(cm.clone(), Default::default(), |handler| {
                let fm = cm.load_file(path.as_path()).expect("failed to load file");
                Ok(c.process_js_file(fm, handler, &opts)
                    .expect("failed to process file"))
            })
        })
        .unwrap();

    output.code
}

pub fn minify_js(configurations: &CliConfigurations) {
    let cm = Arc::<SourceMap>::default();

    let opts = config::Options {
        config: config::Config {
            minify: BoolConfig::new(Some(true)),
            jsc: config::JscConfig {
                target: Some(EsVersion::latest()),
                minify: Some(config::JsMinifyOptions {
                    compress: BoolOrDataConfig::from_bool(true),
                    mangle: BoolOrDataConfig::from_bool(true),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let to_compile = retrieve_js_files(configurations);

    for path in to_compile {
        let code = compile(&path, &cm, &opts);

        match fs::write(&path, code) {
            Ok(_) => println!("File minified: {}", path.display()),
            Err(error) => println!("Failed to write file {}: {}", path.display(), error),
        }
    }
}
