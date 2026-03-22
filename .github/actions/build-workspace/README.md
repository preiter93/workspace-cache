# Build Workspace with workspace-cache

A GitHub Actions composite action that builds Rust workspace binaries with optimized dependency caching using [workspace-cache](https://github.com/preiter93/workspace-cache).

## Usage

```yaml
- name: Build my binary
  uses: ./.github/actions/build-workspace
  with:
    binary: user
    profile: release
```

## Inputs

| Input | Description | Required | Default |
|-------|-------------|----------|---------|
| `binary` | Name of the binary to build | Yes | - |
| `profile` | Build profile (`release` or `debug`) | No | `release` |
| `working-directory` | Directory containing the workspace | No | `.` |
| `workspace-cache-version` | Version of workspace-cache to install | No | `0.1.0-alpha.1` |
| `install-from-git` | Install workspace-cache from git instead of crates.io | No | `false` |

## Outputs

| Output | Description |
|--------|-------------|
| `binary-path` | Path to the built binary (e.g., `.workspace-cache/target/release/user`) |

## Examples

### Basic Usage

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Build User service
        uses: ./.github/actions/build-workspace
        with:
          binary: user
      
      - name: Run binary
        run: ${{ steps.build.outputs.binary-path }}
```

### Debug Build in Subdirectory

```yaml
- name: Build debug binary
  uses: ./.github/actions/build-workspace
  with:
    binary: user
    profile: debug
    working-directory: ./services/user
```

### Specific workspace-cache Version

```yaml
- name: Build with specific version
  uses: ./.github/actions/build-workspace
  with:
    binary: user
    workspace-cache-version: 0.1.0
```

### Install from Git 

```yaml
- name: Build with latest from git
  uses: ./.github/actions/build-workspace
  with:
    binary: user
    install-from-git: true
```

## Caching

The action automatically caches:
- Compiled dependencies in `.workspace-cache/target`
- Cache key: `{OS}-workspace-cache-deps-{Cargo.lock hash}`

This means:
- Different binaries with same dependencies share cache
- Source code changes don't invalidate dependency cache
- Only dependency changes trigger full rebuild

## License

MIT
