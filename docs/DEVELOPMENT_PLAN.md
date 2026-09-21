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
- [x] First engine integration
- [ ] Integration tests with controlled fixtures

### Phase 6 — Nexum Protocol

- [x] Protocol envelope
- [x] Request/response model
- [x] Task APIs
- [x] Transport-neutral event stream
- [x] Error codes
- [ ] Versioning
- [ ] Authentication boundary
- [ ] Compatibility tests

### Phase 7 — Server & CLI

- [x] Server process
- [x] Local server mode
- [x] CLI task commands
- [x] CLI JSON-RPC client foundation
- [ ] Remote connection
- [ ] CLI task creation
- [ ] CLI task control
- [ ] CLI status and logs
- [ ] Configuration

### Phase 8 — Desktop

- [ ] Tauri shell
- [ ] React application
- [ ] Task list
- [ ] Task detail
- [ ] Add-download flow
- [ ] Pause/resume/remove
- [ ] Settings
- [ ] Event-driven updates

### Phase 9 — Browser Integration

- [ ] Browser extension
- [ ] Context-menu integration
- [ ] Link interception
- [ ] Send-to-Nexum flow
- [ ] Server/device selection

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
