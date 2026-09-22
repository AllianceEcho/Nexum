# Changelog

All notable changes to Nexum will be documented here.

## Unreleased

### Added

- **Core**: task lifecycle, scheduler, SQLite persistence, restart recovery, resolver integration, and event collection.
- **Engine**: InMemory engine and HTTP engine with redirect following and progress reporting.
- **Protocol**: JSON-RPC 2.0 task APIs, transport-neutral events, protocol version negotiation, and authentication boundary.
- **Server**: multi-threaded TCP JSON-RPC service with configurable runtime settings and authentication support.
- **CLI**: task management, reusable JSON-RPC client, server configuration, authentication commands, server inspection, and RPC error formatting.
- **Desktop**: Tauri 2 + React application with task management and server settings.
- **Browser**: Manifest V3 extension with context-menu, downloadable-link detection, send-to-Nexum flow, and server/device selection.
- **Security**: credential, authentication, TLS configuration, and rate-limit foundations.
- **Plugin**: manifest, permission, capability, and SDK foundations.
- **Media**: foundational media types and probing structures.

### Documentation

- Synchronized README, architecture, development guides, roadmap, and development plans with the implemented repository state.
- Kept English and Simplified Chinese project documentation aligned.

### Fixed

- **CI**: removed redundant workflow configuration and repaired workspace compatibility issues so the main CI workflow passes formatting, checks, tests, and Clippy.
- **Branding**: refreshed Desktop, Browser Extension, and macOS icon assets and corrected invalid PNG/ICNS payloads.

## [0.1.0] - TBD

- Initial project structure and public development foundation.

For the Chinese version, see [CHANGELOG.zh-CN.md](CHANGELOG.zh-CN.md).
