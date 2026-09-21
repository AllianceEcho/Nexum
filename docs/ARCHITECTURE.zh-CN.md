# 架构设计

## 总体结构

Nexum 以 Core 和 Protocol 边界为中心组织：

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
     Storage Resolver  Scheduler
                        │
                        ▼
                 Engine Adapter
                  /          \
             InMemory        HTTP
```

当前 Server 与 CLI 使用基于 TCP 的按行 JSON-RPC。Protocol 保持传输层中立，因此后续可以在不让 Core 依赖网络实现的情况下增加其他传输方式。

## 核心原则

### Core-first

Core 负责任务生命周期、调度、状态转换、事件收集与整体编排。

### Protocol-first

客户端通过稳定的 Protocol 操作与 Core 通信，而不是直接依赖 Core 内部实现。

### Local-first

默认 Server 绑定本地地址，同时支持配置远程 Server 连接。

### Extensible

Resolver、Engine Adapter、Plugin 与 SDK 都是明确的扩展边界。

## 任务模型

任务生命周期：

```text
Created → Queued → Downloading
                       ├→ Paused
                       ├→ Completed
                       └→ Failed → Retrying → Queued
```

持久化任务状态通过 Storage Repository 边界管理；运行时状态和事件由 Core 管理。

## Resolver

输入处理路径：

```text
Input → Resolver → DownloadTask → Scheduler → Engine
```

当前 Resolver 基础能力覆盖 HTTP/HTTPS、Magnet 与本地来源。

## Engine Adapter

引擎通过统一 Adapter 接口接入。Core 使用 capabilities、task mapping、progress 与生命周期操作，而不是依赖具体引擎的内部数据结构。

当前仓库包含受控 InMemory Engine 和 HTTP Engine。后续引擎可以继续通过同一边界接入。

## Protocol

当前 Protocol 提供：

- JSON-RPC 2.0 请求/响应
- Task API
- 标准错误码与应用错误码
- 传输中立事件信封
- Protocol 版本协商
- 鉴权边界
- Compatibility Tests

Protocol 不绑定具体网络传输方式。

## Server 与客户端

Server 通过 TCP JSON-RPC 暴露 Core。CLI、Desktop 与 Browser 都通过 Protocol 工作，而不是直接调用 Core。

Desktop 使用 Tauri 2 + React；Browser 使用 Manifest V3 扩展。

## Storage

SQLite 提供任务元数据持久化、Schema Versioning、Migration 与重启恢复。InMemory Storage 继续用于测试和轻量运行场景。

## 扩展

Plugin 基础设施已经包含 Manifest、Permission、Capability 与 SDK 结构。插件生命周期管理和可执行插件集成仍属于后续工作。

## Media

Media crate 当前提供基础媒体类型和探测结构。完整媒体探测工作流、分段调度、Mux 与自动化仍属于后续计划。

