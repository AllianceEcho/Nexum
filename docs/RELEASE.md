# Release Process

## Current Status

There is no automated release workflow or version tag in this checkout. `.github/workflows/ci.yml` runs Rust formatting, check, tests, and Clippy on pushes and pull requests; it does not build frontend packages, bundle desktop installers, package the extension, or publish a GitHub Release. The packages and app manifests currently declare `0.1.0`, while the changelog still marks that release as TBD.

`Cargo.lock` is ignored by `.gitignore` and is not tracked; neither frontend package has a tracked lockfile. A fresh checkout can therefore resolve different dependency versions. Updating the local `Cargo.lock` does not change a release tag. Establish a tracked lockfile policy before claiming reproducible dependency resolution for release builds.

## Manual Release Checklist

The following steps describe what the current source tree requires for a release candidate. Tagging and publishing are manual decisions after validation.

1. Set the intended version in the Rust package manifests (`crates/*/Cargo.toml`, `apps/server/Cargo.toml`, `apps/cli/Cargo.toml`, and `apps/desktop/src-tauri/Cargo.toml`), and keep `apps/desktop/package.json`, `apps/desktop/src-tauri/tauri.conf.json`, `apps/extension/package.json`, and `apps/extension/manifest.json` in sync. The JSON-RPC protocol version is a separate compatibility number.
2. Move the release notes from `Unreleased` into a dated entry in both `CHANGELOG.md` and `CHANGELOG.zh-CN.md`.
3. Run the four Rust CI commands from the repository root as listed in the [Development Guide](DEVELOPMENT.md#rust-checks).
4. Run `pnpm install && pnpm build` in `apps/desktop`. Run `pnpm install && pnpm lint && pnpm build` in `apps/extension`. These TypeScript checks are not in CI.
5. Build the Rust command-line artifacts with `cargo build --release -p nexum-server -p nexum-cli`. The binaries are `target/release/nexum-server` and `target/release/nexum-cli`. For desktop installers, use the Tauri 2 CLI on each intended target platform after the frontend build (`cargo tauri build` from `apps/desktop`); Tauri's bundle configuration enables all supported targets for that platform.
6. Package the extension with its root `manifest.json`, `icons/`, and generated `dist/` directory. The manifest points to files in both locations, so `dist/` alone is not a loadable extension.
7. Smoke-test the binaries and native bundle against a local server. Serve a known file over HTTP, queue its URL, call `task start`, then check that the destination bytes and persisted `Completed` state match after the worker finishes. Verify that a task survives a restart with the same `--data-dir` and that an interrupted `Downloading` task returns to `Queued`; another `task start` begins the transfer from byte zero. Check that an active HTTP transfer rejects pause, resume, and remove, rejects data-directory and overlapping destinations, and that a failed transfer preserves any existing destination. The browser extension's HTTP `/jsonrpc` transport is not implemented by that server; do not treat extension task creation as a passing release check yet.
8. After verifying the actual artifacts, create a version tag and GitHub Release manually. Record target platforms and any incomplete integrations in the release notes.

For the Chinese version, see [RELEASE.zh-CN.md](RELEASE.zh-CN.md).
