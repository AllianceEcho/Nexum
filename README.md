# Nexum

[English](README.md) · [简体中文](README.zh-CN.md)

> **An open, local-first download platform for desktop, browser, CLI, and automation.**

Nexum is a Rust-based download platform built around a stable **Core + Protocol** boundary. It brings task management, scheduling, persistence, resolvers, download engines, a TCP server, CLI, desktop client, browser integration, and plugin foundations into one extensible architecture.

[![CI](https://github.com/liveait/Nexum/actions/workflows/ci.yml/badge.svg)](https://github.com/liveait/Nexum/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> **Status:** Nexum is under active development. The repository now has working foundations across Core, Protocol, Server, CLI, Desktop, Browser, Security, Plugins, and Media. Plugin lifecycle, additional resolver/engine integrations, media workflows, automation, and remote management remain in development.

## Why Nexum?

Nexum is designed around a few simple rules:

- **Core-first** — task lifecycle, scheduling, state, events, and orchestration live in a testable core.
- **Protocol-first** — clients communicate through stable protocol operations instead of depending on Core internals.
- **Local-first** — the default setup is easy to run locally while remote connections remain an architectural option.
- **Engine-agnostic** — download engines sit behind a common adapter boundary.
- **Extensible** — Resolver, Engine, Plugin, Capability, and SDK boundaries are explicit.
- **Recoverable** — persistent task state and restart recovery are first-class concerns.
- **Open** — source code, protocol design, documentation, and contribution rules are kept in the repository.

## What is included?

| Component | Role | Current state |
|---|---|---|
| **Nexum Core** | Task lifecycle, scheduler, events, orchestration | Working |
| **Nexum Protocol** | JSON-RPC 2.0 APIs, events, version negotiation | Working |
| **Storage** | SQLite persistence, migrations, restart recovery | Working |
| **Resolver** | HTTP/HTTPS, Magnet, local sources | Foundation working |
| **Engine Adapter** | Common engine boundary | Working |
| **HTTP Engine** | HTTP download execution and progress | Working |
| **Server** | TCP JSON-RPC service | Working |
| **CLI** | Task control, server configuration, inspection | Working |
| **Desktop** | Tauri 2 + React task client | Foundation working |
| **Browser** | Manifest V3 send-to-Nexum integration | Foundation working |
| **Security** | Credentials, authentication, TLS, rate-limit foundations | Foundation working |
| **Plugin SDK** | Manifest, permissions, capabilities, SDK skeleton | Foundation working |
| **Media** | Media types and probing structures | Foundation working |

## Architecture

```text
 Desktop ─────────┐
 Browser ─────────┤
 CLI ─────────────┤
                  ▼
           Nexum Protocol
                  │
                  ▼
             Nexum Core
       ┌──────────┼──────────┐
       │          │          │
    Storage    Resolver   Scheduler
                             │
                             ▼
                      Engine Adapter
                       /          \
                  InMemory         HTTP
```

The current Server and CLI transport is **line-delimited JSON-RPC over TCP**. The Protocol is intentionally transport-neutral, so other transports can be introduced without coupling Core to a network implementation.

### Task lifecycle

```text
Created → Queued → Downloading
                       ├── Paused
                       ├── Completed
                       └── Failed → Retrying → Queued
```

Persistent task metadata is handled through the Storage boundary, while runtime state and events remain owned by Core.

## Supported sources

The current Resolver foundation recognizes:

- **HTTP / HTTPS**
- **Magnet**
- **Local sources**

Resolvers classify and validate inputs before they become download tasks:

```text
Input → Resolver → DownloadTask → Scheduler → Engine
```

## Clients

### CLI

The CLI connects to the local server by default and can target a configured remote server.

Example:

```bash
nexum task list
nexum task create task-1 https://example.com/file.bin ./file.bin
nexum task queue task-1
nexum task start
```

### Desktop

The desktop application uses **Tauri 2 + React** and communicates through the Nexum protocol rather than reaching into Core directly.

Development:

```bash
cd apps/desktop
pnpm install
pnpm dev
```

### Browser extension

The browser integration uses **Manifest V3** and provides a send-to-Nexum flow for downloadable links, including server/device selection.

Build:

```bash
cd apps/extension
pnpm install
pnpm build
```

The generated extension can then be loaded as an unpacked extension in a Chromium-based browser.

## Quick start

### Requirements

- Rust stable
- Cargo
- Node.js LTS
- pnpm
- Linux desktop development dependencies if building the Tauri client

### Check and test the workspace

From the repository root:

```bash
cargo check --workspace
cargo test --workspace
```

For CI-equivalent Rust validation:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Run the server

The server listens on `127.0.0.1:39100` by default.

The exact server command and runtime options are exposed by the CLI. See the [Development Guide](docs/DEVELOPMENT.md) for the current development workflow.

## Protocol

Nexum Protocol currently provides:

- JSON-RPC 2.0 request/response primitives
- Task creation, inspection, queueing, starting, pausing, and control
- Standard and application-specific error codes
- Transport-neutral event envelopes
- Protocol version negotiation
- Authentication boundary
- Compatibility tests

A simplified interaction looks like:

```text
Client
  │
  │ JSON-RPC request
  ▼
Server
  │
  ▼
Nexum Core
  │
  ├── validate / resolve
  ├── persist task
  ├── schedule
  └── execute through Engine Adapter
  │
  └── JSON-RPC response + events
  ▼
Client
```

The protocol boundary is intended to keep Desktop, CLI, Browser, and future automation clients independent from internal Core implementation details.

## Persistence & recovery

SQLite is used for persistent task metadata.

The storage layer provides:

- schema versioning
- migrations
- persistent task state
- restart recovery
- repository boundaries that can be replaced in tests

Core behavior is designed to remain testable without network access. Network and engine behavior belongs in controlled integration tests.

## Security boundary

Nexum includes security foundations for:

- credential handling
- authentication
- TLS configuration
- rate limiting

The default server binding is local. Remote exposure should be treated as an explicit deployment decision and configured with the appropriate authentication and transport protections.

Nexum is not presented as a finished hardened public download service; security-sensitive deployment should follow the project's current documentation and review the implementation before exposure to untrusted networks.

## Extensibility

Nexum deliberately separates extension points:

```text
Resolver ────────┐
Engine Adapter ─┤
Plugin SDK ─────┤
Capabilities ───┤──→ Nexum Core / Protocol
Media ──────────┘
```

The Plugin foundation currently covers:

- Plugin Manifest
- Permissions
- Capabilities
- SDK structures

Plugin lifecycle management and executable plugin integration are still planned.

## Media & automation

The Media layer currently contains foundational media types and probing structures.

The following remain roadmap work:

- Media probing workflow
- Manifest parsing
- Track selection
- Segment scheduling
- Mux / post-processing
- Automation APIs
- AI / MCP integration
- Remote device management

Nexum therefore should currently be understood as a **download platform foundation**, not a finished media automation suite.

## Development status

The repository has working foundations across **Phases 0–9**, plus the foundational pieces of **Phase 10 (Extensibility)** and **Phase 11 (Media & Automation)**.

Implemented foundations include:

- Rust workspace and domain model
- Download task state machine
- Scheduler and task events
- SQLite persistence and restart recovery
- Resolver registry
- InMemory and HTTP engines
- JSON-RPC 2.0 protocol
- Protocol version negotiation
- Authentication boundary
- TCP Server
- CLI client and task management
- Tauri 2 + React Desktop foundation
- Manifest V3 Browser integration
- Security foundations
- Plugin Manifest / Permission / Capability / SDK foundations
- Foundational media structures
- GitHub Actions CI with formatting, workspace checks, tests, and Clippy validation

See the [Roadmap](ROADMAP.md) for the authoritative work breakdown.

## Repository layout

```text
Nexum/
├── crates/
│   ├── core/
│   ├── domain/
│   ├── task/
│   ├── scheduler/
│   ├── storage/
│   ├── resolver/
│   ├── protocol/
│   ├── plugin/
│   ├── security/
│   ├── media/
│   └── engine/
│
├── apps/
│   ├── server/
│   ├── cli/
│   ├── desktop/
│   └── extension/
│
├── docs/
├── ROADMAP.md
├── CHANGELOG.md
├── CONTRIBUTING.md
└── GOVERNANCE.md
```

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Development Guide](docs/DEVELOPMENT.md)
- [Roadmap](ROADMAP.md)
- [Contributing](CONTRIBUTING.md)
- [Governance](GOVERNANCE.md)
- [Development Plan](docs/DEVELOPMENT_PLAN.md)
- [Changelog](CHANGELOG.md)
- [中文 README](README.zh-CN.md)

## Roadmap

The next major areas are:

1. Complete plugin lifecycle and executable extension support
2. Expand resolver and engine integrations
3. Build the media probing and segmented-download pipeline
4. Add automation APIs and AI/MCP integration
5. Improve remote device management
6. Continue hardening clients, protocol compatibility, and security boundaries

Roadmap items are directional rather than fixed release promises.

## Contributing

Contributions are welcome.

Before making a large architectural change, please review the repository's contribution and governance documents and consider an RFC when the change affects public boundaries.

Useful commands:

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
```

Keep documentation synchronized with implemented behavior.

## License

Nexum is licensed under the [MIT License](LICENSE).

Third-party dependencies remain subject to their respective licenses.
