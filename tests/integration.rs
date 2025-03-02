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
    fs::create_dir_all(dir.join("crates/lib-a/src")).unwrap();
    fs::create_dir_all(dir.join("crates/lib-b/src")).unwrap();

    fs::write(
        dir.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/lib-a", "crates/lib-b"]
resolver = "2"

[workspace.dependencies]
serde = "1"
log = "0.4"
"#,
    )
    .unwrap();

    fs::write(
        dir.join("crates/lib-a/Cargo.toml"),
        r#"[package]
name = "lib-a"
version = "0.1.0"
edition = "2021"

[dependencies]
serde.workspace = true
lib-b = { path = "../lib-b" }
"#,
    )
    .unwrap();

    fs::write(
        dir.join("crates/lib-a/src/lib.rs"),
        "pub fn hello() -> &'static str { \"hello\" }\n",
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
        cache_dir.join("crates/lib-a/Cargo.toml").exists(),
        "lib-a/Cargo.toml should exist"
    );
    assert!(
        cache_dir.join("crates/lib-b/Cargo.toml").exists(),
        "lib-b/Cargo.toml should exist"
    );
    assert!(
        cache_dir.join("crates/lib-a/src/lib.rs").exists(),
        "lib-a/src/lib.rs stub should exist"
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
        workspace_toml.contains("crates/lib-a"),
        "should include lib-a member"
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
        .arg("build")
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "build command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        workspace_dir.join("target").exists(),
        "target directory should exist after build"
    );
}

#[test]
fn test_build_command_release() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-release");
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
        workspace_dir.join("target/release").exists(),
        "target/release directory should exist after release build"
    );
}

#[test]
fn test_package_filter() {
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

    // Test filtering to only api + common
    let output = Command::new(workspace_cache_binary())
        .args(["deps", "-p", "api", "-p", "common"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(output.status.success());

    let cache_dir = workspace_dir.join(".workspace-cache");

    // Should have api and common, but not worker
    assert!(
        cache_dir.join("crates/api/Cargo.toml").exists(),
        "should include api"
    );
    assert!(
        cache_dir.join("crates/common/Cargo.toml").exists(),
        "should include common"
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
fn test_build_with_package_filter() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-build-filter");
    fs::create_dir_all(&workspace_dir).unwrap();

    create_test_workspace(&workspace_dir);

    let output = Command::new(workspace_cache_binary())
        .args(["build", "-p", "lib-a"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "build -p lib-a failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_binary_crate_creates_main_rs() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-bin");
    fs::create_dir_all(&workspace_dir).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/mybin/src")).unwrap();

    fs::write(
        workspace_dir.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/mybin"]
"#,
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/mybin/Cargo.toml"),
        r#"[package]
name = "mybin"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )
    .unwrap();
    fs::write(
        workspace_dir.join("crates/mybin/src/main.rs"),
        "fn main() { println!(\"hello\"); }",
    )
    .unwrap();

    let output = Command::new(workspace_cache_binary())
        .arg("deps")
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(output.status.success());

    let stub_main = workspace_dir.join(".workspace-cache/crates/mybin/src/main.rs");
    assert!(stub_main.exists(), "should create main.rs stub for binary");

    let content = fs::read_to_string(&stub_main).unwrap();
    assert!(content.contains("fn main()"), "stub should have main fn");
}

#[test]
fn test_shared_target_dir_caches_deps() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-cache");
    fs::create_dir_all(&workspace_dir).unwrap();

    create_test_workspace(&workspace_dir);

    // Generate deps
    let output = Command::new(workspace_cache_binary())
        .arg("deps")
        .current_dir(&workspace_dir)
        .output()
        .unwrap();
    assert!(output.status.success());

    // Build deps with shared target dir
    let cache_dir = workspace_dir.join(".workspace-cache");
    let target_dir = workspace_dir.join("target");

    let output = Command::new("cargo")
        .args(["build", "--release"])
        .env("CARGO_TARGET_DIR", &target_dir)
        .current_dir(&cache_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "cache build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Now build real workspace - should be fast since deps are cached
    let output = Command::new(workspace_cache_binary())
        .args(["build", "--release"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "real build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_deps_of_command() {
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

    // Test deps-of api: should return api, common, utils
    let output = Command::new(workspace_cache_binary())
        .args(["deps-of", "-p", "api"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
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

    // Test deps-of common: should return common, utils
    let output = Command::new(workspace_cache_binary())
        .args(["deps-of", "-p", "common"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let deps: Vec<&str> = stdout.lines().collect();

    assert!(deps.contains(&"common"), "should include common itself");
    assert!(deps.contains(&"utils"), "should include utils");
    assert_eq!(deps.len(), 2, "should only have common and utils");
}
