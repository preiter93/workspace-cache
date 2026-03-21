use std::fs;
use std::path::Path;
use std::process::Command;

fn workspace_cache_binary() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.pop();
    path.push("workspace-cache");
    path
}

fn create_test_workspace(dir: &Path) {
    fs::create_dir_all(dir.join("crates/bin-a/src")).unwrap();
    fs::create_dir_all(dir.join("crates/lib-b/src")).unwrap();

    fs::write(
        dir.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/bin-a", "crates/lib-b"]
resolver = "2"

[workspace.dependencies]
serde = "1"
log = "0.4"
"#,
    )
    .unwrap();

    fs::write(
        dir.join("crates/bin-a/Cargo.toml"),
        r#"[package]
name = "bin-a"
version = "0.1.0"
edition = "2021"

[dependencies]
serde.workspace = true
lib-b = { path = "../lib-b" }
"#,
    )
    .unwrap();

    fs::write(
        dir.join("crates/bin-a/src/main.rs"),
        "fn main() { println!(\"hello\"); }\n",
    )
    .unwrap();

    fs::write(
        dir.join("crates/lib-b/Cargo.toml"),
        r#"[package]
name = "lib-b"
version = "0.1.0"
edition = "2021"

[dependencies]
log.workspace = true
"#,
    )
    .unwrap();

    fs::write(dir.join("crates/lib-b/src/lib.rs"), "// lib-b\n").unwrap();
}

#[test]
fn test_deps_command_creates_workspace_cache() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace");
    fs::create_dir_all(&workspace_dir).unwrap();

    create_test_workspace(&workspace_dir);

    let output = Command::new(workspace_cache_binary())
        .arg("deps")
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(output.status.success(), "deps command failed: {:?}", output);

    let cache_dir = workspace_dir.join(".workspace-cache");
    assert!(cache_dir.exists(), ".workspace-cache dir should exist");
    assert!(
        cache_dir.join("Cargo.toml").exists(),
        "Cargo.toml should exist"
    );
    assert!(
        cache_dir.join("crates/bin-a/Cargo.toml").exists(),
        "bin-a/Cargo.toml should exist"
    );
    assert!(
        cache_dir.join("crates/lib-b/Cargo.toml").exists(),
        "lib-b/Cargo.toml should exist"
    );
    assert!(
        cache_dir.join("crates/bin-a/src/main.rs").exists(),
        "bin-a/src/main.rs stub should exist"
    );
    assert!(
        cache_dir.join("crates/lib-b/src/lib.rs").exists(),
        "lib-b/src/lib.rs stub should exist"
    );

    let workspace_toml = fs::read_to_string(cache_dir.join("Cargo.toml")).unwrap();
    assert!(
        workspace_toml.contains("[workspace]"),
        "should have workspace section"
    );
    assert!(
        workspace_toml.contains("crates/bin-a"),
        "should include bin-a member"
    );
    assert!(
        workspace_toml.contains("crates/lib-b"),
        "should include lib-b member"
    );
}

#[test]
fn test_deps_command_copies_lockfile() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-lock");
    fs::create_dir_all(&workspace_dir).unwrap();

    create_test_workspace(&workspace_dir);

    fs::write(workspace_dir.join("Cargo.lock"), "# fake lockfile\n").unwrap();

    let output = Command::new(workspace_cache_binary())
        .arg("deps")
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(output.status.success());

    let cache_lock = workspace_dir.join(".workspace-cache/Cargo.lock");
    assert!(cache_lock.exists(), "Cargo.lock should be copied");
}

#[test]
fn test_generated_workspace_is_valid() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-valid");
    fs::create_dir_all(&workspace_dir).unwrap();

    create_test_workspace(&workspace_dir);

    let output = Command::new(workspace_cache_binary())
        .arg("deps")
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(output.status.success());

    let cache_dir = workspace_dir.join(".workspace-cache");

    let check_output = Command::new("cargo")
        .args(["check"])
        .current_dir(&cache_dir)
        .output()
        .unwrap();

    assert!(
        check_output.status.success(),
        "cargo check should succeed on generated workspace: {}",
        String::from_utf8_lossy(&check_output.stderr)
    );
}

#[test]
fn test_build_command() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-build");
    fs::create_dir_all(&workspace_dir).unwrap();

    create_test_workspace(&workspace_dir);

    let output = Command::new(workspace_cache_binary())
        .args(["build"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "build command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        workspace_dir.join("target/debug/bin-a").exists()
            || workspace_dir.join("target/debug/bin-a.exe").exists(),
        "binary should be built"
    );
}

#[test]
fn test_build_command_release() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-build-release");
    fs::create_dir_all(&workspace_dir).unwrap();

    create_test_workspace(&workspace_dir);

    let output = Command::new(workspace_cache_binary())
        .args(["build", "--release"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "build --release command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        workspace_dir.join("target/release/bin-a").exists()
            || workspace_dir.join("target/release/bin-a.exe").exists(),
        "release binary should be built"
    );
}

#[test]
fn test_bin_filter() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-filter");
    fs::create_dir_all(&workspace_dir).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/api/src")).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/worker/src")).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/common/src")).unwrap();

    fs::write(
        workspace_dir.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/api", "crates/worker", "crates/common"]
resolver = "2"

[workspace.dependencies]
axum = "0.7"
tokio = "1"
serde = "1"
"#,
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/common/Cargo.toml"),
        r#"[package]
name = "common"
version = "0.1.0"
edition = "2021"

[dependencies]
serde.workspace = true
"#,
    )
    .unwrap();
    fs::write(workspace_dir.join("crates/common/src/lib.rs"), "").unwrap();

    fs::write(
        workspace_dir.join("crates/api/Cargo.toml"),
        r#"[package]
name = "api"
version = "0.1.0"
edition = "2021"

[dependencies]
common = { path = "../common" }
axum.workspace = true
"#,
    )
    .unwrap();
    fs::write(workspace_dir.join("crates/api/src/main.rs"), "fn main() {}").unwrap();

    fs::write(
        workspace_dir.join("crates/worker/Cargo.toml"),
        r#"[package]
name = "worker"
version = "0.1.0"
edition = "2021"

[dependencies]
common = { path = "../common" }
tokio.workspace = true
"#,
    )
    .unwrap();
    fs::write(
        workspace_dir.join("crates/worker/src/main.rs"),
        "fn main() {}",
    )
    .unwrap();

    // Test filtering to only api (which depends on common)
    let output = Command::new(workspace_cache_binary())
        .args(["deps", "--bin", "api"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "deps --bin api failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let cache_dir = workspace_dir.join(".workspace-cache");

    // Should have api and common, but not worker
    assert!(
        cache_dir.join("crates/api/Cargo.toml").exists(),
        "should include api"
    );
    assert!(
        cache_dir.join("crates/common/Cargo.toml").exists(),
        "should include common (dependency of api)"
    );
    assert!(
        !cache_dir.join("crates/worker").exists(),
        "should NOT include worker"
    );

    let workspace_toml = fs::read_to_string(cache_dir.join("Cargo.toml")).unwrap();
    assert!(
        workspace_toml.contains("crates/api"),
        "workspace should list api"
    );
    assert!(
        workspace_toml.contains("crates/common"),
        "workspace should list common"
    );
    assert!(
        !workspace_toml.contains("crates/worker"),
        "workspace should NOT list worker"
    );
}

#[test]
fn test_build_with_bin_filter() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-build-filter");
    fs::create_dir_all(&workspace_dir).unwrap();

    create_test_workspace(&workspace_dir);

    let output = Command::new(workspace_cache_binary())
        .args(["build", "--bin", "bin-a"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "build --bin bin-a failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_binary_crate_creates_main_rs() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-bin");
    fs::create_dir_all(&workspace_dir).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/my-bin/src")).unwrap();

    fs::write(
        workspace_dir.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/my-bin"]
resolver = "2"
"#,
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/my-bin/Cargo.toml"),
        r#"[package]
name = "my-bin"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/my-bin/src/main.rs"),
        "fn main() { println!(\"real code\"); }",
    )
    .unwrap();

    let output = Command::new(workspace_cache_binary())
        .arg("deps")
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(output.status.success());

    let stub_main = workspace_dir.join(".workspace-cache/crates/my-bin/src/main.rs");
    assert!(stub_main.exists(), "main.rs stub should exist");

    let content = fs::read_to_string(&stub_main).unwrap();
    assert!(
        content.contains("fn main()"),
        "stub should have main function"
    );
}

#[test]
fn test_shared_target_dir_caches_deps() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-target");
    fs::create_dir_all(&workspace_dir).unwrap();

    create_test_workspace(&workspace_dir);

    // Generate cache
    let output = Command::new(workspace_cache_binary())
        .arg("deps")
        .current_dir(&workspace_dir)
        .output()
        .unwrap();
    assert!(output.status.success());

    // Build in cache dir (deps only)
    let cache_dir = workspace_dir.join(".workspace-cache");
    let build_output = Command::new("cargo")
        .args(["build"])
        .current_dir(&cache_dir)
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "cargo build in cache dir failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    // The target directory should be inside .workspace-cache
    assert!(
        cache_dir.join("target").exists(),
        "target dir should exist in cache"
    );
}

#[test]
fn test_resolve_command() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-deps-of");
    fs::create_dir_all(&workspace_dir).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/api/src")).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/worker/src")).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/common/src")).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/utils/src")).unwrap();

    fs::write(
        workspace_dir.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/api", "crates/worker", "crates/common", "crates/utils"]
resolver = "2"
"#,
    )
    .unwrap();

    // common depends on utils
    fs::write(
        workspace_dir.join("crates/common/Cargo.toml"),
        r#"[package]
name = "common"
version = "0.1.0"
edition = "2021"

[dependencies]
utils = { path = "../utils" }
"#,
    )
    .unwrap();
    fs::write(workspace_dir.join("crates/common/src/lib.rs"), "").unwrap();

    // utils has no workspace deps
    fs::write(
        workspace_dir.join("crates/utils/Cargo.toml"),
        r#"[package]
name = "utils"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();
    fs::write(workspace_dir.join("crates/utils/src/lib.rs"), "").unwrap();

    // api depends on common (which depends on utils)
    fs::write(
        workspace_dir.join("crates/api/Cargo.toml"),
        r#"[package]
name = "api"
version = "0.1.0"
edition = "2021"

[dependencies]
common = { path = "../common" }
"#,
    )
    .unwrap();
    fs::write(workspace_dir.join("crates/api/src/main.rs"), "fn main() {}").unwrap();

    // worker depends on common
    fs::write(
        workspace_dir.join("crates/worker/Cargo.toml"),
        r#"[package]
name = "worker"
version = "0.1.0"
edition = "2021"

[dependencies]
common = { path = "../common" }
"#,
    )
    .unwrap();
    fs::write(
        workspace_dir.join("crates/worker/src/main.rs"),
        "fn main() {}",
    )
    .unwrap();

    // Test resolve api: should return api, common, utils
    let output = Command::new(workspace_cache_binary())
        .args(["resolve", "--bin", "api"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "resolve --bin api failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let deps: Vec<&str> = stdout.lines().collect();

    assert!(deps.contains(&"api"), "should include api itself");
    assert!(
        deps.contains(&"common"),
        "should include common (direct dep)"
    );
    assert!(
        deps.contains(&"utils"),
        "should include utils (transitive dep)"
    );
    assert!(!deps.contains(&"worker"), "should NOT include worker");

    // Test resolve worker: should return worker, common, utils
    let output = Command::new(workspace_cache_binary())
        .args(["resolve", "--bin", "worker"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let deps: Vec<&str> = stdout.lines().collect();

    assert!(deps.contains(&"worker"), "should include worker itself");
    assert!(deps.contains(&"common"), "should include common");
    assert!(deps.contains(&"utils"), "should include utils");
    assert!(!deps.contains(&"api"), "should NOT include api");
}

#[test]
fn test_cargo_lock_filtering() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-lock-filter");
    fs::create_dir_all(&workspace_dir).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/api/src")).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/worker/src")).unwrap();

    fs::write(
        workspace_dir.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/api", "crates/worker"]
resolver = "2"
"#,
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/api/Cargo.toml"),
        r#"[package]
name = "api"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1"
"#,
    )
    .unwrap();
    fs::write(workspace_dir.join("crates/api/src/main.rs"), "fn main() {}").unwrap();

    fs::write(
        workspace_dir.join("crates/worker/Cargo.toml"),
        r#"[package]
name = "worker"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["rt"] }
"#,
    )
    .unwrap();
    fs::write(
        workspace_dir.join("crates/worker/src/main.rs"),
        "fn main() {}",
    )
    .unwrap();

    // Generate lockfile
    let output = Command::new("cargo")
        .args(["generate-lockfile"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "Failed to generate lockfile: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let original_lock = fs::read_to_string(workspace_dir.join("Cargo.lock")).unwrap();
    assert!(
        original_lock.contains("serde"),
        "original lockfile should contain serde"
    );
    assert!(
        original_lock.contains("tokio"),
        "original lockfile should contain tokio"
    );

    // Generate cache with only api
    let output = Command::new(workspace_cache_binary())
        .args(["deps", "--bin", "api"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "deps --bin api failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let filtered_lock =
        fs::read_to_string(workspace_dir.join(".workspace-cache/Cargo.lock")).unwrap();

    // Filtered lock should have serde but not tokio
    assert!(
        filtered_lock.contains("serde"),
        "filtered lockfile should contain serde (api dependency)"
    );
    assert!(
        !filtered_lock.contains("tokio"),
        "filtered lockfile should NOT contain tokio (worker-only dependency)"
    );
}

#[test]
fn test_workspace_dependencies_filtering() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-deps-filter");
    fs::create_dir_all(&workspace_dir).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/api/src")).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/worker/src")).unwrap();

    fs::write(
        workspace_dir.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/api", "crates/worker"]
resolver = "2"

[workspace.dependencies]
serde = "1"
tokio = "1"
log = "0.4"
"#,
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/api/Cargo.toml"),
        r#"[package]
name = "api"
version = "0.1.0"
edition = "2021"

[dependencies]
serde.workspace = true
"#,
    )
    .unwrap();
    fs::write(workspace_dir.join("crates/api/src/main.rs"), "fn main() {}").unwrap();

    fs::write(
        workspace_dir.join("crates/worker/Cargo.toml"),
        r#"[package]
name = "worker"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio.workspace = true
log.workspace = true
"#,
    )
    .unwrap();
    fs::write(
        workspace_dir.join("crates/worker/src/main.rs"),
        "fn main() {}",
    )
    .unwrap();

    // Generate cache with only api
    let output = Command::new(workspace_cache_binary())
        .args(["deps", "--bin", "api"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "deps --bin api failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let workspace_toml =
        fs::read_to_string(workspace_dir.join(".workspace-cache/Cargo.toml")).unwrap();

    // Should only have serde in workspace.dependencies, not tokio or log
    assert!(
        workspace_toml.contains("serde"),
        "workspace.dependencies should contain serde"
    );
    assert!(
        !workspace_toml.contains("tokio"),
        "workspace.dependencies should NOT contain tokio"
    );
    assert!(
        !workspace_toml.contains("log"),
        "workspace.dependencies should NOT contain log"
    );
}

#[test]
fn test_bin_not_found_error() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-bin-not-found");
    fs::create_dir_all(&workspace_dir).unwrap();

    create_test_workspace(&workspace_dir);

    let output = Command::new(workspace_cache_binary())
        .args(["deps", "--bin", "nonexistent"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "should fail when binary not found"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found") || stderr.contains("nonexistent"),
        "error message should mention the missing binary"
    );
}

#[test]
fn test_multiple_bins() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-multi-bin");
    fs::create_dir_all(&workspace_dir).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/app/src/bin")).unwrap();

    fs::write(
        workspace_dir.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/app"]
resolver = "2"
"#,
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/app/Cargo.toml"),
        r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/app/src/bin/server.rs"),
        "fn main() {}",
    )
    .unwrap();
    fs::write(
        workspace_dir.join("crates/app/src/bin/cli.rs"),
        "fn main() {}",
    )
    .unwrap();

    // Test that we can target a specific binary
    let output = Command::new(workspace_cache_binary())
        .args(["deps", "--bin", "server"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "deps --bin server failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Test that we can target the other binary too
    let output = Command::new(workspace_cache_binary())
        .args(["deps", "--bin", "cli"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "deps --bin cli failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
