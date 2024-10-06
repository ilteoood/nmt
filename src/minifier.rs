//! Minify JavaScript files

use anyhow::Error;
use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_minifier::{Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::{fs, path::PathBuf};

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
fn build_minifier() -> impl Fn(&PathBuf) -> Result<String, Error> {
    let allocator = Allocator::default();

    let minifier_options = MinifierOptions::default();
    let codegen_options = CodegenOptions {
        minify: true,
        ..Default::default()
    };

    return move |path: &PathBuf| -> Result<String, Error> {
        let source_text = std::fs::read_to_string(path)?;
        let source_type = SourceType::from_path(path)?;

        let ret = Parser::new(&allocator, source_text.as_str(), source_type).parse();
        let mut program = ret.program;

        let ret = Minifier::new(minifier_options).build(&allocator, &mut program);
        Ok(CodeGenerator::new()
            .with_mangler(ret.mangler)
            .with_options(codegen_options)
            .build(&program)
            .source_text)
    };
}

/// Minify JavaScript files
///
/// This function takes a vector of `PathBuf`s and minifies each file. The
/// minified file is then written to the same location as the original file.
pub fn minify_js(configurations: &CliConfigurations) {
    let to_minify = retrieve_files_by_extension(configurations, "js");
    let minifier = build_minifier();

    for path in to_minify {
        let transform_output = minifier(&path);

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

        let minifier = build_minifier();

        assert_eq!(
            minifier(&js_path).unwrap(),
            "import path from \"path\";const stream=import(\"stream\");const fs=import.meta.resolve(\"fs\");export default function(d){return path.extname(d)===\".md\"}"
        );
    }

    #[test]
    fn test_compile_cjs() {
        let js_path = retrieve_tests_ilteoood().join("legit.js");

        let minifier = build_minifier();

        assert_eq!(
            minifier(&js_path).unwrap(),
            "(function(){const a=require(\"path\");const b=require.resolve(\"stream\");require(\"depd\")(\"body-parser\");a.join(require(\"module\"))})(),module.exports=function(a){return path.extname(a)===\".md\"};"
        );
    }
}
