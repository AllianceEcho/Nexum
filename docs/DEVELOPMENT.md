# Development Guide

## Environment

Nexum Core uses Rust. Client interfaces use TypeScript/React, and the desktop application uses Tauri.

Recommended tools: Rust stable, Cargo, Node.js LTS, and pnpm.

## Workspace

The Rust workspace lives at the repository root under crates/.

## Development Principles

- Keep module boundaries clear.
- Document and test public APIs.
- Make core state transitions testable.
- Keep UI logic out of the Core.
- Keep engine-specific details out of the Domain Model.
- Submit an RFC before destructive architectural changes.

## Common Commands

- cargo check --workspace
- cargo test --workspace
- cargo fmt --all
- cargo clippy --workspace --all-targets

More complete local development and debugging instructions will be added as the project becomes runnable.

For the Chinese version, see [DEVELOPMENT.zh-CN.md](DEVELOPMENT.zh-CN.md).
