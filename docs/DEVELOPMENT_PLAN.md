# Nexum Development Plan

## 1. Development Principles

Nexum will be developed from the core outward.

1. Define the domain model before building UI.
2. Define the task lifecycle before implementing scheduling.
3. Define the protocol around stable core operations.
4. Keep persistence behind explicit interfaces.
5. Add engine integrations only after the core abstractions are testable.
6. Every milestone should leave the repository in a buildable and testable state.

## 2. Milestones

### Phase 0 — Project Foundation

- [x] Repository structure
- [x] MIT license
- [x] Contribution and governance documents
- [x] English and Simplified Chinese documentation
- [x] Rust workspace
- [x] GitHub CI
- [ ] Verified clean workspace build on CI
- [ ] Formal formatting and linting policy

### Phase 1 — Domain & Task Core

- [x] Domain model
- [x] Task state machine
- [x] Explicit transition validation
- [x] Task service
- [x] Task events
- [x] Unit tests

### Phase 2 — Scheduler

- [x] Queue abstraction
- [x] Concurrency limits
- [x] Priority
- [x] Retry policy
- [x] Pause/resume scheduling
- [x] Bandwidth policy abstraction
- [x] Scheduler events

### Phase 3 — Storage

- [x] Repository traits
- [x] In-memory repository
- [x] SQLite implementation
- [x] Schema versioning
- [x] Migration mechanism
- [x] Task persistence
- [x] Restart recovery

### Phase 4 — Resolver

- [x] Resolver trait and request/result model
- [x] HTTP/HTTPS classification and validation
- [x] Magnet classification and validation
- [x] Local source handling
- [x] Resolver registry
- [x] Resolver error model
- [x] Resolver tests

### Phase 5 — Engine Adapter

- [x] Adapter trait
- [x] Engine capabilities
- [x] Task and progress mapping
- [x] Pause/resume/remove mapping
- [x] InMemory engine
- [x] HTTP engine with redirect following
- [x] Controlled engine tests

### Phase 6 — Nexum Protocol

- [x] JSON-RPC 2.0 envelope
- [x] Request/response model
- [x] Task APIs
- [x] Transport-neutral event envelopes
- [x] Error codes
- [x] Protocol version negotiation
- [x] Authentication boundary
- [x] Compatibility tests

### Phase 7 — Server & CLI

- [x] TCP server process
- [x] Local server mode
- [x] CLI JSON-RPC client
- [x] Task creation and control
- [x] Configured server connection
- [x] Authentication and server inspection commands
- [x] Server configuration

### Phase 8 — Desktop

- [x] Tauri 2 shell
- [x] React application
- [x] Task list and detail views
- [x] Add-download flow
- [x] Pause/resume/remove
- [x] Server settings
- [x] Event-driven/polling updates

### Phase 9 — Browser Integration

- [x] Manifest V3 extension
- [x] Context-menu integration
- [x] Downloadable-link interception
- [x] Send-to-Nexum flow
- [x] Server/device selection

### Phase 10 — Extensibility

- [x] Plugin manifest
- [x] Permission model
- [x] Capability API
- [x] Plugin SDK skeleton
- [ ] Plugin lifecycle
- [ ] Resolver plugins
- [ ] Engine plugins

### Phase 11 — Media & Automation

- [x] Foundational media data structures
- [ ] Media probing workflow
- [ ] Manifest parsing
- [ ] Track selection
- [ ] Segment scheduling
- [ ] Mux/post-processing
- [ ] Automation API
- [ ] AI/MCP integration
- [ ] Remote device management

## 3. Delivery Order

Foundation → Domain → Task State Machine → Task Service → Scheduler → Storage → Resolver → Engine Adapter → Protocol → Server/CLI → Desktop/Browser → Extensibility → Media/Automation.

## 4. Engineering Rules

### Tests

Core behavior must be testable without network access. Network and engine tests belong in controlled integration-test layers.

### Dependencies

Prefer small, well-scoped dependencies. A dependency should solve a concrete problem.

### API Stability

Anything exposed outside a crate boundary should be treated as an API. Breaking changes should be deliberate and documented.

### Error Handling

Errors should preserve actionable context and should not be reduced to opaque strings at subsystem boundaries.

### Observability

Core operations should emit structured events and should move toward structured logs rather than ad-hoc output.

### Compatibility

Protocol, storage schema, and plugin APIs require explicit versioning strategies before becoming stable public interfaces.

## 5. Git Workflow

- main remains buildable.
- Feature work uses focused branches.
- Small fixes may go through a PR.
- Architectural changes require an RFC.
- Each milestone should be represented by reviewable commits.
- Releases are tagged from known-good commits.

## 6. Current Focus

Phases 0–9 have working foundations in the repository. Phase 10 has the plugin manifest, permission, capability, and SDK foundations. The next architectural focus is the transport layer and plugin lifecycle, followed by the media and automation work described in Phase 11.
