# Development Guide

## Environment

Nexum Core uses Rust. Client interfaces use TypeScript/React, and the desktop application uses Tauri 2.

Recommended tools:

- Rust stable
- Cargo
- Node.js LTS
- pnpm

## Workspace

The Rust workspace lives at the repository root:

```text
crates/
├── core/
├── domain/
├── task/
├── scheduler/
├── storage/
├── resolver/
├── protocol/
├── plugin/
├── security/
├── media/
└── engine/

apps/
├── server/
├── cli/
├── desktop/
└── extension/
```

## Development Principles

- Keep module boundaries clear.
- Document and test public APIs.
- Make core state transitions testable.
- Keep UI logic out of the Core.
- Keep engine-specific details out of the Domain Model.
- Submit an RFC before destructive architectural changes.
- Keep documentation synchronized with implemented behavior.

## Common Commands

```bash
cargo check --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets
```

For CI-equivalent validation:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Running the Server

The server listens on `127.0.0.1:39100` by default. The address and runtime configuration are exposed by the server command-line interface.

## Running the CLI

The CLI connects to the local server by default. A different server can be selected with the server option.

Example:

```bash
nexum task list
nexum task create task-1 https://example.com/file.bin ./file.bin
nexum task queue task-1
nexum task start
```

## Desktop and Browser

Desktop development lives under `apps/desktop` and uses Tauri 2 with a React frontend.

Browser integration lives under `apps/extension` and uses a Manifest V3 extension.

## Tests

Core behavior should remain testable without network access. Network and engine behavior belongs in controlled integration tests.

For the Chinese version, see [DEVELOPMENT.zh-CN.md](DEVELOPMENT.zh-CN.md).
