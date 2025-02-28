use std::fs;
use std::path::Path;
use std::process::Command;

fn workspace_cache_binary() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    path.pop(); // remove deps
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
serde = "1"
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
log = "0.4"

[dev-dependencies]
tempfile = "3"

[build-dependencies]
cc = "1"
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
        cache_dir.join("src/main.rs").exists(),
        "src/main.rs should exist"
    );

    let cargo_toml = fs::read_to_string(cache_dir.join("Cargo.toml")).unwrap();

    assert!(
        cargo_toml.contains("[workspace]"),
        "should have empty workspace section"
    );
    assert!(
        cargo_toml.contains("serde"),
        "should include serde dependency"
    );
    assert!(cargo_toml.contains("log"), "should include log dependency");
    assert!(
        !cargo_toml.contains("lib-a"),
        "should not include path dependency lib-a"
    );
    assert!(
        !cargo_toml.contains("lib-b"),
        "should not include path dependency lib-b"
    );

    assert!(
        cargo_toml.contains("[dev-dependencies]"),
        "should have dev-dependencies section"
    );
    assert!(
        cargo_toml.contains("tempfile"),
        "should include tempfile dev-dependency"
    );

    assert!(
        cargo_toml.contains("[build-dependencies]"),
        "should have build-dependencies section"
    );
    assert!(
        cargo_toml.contains("cc"),
        "should include cc build-dependency"
    );

    let main_rs = fs::read_to_string(cache_dir.join("src/main.rs")).unwrap();
    assert!(main_rs.contains("fn main()"), "should have main function");
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
fn test_features_are_preserved() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-features");
    fs::create_dir_all(&workspace_dir).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/app/src")).unwrap();

    fs::write(
        workspace_dir.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/app"]
"#,
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/app/Cargo.toml"),
        r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"], default-features = false }
"#,
    )
    .unwrap();

    fs::write(workspace_dir.join("crates/app/src/lib.rs"), "").unwrap();

    let output = Command::new(workspace_cache_binary())
        .arg("deps")
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(output.status.success());

    let cargo_toml = fs::read_to_string(workspace_dir.join(".workspace-cache/Cargo.toml")).unwrap();

    assert!(
        cargo_toml.contains("derive"),
        "should preserve features: {}",
        cargo_toml
    );
    assert!(
        cargo_toml.contains("default-features = false"),
        "should preserve default-features: {}",
        cargo_toml
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
serde = "1"
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
axum = "0.7"
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
tokio = "1"
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

    let cargo_toml = fs::read_to_string(workspace_dir.join(".workspace-cache/Cargo.toml")).unwrap();

    assert!(
        cargo_toml.contains("axum"),
        "should include axum (api dep): {}",
        cargo_toml
    );
    assert!(
        cargo_toml.contains("serde"),
        "should include serde (common dep): {}",
        cargo_toml
    );
    assert!(
        !cargo_toml.contains("tokio"),
        "should NOT include tokio (worker dep): {}",
        cargo_toml
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
