# 架构设计

## 总体结构

Nexum 将下载能力组织为分层架构：

```text
Client
  │
  ▼
Nexum Protocol
  │
  ▼
Nexum Core
  ├── Task
  ├── Scheduler
  ├── Resolver
  ├── Storage
  ├── Plugin
  └── Engine Adapter
          ├── curl
          ├── aria2
          └── libtorrent
```

## 核心原则

### Core-first

下载任务的生命周期、调度、状态和事件由 Core 统一管理。

### Protocol-first

客户端不直接依赖内部实现，而是通过稳定的 Protocol 与 Core 通信。

### Local-first

本地运行是默认路径；协议和身份模型同时为远程部署保留扩展能力。

## 任务模型

核心任务状态遵循明确的状态机：

```text
Created → Queued → Downloading
                       ├→ Paused
                       ├→ Completed
                       └→ Failed → Retry
```

持久化数据进入 Storage；实时状态和事件由 Core 管理。

## Resolver

统一输入经过 Resolver 转换：

```text
Input → Resolver → ResolvedDownload → DownloadTask → Scheduler → Engine
```

## Engine Adapter

引擎通过统一接口接入 Core。Core 不应依赖某个具体引擎的内部数据结构。

## Event Model

典型事件包括：

- TaskCreated
- TaskQueued
- TaskStarted
- TaskProgress
- TaskPaused
- TaskResumed
- TaskCompleted
- TaskFailed
- TaskRetrying
- TaskRemoved

后续协议会定义这些事件的稳定表示。

## 数据存储

SQLite 用于持久化任务元数据、配置和必要的历史信息。实时下载状态不以数据库轮询作为唯一来源，而由 Core 的事件模型驱动。

## 扩展

插件通过 Manifest、Permission 和 Capability API 获得明确能力边界。扩展 API 应尽量与 Core 内部实现解耦。
