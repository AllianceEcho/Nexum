# Nexum

[English](README.md) · [简体中文](README.zh-CN.md)

> A Rust workspace for download task management and client integrations.

Nexum is under active development. Its runnable path is a local TCP JSON-RPC server for creating and controlling tasks. The server uses an in-memory repository and engine: starting a task changes its state but does **not** download its source to the destination.

[![CI](https://github.com/liveait/Nexum/actions/workflows/ci.yml/badge.svg)](https://github.com/liveait/Nexum/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Current status

The CLI and early Tauri desktop client can call the local server; the desktop client requires the server to run separately. SQLite storage and an HTTP engine exist as libraries but are not selected by the server. The browser extension prototype calls HTTP `/jsonrpc`, which the TCP-only server does not provide, so it cannot submit tasks to this server yet.

Authentication, TLS, rate limiting, executable plugins, and real media processing are not active in the running server. See [Architecture](docs/ARCHITECTURE.md) for the code-level boundaries and current call paths, and the [Development Plan](docs/DEVELOPMENT_PLAN.md) for remaining integration work.

## Run the local task flow

Requirements: Rust with edition 2024 support (Rust 1.85 or newer) and Cargo. From the repository root, start the server in one terminal:

```bash
cargo run -p nexum-server
```

In another terminal, use the CLI:

```bash
cargo run -p nexum-cli -- task create task-1 https://example.com/file.bin ./file.bin
cargo run -p nexum-cli -- task queue task-1
cargo run -p nexum-cli -- task start
cargo run -p nexum-cli -- task list
```

`task start` marks the queued task as `Downloading` through the in-memory engine. It does not create `./file.bin`, and tasks are lost when the server exits. The server binds to `127.0.0.1:39100` by default; use `--port PORT` on the server or `--server ADDR` on the CLI to change the connection. The server does not enforce authentication, even when `--require-auth` is set, so do not expose it to an untrusted network. See the [Development Guide](docs/DEVELOPMENT.md) for client setup, all Rust checks, and platform dependencies.

## Documentation and contribution

- [Architecture](docs/ARCHITECTURE.md)
- [Development Guide](docs/DEVELOPMENT.md)
- [Development Plan](docs/DEVELOPMENT_PLAN.md)
- [Changelog](CHANGELOG.md)
- [Contributing](CONTRIBUTING.md)
- [Governance](GOVERNANCE.md)
- [Chinese README](README.zh-CN.md)

Nexum is licensed under the [MIT License](LICENSE). Third-party dependencies retain their respective licenses.
