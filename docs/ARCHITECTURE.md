# Architecture

## Overview

Nexum is organized around a Core and a protocol boundary:

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
    Storage Resolver Scheduler
                       │
                       ▼
                Engine Adapter
                 /          \
            InMemory        HTTP
```

The current server and CLI transport is line-delimited JSON-RPC over TCP. The protocol layer is intentionally transport-neutral so other transports can be introduced without coupling Core to a network implementation.

## Core Principles

### Core-first

The Core owns task lifecycle, scheduling, state transitions, event collection, and orchestration.

### Protocol-first

Clients communicate through stable protocol operations instead of depending on Core internals.

### Local-first

The default server binding is local. The architecture also supports configured remote server connections.

### Extensible

Resolver, Engine Adapter, Plugin, and SDK boundaries are explicit extension points.

## Task Model

The task lifecycle is:

```text
Created → Queued → Downloading
                       ├→ Paused
                       ├→ Completed
                       └→ Failed → Retrying → Queued
```

Persistent task state is handled through the Storage repository boundary. Runtime task state and events are owned by Core.

## Resolver

Input handling follows:

```text
Input → Resolver → DownloadTask → Scheduler → Engine
```

The current resolver foundation handles HTTP/HTTPS, Magnet, and local sources.

## Engine Adapter

Engines integrate through a common adapter interface. Core works with capabilities, task mappings, progress, and lifecycle operations rather than engine-specific data structures.

The repository currently contains a controlled InMemory engine and an HTTP engine. Additional engine integrations can be added behind the same boundary.

## Protocol

The Protocol currently provides:

- JSON-RPC 2.0 request/response primitives
- Task APIs
- Standard and application error codes
- Transport-neutral event envelopes
- Protocol version negotiation
- Authentication boundary
- Compatibility tests

The protocol does not require a particular network transport.

## Server and Clients

The Server exposes Core through JSON-RPC over TCP. The CLI, Desktop, and Browser integrations consume the protocol rather than calling Core directly.

The Desktop client uses Tauri 2 + React. The Browser integration uses a Manifest V3 extension.

## Storage

SQLite provides persistent task metadata with schema versioning, migrations, and restart recovery. In-memory storage remains useful for tests and lightweight runtime scenarios.

## Extensions

Plugin foundations define manifests, permissions, capabilities, and SDK structures. Plugin lifecycle management and executable plugin integration remain future work.

## Media

The media crate currently provides foundational media types and probing structures. Full media probing workflows, segment scheduling, muxing, and automation remain planned.

For the Chinese version, see [ARCHITECTURE.zh-CN.md](ARCHITECTURE.zh-CN.md).
