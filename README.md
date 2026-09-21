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

- **Nexum Core** — task lifecycle, scheduling, state, events, and coordination.
- **Nexum Protocol** — unified communication between clients and the core (JSON-RPC 2.0, version negotiation, auth boundary).
- **Engine Adapter** — integration layer for download engines (InMemory, HTTP with redirect following).
- **Resolver** — turns inputs such as URLs and Magnet links into normalized download tasks.
- **Server** — remote download services (TCP, configurable, graceful shutdown).
- **CLI** — terminal and automation interface (JSON-RPC client, config management, auth).
- **Desktop** — Tauri + React desktop application (full task management).
- **Browser** — browser integration (planned).
- **Plugin SDK** — APIs for extensions and third-party capabilities (manifest, permissions, capabilities skeleton).

## Development Status

**Phase 0–7: Core completed.** Phase 8 (Desktop): Tauri + React with full task management. Phase 9–11: Browser extension, plugin SDK, media & automation.

- Core task lifecycle with state machine (Created → Queued → Downloading → Completed/Failed/Paused/Retrying)
- JSON-RPC 2.0 protocol with version negotiation and auth boundary
- Scheduler with priority, concurrency limits, retry policy, bandwidth control
- Storage: In-memory and SQLite with schema migration and recovery
- Resolver: HTTP/HTTPS, Magnet, Local source classification
- Engine Adapters: InMemory (full lifecycle), HTTP (redirect following, progress)
- Server: TCP with configurable settings, credentials, graceful shutdown
- CLI: Full task CRUD, config management, authentication, server commands
- Desktop: Tauri 2.0 + React 19 with task list, create, queue, pause/resume, remove

See:

- [Architecture](docs/ARCHITECTURE.md)
- [Development Guide](docs/DEVELOPMENT.md)
- [Roadmap](ROADMAP.md)
- [Contributing](CONTRIBUTING.md)
- [Governance](GOVERNANCE.md)
- [Development Plan](docs/DEVELOPMENT_PLAN.md)

For the Chinese version, see [README.zh-CN.md](README.zh-CN.md).

## License

Nexum is licensed under the [MIT License](LICENSE).
Third-party dependencies remain subject to their respective licenses.
