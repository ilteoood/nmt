use std::path::PathBuf;

use oxc_resolver::{ResolveOptions, Resolver};

fn resolve(path: PathBuf, specifier: String) -> String {
    let options = ResolveOptions {
        ..ResolveOptions::default()
    };

    match Resolver::new(options).resolve(path, &specifier) {
        Err(error) => format!("Error: {error}"),
        Ok(resolution) => format!("Resolved: {:?}", resolution.full_path()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve() {
        let path =
            PathBuf::from("/Users/ilteoood/Documents/git/personal/xdcc-mule/packages/server/dist");
        let specifier = "desm".to_string();
        let result = resolve(path, specifier);

        assert_eq!(result, "Resolved: foo/bar.js");
    }
}
