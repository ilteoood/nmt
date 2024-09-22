#![allow(clippy::print_stdout)]
use std::{
    collections::{HashSet, VecDeque},
    path::PathBuf,
};

use oxc_allocator::Allocator;
use oxc_ast::{ast::Expression, Visit};
use oxc_parser::{ParseOptions, Parser};
use oxc_resolver::{ResolveOptions, Resolver};
use oxc_span::SourceType;

struct Visitor {
    modules_to_visit: HashSet<String>,
    files_to_visit: VecDeque<PathBuf>,
    paths_found: HashSet<PathBuf>,
    current_path: PathBuf,
}

impl<'a> Visitor {
    fn new(path: PathBuf) -> Self {
        Self {
            modules_to_visit: HashSet::new(),
            files_to_visit: VecDeque::from([path.join("dist").join("index.js")]),
            paths_found: HashSet::new(),
            current_path: PathBuf::new(),
        }
    }

    fn retrieve_file_path(&mut self, module: String) -> PathBuf {
        let parent_path = self.current_path.parent().unwrap();

        [
            parent_path.join(&module),
            parent_path.join(format!("{}.js", &module)),
            parent_path.join(module).join("index.js"),
        ]
        .into_iter()
        .find(|path| path.is_file())
        .unwrap()
    }

    fn add_path_to_visit(&mut self, path: PathBuf) {
        let is_new_path = self.add_path(path.clone());
        if is_new_path {
            self.files_to_visit.push_back(path);
        }
    }

    fn add_path(&mut self, path: PathBuf) -> bool {
        let path = path.canonicalize().unwrap();
        self.paths_found.insert(path.clone())
    }

    fn insert_module_to_visit(&mut self, module: String) {
        if module.ends_with(".json") {
            self.add_path(self.current_path.parent().unwrap().join(module));
        } else if module.starts_with("..") || module.starts_with(".") {
            let path = self.retrieve_file_path(module);
            self.add_path_to_visit(path);
        } else if !module.starts_with("node:") {
            self.modules_to_visit.insert(module);
        }
    }

    fn insert_first_argument(&mut self, it: &oxc_ast::ast::CallExpression<'a>) {
        if let Some(Expression::StringLiteral(lit)) = &it.arguments[0].as_expression() {
            self.insert_module_to_visit(lit.value.to_string());
        }
    }

    fn resolve_modules_to_visit(&mut self) {
        let resolver = Resolver::new(ResolveOptions::default());

        let paths_to_add: Vec<PathBuf> = self
            .modules_to_visit
            .iter()
            .map(
                |specifier| match resolver.resolve(&self.current_path, specifier) {
                    Err(_) => None,
                    Ok(resolution) => Some(resolution.full_path()),
                },
            )
            .flatten()
            .collect();

        for path in paths_to_add {
            self.add_path_to_visit(path);
        }

        self.modules_to_visit.clear();
    }

    fn run(&mut self) -> HashSet<PathBuf> {
        loop {
            match self.files_to_visit.pop_front() {
                Some(path) => {
                    self.visit_path(path);

                    self.resolve_modules_to_visit();
                }
                None => break,
            }
        }

        self.paths_found.clone()
    }

    fn visit_path(&mut self, path: PathBuf) {
        self.current_path = path;

        match std::fs::read_to_string(&self.current_path) {
            Err(error) => {
                println!("Read error: {error} at {}", self.current_path.display());
            }
            Ok(source_text) => {
                let allocator = Allocator::default();
                let source_type = SourceType::from_path(&self.current_path).unwrap();

                let ret = Parser::new(&allocator, &source_text, source_type)
                    .with_options(ParseOptions {
                        parse_regular_expression: true,
                        ..ParseOptions::default()
                    })
                    .parse();

                self.visit_program(&ret.program);
            }
        }
    }
}

impl<'a> Visit<'a> for Visitor {
    fn visit_import_declaration(&mut self, it: &oxc_ast::ast::ImportDeclaration<'a>) {
        self.insert_module_to_visit(it.source.to_string());
    }

    fn visit_call_expression(&mut self, it: &oxc_ast::ast::CallExpression<'a>) {
        match it.common_js_require() {
            Some(lit) => self.insert_module_to_visit(lit.value.to_string()),
            None => {
                if it.callee.is_specific_member_access("require", "resolve")
                    && it.callee_name() == Some("resolve")
                {
                    self.insert_first_argument(it);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    fn retrieve_tests_dir() -> PathBuf {
        let current_dir = env::current_dir().unwrap();
        current_dir.join("tests")
    }

    #[test]
    fn test_esm_specifier() {
        let path = retrieve_tests_dir()
            .join("node_modules")
            .join("ilteoood")
            .join("legit.esm.js");

        // Visitor::new().visit_path(path);

        // assert_eq!(result.unwrap(), HashSet::from(["path".to_string()]));
    }

    #[test]
    fn test_cjs_specifier() {
        let path = retrieve_tests_dir()
            .join("node_modules")
            .join("ilteoood")
            .join("legit.js");

        // Visitor::new().visit_path(path);

        /*assert_eq!(
            result.unwrap(),
            HashSet::from(["path".to_string(), "stream".to_string()])
        );*/
    }
}

#[cfg(test)]
mod resolve_tests {
    use super::*;
    use std::{env, fs};

    fn retrieve_tests_dir() -> PathBuf {
        let current_dir = env::current_dir().unwrap();
        current_dir.join("tests")
    }

    #[test]
    fn test_resolve() {
        let path =
            PathBuf::from("/Users/ilteoood/Documents/git/personal/xdcc-mule/packages/server");

        let mut visitor = Visitor::new(path);

        let result = visitor.run();

        let mut array: Vec<String> = result
            .into_iter()
            .map(|path| path.to_str().unwrap().to_owned())
            .collect();
        array.sort();

        fs::write("./test.json", format!("{:?}", array)).unwrap();

        assert_eq!(array.len(), 1001);
    }
}
