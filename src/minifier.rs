//! Minify JavaScript files

use std::{collections::HashSet, fs, path::PathBuf, sync::Arc};

use swc::{config, try_with_handler, BoolConfig, BoolOrDataConfig};
use swc_common::{SourceMap, GLOBALS};
use swc_ecma_ast::EsVersion;
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
                let fm = cm
                    .load_file(path.as_path())
                    .unwrap_or_else(|_| panic!("failed to load file: {}", path.display()));

                c.process_js_file(fm, handler, &opts)
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
pub fn minify_js(module_graph: &HashSet<PathBuf>) {
    let compiler = build_compiler();

    for path in module_graph.iter() {
        let transform_output = compiler(path);

        match transform_output {
            Ok(code) => match fs::write(path, code) {
                Ok(_) => println!("File minified: {}", path.display()),
                Err(error) => println!("Failed to write file {}: {}", path.display(), error),
            },
            Err(error) => println!("Failed to minify file {}: {}", path.display(), error),
        }
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
            compiler(&js_path).unwrap(),
            "import t from\"path\";import(\"stream\");export default function(r){return\".md\"===t.extname(r)}"
        );
    }

    #[test]
    fn test_compile_cjs() {
        let js_path = retrieve_tests_ilteoood().join("legit.js");

        let compiler = build_compiler();

        assert_eq!(
            compiler(&js_path).unwrap(),
            "const e=require(\"path\");require.resolve(\"stream\"),module.exports=function(r){return\".md\"===e.extname(r)};"
        );
    }
}
