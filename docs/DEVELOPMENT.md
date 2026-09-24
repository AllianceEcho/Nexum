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
cargo run -p nexum-cli -- task create task-1 https://example.com/ ./example.html
cargo run -p nexum-cli -- task queue task-1
cargo run -p nexum-cli -- task start
```

The built binaries are named `nexum-server` and `nexum-cli`. Use `cargo run -p nexum-server -- --help` and `cargo run -p nexum-cli -- --help` for current flags and commands. The CLI defaults to `127.0.0.1:39100`; put `--server ADDR` before `task` or `server` to override it.

The server stores task metadata and progress in `nexum.sqlite` under `--data-dir` (default `./data`, relative to the server's working directory). It creates the directory if needed and holds a lock on `nexum.lock` there until exit, so only one server can use that directory. It opens the database and recovers tasks before listening; directory, lock, database, or recovery errors stop startup. On restart, previously downloading, paused, or retrying tasks become queued in memory and in SQLite, but the server does not start them automatically. `--max-connections` is displayed but not enforced, and `--require-auth` does not enable authentication; `server.auth` reports `none`.

`task start` selects a queued HTTP/HTTPS task and returns its ID after launching a worker. Check `task list` again for `Completed` and the downloaded file; completion is asynchronous. The server rejects destinations inside its data directory, symbolic-link destinations, and overlapping active destinations. The worker stages a complete response in a `.part` file in the destination directory before renaming it into place. A failed transfer keeps an existing destination, logs an error on the server, and is queued again while the retry policy allows; another `task start` is required to retry it. Progress resets to zero when the task is claimed, then receives its final byte count only after a successful transfer. Active HTTP transfers cannot be paused, resumed, or removed. Magnet and local-file sources have no transfer engine yet. Restarted transfers begin from byte zero.

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
