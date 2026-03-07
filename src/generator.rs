use crate::metadata::{ExtractedWorkspace, WorkspaceMember};
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

    copy_workspace_manifest(workspace_root, &cache_dir, &workspace.members)?;
    copy_lockfile(workspace_root, &cache_dir)?;

    for member in &workspace.members {
        generate_member_stub(&cache_dir, member, workspace_root)?;
    }

    Ok(())
}

fn copy_workspace_manifest(
    workspace_root: &Path,
    cache_dir: &Path,
    members: &[WorkspaceMember],
) -> io::Result<()> {
    let source = workspace_root.join("Cargo.toml");
    let content = fs::read_to_string(&source)?;

    let mut doc = content
        .parse::<DocumentMut>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if let Some(workspace) = doc.get_mut("workspace") {
        let member_paths: Array = members
            .iter()
            .map(|m| m.path.display().to_string())
            .collect();

        workspace["members"] = toml_edit::value(member_paths);

        // Remove exclude since we're creating a minimal workspace
        if let Some(table) = workspace.as_table_mut() {
            table.remove("exclude");
        }
    }

    fs::write(cache_dir.join("Cargo.toml"), doc.to_string())
}

fn generate_member_stub(
    cache_dir: &Path,
    member: &WorkspaceMember,
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

fn copy_lockfile(workspace_root: &Path, cache_dir: &Path) -> io::Result<()> {
    let source = workspace_root.join("Cargo.lock");
    if source.exists() {
        fs::copy(&source, cache_dir.join("Cargo.lock"))?;
    }
    Ok(())
}
