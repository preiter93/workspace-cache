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
    fs::create_dir_all(dir.join("crates/user/src")).unwrap();
    fs::create_dir_all(dir.join("crates/pkg-a/src")).unwrap();

    fs::write(
        dir.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/user", "crates/pkg-a"]
resolver = "2"

[workspace.dependencies]
serde = "1"
log = "0.4"
"#,
    )
    .unwrap();

    fs::write(
        dir.join("crates/user/Cargo.toml"),
        r#"[package]
name = "user"
version = "0.1.0"
edition = "2021"

[dependencies]
serde.workspace = true
pkg-a = { path = "../pkg-a" }
"#,
    )
    .unwrap();

    fs::write(
        dir.join("crates/user/src/main.rs"),
        "fn main() { println!(\"hello\"); }\n",
    )
    .unwrap();

    fs::write(
        dir.join("crates/pkg-a/Cargo.toml"),
        r#"[package]
name = "pkg-a"
version = "0.1.0"
edition = "2021"

[dependencies]
log.workspace = true
"#,
    )
    .unwrap();

    fs::write(dir.join("crates/pkg-a/src/lib.rs"), "// pkg-a\n").unwrap();
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
        cache_dir.join("crates/user/Cargo.toml").exists(),
        "user/Cargo.toml should exist"
    );
    assert!(
        cache_dir.join("crates/pkg-a/Cargo.toml").exists(),
        "pkg-a/Cargo.toml should exist"
    );
    assert!(
        cache_dir.join("crates/user/src/main.rs").exists(),
        "user/src/main.rs stub should exist"
    );
    assert!(
        cache_dir.join("crates/pkg-a/src/lib.rs").exists(),
        "pkg-a/src/lib.rs stub should exist"
    );

    let workspace_toml = fs::read_to_string(cache_dir.join("Cargo.toml")).unwrap();
    assert!(
        workspace_toml.contains("[workspace]"),
        "should have workspace section"
    );
    assert!(
        workspace_toml.contains("crates/user"),
        "should include user member"
    );
    assert!(
        workspace_toml.contains("crates/pkg-a"),
        "should include pkg-a member"
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
        workspace_dir.join("target/debug/user").exists()
            || workspace_dir.join("target/debug/user.exe").exists(),
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
        "build --release failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        workspace_dir.join("target/release/user").exists()
            || workspace_dir.join("target/release/user.exe").exists(),
        "release binary should be built"
    );
}

#[test]
fn test_bin_filter() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-filter");
    fs::create_dir_all(&workspace_dir).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/user/src")).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/order/src")).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/pkg-a/src")).unwrap();

    fs::write(
        workspace_dir.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/user", "crates/order", "crates/pkg-a"]
resolver = "2"

[workspace.dependencies]
axum = "0.7"
tokio = "1"
serde = "1"
"#,
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/pkg-a/Cargo.toml"),
        r#"[package]
name = "pkg-a"
version = "0.1.0"
edition = "2021"

[dependencies]
serde.workspace = true
"#,
    )
    .unwrap();
    fs::write(workspace_dir.join("crates/pkg-a/src/lib.rs"), "").unwrap();

    fs::write(
        workspace_dir.join("crates/user/Cargo.toml"),
        r#"[package]
name = "user"
version = "0.1.0"
edition = "2021"

[dependencies]
pkg-a = { path = "../pkg-a" }
axum.workspace = true
"#,
    )
    .unwrap();
    fs::write(
        workspace_dir.join("crates/user/src/main.rs"),
        "fn main() {}",
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/order/Cargo.toml"),
        r#"[package]
name = "order"
version = "0.1.0"
edition = "2021"

[dependencies]
pkg-a = { path = "../pkg-a" }
tokio.workspace = true
"#,
    )
    .unwrap();
    fs::write(
        workspace_dir.join("crates/order/src/main.rs"),
        "fn main() {}",
    )
    .unwrap();

    // Test filtering to only user (which depends on pkg-a)
    let output = Command::new(workspace_cache_binary())
        .args(["deps", "--bin", "user"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "deps --bin user failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let cache_dir = workspace_dir.join(".workspace-cache");

    // Should have user and pkg-a, but not order
    assert!(
        cache_dir.join("crates/user/Cargo.toml").exists(),
        "should include user"
    );
    assert!(
        cache_dir.join("crates/pkg-a/Cargo.toml").exists(),
        "should include pkg-a (dependency of user)"
    );
    assert!(
        !cache_dir.join("crates/order").exists(),
        "should NOT include order"
    );

    let workspace_toml = fs::read_to_string(cache_dir.join("Cargo.toml")).unwrap();
    assert!(
        workspace_toml.contains("crates/user"),
        "workspace should list user"
    );
    assert!(
        workspace_toml.contains("crates/pkg-a"),
        "workspace should list pkg-a"
    );
    assert!(
        !workspace_toml.contains("crates/order"),
        "workspace should NOT list order"
    );
}

#[test]
fn test_build_with_bin_filter() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-build-filter");
    fs::create_dir_all(&workspace_dir).unwrap();

    create_test_workspace(&workspace_dir);

    let output = Command::new(workspace_cache_binary())
        .args(["build", "--bin", "user"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "build --bin user failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_binary_crate_creates_main_rs() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-bin");
    fs::create_dir_all(&workspace_dir).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/user/src")).unwrap();

    fs::write(
        workspace_dir.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/user"]
resolver = "2"
"#,
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/user/Cargo.toml"),
        r#"[package]
name = "user"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/user/src/main.rs"),
        "fn main() { println!(\"real code\"); }",
    )
    .unwrap();

    let output = Command::new(workspace_cache_binary())
        .arg("deps")
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(output.status.success());

    let stub_main = workspace_dir.join(".workspace-cache/crates/user/src/main.rs");
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
    let workspace_dir = temp.path().join("test-workspace-resolve");
    fs::create_dir_all(&workspace_dir).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/user/src")).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/order/src")).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/pkg-a/src")).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/pkg-b/src")).unwrap();

    fs::write(
        workspace_dir.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/user", "crates/order", "crates/pkg-a", "crates/pkg-b"]
resolver = "2"
"#,
    )
    .unwrap();

    // pkg-a depends on pkg-b
    fs::write(
        workspace_dir.join("crates/pkg-a/Cargo.toml"),
        r#"[package]
name = "pkg-a"
version = "0.1.0"
edition = "2021"

[dependencies]
pkg-b = { path = "../pkg-b" }
"#,
    )
    .unwrap();
    fs::write(workspace_dir.join("crates/pkg-a/src/lib.rs"), "").unwrap();

    // pkg-b has no workspace deps
    fs::write(
        workspace_dir.join("crates/pkg-b/Cargo.toml"),
        r#"[package]
name = "pkg-b"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();
    fs::write(workspace_dir.join("crates/pkg-b/src/lib.rs"), "").unwrap();

    // user depends on pkg-a (which depends on pkg-b)
    fs::write(
        workspace_dir.join("crates/user/Cargo.toml"),
        r#"[package]
name = "user"
version = "0.1.0"
edition = "2021"

[dependencies]
pkg-a = { path = "../pkg-a" }
"#,
    )
    .unwrap();
    fs::write(
        workspace_dir.join("crates/user/src/main.rs"),
        "fn main() {}",
    )
    .unwrap();

    // order depends on pkg-a
    fs::write(
        workspace_dir.join("crates/order/Cargo.toml"),
        r#"[package]
name = "order"
version = "0.1.0"
edition = "2021"

[dependencies]
pkg-a = { path = "../pkg-a" }
"#,
    )
    .unwrap();
    fs::write(
        workspace_dir.join("crates/order/src/main.rs"),
        "fn main() {}",
    )
    .unwrap();

    // Test resolve user: should return user, pkg-a, pkg-b
    let output = Command::new(workspace_cache_binary())
        .args(["resolve", "--bin", "user"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "resolve --bin user failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let deps: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.split_whitespace().nth(1))
        .collect();

    assert!(deps.contains(&"user"), "should include user itself");
    assert!(deps.contains(&"pkg-a"), "should include pkg-a (direct dep)");
    assert!(
        deps.contains(&"pkg-b"),
        "should include pkg-b (transitive dep)"
    );
    assert!(!deps.contains(&"order"), "should NOT include order");

    // Test resolve order: should return order, pkg-a, pkg-b
    let output = Command::new(workspace_cache_binary())
        .args(["resolve", "--bin", "order"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let deps: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.split_whitespace().nth(1))
        .collect();

    assert!(deps.contains(&"order"), "should include order itself");
    assert!(deps.contains(&"pkg-a"), "should include pkg-a");
    assert!(deps.contains(&"pkg-b"), "should include pkg-b");
    assert!(!deps.contains(&"user"), "should NOT include user");
}

#[test]
fn test_cargo_lock_filtering() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-lock-filter");
    fs::create_dir_all(&workspace_dir).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/user/src")).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/order/src")).unwrap();

    fs::write(
        workspace_dir.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/user", "crates/order"]
resolver = "2"
"#,
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/user/Cargo.toml"),
        r#"[package]
name = "user"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1"
"#,
    )
    .unwrap();
    fs::write(
        workspace_dir.join("crates/user/src/main.rs"),
        "fn main() {}",
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/order/Cargo.toml"),
        r#"[package]
name = "order"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["rt"] }
"#,
    )
    .unwrap();
    fs::write(
        workspace_dir.join("crates/order/src/main.rs"),
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

    // Generate cache with only user
    let output = Command::new(workspace_cache_binary())
        .args(["deps", "--bin", "user"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "deps --bin user failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let filtered_lock =
        fs::read_to_string(workspace_dir.join(".workspace-cache/Cargo.lock")).unwrap();

    // Filtered lock should have serde but not tokio
    assert!(
        filtered_lock.contains("serde"),
        "filtered lockfile should contain serde (user dependency)"
    );
    assert!(
        !filtered_lock.contains("tokio"),
        "filtered lockfile should NOT contain tokio (order-only dependency)"
    );
}

#[test]
fn test_workspace_dependencies_filtering() {
    let temp = tempfile::tempdir().unwrap();
    let workspace_dir = temp.path().join("test-workspace-deps-filter");
    fs::create_dir_all(&workspace_dir).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/user/src")).unwrap();
    fs::create_dir_all(workspace_dir.join("crates/order/src")).unwrap();

    fs::write(
        workspace_dir.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/user", "crates/order"]
resolver = "2"

[workspace.dependencies]
serde = "1"
tokio = "1"
log = "0.4"
"#,
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/user/Cargo.toml"),
        r#"[package]
name = "user"
version = "0.1.0"
edition = "2021"

[dependencies]
serde.workspace = true
"#,
    )
    .unwrap();
    fs::write(
        workspace_dir.join("crates/user/src/main.rs"),
        "fn main() {}",
    )
    .unwrap();

    fs::write(
        workspace_dir.join("crates/order/Cargo.toml"),
        r#"[package]
name = "order"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio.workspace = true
log.workspace = true
"#,
    )
    .unwrap();
    fs::write(
        workspace_dir.join("crates/order/src/main.rs"),
        "fn main() {}",
    )
    .unwrap();

    // Generate cache with only user
    let output = Command::new(workspace_cache_binary())
        .args(["deps", "--bin", "user"])
        .current_dir(&workspace_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "deps --bin user failed: {}",
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
