mod builder;
mod cli;
mod dockerfile;
mod generator;
mod metadata;

use anyhow::Result;
use cli::Command;

fn main() -> Result<()> {
    let cli = cli::parse();

    match cli.command {
        Command::Deps { package } => {
            let meta = metadata::get_metadata()?;

            let packages = if package.is_empty() {
                package
            } else {
                metadata::resolve_workspace_deps(&meta, &package)
            };

            let workspace = metadata::extract_workspace(&meta, &packages);
            generator::generate_minimal_workspace(&workspace, meta.workspace_root.as_std_path())?;
            println!("Generated .workspace-cache/");
        }
        Command::Build { release, package } => {
            builder::run_build(release, &package)?;
        }
        Command::Resolve { package } => {
            let meta = metadata::get_metadata()?;
            let resolved = metadata::resolve_workspace_deps(&meta, &package);
            for name in &resolved {
                println!("{}", name);
            }
        }
        Command::Dockerfile {
            package,
            base_image,
            runtime_image,
            output,
        } => {
            let meta = metadata::get_metadata()?;
            let resolved = metadata::resolve_workspace_deps(&meta, &[package.clone()]);
            let workspace = metadata::extract_workspace(&meta, &resolved);

            let config = dockerfile::DockerfileConfig {
                package,
                base_image,
                runtime_image,
                members: workspace.members,
            };

            let output_path = output.as_ref().map(std::path::Path::new);
            dockerfile::generate(&config, output_path)?;
        }
    }

    Ok(())
}
