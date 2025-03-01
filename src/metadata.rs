use cargo_metadata::{Metadata, MetadataCommand, Package, Target};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    pub path: PathBuf,
    pub is_bin: bool,
    pub is_lib: bool,
    pub bins: Vec<String>,
}

pub struct ExtractedWorkspace {
    pub members: Vec<WorkspaceMember>,
}

pub fn get_metadata() -> Result<Metadata, cargo_metadata::Error> {
    MetadataCommand::new().exec()
}

pub fn extract_workspace(metadata: &Metadata, package_filter: &[String]) -> ExtractedWorkspace {
    let packages_to_process: Vec<&Package> = if package_filter.is_empty() {
        metadata
            .workspace_members
            .iter()
            .filter_map(|id| metadata.packages.iter().find(|p| &p.id == id))
            .collect()
    } else {
        metadata
            .workspace_members
            .iter()
            .filter_map(|id| metadata.packages.iter().find(|p| &p.id == id))
            .filter(|p| package_filter.contains(&p.name))
            .collect()
    };

    let members: Vec<WorkspaceMember> = packages_to_process
        .iter()
        .map(|pkg| extract_member_info(pkg, metadata))
        .collect();

    ExtractedWorkspace { members }
}

fn extract_member_info(pkg: &Package, metadata: &Metadata) -> WorkspaceMember {
    let manifest_dir = pkg.manifest_path.parent().unwrap();
    let relative_path = manifest_dir
        .strip_prefix(&metadata.workspace_root)
        .unwrap_or(manifest_dir);

    let is_lib = pkg.targets.iter().any(|t| is_lib_target(t));
    let is_bin = pkg.targets.iter().any(|t| is_bin_target(t));

    let bins: Vec<String> = pkg
        .targets
        .iter()
        .filter(|t| is_bin_target(t))
        .map(|t| t.name.clone())
        .collect();

    WorkspaceMember {
        path: relative_path.as_std_path().to_path_buf(),
        is_bin,
        is_lib,
        bins,
    }
}

fn is_lib_target(target: &Target) -> bool {
    target.kind.iter().any(|k| {
        k == "lib"
            || k == "rlib"
            || k == "dylib"
            || k == "staticlib"
            || k == "cdylib"
            || k == "proc-macro"
    })
}

fn is_bin_target(target: &Target) -> bool {
    target.kind.iter().any(|k| k == "bin")
}
