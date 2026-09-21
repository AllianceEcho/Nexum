# Changelog

All notable changes to Nexum will be documented here.

## Unreleased

### Added

- **Protocol**: Protocol versioning (`ProtocolVersion` V1), version negotiation via `server.version` and `server.auth` RPC methods
- **Protocol**: Optional `credential` field on `RpcRequest` for future authentication (backward-compatible via `#[serde(default)]`)
- **Protocol**: Re-exports all security types: `Credential`, `AuthenticationScheme`, `AuthenticationError`, `TlsConfig`, `RateLimit`, `PathPattern`
- **Protocol**: 20 tests (up from 8) covering versioning, credential parsing, server methods
- **Core**: 10 integration tests (up from 4) covering full lifecycle, concurrent limits, event draining, SQLite persistence, task cleanup
- **Engine**: HTTP redirect following (up to 5), progress reporting during download, destination directory auto-creation
- **Server**: Configurable `--port`, `--data-dir`, `--max-connections`, `--require-auth`, config file parsing, version/help output, credential logging
- **CLI**: `config get-server` / `config set-server`, `auth set` / `auth clear`, `server ping` / `server version` / `server auth`, `--version` flag, RPC error formatting
- **Security**: `Credential` enum (None, Bearer, ApiKey), `AuthenticationScheme`, `TlsConfig`, `AuthenticationError`, `CredentialStore`, `RateLimit`
- **Plugin**: `Permission` enum, `PathPattern`, `Capability`, `PluginManifest` with builder pattern
- **Media**: `MediaType`, `Track`, `MediaProbe` with builder, `MuxSpec`
- **Desktop**: Tauri 2.0 + React 19 foundation, full task management UI (list, create, queue, pause, resume, remove)

## [0.1.0] - 2025-XX-XX

- Initial project structure (workspace, docs, CI)

For the Chinese version, see [CHANGELOG.zh-CN.md](CHANGELOG.zh-CN.md).
