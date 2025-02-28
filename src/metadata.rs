use cargo_metadata::{Metadata, MetadataCommand, Package};
use std::collections::BTreeMap;

pub struct Dependency {
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
    pub default_features: bool,
}

pub struct ExtractedDeps {
    pub dependencies: BTreeMap<String, Dependency>,
    pub dev_dependencies: BTreeMap<String, Dependency>,
    pub build_dependencies: BTreeMap<String, Dependency>,
}

pub fn get_metadata() -> Result<Metadata, cargo_metadata::Error> {
    MetadataCommand::new().exec()
}

pub fn extract_dependencies(metadata: &Metadata, package_filter: &[String]) -> ExtractedDeps {
    let mut deps: BTreeMap<String, Dependency> = BTreeMap::new();
    let mut dev_deps: BTreeMap<String, Dependency> = BTreeMap::new();
    let mut build_deps: BTreeMap<String, Dependency> = BTreeMap::new();

    let workspace_member_ids: std::collections::HashSet<_> =
        metadata.workspace_members.iter().collect();

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

    for pkg in packages_to_process {
        collect_package_deps(
            pkg,
            &workspace_member_ids,
            metadata,
            &mut deps,
            &mut dev_deps,
            &mut build_deps,
        );
    }

    ExtractedDeps {
        dependencies: deps,
        dev_dependencies: dev_deps,
        build_dependencies: build_deps,
    }
}

fn collect_package_deps(
    pkg: &Package,
    workspace_members: &std::collections::HashSet<&cargo_metadata::PackageId>,
    metadata: &Metadata,
    deps: &mut BTreeMap<String, Dependency>,
    dev_deps: &mut BTreeMap<String, Dependency>,
    build_deps: &mut BTreeMap<String, Dependency>,
) {
    for dep in &pkg.dependencies {
        if is_path_or_workspace_dep(dep, workspace_members, metadata) {
            continue;
        }

        let dependency = Dependency {
            name: dep.name.clone(),
            version: dep.req.to_string(),
            features: dep.features.clone(),
            default_features: dep.uses_default_features,
        };

        let target_map = match dep.kind {
            cargo_metadata::DependencyKind::Development => &mut *dev_deps,
            cargo_metadata::DependencyKind::Build => &mut *build_deps,
            _ => &mut *deps,
        };

        target_map
            .entry(dep.name.clone())
            .and_modify(|existing| {
                for f in &dependency.features {
                    if !existing.features.contains(f) {
                        existing.features.push(f.clone());
                    }
                }
            })
            .or_insert(dependency);
    }
}

fn is_path_or_workspace_dep(
    dep: &cargo_metadata::Dependency,
    workspace_members: &std::collections::HashSet<&cargo_metadata::PackageId>,
    metadata: &Metadata,
) -> bool {
    if dep.path.is_some() {
        return true;
    }

    for pkg_id in workspace_members {
        if let Some(pkg) = metadata.packages.iter().find(|p| &p.id == *pkg_id) {
            if pkg.name == dep.name {
                return true;
            }
        }
    }

    false
}
