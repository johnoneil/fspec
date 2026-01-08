# CI and Release Setup

This document describes the CI/CD setup for cross-platform builds and releases.

## CI Workflow (`.github/workflows/ci.yml`)

The CI workflow runs on every push and pull request, and includes:

### Jobs

1. **Format Check** (`fmt`)
   - Runs only on Linux (Ubuntu 24.04)
   - Checks that all code is properly formatted with `cargo fmt`

2. **Test Suite** (`test`)
   - Runs on all three platforms:
     - Linux (Ubuntu 24.04) - `x86_64-unknown-linux-gnu`
     - Windows (Windows 2022) - `x86_64-pc-windows-msvc`
     - macOS (macOS 14) - `x86_64-apple-darwin`
   - Runs the full test suite with `cargo test`
   - Verifies the tool runs correctly with `cargo run`

3. **Build Release** (`build`)
   - Builds release binaries for all three platforms
   - Uploads artifacts that are retained for 90 days
   - Artifacts can be downloaded from the Actions UI for testing

## Release Workflow (`.github/workflows/release.yml`)

The release workflow creates GitHub releases with downloadable binaries.

### Triggers

- **Automatic**: Pushes to tags matching `v*` (e.g., `v0.1.0`)
- **Manual**: Can be triggered from the Actions UI with a version input

### Process

1. Builds release binaries for all platforms
2. Creates archives:
   - Linux/macOS: `.tar.gz` files
   - Windows: `.zip` files
3. Creates a GitHub release with:
   - Tag name matching the version
   - Release notes with download links
   - Attached binary archives

### Creating a Release

#### Option 1: Using Git Tags (Recommended)

```bash
# Update version in Cargo.toml files first
git tag v0.1.0
git push origin v0.1.0
```

This will automatically trigger the release workflow.

#### Option 2: Manual Trigger

1. Go to Actions → Release workflow
2. Click "Run workflow"
3. Enter the version tag (e.g., `v0.1.0`)
4. Click "Run workflow"

## Artifacts

### CI Artifacts

- Available in the Actions UI for each workflow run
- Retained for 90 days
- Useful for testing builds before creating a release

### Release Artifacts

- Attached to GitHub releases
- Permanently available for download
- Named with platform identifiers:
  - `fspec-x86_64-unknown-linux-gnu.tar.gz`
  - `fspec-x86_64-pc-windows-msvc.zip`
  - `fspec-x86_64-apple-darwin.tar.gz`

## Platform Support

Currently supported platforms (x86_64 only):

- **Linux**: `x86_64-unknown-linux-gnu`
- **Windows**: `x86_64-pc-windows-msvc`
- **macOS**: `x86_64-apple-darwin`

### Future Expansion

To add more platforms (e.g., ARM64, other architectures):

1. Add entries to the `matrix.include` arrays in both workflows
2. Update the artifact naming convention if needed
3. Test the builds

Example for adding ARM64 macOS:

```yaml
- os: macos-14
  target: aarch64-apple-darwin
  artifact_name: fspec-aarch64-apple-darwin
  binary_name: fspec
```

## Notes

- All builds use the stable Rust toolchain
- Cargo cache is used to speed up builds
- Format checking only runs on Linux (no need to check on all platforms)
- Tests run on all platforms to catch platform-specific issues
- Release binaries are built with `--release` flag for optimization

