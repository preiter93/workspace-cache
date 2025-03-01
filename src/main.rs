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
            let workspace = metadata::extract_workspace(&meta, &package);
            generator::generate_minimal_workspace(&workspace, meta.workspace_root.as_std_path())?;
            println!("Generated .workspace-cache/");
        }
        Command::Build { release, package } => {
            builder::run_build(release, &package)?;
        }
    }

    Ok(())
}
