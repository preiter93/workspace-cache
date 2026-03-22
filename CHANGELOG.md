# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0-alpha.2] - 2026-03-22

### 🚀 Features

- Resolve returns path
- *(action)* Add install from git option
- Split action into install & build

### 🐛 Bug Fixes

- *(lint)* Satisfy clippy
- Fix integration tests
- *(ci)* Default to latest version

### 📚 Documentation

- *(README)* Document usage in CI
- *(example)* Rename example crates to make dependencies clearer
- *(README)* Update CI usage section
- *(README)* Mention github action

### 🔧 Refactor

- [**breaking**] Rename resolve command to members

### ⚙️ Miscellaneous Tasks

- Add cargo toml metadata
- Add workflow to build example workspace
- Fix caching of the example workspace
- Update to checkout v6
- Remove debug build artifacts step
- Use resolve command to discover dependencies
- Move workspace build in separate action
- Update to checkout v6
- *(actions)* Improve output of actions

### Build

- Ignore relase changelog

## [0.1.0-alpha.1] - 2026-03-21

### 🚀 Features

- Support for member filtering
- Create minima workspace
- Add resolve flag to show dependent members
- Add support to generate dockerfile
- Change comment on first docker step
- Filter unused dependencies from minimal cargo.toml
- Filter workspace.dependencies and Cargo.lock
- Add --no-deps flag for faster builds
- [**breaking**] Change CLI from --package to --bin targeting
- *(dockerfile)* Allow setting the build profile
- [**breaking**] Rename --no-deps to --fast
- Add flag to specify from where to install from

### 🐛 Bug Fixes

- Include dependent workspace
- Use filtered workspace manifest in dockerfile builder stage
- Install workspace-cache from git in generated Dockerfile
- Exclude example folder
- *(lint)* Fix linter errors
- *(dockerfile)* Clean workspace to force rebuild
- *(lint)* Fix linter errors
- *(lint)* Fix linter errors

### 📚 Documentation

- Update Docker example in readme
- Simplify readme
- Change to install from repo
- Add testing section
- *(dockerfile)* Update dockerfile comment
- *(README)* Emphasize the dockerfile command

### 🔧 Refactor

- Use minijnina for dockerfile generation

### ⚙️ Miscellaneous Tasks

- Add another docker layer in generated docker
- Add license
- Use newest rust image
- Add CI and release workflows

### Build

- *(deps)* Set to prerelease version

### Deps

- Bump toml edit and minininja


