use crate::metadata::{ExtractedWorkspace, ResolvedPackage};
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;
use toml_edit::{Array, DocumentMut};

const CACHE_DIR: &str = ".workspace-cache";

pub fn generate_minimal_workspace(
    workspace: &ExtractedWorkspace,
    workspace_root: &Path,
) -> io::Result<()> {
    let cache_dir = workspace_root.join(CACHE_DIR);

    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)?;
    }
    fs::create_dir_all(&cache_dir)?;

    copy_workspace_manifest(workspace_root, &cache_dir, workspace)?;
    copy_lockfile(workspace_root, &cache_dir, &workspace.resolved_packages)?;

    for member in &workspace.members {
        generate_member_stub(&cache_dir, member, workspace_root)?;
    }

    Ok(())
}

fn copy_workspace_manifest(
    workspace_root: &Path,
    cache_dir: &Path,
    workspace: &ExtractedWorkspace,
) -> io::Result<()> {
    let source = workspace_root.join("Cargo.toml");
    let content = fs::read_to_string(&source)?;

    let mut doc = content
        .parse::<DocumentMut>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if let Some(ws) = doc.get_mut("workspace") {
        let member_paths: Array = workspace
            .members
            .iter()
            .map(|m| m.path.display().to_string())
            .collect();

        ws["members"] = toml_edit::value(member_paths);

        if let Some(table) = ws.as_table_mut() {
            table.remove("exclude");
        }

        filter_workspace_dependencies(ws, &workspace.used_dependencies);
    }

    fs::write(cache_dir.join("Cargo.toml"), doc.to_string())
}

fn filter_workspace_dependencies(workspace: &mut toml_edit::Item, used_deps: &HashSet<String>) {
    if let Some(deps) = workspace.get_mut("dependencies") {
        if let Some(table) = deps.as_table_mut() {
            let keys_to_remove: Vec<String> = table
                .iter()
                .filter(|(key, _)| !used_deps.contains(*key))
                .map(|(key, _)| key.to_string())
                .collect();

            for key in keys_to_remove {
                table.remove(&key);
            }
        }
    }
}

fn generate_member_stub(
    cache_dir: &Path,
    member: &crate::metadata::WorkspaceMember,
    workspace_root: &Path,
) -> io::Result<()> {
    let member_dir = cache_dir.join(&member.path);
    let src_dir = member_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    let original_manifest = workspace_root.join(&member.path).join("Cargo.toml");
    if original_manifest.exists() {
        fs::copy(&original_manifest, member_dir.join("Cargo.toml"))?;
    }

    if member.is_lib {
        fs::write(src_dir.join("lib.rs"), "// stub\n")?;
    }

    if member.is_bin {
        if member.bins.len() <= 1 {
            fs::write(src_dir.join("main.rs"), "fn main() {}\n")?;
        } else {
            let bin_dir = src_dir.join("bin");
            fs::create_dir_all(&bin_dir)?;
            for bin in &member.bins {
                fs::write(bin_dir.join(format!("{bin}.rs")), "fn main() {}\n")?;
            }
        }
    }

    if !member.is_bin && !member.is_lib {
        fs::write(src_dir.join("lib.rs"), "// stub\n")?;
    }

    Ok(())
}

fn copy_lockfile(
    workspace_root: &Path,
    cache_dir: &Path,
    resolved_packages: &HashSet<ResolvedPackage>,
) -> io::Result<()> {
    let source = workspace_root.join("Cargo.lock");
    if !source.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&source)?;

    let mut doc = content
        .parse::<DocumentMut>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if let Some(packages) = doc.get_mut("package") {
        if let Some(arr) = packages.as_array_of_tables_mut() {
            arr.retain(|table| {
                let name = table.get("name").and_then(|v| v.as_str());
                let version = table.get("version").and_then(|v| v.as_str());

                match (name, version) {
                    (Some(n), Some(v)) => resolved_packages.contains(&ResolvedPackage {
                        name: n.to_string(),
                        version: v.to_string(),
                    }),
                    _ => true,
                }
            });
        }
    }

    fs::write(cache_dir.join("Cargo.lock"), doc.to_string())
}
