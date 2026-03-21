use std::process::Command;

pub fn run_build(release: bool, bins: &[String]) -> anyhow::Result<()> {
    let mut args = vec!["build"];

    if bins.is_empty() {
        args.push("--workspace");
    } else {
        for bin in bins {
            args.push("--bin");
            args.push(bin);
        }
    }

    if release {
        args.push("--release");
    }

    let status = Command::new("cargo").args(&args).status()?;

    if !status.success() {
        anyhow::bail!("cargo build failed");
    }

    Ok(())
}
