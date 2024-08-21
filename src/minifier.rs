use std::{fs, path::PathBuf, sync::Arc};

use crate::configurations::CliConfigurations;
use swc::{config, try_with_handler, BoolConfig, BoolOrDataConfig};
use swc_common::{SourceMap, GLOBALS};
use swc_ecma_ast::EsVersion;

use crate::glob::retrieve_glob_paths;

fn retrieve_js_files(configurations: &CliConfigurations) -> Vec<PathBuf> {
    let js_glob_path = configurations
        .node_modules_location
        .join("**")
        .join("*.*js");
    let js_glob_path = js_glob_path.display();

    retrieve_glob_paths(vec![js_glob_path.to_string()])
        .into_iter()
        .filter(|path| path.is_file())
        .collect()
}

fn build_compiler() -> impl Fn(&PathBuf) -> String {
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

    return move |path: &PathBuf| -> String {
        let c = swc::Compiler::new(cm.clone());
        let output = GLOBALS
            .set(&Default::default(), || {
                try_with_handler(cm.clone(), Default::default(), |handler| {
                    let fm = cm
                        .load_file(path.as_path())
                        .unwrap_or_else(|_| panic!("failed to load file: {}", path.display()));
                    Ok(c.process_js_file(fm, handler, &opts)
                        .expect("failed to process file"))
                })
            })
            .unwrap();

        output.code
    };
}

pub fn minify_js(configurations: &CliConfigurations) {
    let to_compile = retrieve_js_files(configurations);
    let compiler = build_compiler();

    for path in to_compile {
        let code = compiler(&path);

        match fs::write(&path, code) {
            Ok(_) => println!("File minified: {}", path.display()),
            Err(error) => println!("Failed to write file {}: {}", path.display(), error),
        }
    }
}

#[cfg(test)]
mod tests_retrieve_js_files {
    use std::env;

    use serial_test::serial;

    use super::*;

    fn retrieve_tests_ilteoood() -> PathBuf {
        PathBuf::from(env::current_dir().unwrap())
            .join("tests")
            .join("node_modules")
            .join("ilteoood")
    }

    #[test]
    #[serial(fs)]
    fn test_retrieve_js_files() {
        let js_paths = retrieve_tests_ilteoood();

        assert_eq!(
            retrieve_js_files(&CliConfigurations {
                node_modules_location: PathBuf::from(&js_paths),
                ..Default::default()
            }),
            vec![
                PathBuf::from(&js_paths).join("legit.esm.js"),
                PathBuf::from(&js_paths).join("legit.js"),
                PathBuf::from(&js_paths).join("unlegit.min.js"),
            ]
        );
    }
}

#[cfg(test)]
mod tests_compile {
    use std::env;

    use super::*;

    fn retrieve_tests_ilteoood() -> PathBuf {
        PathBuf::from(env::current_dir().unwrap())
            .join("tests")
            .join("node_modules")
            .join("ilteoood")
    }

    #[test]
    fn test_compile_esm() {
        let js_path = retrieve_tests_ilteoood().join("legit.esm.js");

        let compiler = build_compiler();

        assert_eq!(
            compiler(&js_path),
            "import t from\"path\";export default function(e){return\".md\"===t.extname(e)}"
        );
    }

    #[test]
    fn test_compile_cjs() {
        let js_path = retrieve_tests_ilteoood().join("legit.js");

        let compiler = build_compiler();

        assert_eq!(
            compiler(&js_path),
            "const e=require(\"path\");module.exports=function(t){return\".md\"===e.extname(t)};"
        );
    }
}
