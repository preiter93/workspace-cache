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
| `workspace-cache-version` | Version of workspace-cache to install | No | `latest` |

## Outputs

| Output | Description |
|--------|-------------|
| `binary-path` | Path to the built binary (e.g., `.workspace-cache/target/release/api`) |

## Examples

### Basic Usage

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Build API service
        uses: ./.github/actions/build-workspace
        with:
          binary: api
      
      - name: Run binary
        run: ${{ steps.build.outputs.binary-path }}
```

### Debug Build in Subdirectory

```yaml
- name: Build debug binary
  uses: ./.github/actions/build-workspace
  with:
    binary: api
    profile: debug
    working-directory: ./services/api
```

### Specific workspace-cache Version

```yaml
- name: Build with specific version
  uses: ./.github/actions/build-workspace
  with:
    binary: api
    workspace-cache-version: 0.1.0
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
