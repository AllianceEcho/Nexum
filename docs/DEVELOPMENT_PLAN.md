# Nexum Development Plan

This plan distinguishes code-level foundations from an end-to-end feature available through the current server and clients. Checked items are present in the repository; unchecked items still need implementation or wiring. Planned work describes direction, not a fixed release schedule. See [Architecture](ARCHITECTURE.md) for current call paths and [Contributing](../CONTRIBUTING.md) for engineering and submission guidance.

## 1. Milestones

### Phase 0 - Project Foundation

- [x] Repository structure, MIT license, and contribution/governance documents
- [x] English and Simplified Chinese documentation
- [x] Rust workspace with server, CLI, desktop Tauri crate, and core crates
- [x] GitHub CI workflow configured for formatting, workspace check/test, and Clippy

### Phase 1 - Domain and Task Core

- [x] Domain value types and download task model
- [x] Validated task state machine
- [x] In-memory task service and task events
- [x] Unit tests for task lifecycle and transitions

### Phase 2 - Scheduler

- [x] Priority queue and concurrent-task limit
- [x] Retry policy and pause/resume operations
- [x] Bandwidth-policy interface and limit calculation
- [x] Scheduler events and controlled unit tests
- [ ] Apply calculated bandwidth limits to transfers
- [ ] Automatically dispatch queued work and retries; HTTP workers currently require an explicit `task.start`

### Phase 3 - Storage

- [x] `TaskRepository` and in-memory implementation
- [x] SQLite task metadata/progress repository with schema version and migration
- [x] `Core::recover` to rebuild queued work and persist normalized recovery states
- [x] Open SQLite and call recovery in the server startup path
- [x] Verify task continuity across a real server restart

### Phase 4 - Resolver

- [x] Resolver request/result/error model and registry
- [x] HTTP/HTTPS validation, magnet `xt=urn:btih:` parameter presence check, and existing-local-path validation
- [x] Resolver tests and Core task-creation validation
- [x] Route queued HTTP/HTTPS tasks to an HTTP worker from server `task.start`
- [ ] Route magnet and local-file sources to compatible transfer engines
- [ ] Add actual magnet and local-source transfer paths

### Phase 5 - Engine Adapter

- [x] Adapter capabilities, task mapping, and engine registry
- [x] Simulated in-memory engine and blocking HTTP GET engine with redirect following
- [x] Controlled engine tests
- [x] Run the HTTP engine from server `task.start`; CLI and Desktop use this same RPC
- [x] Stage HTTP downloads in a `.part` file and rename only a complete response to the destination
- [ ] Add nonblocking transfer progress, cancellation, and supported pause/resume behavior for real transfers

### Phase 6 - Nexum Protocol and Security

- [x] JSON-RPC 2.0 request/response and error objects
- [x] Task create/get/list/queue/start/pause/resume/remove and server inspection methods
- [x] V1 version type, request field, and `server.version` method
- [x] Task/scheduler event envelopes and buffering
- [x] Credential, TLS, and rate-limit types with protocol/security unit tests
- [ ] Enforce protocol compatibility beyond envelope validation
- [ ] Publish events through a server transport and expose them to clients
- [ ] Validate credentials and enforce authentication/TLS/rate limiting where configured

### Phase 7 - Server and CLI

- [x] Loopback TCP server using line-delimited JSON-RPC
- [x] CLI TCP client with task-control, address configuration, and server inspection commands
- [x] Server CLI flags and key-value configuration parsing
- [ ] Apply `require_auth` and `max_connections`; both are currently parsed but not enforced
- [x] Use `data_dir` for SQLite persistence and restart recovery
- [x] Make normal `task.start` launch a real HTTP/HTTPS download for supported sources
- [ ] Automatically dispatch retries and expose transfer errors to clients instead of only server logs

### Phase 8 - Desktop

- [x] Tauri 2 + React application and TCP JSON-RPC command bridge
- [x] Task list, add, queue/start, pause/resume, and remove UI
- [x] Editable server address and refresh after actions/address changes
- [ ] Add periodic or event-driven updates for progress and externally changed tasks
- [ ] Persist server settings and surface connection/action failures reliably

### Phase 9 - Browser Integration

- [x] Manifest V3 extension shell, link context menu, and downloadable-link badge heuristic
- [x] Popup field for storing one server address
- [ ] Connect send-to-Nexum to a supported transport; the extension posts HTTP `/jsonrpc` while the server only speaks TCP
- [ ] Use the saved `server` address in the background script; it currently reads an `address` field instead
- [ ] Handle the content script's send message and complete an end-to-end task-creation test
- [ ] Add device selection if multi-device delivery is still a product requirement

### Phase 10 - Extensibility

- [x] Plugin manifest, permission, and capability data models
- [x] `PluginManager` state transitions and tests
- [x] `EngineProvider` and `ResolverProvider` traits
- [ ] Invoke plugin lifecycle implementations and load executable plugin entries
- [ ] Register plugin-provided engines/resolvers with Core; current initialization only changes manager state
- [ ] Enforce permissions and define a stable SDK/runtime contract

### Phase 11 - Media and Automation

- [x] Foundational media, job, workflow, and MCP request types
- [x] Workflow dependency ordering and simulated in-memory job API
- [ ] Probe real media and parse manifests
- [ ] Implement track selection, segment scheduling, muxing, and post-processing
- [ ] Execute and persist real jobs/workflows instead of fabricating completed results
- [ ] Expose an external automation endpoint and integrate AI/MCP where required
- [ ] Implement remote device management

## 2. Delivery Order From Current Code

1. Complete the HTTP runtime path with incremental progress, cancellation or supported pause/resume, automatic retry dispatch, and client-visible transfer errors.
2. Complete server/client contracts: authenticated transport where configured, observable events, and a browser-compatible endpoint or bridge.
3. Connect plugin providers and enforce their declared permissions.
4. Replace simulated media operations with real processing, then expose automation and remote-device workflows.

## 3. Current Focus

Phases 0-9 have varying levels of scaffolding and library coverage. The running server persists tasks in SQLite, recovers them on restart, and launches a real HTTP/HTTPS worker for `task.start`. It records final progress and completion after a successful transfer. An active HTTP transfer cannot be paused, resumed, or removed; there is no incremental progress, cancellation, or automatic retry dispatch. Magnet and local-file transfers remain unsupported. Plugin and media crates contain more than data types, but their provider callbacks and real processing are not integrated into the product path.
