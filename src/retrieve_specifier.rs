#![allow(clippy::print_stdout)]
use std::{collections::HashSet, path::PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::{ast::Expression, Visit};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

struct Visitor {
    modules_to_visit: HashSet<String>,
    modules_visited: HashSet<String>,
}

impl<'a> Visitor {
    fn new() -> Self {
        Self {
            modules_to_visit: HashSet::new(),
            modules_visited: HashSet::new(),
        }
    }

    fn insert_module_to_visit(&mut self, module: String) {
        if !module.starts_with("node:") && !self.modules_visited.contains(&module) {
            self.modules_to_visit.insert(module);
        }
    }

    fn insert_first_argument(&mut self, it: &oxc_ast::ast::CallExpression<'a>) {
        if let Some(Expression::StringLiteral(lit)) = &it.arguments[0].as_expression() {
            self.insert_module_to_visit(lit.value.to_string());
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

fn specifier(path: &PathBuf) -> Result<HashSet<String>, ()> {
    let source_text = std::fs::read_to_string(path).unwrap();
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();

    let ret = Parser::new(&allocator, &source_text, source_type)
        .with_options(ParseOptions {
            parse_regular_expression: true,
            ..ParseOptions::default()
        })
        .parse();

    let mut visitor = Visitor::new();

    visitor.visit_program(&ret.program);

    Ok(visitor.modules_to_visit)
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

        let result = specifier(&path);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), HashSet::from(["path".to_string()]));
    }

    #[test]
    fn test_cjs_specifier() {
        let path = retrieve_tests_dir()
            .join("node_modules")
            .join("ilteoood")
            .join("legit.js");

        let result = specifier(&path);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            HashSet::from(["path".to_string(), "stream".to_string()])
        );
    }
}
