//! Minify JavaScript files

use anyhow::anyhow;
use std::{fs, path::PathBuf, sync::Arc};
use swc::{config, try_with_handler, BoolConfig, BoolOrDataConfig};
use swc_common::{SourceMap, GLOBALS};
use swc_ecma_ast::EsVersion;

use crate::{configurations::CliConfigurations, glob::retrieve_glob_paths};

/// Retrieve JavaScript files from the node_modules directory
///
/// This function retrieves all JavaScript files from the node_modules directory
/// and returns them as a vector of `PathBuf`s.
fn retrieve_files_by_extension(
    configurations: &CliConfigurations,
    extension: &str,
) -> Vec<PathBuf> {
    let js_glob_path = configurations
        .node_modules_location
        .join("**")
        .join(format!("*.*{}", extension));

    let js_glob_path = js_glob_path.display();

    retrieve_glob_paths(vec![js_glob_path.to_string()])
        .into_iter()
        .filter(|path| path.is_file())
        .collect()
}

/// Build a compiler for minifying JavaScript files
///
/// This function builds a compiler for minifying JavaScript files. The compiler
/// is configured to use the latest ECMAScript version and to minify the code.
fn build_compiler() -> impl Fn(&PathBuf) -> Result<String, String> {
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

    return move |path: &PathBuf| -> Result<String, String> {
        let c = swc::Compiler::new(cm.clone());
        let output = GLOBALS.set(&Default::default(), || {
            try_with_handler(cm.clone(), Default::default(), |handler| {
                match cm.load_file(path.as_path()) {
                    Ok(fm) => c.process_js_file(fm, handler, &opts),
                    Err(err) => Err(anyhow!("failed to load file: {}: {}", path.display(), err)),
                }
            })
        });

        output
            .map(|output| output.code)
            .map_err(|error| format!("failed to process file: {}", error))
    };
}

/// Minify JavaScript files
///
/// This function takes a vector of `PathBuf`s and minifies each file. The
/// minified file is then written to the same location as the original file.
pub fn minify_js(configurations: &CliConfigurations) {
    let to_minify = retrieve_files_by_extension(configurations, "js");
    let compiler = build_compiler();

    for path in to_minify {
        let transform_output = compiler(&path);

        match transform_output {
            Ok(code) => match fs::write(&path, code) {
                Ok(_) => println!("File minified: {}", path.display()),
                Err(error) => println!("Failed to write file {}: {}", path.display(), error),
            },
            Err(error) => println!("Failed to minify file {}: {}", path.display(), error),
        }
    }
}

pub fn minify_json(configurations: &CliConfigurations) {
    let to_minify = retrieve_files_by_extension(configurations, "json");

    for path in to_minify {
        match fs::read_to_string(&path) {
            Ok(json_string) => match serde_json::from_str::<serde_json::Value>(&json_string) {
                Ok(json_reader) => match serde_json::to_string(&json_reader) {
                    Ok(minified_json_string) => match fs::write(&path, minified_json_string) {
                        Ok(_) => println!("File minified: {}", path.display()),
                        Err(error) => {
                            println!("Failed to write file {}: {}", path.display(), error)
                        }
                    },
                    Err(error) => println!("Failed to minify file {}: {}", path.display(), error),
                },
                Err(error) => println!("Failed to parse file {}: {}", path.display(), error),
            },
            Err(error) => println!("Failed to read file {}: {}", path.display(), error),
        }
    }
}

pub fn minify(configurations: &CliConfigurations) {
    minify_js(configurations);
    minify_json(configurations);
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
            compiler(&js_path).unwrap(),
            "import t from\"path\";import(\"stream\"),import.meta.resolve(\"fs\");export default function(e){return\".md\"===t.extname(e)}"
        );
    }

    #[test]
    fn test_compile_cjs() {
        let js_path = retrieve_tests_ilteoood().join("legit.js");

        let compiler = build_compiler();

        assert_eq!(
            compiler(&js_path).unwrap(),
            "!function(){let e=require(\"path\");require.resolve(\"stream\"),require(\"depd\")(\"body-parser\"),e.join(require(\"module\"))}(),module.exports=function(e){return\".md\"===path.extname(e)};"
        );
    }
}
