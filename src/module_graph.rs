#![allow(clippy::print_stdout)]
use std::{
    collections::{HashSet, VecDeque},
    path::PathBuf,
};

use oxc_allocator::Allocator;
use oxc_ast::{ast::Expression, Visit};
use oxc_parser::{ParseOptions, Parser};
use oxc_resolver::{ResolveOptions, Resolver};
use oxc_span::{ModuleKind, SourceType};

use crate::configurations::CliConfigurations;

pub struct Visitor {
    modules_to_visit: HashSet<String>,
    files_to_visit: VecDeque<PathBuf>,
    paths_found: HashSet<PathBuf>,
    current_path: PathBuf,
    resolver: Resolver,
}

impl<'a> Visitor {
    pub fn new(configurations: &CliConfigurations) -> Self {
        Self {
            modules_to_visit: HashSet::new(),
            files_to_visit: VecDeque::from([configurations.entry_point_location.clone()]),
            paths_found: HashSet::from([configurations.entry_point_location.clone()]),
            current_path: PathBuf::new(),
            resolver: Resolver::new(ResolveOptions {
                condition_names: match SourceType::from_path(&configurations.entry_point_location) {
                    Ok(source_type) => match source_type.module_kind() {
                        ModuleKind::Script => vec!["node".to_owned(), "require".to_owned()],
                        ModuleKind::Module => vec!["node".to_owned(), "import".to_owned()],
                        _ => vec![],
                    },
                    Err(_) => vec![],
                },
                ..Default::default()
            }),
        }
    }

    fn retrieve_file_path(&mut self, module: String) -> Option<PathBuf> {
        let parent_path = self.current_path.parent().unwrap();

        [
            parent_path.join(&module),
            parent_path.join(format!("{}.js", &module)),
            parent_path.join(module).join("index.js"),
        ]
        .into_iter()
        .find(|path| path.is_file())
    }

    fn add_path_to_visit(&mut self, path: PathBuf) {
        let is_new_path = self.add_path(path.clone());
        if is_new_path {
            self.files_to_visit.push_back(path);
        }
    }

    fn add_path(&mut self, path: PathBuf) -> bool {
        match path.canonicalize() {
            Ok(path) => self.paths_found.insert(path),
            Err(err) => {
                println!("Error while processing {}: {}", path.display(), err);
                false
            }
        }
    }

    fn insert_module_to_visit(&mut self, module: String) {
        if module.ends_with(".json") || module.ends_with(".node") {
            self.add_path(self.current_path.parent().unwrap().join(module));
        } else if module.starts_with("..") || module.starts_with(".") {
            if let Some(path) = self.retrieve_file_path(module) {
                self.add_path_to_visit(path);
            }
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
        let paths_to_add: Vec<PathBuf> = self
            .modules_to_visit
            .iter()
            .filter_map(
                |specifier| match self.resolver.resolve(&self.current_path, specifier) {
                    Err(_) => None,
                    Ok(resolution) => Some(resolution.full_path()),
                },
            )
            .collect();

        for path in paths_to_add {
            self.add_path_to_visit(path);
        }

        self.modules_to_visit.clear();
    }

    pub fn run(&mut self) -> HashSet<PathBuf> {
        while let Some(path) = self.files_to_visit.pop_front() {
            self.visit_path(path);

            self.resolve_modules_to_visit();
        }

        self.paths_found.drain().collect()
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

    fn deep_call_expression(&mut self, it: &oxc_ast::ast::CallExpression<'a>) {
        self.visit_expression(it.callee.get_inner_expression());
        self.visit_arguments(&it.arguments);
    }
}

impl<'a> Visit<'a> for Visitor {
    fn visit_import_declaration(&mut self, it: &oxc_ast::ast::ImportDeclaration<'a>) {
        self.insert_module_to_visit(it.source.to_string());
    }

    fn visit_call_expression(&mut self, it: &oxc_ast::ast::CallExpression<'a>) {
        match it.common_js_require() {
            Some(lit) => self.insert_module_to_visit(lit.value.to_string()),
            None => match &it.callee {
                Expression::StaticMemberExpression(static_member_expression) => {
                    if it.callee.is_specific_member_access("require", "resolve") {
                        self.insert_first_argument(it);
                    } else if let Expression::MetaProperty(meta_property) =
                        &static_member_expression.object
                    {
                        if meta_property.meta.name.as_str() == "import"
                            && meta_property.property.name.as_str() == "meta"
                            && it.callee_name() == Some("resolve")
                        {
                            self.insert_first_argument(it);
                        }
                    } else {
                        self.deep_call_expression(it);
                    }
                }
                _ => self.deep_call_expression(it),
            },
        }
    }

    fn visit_export_named_declaration(&mut self, it: &oxc_ast::ast::ExportNamedDeclaration<'a>) {
        if let Some(source) = it.source.as_ref() {
            self.insert_module_to_visit(source.to_string());
        }
    }

    fn visit_export_all_declaration(&mut self, it: &oxc_ast::ast::ExportAllDeclaration<'a>) {
        self.insert_module_to_visit(it.source.to_string());
    }

    fn visit_import_expression(&mut self, it: &oxc_ast::ast::ImportExpression<'a>) {
        if let oxc_ast::ast::Expression::StringLiteral(source_lit) = &it.source {
            self.insert_module_to_visit(source_lit.value.to_string());
        }
    }
}

#[cfg(test)]
mod specifier_tests {
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

        let mut visitor = Visitor::new(&CliConfigurations {
            entry_point_location: path.clone(),
            ..Default::default()
        });
        visitor.visit_path(path);

        assert_eq!(
            visitor.modules_to_visit,
            HashSet::from(["path".to_string(), "stream".to_string(), "fs".to_string()])
        );
    }

    #[test]
    fn test_esm_export() {
        let path = retrieve_tests_dir()
            .join("node_modules")
            .join("ilteoood")
            .join("unlegit.min.js");

        let mut visitor = Visitor::new(&CliConfigurations {
            entry_point_location: path.clone(),
            ..Default::default()
        });
        visitor.visit_path(path);

        assert_eq!(
            visitor.modules_to_visit,
            HashSet::from(["fastify".to_string(), "stream".to_string()])
        );
    }

    #[test]
    fn test_cjs_specifier() {
        let path = retrieve_tests_dir()
            .join("node_modules")
            .join("ilteoood")
            .join("legit.js");

        let mut visitor = Visitor::new(&CliConfigurations {
            entry_point_location: path.clone(),
            ..Default::default()
        });
        visitor.visit_path(path);

        assert_eq!(
            visitor.modules_to_visit,
            HashSet::from([
                "path".to_owned(),
                "stream".to_owned(),
                "module".to_owned(),
                "depd".to_owned()
            ])
        );
    }
}

#[cfg(test)]
mod resolve_tests {
    use super::*;
    use std::env;

    fn retrieve_tests_dir() -> PathBuf {
        let current_dir = env::current_dir().unwrap();
        current_dir.join("tests")
    }

    #[test]
    fn test_resolve() {
        let tests_dir = retrieve_tests_dir();
        let path = tests_dir.join("index.js");

        let mut visitor = Visitor::new(&CliConfigurations {
            entry_point_location: path.clone(),
            ..Default::default()
        });

        let result = visitor.run();

        assert_eq!(
            result,
            HashSet::from([
                path,
                tests_dir
                    .join("node_modules")
                    .join("ilteoood")
                    .join("legit.js")
            ])
        );
    }
}
