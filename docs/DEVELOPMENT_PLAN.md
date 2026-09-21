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
- [ ] Error model

#### 1B — Task State Machine

- [ ] Created
- [ ] Queued
- [ ] Downloading
- [ ] Paused
- [ ] Completed
- [ ] Failed
- [ ] Retry
- [ ] Explicit transition validation
- [ ] Unit tests for valid and invalid transitions

#### 1C — Task Service

- [ ] Create task
- [ ] Queue task
- [ ] Pause/resume task
- [ ] Complete/fail task
- [ ] Retry task
- [ ] Remove task
- [ ] Task events

**Definition of done:** task behavior is deterministic and fully unit-tested without a real download engine.

### Phase 2 — Scheduler

- [x] Queue abstraction
- [x] Concurrency limits
- [x] Priority
- [ ] Retry policy
- [ ] Pause/resume scheduling
- [ ] Bandwidth policy abstraction
- [ ] Scheduler events

**Definition of done:** scheduler behavior can be tested with fake tasks and no network access.

### Phase 3 — Storage

- [ ] Repository traits
- [ ] SQLite implementation
- [ ] Schema versioning
- [ ] Migration mechanism
- [ ] Task persistence
- [ ] Recovery after restart

### Phase 4 — Resolver

- [ ] HTTP/HTTPS URL resolver
- [ ] Magnet resolver
- [ ] Local source validation
- [ ] Resolver registry
- [ ] Resolver error model
- [ ] Resolver tests

### Phase 5 — Engine Adapter

- [ ] Adapter trait
- [ ] Engine capabilities
- [ ] Task mapping
- [ ] Progress mapping
- [ ] Pause/resume/remove mapping
- [ ] First engine integration
- [ ] Integration tests with controlled fixtures

### Phase 6 — Nexum Protocol

- [ ] Protocol envelope
- [ ] Request/response model
- [ ] Task APIs
- [ ] Event stream
- [ ] Error codes
- [ ] Versioning
- [ ] Authentication boundary
- [ ] Compatibility tests

### Phase 7 — Server & CLI

- [ ] Server process
- [ ] Local server mode
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

The first coding milestone is Phase 1A + Phase 1B:

1. Implement domain identifiers and download primitives.
2. Implement the task model and state machine.
3. Add transition tests for the initial lifecycle.
4. Keep the implementation engine-independent.
5. Begin the scheduler only after this foundation is stable.
