use std::process::Command;

pub fn run_build(release: bool, packages: &[String]) -> anyhow::Result<()> {
    let mut args = vec!["build"];

    if packages.is_empty() {
        args.push("--workspace");
    } else {
        for pkg in packages {
            args.push("-p");
            args.push(pkg);
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
