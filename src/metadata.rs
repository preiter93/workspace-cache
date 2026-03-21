use cargo_metadata::{Metadata, MetadataCommand, Package, PackageId, Target};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    pub name: String,
    pub path: PathBuf,
    pub is_bin: bool,
    pub is_lib: bool,
    pub bins: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: String,
}

pub struct ExtractedWorkspace {
    pub members: Vec<WorkspaceMember>,
    pub used_dependencies: HashSet<String>,
    pub resolved_packages: HashSet<ResolvedPackage>,
}

pub fn get_metadata(no_deps: bool) -> Result<Metadata, cargo_metadata::Error> {
    let mut cmd = MetadataCommand::new();
    if no_deps {
        cmd.no_deps();
    }
    cmd.exec()
}

pub fn extract_workspace(metadata: &Metadata, package_filter: &[String]) -> ExtractedWorkspace {
    let workspace_member_names: HashSet<String> = metadata
        .workspace_members
        .iter()
        .filter_map(|id| metadata.packages.iter().find(|p| &p.id == id))
        .map(|p| p.name.to_string())
        .collect();

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
            .filter(|p| package_filter.contains(&p.name.to_string()))
            .collect()
    };

    let members: Vec<WorkspaceMember> = packages_to_process
        .iter()
        .map(|pkg| extract_member_info(pkg, metadata))
        .collect();

    let used_dependencies =
        collect_used_dependencies(&packages_to_process, &workspace_member_names);

    let resolved_packages = collect_resolved_packages(metadata, &packages_to_process);

    ExtractedWorkspace {
        members,
        used_dependencies,
        resolved_packages,
    }
}

fn collect_used_dependencies(
    packages: &[&Package],
    workspace_members: &HashSet<String>,
) -> HashSet<String> {
    let mut deps = HashSet::new();

    for pkg in packages {
        for dep in &pkg.dependencies {
            let dep_name = dep.name.clone();
            if workspace_members.contains(&dep_name) {
                continue;
            }
            deps.insert(dep_name);
        }
    }

    deps
}

fn collect_resolved_packages(
    metadata: &Metadata,
    start_packages: &[&Package],
) -> HashSet<ResolvedPackage> {
    let mut resolved = HashSet::new();

    let Some(resolve) = &metadata.resolve else {
        return resolved;
    };

    let mut to_visit: Vec<&PackageId> = start_packages.iter().map(|p| &p.id).collect();
    let mut visited: HashSet<&PackageId> = HashSet::new();

    while let Some(pkg_id) = to_visit.pop() {
        if visited.contains(pkg_id) {
            continue;
        }
        visited.insert(pkg_id);

        if let Some(pkg) = metadata.packages.iter().find(|p| &p.id == pkg_id) {
            resolved.insert(ResolvedPackage {
                name: pkg.name.to_string(),
                version: pkg.version.to_string(),
            });
        }

        if let Some(node) = resolve.nodes.iter().find(|n| &n.id == pkg_id) {
            for dep_id in &node.dependencies {
                if !visited.contains(dep_id) {
                    to_visit.push(dep_id);
                }
            }
        }
    }

    resolved
}

pub fn resolve_workspace_deps(metadata: &Metadata, packages: &[String]) -> Vec<String> {
    let workspace_member_names: HashSet<String> = metadata
        .workspace_members
        .iter()
        .filter_map(|id| metadata.packages.iter().find(|p| &p.id == id))
        .map(|p| p.name.to_string())
        .collect();

    let mut result: HashSet<String> = HashSet::new();
    let mut to_visit: Vec<String> = packages.to_vec();

    while let Some(pkg_name) = to_visit.pop() {
        if result.contains(&pkg_name) {
            continue;
        }

        let pkg = metadata
            .workspace_members
            .iter()
            .filter_map(|id| metadata.packages.iter().find(|p| &p.id == id))
            .find(|p| p.name == pkg_name);

        let Some(pkg) = pkg else {
            continue;
        };

        result.insert(pkg_name.clone());

        for dep in &pkg.dependencies {
            let dep_name = dep.name.clone();
            if workspace_member_names.contains(&dep_name) && !result.contains(&dep_name) {
                to_visit.push(dep_name);
            }
        }
    }

    let mut sorted: Vec<String> = result.into_iter().collect();
    sorted.sort();
    sorted
}

/// Find which package contains each binary and return a mapping of binary name -> package name
pub fn resolve_bins_to_packages(metadata: &Metadata, bins: &[String]) -> HashMap<String, String> {
    let mut bin_to_package = HashMap::new();

    for bin_name in bins {
        for member_id in &metadata.workspace_members {
            let Some(pkg) = metadata.packages.iter().find(|p| &p.id == member_id) else {
                continue;
            };

            let has_bin = pkg
                .targets
                .iter()
                .any(|t| is_bin_target(t) && t.name.clone() == *bin_name);

            if has_bin {
                bin_to_package.insert(bin_name.clone(), pkg.name.to_string());
                break;
            }
        }
    }

    bin_to_package
}

/// Get all binary names from workspace packages
pub fn get_all_bins(metadata: &Metadata) -> Vec<String> {
    metadata
        .workspace_members
        .iter()
        .filter_map(|id| metadata.packages.iter().find(|p| &p.id == id))
        .flat_map(|pkg| {
            pkg.targets
                .iter()
                .filter(|t| is_bin_target(t))
                .map(|t| t.name.clone())
        })
        .collect()
}

fn extract_member_info(pkg: &Package, metadata: &Metadata) -> WorkspaceMember {
    let manifest_dir = pkg.manifest_path.parent().unwrap();
    let relative_path = manifest_dir
        .strip_prefix(&metadata.workspace_root)
        .unwrap_or(manifest_dir);

    let is_lib = pkg.targets.iter().any(is_lib_target);
    let is_bin = pkg.targets.iter().any(is_bin_target);

    let bins: Vec<String> = pkg
        .targets
        .iter()
        .filter(|t| is_bin_target(t))
        .map(|t| t.name.clone())
        .collect();

    WorkspaceMember {
        name: pkg.name.to_string(),
        path: relative_path.as_std_path().to_path_buf(),
        is_bin,
        is_lib,
        bins,
    }
}

fn is_lib_target(target: &Target) -> bool {
    target.kind.iter().any(|k| {
        let kind = k.to_string();
        kind == "lib"
            || kind == "rlib"
            || kind == "dylib"
            || kind == "staticlib"
            || kind == "cdylib"
            || kind == "proc-macro"
    })
}

fn is_bin_target(target: &Target) -> bool {
    target.kind.iter().any(|k| k.to_string() == "bin")
}
