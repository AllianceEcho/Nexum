# Changelog

All notable changes to Nexum will be documented here.

## Unreleased

### Added

- **Core**: task lifecycle, scheduler, resolver integration, and event collection. SQLite storage and restart recovery now support the running server; recovery persists the normalized `Queued` state of previously downloading, paused, or retrying tasks.
- **Engine**: in-memory and HTTP adapters. The HTTP adapter follows redirects, stages the response in a `.part` file, renames a complete download into place, and reports progress after each written response chunk. It still has no cancellation path.
- **Protocol**: JSON-RPC 2.0 task and server-info methods, a version identifier, credential fields, and event envelope/buffer types. `TaskView`, returned by `task.get` and `task.list`, includes persisted progress and an `error` field for the most recent transfer error. The server does not stream events or enforce credentials.
- **Server**: localhost, line-delimited TCP JSON-RPC with one thread per connection. It holds a data-directory lock, stores tasks in `data_dir/nexum.sqlite`, recovers them before listening, and fails startup on directory, lock, database, or recovery errors. `task.start` launches an HTTP/HTTPS worker and returns before transfer completion. The server persists progress when a response has added 1 MiB or 250 ms have elapsed, then flushes the final snapshot before persisting `Completed`. Failures preserve the error in `TaskView` and requeue while the default three-retry budget remains; after the budget is exhausted the task stays `Failed`. Retries are not automatically dispatched, so another `task.start` is required, and a new claim clears the previous error. Active HTTP transfers reject pause, resume, and remove. Connection limits and required authentication are not enforced.
- **CLI**: task commands, a reusable TCP JSON-RPC client, saved server address and credential fields, server inspection, and RPC error formatting.
- **Desktop**: Tauri 2 + React task UI connected to the TCP server, with a server-address field and manual task refresh.
- **Browser**: Manifest V3 context menu, link detection, and popup configuration. Its HTTP send request is not yet compatible with the TCP-only server.
- **Security**: credential, TLS, and rate-limit types; server-side authentication, TLS, and rate limiting remain unimplemented.
- **Plugin**: manifest, permission, capability, lifecycle-state, and provider-trait foundations. Core tracks plugin manifests but does not yet load provider engines or resolvers.
- **Media**: media and workflow models with simulated, in-memory probing and job execution; no real media processing is wired in.

### Documentation

- Synchronized README, architecture, development guides, and the consolidated development plan with the implemented repository state.
- Kept English and Simplified Chinese project documentation aligned.

### Fixed

- **CI**: removed redundant workflow configuration and repaired workspace compatibility issues so the main CI workflow passes formatting, checks, tests, and Clippy.
- **Branding**: refreshed Desktop, Browser Extension, and macOS icon assets and corrected invalid PNG/ICNS payloads.

## [0.1.0] - TBD

- Initial project structure and public development foundation.

For the Chinese version, see [CHANGELOG.zh-CN.md](CHANGELOG.zh-CN.md).
