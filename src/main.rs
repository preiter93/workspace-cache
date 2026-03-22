mod builder;
mod cli;
mod dockerfile;
mod generator;
mod metadata;

use anyhow::{Result, bail};
use cli::Command;

fn main() -> Result<()> {
    let cli = cli::parse();

    match cli.command {
        Command::Deps { bin, output, fast } => {
            let meta = metadata::get_metadata(fast)?;

            let packages = if bin.is_empty() {
                vec![]
            } else {
                let bin_to_pkg = metadata::resolve_bins_to_packages(&meta, &bin);

                // Check all binaries were found
                for b in &bin {
                    if !bin_to_pkg.contains_key(b) {
                        let available = metadata::get_all_bins(&meta);
                        bail!(
                            "Binary '{b}' not found in workspace. Available binaries: {available:?}"
                        );
                    }
                }

                let pkgs: Vec<String> = bin_to_pkg.values().cloned().collect();
                metadata::resolve_workspace_deps(&meta, &pkgs)
            };

            let workspace = metadata::extract_workspace(&meta, &packages);
            eprintln!(
                "Debug: {} workspace members, {} used deps, {} resolved packages",
                workspace.members.len(),
                workspace.used_dependencies.len(),
                workspace.resolved_packages.len()
            );
            generator::generate_minimal_workspace(
                &workspace,
                meta.workspace_root.as_std_path(),
                output.as_deref(),
            )?;
            let dir_name = output.as_deref().unwrap_or(generator::DEFAULT_CACHE_DIR);
            println!("Generated {dir_name}/");
        }
        Command::Build { release, bin } => {
            builder::run_build(release, &bin)?;
        }
        Command::Resolve { bin } => {
            let meta = metadata::get_metadata(false)?;
            let bin_to_pkg = metadata::resolve_bins_to_packages(&meta, &bin);

            for b in &bin {
                if !bin_to_pkg.contains_key(b) {
                    let available = metadata::get_all_bins(&meta);
                    bail!("Binary '{b}' not found in workspace. Available binaries: {available:?}");
                }
            }

            let pkgs: Vec<String> = bin_to_pkg.values().cloned().collect();
            let resolved = metadata::resolve_workspace_deps(&meta, &pkgs);
            let workspace = metadata::extract_workspace(&meta, &resolved);

            for member in &workspace.members {
                println!("{} {}", member.path.display(), member.name);
            }
        }
        Command::Dockerfile {
            bin,
            profile,
            from_git,
            tool_version,
            base_image,
            runtime_image,
            output,
            fast,
        } => {
            let meta = metadata::get_metadata(false)?;
            let bin_to_pkg = metadata::resolve_bins_to_packages(&meta, std::slice::from_ref(&bin));

            let Some(pkg_name) = bin_to_pkg.get(&bin) else {
                let available = metadata::get_all_bins(&meta);
                bail!("Binary '{bin}' not found in workspace. Available binaries: {available:?}");
            };

            let resolved = metadata::resolve_workspace_deps(&meta, std::slice::from_ref(pkg_name));
            let workspace = metadata::extract_workspace(&meta, &resolved);

            let config = dockerfile::DockerfileConfig {
                bin,
                profile,
                base_image,
                runtime_image,
                members: workspace.members,
                fast,
                from_git,
                tool_version,
            };

            let output_path = output.as_ref().map(std::path::Path::new);
            dockerfile::generate(&config, output_path)?;
        }
    }

    Ok(())
}
