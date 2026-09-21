# Nexum

**An open download platform.**

Nexum is an open, modern, and extensible download platform.

## Vision

Nexum provides a unified download core with clients and tools that can run across different environments.

Core principles:

- **Core-first** — download capabilities are organized around a stable, testable core.
- **Protocol-first** — clients, servers, browser integrations, and automation communicate through a unified protocol.
- **Local-first** — local usage stays simple while the architecture remains ready for remote devices and servers.
- **Extensible** — Engine Adapters, Resolvers, Plugins, and SDKs provide clear extension points.
- **Open** — source code, protocols, and contribution processes remain open and transparent.

## Components

- **Nexum Core** — task lifecycle, scheduling, state, events, persistence coordination, and orchestration.
- **Nexum Protocol** — JSON-RPC 2.0 request/response APIs, protocol version negotiation, authentication boundary, and transport-neutral events.
- **Engine Adapter** — common engine boundary with the controlled InMemory engine and HTTP engine.
- **Resolver** — source classification and validation for HTTP/HTTPS, Magnet, and local sources.
- **Server** — TCP JSON-RPC service with configurable runtime settings and authentication support.
- **CLI** — terminal client for task operations, server configuration, authentication, and server inspection.
- **Desktop** — Tauri 2 + React desktop client for task management and server configuration.
- **Browser** — Manifest V3 browser integration for sending downloadable links to Nexum.
- **Plugin SDK** — plugin manifest, permissions, capabilities, and SDK foundations.
- **Media** — foundational media types and probing structures for later media workflows.

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

The Protocol is intentionally transport-neutral. The current server/client foundation uses line-delimited JSON-RPC over TCP; additional transports can be added without changing Core APIs.

## Development Status

Phases 0–9 have working foundations in the repository. Phase 10 provides plugin manifest, permission, capability, and SDK foundations. Media and automation work remains planned.

Current highlights:

- Core task lifecycle and scheduler
- SQLite persistence and restart recovery
- Resolver registry with HTTP/HTTPS, Magnet, and local source handling
- Engine Adapter boundary with InMemory and HTTP engines
- JSON-RPC 2.0 task APIs, events, version negotiation, and authentication boundary
- TCP server and CLI client
- Tauri 2 + React desktop application
- Manifest V3 browser extension
- Plugin manifest, permissions, capabilities, and SDK skeleton
- Foundational media data structures

See:

- [Architecture](docs/ARCHITECTURE.md)
- [Development Guide](docs/DEVELOPMENT.md)
- [Roadmap](ROADMAP.md)
- [Contributing](CONTRIBUTING.md)
- [Governance](GOVERNANCE.md)
- [Development Plan](docs/DEVELOPMENT_PLAN.md)
- [Changelog](CHANGELOG.md)

For the Chinese version, see [README.zh-CN.md](README.zh-CN.md).

## License

Nexum is licensed under the [MIT License](LICENSE).
Third-party dependencies remain subject to their respective licenses.
