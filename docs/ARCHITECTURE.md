# Architecture

## Overview

Nexum organizes download capabilities into clear layers: Client → Nexum Protocol → Nexum Core → Engine Adapter.

## Core Principles

### Core-first

The Core owns task lifecycle, scheduling, state, and events.

### Protocol-first

Clients communicate with the Core through a stable Protocol rather than depending on internal implementation details.

### Local-first

Local execution is the default path while the protocol and identity model remain ready for remote deployment.

## Task Model

The core task lifecycle is Created → Queued → Downloading → Paused / Completed / Failed → Retry.

Persistent data belongs in Storage; real-time state and events are managed by the Core.

## Resolver

Inputs are normalized through Input → Resolver → ResolvedDownload → DownloadTask → Scheduler → Engine.

## Engine Adapter

Engines integrate through a common adapter boundary. The Core should not depend on engine-specific internal data structures.

## Event Model

Typical events include TaskCreated, TaskQueued, TaskStarted, TaskProgress, TaskPaused, TaskResumed, TaskCompleted, TaskFailed, TaskRetrying, and TaskRemoved.

## Storage

SQLite stores task metadata, configuration, and required history. Real-time state should be driven by the Core event model rather than database polling alone.

## Extensions

Plugins receive explicit capability boundaries through manifests, permissions, and capability APIs.

For the Chinese version, see [ARCHITECTURE.zh-CN.md](ARCHITECTURE.zh-CN.md).
