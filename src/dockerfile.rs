use crate::metadata::WorkspaceMember;
use minijinja::{context, Environment};
use serde::Serialize;
use std::io::{self, Write};
use std::path::Path;

const TEMPLATE: &str = r#"FROM {{ base_image }} AS base
WORKDIR /app
COPY --from=workspace-cache /workspace-cache /usr/local/bin/workspace-cache

# Prepare minimal workspace
FROM base AS planner
COPY . .
RUN workspace-cache deps -p {{ package }}

# Build dependencies
FROM base AS deps
COPY --from=planner /app/.workspace-cache .
RUN cargo build --release

# Build the binary
FROM deps AS builder
RUN rm -rf {% for member in members %}{{ member.path }}/src{% if not loop.last %} {% endif %}{% endfor %}
{%- for member in members %}
COPY {{ member.path }} {{ member.path }}
{%- endfor %}
RUN cargo build --release -p {{ package }}

# Runtime
FROM {{ runtime_image }} AS runtime
COPY --from=builder /app/target/release/{{ package }} /usr/local/bin/{{ package }}
ENTRYPOINT ["/usr/local/bin/{{ package }}"]
"#;

#[derive(Serialize)]
struct MemberContext {
    path: String,
}

pub struct DockerfileConfig {
    pub package: String,
    pub base_image: String,
    pub runtime_image: String,
    pub members: Vec<WorkspaceMember>,
}

pub fn generate(config: &DockerfileConfig, output: Option<&Path>) -> io::Result<()> {
    let mut env = Environment::new();
    env.add_template("Dockerfile", TEMPLATE)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let template = env
        .get_template("Dockerfile")
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let members: Vec<MemberContext> = config
        .members
        .iter()
        .map(|m| MemberContext {
            path: m.path.display().to_string(),
        })
        .collect();

    let dockerfile = template
        .render(context! {
            base_image => &config.base_image,
            runtime_image => &config.runtime_image,
            package => &config.package,
            members => members,
        })
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    match output {
        Some(path) => std::fs::write(path, dockerfile),
        None => {
            io::stdout().write_all(dockerfile.as_bytes())?;
            Ok(())
        }
    }
}
