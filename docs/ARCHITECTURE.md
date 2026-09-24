# Architecture

This document describes the behavior present in the repository. A type or trait in a crate does not imply that the server or clients use it yet.

## Runtime Boundary

```text
CLI ───────────────┐
Desktop (Tauri) ───┼── line-delimited JSON-RPC 2.0 / TCP
                   ▼
              Local server
                   │
              RpcDispatcher
                   │
                   ▼
                 Core
       ┌───────────┼────────────┐
  TaskService   Scheduler   TaskRepository
       │            │              │
  Task events   Queue/events   InMemory (server)
                    │           SQLite (injectable)
                    ▼
              EngineRegistry
             /              \
       InMemoryEngine     HttpEngine

Browser extension ── HTTP POST /jsonrpc (no matching server endpoint yet)
```

The server listens on `127.0.0.1:39100` by default. Its address is always bound to loopback in the current implementation; `--port` changes only the port. The CLI can connect to a configured TCP address, and the desktop UI accepts a server address. The browser extension's HTTP request is incompatible with the current TCP-only server.

## Crates and Responsibilities

| Crate | Implemented role |
| --- | --- |
| `domain` | Task ID, source, destination, and progress value types. |
| `task` | In-memory task service, validated state transitions, and task events. |
| `scheduler` | Explicit queue operations, priority, concurrency limit, retry policy, bandwidth-policy calculation, and scheduler events. It does not perform network I/O or automatically start queued work. |
| `storage` | `TaskRepository` with in-memory and SQLite implementations. SQLite has a schema version and migration. |
| `resolver` | HTTP/HTTPS, magnet, and local-source classification and validation through a registry. |
| `engine` | Adapter/registry APIs, simulated in-memory engine, and blocking HTTP GET engine. |
| `core` | Coordinates task, scheduler, resolver, engine, repository, and plugin manager state. |
| `security` | Credential, TLS configuration, rate-limit, and credential-store types. |
| `protocol` | JSON-RPC request/response dispatch, task/server methods, error objects, and event envelopes. |
| `plugin` | Manifest/permission/capability types, provider traits, and a plugin-manager state machine. |
| `media` | Media and workflow data types, dependency ordering, and simulated in-memory jobs. |

## Task Flow

`Core::create_task` validates the source through `ResolverRegistry`, creates a `DownloadTask`, and writes it to the injected repository. `queue_task` adds it to the scheduler. `start_next` explicitly chooses the next task and defaults to `InMemoryEngine`; `start_next_with_engine("http")` is a library API, not a JSON-RPC method or CLI option. The default server therefore does not download files when `task.start` is called.

The task state machine permits:

```text
Created     → Queued
Queued      → Downloading
Downloading → Paused | Completed | Failed
Paused      → Queued | Downloading
Completed   → Queued
Failed      → Retrying → Queued
```

The scheduler enforces its concurrency count when `start_next` or `resume` is called and can requeue failures under its retry policy. Bandwidth policy currently calculates limits; the HTTP transfer does not apply them. Task and scheduler events are collected in Core and can be drained, but the server does not publish them to clients.

## Engines and Sources

The resolver accepts HTTP/HTTPS URLs, magnet URIs containing an `xt=urn:btih:` parameter, and existing local paths; it does not validate the magnet hash itself. Resolving a source does not select a matching engine. `HttpEngine` can perform a blocking GET to a destination file and follows up to five redirects. Its pause/resume operations are unsupported, and its transfer does not expose incremental progress while the call is blocked. The in-memory engine simulates lifecycle operations and does not transfer bytes. There is no magnet or local-file transfer engine.

## Persistence and Recovery

`Core<R>` accepts a `TaskRepository`. The SQLite repository persists task metadata and progress, and `Core::recover` restores stored tasks; previously active, paused, or retrying tasks return to the queue. These paths have library tests. The server currently constructs `Core::new`, which uses `InMemoryRepository`, and never calls `recover`; its `data_dir` setting only creates a directory. Tasks do not survive a server restart.

## Protocol and Clients

The server reads one JSON-RPC request per TCP line and writes a response line for requests with IDs. The dispatcher supports `task.get`, `task.list`, `task.create`, `task.queue`, `task.start`, `task.pause`, `task.resume`, `task.remove`, `server.version`, and `server.auth`. There is no general task-update method. The protocol has a V1 version field and a version-inspection method, but no negotiated feature set or version enforcement beyond JSON-RPC `2.0` envelope validation. Event-envelope conversion and buffering exist in the crate, without a server subscription or stream.

The CLI uses the TCP protocol for task control and server inspection. The Tauri 2 + React desktop app uses Tauri commands as a TCP JSON-RPC client, with task list, add, and control views. It refreshes after actions or server-address changes, without periodic polling or pushed events; its server address is held only in component state. The Manifest V3 browser extension contains context-menu and link-badge UI, but its send flow posts HTTP to `/jsonrpc` and has no compatible endpoint. Its popup writes the `server` storage key while its background script reads `address`, so the saved address is ignored. It has no device selection.

Credentials can be carried in protocol requests, and `server.auth` reports only `none`. The server does not validate credentials. Its `require_auth` and `max_connections` configuration values are parsed but not enforced. TLS and rate-limit types are not wired into the server.

## Extension and Media Boundaries

`PluginManager` registers manifests and tracks load/start/stop states, and the crate defines `EngineProvider` and `ResolverProvider` traits. Core does not call plugin implementations or register their adapters; `init_plugins` only advances manager state. Permission declarations are metadata, not an enforced sandbox. Dynamic loading and executable plugin integration are not present.

The media crate defines probe, manifest, track, segment, mux, pipeline, job, workflow, and MCP request types. Workflow dependency ordering and in-memory job bookkeeping are implemented. `MediaProcessor::probe` returns default metadata, while job execution fabricates a completed result; no file probing, manifest parsing, segment processing, muxing, external automation endpoint, or MCP integration is wired up.

For the Chinese version, see [ARCHITECTURE.zh-CN.md](ARCHITECTURE.zh-CN.md).
