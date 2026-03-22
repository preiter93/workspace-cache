# Install workspace-cache

A GitHub Actions composite action that installs the [workspace-cache](https://github.com/preiter93/workspace-cache) tool with binary caching for faster workflow runs.

## Usage

```yaml
- name: Install workspace-cache
  uses: ./.github/actions/install-workspace-cache
```

## Inputs

| Input | Description | Required | Default |
|-------|-------------|----------|---------|
| `version` | Version of workspace-cache to install | No | `0.1.0-alpha.1` |
| `install-from-git` | Install from git instead of crates.io | No | `false` |

## Examples

### Basic Usage (Default Version)

```yaml
- name: Install workspace-cache
  uses: ./.github/actions/install-workspace-cache
```

### Specific Version

```yaml
- name: Install workspace-cache
  uses: ./.github/actions/install-workspace-cache
  with:
    version: 0.1.0
```

### Install from Git (Latest Development Version)

```yaml
- name: Install workspace-cache
  uses: ./.github/actions/install-workspace-cache
  with:
    install-from-git: true
```

### Complete Workflow Example

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install workspace-cache
        uses: ./.github/actions/install-workspace-cache
      
      - name: Build binary
        uses: ./.github/actions/build-workspace
        with:
          binary: user
```

## Caching

The action caches the compiled `workspace-cache` binary in `~/.cargo/bin/` with:
- **Key for crates.io**: `workspace-cache-{version}-{OS}`
- **Key for git**: `workspace-cache-git-{OS}`

This means:
- First run: Compiles and installs workspace-cache (~1-2 minutes)
- Subsequent runs: Restores from cache (< 5 seconds)
- Cache is OS-specific to handle platform differences

## License

MIT
