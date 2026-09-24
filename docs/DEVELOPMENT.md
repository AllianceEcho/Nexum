# Development Guide

## Environment

Nexum is a Rust 2024-edition workspace with a Tauri 2 desktop client. The desktop frontend uses TypeScript and React; the browser extension is written in TypeScript.

- Rust stable (1.85 or newer), Cargo, `rustfmt`, and Clippy
- Node.js LTS and pnpm for the two frontend packages
- Tauri 2 system dependencies and the Tauri CLI when running or bundling the native desktop app

The Ubuntu CI job installs `libgtk-3-dev` and `libwebkit2gtk-4.1-dev` before checking the Rust workspace. Platform-specific Tauri prerequisites also apply to local builds.

## Workspace

The root `Cargo.toml` includes all crates below and `apps/desktop/src-tauri`. The browser extension is a separate frontend package.

```text
crates/
  core/ domain/ task/ scheduler/ storage/ resolver/
  protocol/ plugin/ security/ media/ engine/
apps/
  server/                 # nexum-server binary
  cli/                    # nexum-cli binary
  desktop/                # React/Vite and src-tauri/
  extension/              # Manifest V3 extension
```

Keep UI logic out of Core and engine-specific details out of the Domain Model. Add focused tests and update both language versions of affected documentation when behavior changes. Check [Contributing](../CONTRIBUTING.md) for changes that require an RFC.

## Rust Checks

From the repository root, run the same commands as `.github/workflows/ci.yml`:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

CI runs these Rust checks on Ubuntu. It does not currently build or check the TypeScript packages or produce release artifacts.

## Server and CLI

Start the local server in one terminal:

```bash
cargo run -p nexum-server -- --port 39100
```

It binds to `127.0.0.1:39100` by default and serves newline-delimited JSON-RPC 2.0 over TCP. Run the CLI in another terminal:

```bash
cargo run -p nexum-cli -- task list
cargo run -p nexum-cli -- task create task-1 https://example.com/file.bin ./file.bin
cargo run -p nexum-cli -- task queue task-1
cargo run -p nexum-cli -- task start
```

The built binaries are named `nexum-server` and `nexum-cli`. Use `cargo run -p nexum-server -- --help` and `cargo run -p nexum-cli -- --help` for current flags and commands. The CLI defaults to `127.0.0.1:39100`; put `--server ADDR` before `task` or `server` to override it.

The running server constructs `Core` with an in-memory repository, so tasks do not survive a restart. Its `--data-dir` option only creates a directory at present. `--max-connections` is displayed but not enforced, and `--require-auth` does not enable authentication; `server.auth` reports `none`. The exposed `task start` path uses the in-memory engine and does not download the example file.

## Desktop

Start the Vite frontend from `apps/desktop`:

```bash
cd apps/desktop
pnpm install
pnpm dev
```

Vite uses port `1420`. The frontend calls Tauri commands, so testing the full application requires a running Nexum server and a native Tauri window. With the Tauri 2 CLI installed, keep Vite running and start `cargo tauri dev` from `apps/desktop` in another terminal. `pnpm build` runs the TypeScript compiler and Vite build; this is a separate check from Rust CI.

## Browser Extension

From `apps/extension`:

```bash
cd apps/extension
pnpm install
pnpm lint
pnpm build
```

The extension's `pnpm dev` watches and rebuilds files. Load `apps/extension` as the unpacked extension: its root `manifest.json` references assets under `dist/` and icons under `icons/`.

The extension currently sends HTTP requests to `/jsonrpc`, while `nexum-server` only accepts TCP JSON-RPC. Its send-to-Nexum action cannot create tasks against the current server until an HTTP bridge or matching transport is implemented.

For the Chinese version, see [DEVELOPMENT.zh-CN.md](DEVELOPMENT.zh-CN.md).
