# Build Workspace with workspace-cache

A GitHub Actions composite action that builds Rust workspace binaries with optimized dependency caching using [workspace-cache](https://github.com/preiter93/workspace-cache).

**Note:** This action requires `workspace-cache` to be installed first. Use the [install-workspace-cache](../install-workspace-cache) action.

## Usage

```yaml
- name: Install workspace-cache
  uses: ./.github/actions/install-workspace-cache

- name: Build my binary
  uses: ./.github/actions/build-workspace
  with:
    binary: user
```

## Inputs

| Input | Description | Required | Default |
|-------|-------------|----------|---------|
| `binary` | Name of the binary to build | Yes | - |
| `profile` | Build profile (`release` or `debug`) | No | `release` |
| `working-directory` | Directory containing the workspace | No | `.` |

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
      
      - name: Install workspace-cache
        uses: ./.github/actions/install-workspace-cache
      
      - name: Build binary
        id: build
        uses: ./.github/actions/build-workspace
        with:
          binary: user
      
      - name: Run binary
        run: ${{ steps.build.outputs.binary-path }}
```

### Multiple Binaries with Matrix

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        binary: [user, order]
    steps:
      - uses: actions/checkout@v4
      
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install workspace-cache
        uses: ./.github/actions/install-workspace-cache
      
      - name: Build ${{ matrix.binary }}
        uses: ./.github/actions/build-workspace
        with:
          binary: ${{ matrix.binary }}
```

### Debug Build in Subdirectory

```yaml
- name: Install workspace-cache
  uses: ./.github/actions/install-workspace-cache

- name: Build debug binary
  uses: ./.github/actions/build-workspace
  with:
    binary: user
    profile: debug
    working-directory: ./services/user
```

### Using Latest Development Version

```yaml
- name: Install workspace-cache from git
  uses: ./.github/actions/install-workspace-cache
  with:
    install-from-git: true

- name: Build binary
  uses: ./.github/actions/build-workspace
  with:
    binary: user
```

## How It Works

1. **Generate minimal workspace** - Creates `.workspace-cache/` with dependency stubs
2. **Cache dependencies** - Uses GitHub Actions cache with Cargo.lock hash as key
3. **Build dependencies** - Compiles external dependencies (cached step)
4. **Copy real sources** - Replaces stubs with actual workspace code
5. **Build binary** - Compiles workspace crates with real sources

On subsequent runs, if dependencies haven't changed, step 3 completes in seconds instead of minutes.

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
