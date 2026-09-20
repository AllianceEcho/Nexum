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
- **Nexum Protocol** — unified communication between clients and the core.
- **Engine Adapter** — integration layer for download engines.
- **Resolver** — turns inputs such as URLs and Magnet links into normalized download tasks.
- **Server** — remote download services.
- **CLI** — terminal and automation interface.
- **Desktop** — desktop graphical client.
- **Browser** — browser integration.
- **Plugin SDK** — APIs for extensions and third-party capabilities.

## Development Status

Nexum is in early development. The project is establishing its core architecture, protocol, and engineering foundations before expanding into desktop, remote, extension, and media capabilities.

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
