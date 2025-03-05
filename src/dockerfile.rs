use crate::metadata::WorkspaceMember;
use std::io::{self, Write};
use std::path::Path;

pub struct DockerfileConfig {
    pub package: String,
    pub base_image: String,
    pub runtime_image: String,
    pub members: Vec<WorkspaceMember>,
}

pub fn generate(config: &DockerfileConfig, output: Option<&Path>) -> io::Result<()> {
    let dockerfile = render(config);

    match output {
        Some(path) => std::fs::write(path, dockerfile),
        None => {
            io::stdout().write_all(dockerfile.as_bytes())?;
            Ok(())
        }
    }
}

fn render(config: &DockerfileConfig) -> String {
    let mut lines = Vec::new();

    lines.push(format!("FROM {} AS base", config.base_image));
    lines.push("WORKDIR /app".to_string());
    lines.push(
        "COPY --from=workspace-cache /workspace-cache /usr/local/bin/workspace-cache".to_string(),
    );
    lines.push(String::new());

    lines.push("# Prepare dependencies".to_string());
    lines.push("FROM base AS planner".to_string());
    lines.push("COPY . .".to_string());
    lines.push(format!("RUN workspace-cache deps -p {}", config.package));
    lines.push(String::new());

    lines.push("# Build dependencies".to_string());
    lines.push("FROM base AS builder".to_string());
    lines.push("COPY --from=planner /app/.workspace-cache ./.workspace-cache".to_string());
    lines.push("COPY --from=planner /app/Cargo.lock ./Cargo.lock".to_string());
    lines.push("RUN cd .workspace-cache && cargo build --release".to_string());
    lines.push(String::new());

    lines.push("# Build the binary".to_string());
    lines.push("COPY Cargo.toml Cargo.lock ./".to_string());

    for member in &config.members {
        let path = member.path.display();
        lines.push(format!("COPY {} {}", path, path));
    }

    lines.push(format!(
        "RUN workspace-cache build --release -p {}",
        config.package
    ));
    lines.push(String::new());

    lines.push("# Runtime".to_string());
    lines.push(format!("FROM {} AS runtime", config.runtime_image));
    lines.push(format!(
        "COPY --from=builder /app/target/release/{} /usr/local/bin/{}",
        config.package, config.package
    ));
    lines.push(format!(
        "ENTRYPOINT [\"/usr/local/bin/{}\"]",
        config.package
    ));

    lines.join("\n") + "\n"
}
