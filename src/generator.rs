use crate::metadata::ExtractedDeps;
use std::fs;
use std::io;
use std::path::Path;

const CACHE_DIR: &str = ".workspace-cache";

pub fn generate_minimal_workspace(deps: &ExtractedDeps, workspace_root: &Path) -> io::Result<()> {
    let cache_dir = workspace_root.join(CACHE_DIR);
    let src_dir = cache_dir.join("src");

    fs::create_dir_all(&src_dir)?;

    generate_cargo_toml(&cache_dir, deps)?;
    generate_dummy_main(&src_dir)?;
    copy_lockfile(workspace_root, &cache_dir)?;

    Ok(())
}

fn generate_cargo_toml(cache_dir: &Path, deps: &ExtractedDeps) -> io::Result<()> {
    let mut content = String::new();
    content.push_str("[package]\n");
    content.push_str("name = \"workspace-cache-deps\"\n");
    content.push_str("version = \"0.1.0\"\n");
    content.push_str("edition = \"2021\"\n");
    content.push_str("\n[workspace]\n");

    if !deps.dependencies.is_empty() {
        content.push_str("\n[dependencies]\n");
        for dep in deps.dependencies.values() {
            content.push_str(&format_dependency(
                &dep.name,
                &dep.version,
                &dep.features,
                dep.default_features,
            ));
            content.push('\n');
        }
    }

    if !deps.dev_dependencies.is_empty() {
        content.push_str("\n[dev-dependencies]\n");
        for dep in deps.dev_dependencies.values() {
            content.push_str(&format_dependency(
                &dep.name,
                &dep.version,
                &dep.features,
                dep.default_features,
            ));
            content.push('\n');
        }
    }

    if !deps.build_dependencies.is_empty() {
        content.push_str("\n[build-dependencies]\n");
        for dep in deps.build_dependencies.values() {
            content.push_str(&format_dependency(
                &dep.name,
                &dep.version,
                &dep.features,
                dep.default_features,
            ));
            content.push('\n');
        }
    }

    fs::write(cache_dir.join("Cargo.toml"), content)
}

fn format_dependency(
    name: &str,
    version: &str,
    features: &[String],
    default_features: bool,
) -> String {
    let mut parts = Vec::new();

    parts.push(format!("version = \"{}\"", version));

    if !features.is_empty() {
        let features_str: Vec<_> = features.iter().map(|f| format!("\"{}\"", f)).collect();
        parts.push(format!("features = [{}]", features_str.join(", ")));
    }

    if !default_features {
        parts.push("default-features = false".to_string());
    }

    format!("{} = {{ {} }}", name, parts.join(", "))
}

fn generate_dummy_main(src_dir: &Path) -> io::Result<()> {
    fs::write(src_dir.join("main.rs"), "fn main() {}\n")
}

fn copy_lockfile(workspace_root: &Path, cache_dir: &Path) -> io::Result<()> {
    let source = workspace_root.join("Cargo.lock");
    if source.exists() {
        fs::copy(&source, cache_dir.join("Cargo.lock"))?;
    }
    Ok(())
}
