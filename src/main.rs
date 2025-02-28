mod builder;
mod cli;
mod generator;
mod metadata;

use anyhow::Result;
use cli::Command;

fn main() -> Result<()> {
    let cli = cli::parse();

    match cli.command {
        Command::Deps { package } => {
            let meta = metadata::get_metadata()?;
            let deps = metadata::extract_dependencies(&meta, &package);
            generator::generate_minimal_workspace(&deps, meta.workspace_root.as_std_path())?;
            println!("Generated .workspace-cache/");
        }
        Command::Build { release, package } => {
            builder::run_build(release, &package)?;
        }
    }

    Ok(())
}
