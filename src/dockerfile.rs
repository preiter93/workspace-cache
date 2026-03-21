use crate::metadata::WorkspaceMember;
use minijinja::{context, Environment};
use serde::Serialize;
use std::io::{self, Write};
use std::path::Path;

const TEMPLATE: &str = r#"# Stage 1: Install workspace-cache tool
FROM {{ base_image }} AS base
WORKDIR /app
RUN cargo install --git https://github.com/preiter93/workspace-cache workspace-cache

# Stage 2: Generate minimal workspace with stub sources for dependency resolution
FROM base AS planner
COPY . .
RUN workspace-cache deps --bin {{ bin }}{% if fast %} --fast{% endif %}

# Stage 3: Build dependencies only (cached until Cargo.toml/Cargo.lock change)
FROM base AS deps
COPY --from=planner /app/.workspace-cache .
RUN cargo build{% if release %} --release{% endif %}

# Stage 4: Build the actual binary with real source code
FROM deps AS builder
RUN rm -rf {% for member in members %}{{ member.path }}/src{% if not loop.last %} {% endif %}{% endfor %}
{%- for member in members %}
COPY {{ member.path }} {{ member.path }}
{%- endfor %}
# Clean workspace crates to force rebuild with real sources. Docker COPY
# preserves original file mtimes, which confuses cargo's fingerprinting.
# This only removes workspace crate artifacts - dependencies stay cached.
RUN cargo clean{% if release %} --release{% endif %}{% for member in members %} -p {{ member.name }}{% endfor %}
RUN cargo build{% if release %} --release{% endif %} --bin {{ bin }}

# Stage 5: Minimal runtime image
FROM {{ runtime_image }} AS runtime
COPY --from=builder /app/target/{{ profile_dir }}/{{ bin }} /usr/local/bin/{{ bin }}
ENTRYPOINT ["/usr/local/bin/{{ bin }}"]
"#;

#[derive(Serialize)]
struct MemberContext {
    name: String,
    path: String,
}

pub struct DockerfileConfig {
    pub bin: String,
    pub profile: String,
    pub base_image: String,
    pub runtime_image: String,
    pub members: Vec<WorkspaceMember>,
    pub fast: bool,
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
            name: m.name.clone(),
            path: m.path.display().to_string(),
        })
        .collect();

    let release = config.profile == "release";
    let profile_dir = if release { "release" } else { "debug" };

    let dockerfile = template
        .render(context! {
            base_image => &config.base_image,
            runtime_image => &config.runtime_image,
            bin => &config.bin,
            release => release,
            profile_dir => profile_dir,
            members => members,
            fast => config.fast,
        })
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if let Some(path) = output {
        std::fs::write(path, dockerfile)
    } else {
        io::stdout().write_all(dockerfile.as_bytes())?;
        Ok(())
    }
}
