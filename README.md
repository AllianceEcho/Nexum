# Nexum

**An open download platform.**

**One core. Any client. Anywhere.**

Nexum is an open, modern, and extensible download platform built around a Rust-first core. It separates download orchestration from clients and transport so the same core can serve desktop, browser, CLI, server, and future automation workflows.

## Why Nexum

Nexum is designed around a small set of architectural principles:

- **Core-first** — download capabilities live in a stable, testable core.
- **Protocol-first** — clients, servers, browser integrations, and automation share a clear protocol boundary.
- **Local-first** — local usage stays simple while the architecture remains ready for remote devices and servers.
- **Extensible** — Engine Adapters, Resolvers, Plugins, and SDKs provide explicit extension points.
- **Transport-neutral** — protocol types are independent from the network transport.
- **Open** — source code, documentation, protocols, and contribution processes remain open and transparent.

## What Nexum provides

| Layer | Responsibility |
| --- | --- |
| **Core** | Task lifecycle, scheduling, persistence coordination, recovery, orchestration, and events |
| **Protocol** | JSON-RPC 2.0 APIs, version negotiation, authentication boundary, and event representation |
| **Engine Adapter** | Stable boundary between Core and concrete download engines |
| **Resolver** | Source classification and validation |
| **Server** | Local/remote TCP JSON-RPC service |
| **Clients** | CLI, Desktop, and Browser integrations |
| **Extensibility** | Plugin manifest, permissions, capabilities, and SDK foundations |
| **Media** | Foundational media models and probing structures |

## Architecture

```text
 Desktop ───────┐
 Browser ───────┤
 CLI ───────────┤
                ▼
         Nexum Protocol
                │
                ▼
           Nexum Core
        ┌───────┼────────┐
        │       │        │
     Storage Resolver  Scheduler
                        │
                        ▼
                 Engine Adapter
                  /          \
             InMemory        HTTP
```

The **Core** owns task state and orchestration. Clients do not need to know how a download engine is implemented.

The **Protocol** is intentionally transport-neutral. The current server/client foundation uses line-delimited JSON-RPC over TCP; additional transports can be introduced later without changing the Core API boundary.

## Main components

### Nexum Core

The Rust core currently provides:

- task creation and lifecycle management
- queueing and priority scheduling
- concurrency limits
- retry policies
- bandwidth policy abstractions
- task and scheduler events
- SQLite persistence
- restart recovery
- source resolution
- engine coordination

### Nexum Protocol

The protocol currently provides:

- JSON-RPC 2.0 request/response handling
- task operations
- event representation
- protocol version information
- authentication boundary
- structured errors
- transport-neutral protocol types

The protocol keeps clients independent from Core implementation details.

### Engine Adapter

Engine Adapter defines the boundary between task orchestration and download execution.

Current engine foundations include:

- **InMemory Engine** — deterministic engine behavior for core workflows and testing.
- **HTTP Engine** — HTTP/HTTPS download execution with progress tracking.

Additional engines can be integrated through the adapter boundary without changing the task model.

### Resolver

Resolver classifies and validates input sources.

Current foundations cover:

- HTTP
- HTTPS
- Magnet
- local sources

Resolvers are registry-based so additional source types can be introduced independently.

### Server

The server exposes Nexum Protocol over TCP and provides a foundation for:

- local desktop clients
- CLI clients
- remote clients
- authentication-aware deployments
- future transport expansion

The server is separated from Core so the same Core can be embedded or hosted.

### Clients

**CLI**

The command-line client supports task operations, server configuration, authentication, and server inspection.

**Desktop**

The desktop application uses **Tauri 2 + React + TypeScript** and provides task management, download creation, server configuration, and status updates.

**Browser**

The browser integration uses **Manifest V3** and can send downloadable links to a configured Nexum server.

## Extensibility

Current plugin foundations include:

- plugin manifest
- plugin identity and metadata
- permission model
- capability model
- Plugin SDK skeleton

Planned extension work includes:

- plugin lifecycle management
- resolver plugins
- engine plugins
- richer automation APIs
- AI/MCP integration
- remote device management

The extension model is capability-oriented so future plugins can request only the access they need.

## Media and automation

The Media crate currently provides foundational data structures for future media workflows.

Planned capabilities include:

- media probing
- manifest parsing
- track selection
- segment scheduling
- muxing and post-processing
- automation APIs
- AI/MCP integration

These areas are being developed incrementally rather than being tightly coupled to the initial download core.

## Repository layout

```text
Nexum/
├── apps/
│   ├── cli/             # Command-line client
│   ├── desktop/         # Tauri desktop application
│   ├── server/          # TCP JSON-RPC server
│   └── extension/       # Browser extension
├── crates/
│   ├── core/            # Core orchestration
│   ├── domain/          # Shared domain types
│   ├── engine/          # Engine Adapter and engines
│   ├── media/           # Media foundations
│   ├── plugin/          # Plugin foundations
│   ├── protocol/        # JSON-RPC protocol
│   ├── resolver/        # Source resolution
│   ├── scheduler/       # Scheduling
│   ├── security/        # Authentication and security foundations
│   ├── storage/         # Persistence
│   └── task/            # Task lifecycle
├── docs/                # Architecture and development documentation
└── .github/             # CI and contribution templates
```

## Quick start

### Requirements

For Rust development, install a current stable Rust toolchain with Cargo.

For desktop development, install the Tauri 2 system dependencies required by your operating system.

### Build the workspace

```bash
cargo check --workspace --all-targets
```

### Run tests

```bash
cargo test --workspace
```

### Check formatting

```bash
cargo fmt --all -- --check
```

### Run Clippy

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

CI runs these workspace validation steps together. Current CI health should always be checked from the latest GitHub Actions result rather than inferred from this document.

## Development status

The repository has working foundations across the first major architecture phases:

- **Phase 0 — Project Foundation:** ✓
- **Phase 1 — Domain & Task Core:** ✓
- **Phase 2 — Scheduler:** ✓
- **Phase 3 — Storage:** ✓
- **Phase 4 — Resolver:** ✓
- **Phase 5 — Engine Adapter:** ✓
- **Phase 6 — Nexum Protocol:** ✓ foundation
- **Phase 7 — Server & CLI:** ✓ foundation
- **Phase 8 — Desktop:** ✓ foundation
- **Phase 9 — Browser Integration:** ✓ foundation
- **Phase 10 — Extensibility:** ✓ foundations; lifecycle and runtime plugins pending
- **Phase 11 — Media & Automation:** in progress

A phase marked as a foundation does not mean every planned feature in that phase is complete. Detailed implementation status is tracked in the development plan and roadmap.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Development Guide](docs/DEVELOPMENT.md)
- [Development Plan](docs/DEVELOPMENT_PLAN.md)
- [Roadmap](ROADMAP.md)
- [Contributing](CONTRIBUTING.md)
- [Governance](GOVERNANCE.md)
- [Security](SECURITY.md)
- [Support](SUPPORT.md)
- [Changelog](CHANGELOG.md)
- [RFCs](docs/RFC/README.md)
- [Architecture Decisions](docs/decisions/README.md)

For Chinese documentation, see [README.zh-CN.md](README.zh-CN.md).

## Contributing

Nexum welcomes contributions in code, documentation, testing, tooling, and design.

The contribution policy is intentionally simple:

- small fixes can go directly through a pull request
- larger feature work should be discussed first
- architecture changes require an RFC
- protocol changes require an RFC
- database migrations require an RFC or architecture decision
- security issues should follow the security reporting process

See [CONTRIBUTING.md](CONTRIBUTING.md) for the complete workflow.

## License

Nexum is licensed under the [MIT License](LICENSE).

Third-party dependencies remain subject to their respective licenses.
