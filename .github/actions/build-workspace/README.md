# Build Workspace with workspace-cache

A GitHub Actions composite action that builds Rust workspace binaries with optimized dependency caching using [workspace-cache](https://github.com/preiter93/workspace-cache).

**Note:** This action requires `workspace-cache` to be installed first. Use the [install-workspace-cache](../install-workspace-cache) action.

## Usage

```yaml
- name: Install workspace-cache
  uses: ./.github/actions/install-workspace-cache

- name: Build
  uses: ./.github/actions/build-workspace
  with:
    binary: user
    working-directory: services

- name: Run tests
  working-directory: services/.workspace-cache
  run: cargo test -p user --verbose

- name: Clippy
  working-directory: services/.workspace-cache
  run: cargo clippy -p user -- -D warnings
```

**Important:** Run tests and other cargo commands from `services/.workspace-cache` where the complete workspace is built.

## Inputs

| Input | Description | Required | Default |
|-------|-------------|----------|---------|
| `binary` | Name of the binary to build | Yes | - |
| `profile` | Build profile (`release` or `debug`) | No | `debug` |
| `working-directory` | Directory containing the workspace | No | `.` |
| `build-tests` | Build test targets to cache dev-dependencies | No | `true` |

## Outputs

| Output | Description |
|--------|-------------|
| `binary-path` | Path to the built binary (e.g., `.workspace-cache/target/debug/user`) |

## Examples

### Matrix Build with Tests

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        service: [user, order]
    steps:
      - uses: actions/checkout@v6
      
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      
      - name: Install workspace-cache
        uses: ./.github/actions/install-workspace-cache
      
      - name: Build ${{ matrix.service }}
        uses: ./.github/actions/build-workspace
        with:
          binary: ${{ matrix.service }}
          working-directory: services
      
      - name: Run tests
        working-directory: services/.workspace-cache
        run: cargo test -p ${{ matrix.service }} --verbose
      
      - name: Clippy
        working-directory: services/.workspace-cache
        run: cargo clippy -p ${{ matrix.service }} -- -D warnings
```

### Binary-Only Build (Skip Tests)

```yaml
- name: Build binary only
  uses: ./.github/actions/build-workspace
  with:
    binary: user
    build-tests: false  # Skip test dependencies for faster builds
```



### Release Build

```yaml
- name: Build release binary
  uses: ./.github/actions/build-workspace
  with:
    binary: user
    profile: release

- name: Run binary
  run: .workspace-cache/target/release/user
```

## How It Works

1. Generates minimal workspace with dependency stubs in `.workspace-cache/` inside the working directory
2. Caches and builds dependencies (fast when dependencies unchanged)
3. Copies real source code and workspace members to `.workspace-cache/`
4. Builds the binary with tests

Run tests and other cargo commands from `.workspace-cache` where all sources and dependencies are built.

Dependencies are cached using `{OS}-workspace-cache-deps-{binary}-{profile}-{Cargo.lock hash}`, so they only rebuild when the binary, profile, or Cargo.lock changes.

## License

MIT