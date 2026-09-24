# Nexum

[English](README.md) · [简体中文](README.zh-CN.md)

> A Rust workspace for download task management and client integrations.

Nexum is under active development. Its runnable path is a local TCP JSON-RPC server for creating and controlling tasks. The server stores tasks in SQLite. Starting a queued HTTP/HTTPS task downloads it to the destination through a background worker.

[![CI](https://github.com/liveait/Nexum/actions/workflows/ci.yml/badge.svg)](https://github.com/liveait/Nexum/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Current status

The CLI and early Tauri desktop client can call the local server; the desktop client requires the server to run separately. The server opens `nexum.sqlite` under its data directory (default `./data`) and recovers stored tasks on startup. `task.start` selects HTTP/HTTPS tasks for a real download. Magnet and local-file sources can be created but have no transfer path yet. The browser extension prototype calls HTTP `/jsonrpc`, which the TCP-only server does not provide, so it cannot submit tasks to this server yet.

Authentication, TLS, rate limiting, executable plugins, and real media processing are not active in the running server. See [Architecture](docs/ARCHITECTURE.md) for the code-level boundaries and current call paths, and the [Development Plan](docs/DEVELOPMENT_PLAN.md) for remaining integration work.

## Run the local task flow

Requirements: Rust with edition 2024 support (Rust 1.85 or newer) and Cargo. From the repository root, start the server in one terminal:

```bash
cargo run -p nexum-server
```

In another terminal, use the CLI:

```bash
cargo run -p nexum-cli -- task create task-1 https://example.com/ ./example.html
cargo run -p nexum-cli -- task queue task-1
cargo run -p nexum-cli -- task start
cargo run -p nexum-cli -- task list
```

`task start` returns after launching an HTTP worker. On success it writes `./example.html`, records the final byte count, and marks the task `Completed`; `task list` may show `Downloading` until then. The destination is replaced only after the full response is staged in a temporary `.part` file. There is no incremental progress, automatic retry dispatch, or pause/cancel for an active HTTP transfer. A task survives a server restart, but an interrupted `Downloading` task is reset to `Queued` and starts from the beginning only after another `task start`. The server binds to `127.0.0.1:39100` by default; use `--port PORT` on the server or `--server ADDR` on the CLI to change the connection. The server does not enforce authentication, even when `--require-auth` is set, so do not expose it to an untrusted network. See the [Development Guide](docs/DEVELOPMENT.md) for client setup, all Rust checks, and platform dependencies.

## Documentation and contribution

- [Architecture](docs/ARCHITECTURE.md)
- [Development Guide](docs/DEVELOPMENT.md)
- [Development Plan](docs/DEVELOPMENT_PLAN.md)
- [Changelog](CHANGELOG.md)
- [Contributing](CONTRIBUTING.md)
- [Governance](GOVERNANCE.md)
- [Chinese README](README.zh-CN.md)

Nexum is licensed under the [MIT License](LICENSE). Third-party dependencies retain their respective licenses.
