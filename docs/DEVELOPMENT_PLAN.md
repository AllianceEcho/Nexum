# Nexum Development Plan

## 1. Development Principles

Nexum will be developed from the core outward.

1. Define the domain model before building UI.
2. Define the task lifecycle before implementing scheduling.
3. Define the protocol around stable core operations.
4. Keep persistence behind explicit interfaces.
5. Add engine integrations only after the core abstractions are testable.
6. Every milestone must leave the repository in a buildable and testable state.

## 2. Milestones

### Phase 0 — Project Foundation

- [x] Repository structure
- [x] MIT license
- [x] Contribution and governance documents
- [x] English and Simplified Chinese documentation
- [x] Rust workspace
- [x] GitHub CI
- [ ] Workspace builds cleanly on CI
- [ ] Basic formatting and linting policy

**Definition of done:** a fresh checkout can run the documented Rust checks successfully.

### Phase 1 — Domain & Task Core

Goal: define what a download task is and how it behaves.

#### 1A — Domain Model

- [x] Task identifiers
- [x] Download source model
- [x] Destination model
- [x] Basic task metadata
- [x] Progress model
- [x] Error model

#### 1B — Task State Machine

- [x] Created
- [x] Queued
- [x] Downloading
- [x] Paused
- [x] Completed
- [x] Failed
- [x] Retry
- [x] Explicit transition validation
- [x] Unit tests for valid and invalid transitions

#### 1C — Task Service

- [x] Create task
- [x] Queue task
- [x] Pause/resume task
- [x] Complete/fail task
- [x] Retry task
- [x] Remove task
- [x] Task events

**Definition of done:** task behavior is deterministic and fully unit-tested without a real download engine.

### Phase 2 — Scheduler

- [x] Queue abstraction
- [x] Concurrency limits
- [x] Priority
- [x] Retry policy
- [x] Pause/resume scheduling
- [x] Bandwidth policy abstraction
- [x] Scheduler events

**Definition of done:** scheduler behavior can be tested with fake tasks and no network access.

### Phase 3 — Storage

- [x] Repository traits
- [x] SQLite implementation
- [x] Schema versioning
- [x] Migration mechanism
- [x] Task persistence
- [x] Recovery after restart

### Phase 4 — Resolver

- [x] Resolver trait and request/result model
- [x] HTTP/HTTPS source classification
- [x] Magnet source classification
- [x] HTTP/HTTPS URL resolver
- [x] Magnet resolver
- [x] Local source validation
- [x] Resolver registry
- [x] Resolver error model
- [x] Resolver tests

### Phase 5 — Engine Adapter

- [x] Adapter trait
- [x] Engine capabilities
- [x] Task mapping
- [x] Progress mapping
- [x] Pause/resume/remove mapping
- [x] First engine integration (InMemory)
- [x] Second engine integration (HTTP with redirect following)
- [x] Integration tests with controlled fixtures

### Phase 6 — Nexum Protocol

- [x] Protocol envelope
- [x] Request/response model
- [x] Task APIs (list, get, create, queue, start, pause, resume, remove)
- [x] Transport-neutral event stream
- [x] Error codes (JSON-RPC 2.0 standard + custom)
- [x] Versioning (ProtocolVersion V1, server.version RPC)
- [x] Authentication boundary (Credential, AuthenticationScheme, server.auth RPC)
- [x] Compatibility tests (version negotiation, credential passthrough)

### Phase 7 — Server & CLI

- [x] Server process (TCP listener, multi-threaded)
- [x] Local server mode (default 127.0.0.1:39100)
- [x] CLI task commands (list, get, create, queue, start, pause, resume, remove)
- [x] CLI JSON-RPC client (TCP socket, line-based protocol)
- [x] Remote connection (configured server, timeout handling)
- [x] CLI task creation (id, source, destination via CLI args)
- [x] CLI task control (all task operations)
- [x] CLI status and logs (RPC error formatting, server auth/version)
- [x] Configuration (CLI flags, config file parsing, server config)

### Phase 8 — Desktop

- [x] Tauri shell (Tauri 2.0, configurable window)
- [x] React application (React 19, Vite)
- [x] Task list (full CRUD, state display, progress)
- [x] Task detail (expandable rows, byte formatting)
- [x] Add-download flow (form, validation, create via RPC)
- [x] Pause/resume/remove (connected to server)
- [x] Settings (server configuration)
- [x] Event-driven updates (RPC polling, error handling)

### Phase 9 — Browser Integration

- [x] Browser extension (Manifest v3)
- [x] Context-menu integration ("Send to Nexum" on links)
- [x] Link interception (hover detection, downloadable URLs)
- [x] Send-to-Nexum flow (background service worker, notifications)
- [x] Server/device selection (chrome.storage.local, popup config)

### Phase 10 — Extensibility

- [x] Plugin manifest (id, name, version, description, author, license, entry)
- [x] Permission model (None, Read, Write, Network, Execute)
- [x] Capability API (name, version, features)
- [x] Plugin SDK skeleton (PathPattern)
- [ ] Plugin lifecycle (install/unload, version validation)
- [ ] Resolver plugins (custom source type handlers)
- [ ] Engine plugins (custom download engine adapters)

### Phase 10 — Extensibility

- [ ] Plugin manifest
- [ ] Permission model
- [ ] Capability API
- [ ] Plugin SDK
- [ ] Resolver plugins
- [ ] Engine plugins
- [ ] Plugin lifecycle

### Phase 11 — Media & Automation

- [ ] Media probe
- [ ] Manifest parsing
- [ ] Track selection
- [ ] Segment scheduling
- [ ] Mux/post-processing
- [ ] Automation API
- [ ] AI/MCP integration

## 3. Delivery Order

Foundation → Domain → Task State Machine → Task Service → Scheduler → Storage → Resolver → Engine Adapter → Protocol → Server/CLI → Desktop/Browser.

Desktop development will begin only after the Core and Protocol have stable enough boundaries.

## 4. Engineering Rules

### Tests

Core behavior must be testable without network access. Network and engine tests belong to integration-test layers.

### Dependencies

Prefer small, well-scoped dependencies. A dependency should solve a concrete problem.

### API Stability

Anything exposed outside a crate boundary should be treated as an API. Breaking changes should be deliberate and documented.

### Error Handling

Errors should preserve actionable context and should not be reduced to opaque strings at subsystem boundaries.

### Observability

Core operations should eventually emit structured events and structured logs rather than ad-hoc print statements.

### Compatibility

Protocol, storage schema, and plugin APIs require explicit versioning strategies before becoming public.

## 5. Git Workflow

- main remains buildable.
- Feature work uses focused branches.
- Small fixes may go through a PR.
- Architectural changes require an RFC.
- Each milestone should be represented by reviewable commits.
- Releases are tagged from known-good commits.

## 6. Immediate Work

**Phase 2 — Scheduler is complete.** Phase 3 — Storage is implemented through the repository boundary, SQLite, schema versioning, migrations, task persistence, and initial restart recovery. Phase 4 — Resolver now provides source-specific validation and is integrated into Core task creation. Phase 5 — Engine Adapter has the adapter boundary, capabilities, task mapping, lifecycle mapping, and an initial HTTP engine. Phase 6 — Protocol now has JSON-RPC request/response primitives, task APIs, application error codes, notification handling, and a transport-neutral event buffer. Next: define the concrete event transport and compatibility/versioning boundaries.
