# Nexum

**An open download platform.**

Nexum 是一个开放、现代、可扩展的下载平台。

## 愿景

Nexum 提供统一的下载核心，以及可以运行在不同环境中的客户端与工具。

核心原则：

- **Core-first**：下载能力集中在稳定、可测试的核心层。
- **Protocol-first**：客户端、服务端、浏览器与自动化工具通过统一协议协作。
- **Local-first**：本地使用简单直接，同时为远程设备与服务器保留扩展空间。
- **Extensible**：通过 Engine Adapter、Resolver、Plugin 与 SDK 扩展能力。
- **Open**：源码、协议与贡献流程保持开放透明。

## 架构

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

Protocol 保持传输层中立。目前 Server / CLI 使用基于 TCP 的按行 JSON-RPC；后续可以在不改变 Core API 的情况下增加其他传输方式。

## 主要组件

- **Nexum Core** — 任务生命周期、调度、状态、事件、持久化协调与核心编排。
- **Nexum Protocol** — JSON-RPC 2.0 请求/响应、协议版本协商、鉴权边界与传输中立事件。
- **Engine Adapter** — 统一引擎接口，目前包含受控 InMemory Engine 与 HTTP Engine。
- **Resolver** — HTTP/HTTPS、Magnet、本地来源的分类与校验。
- **Server** — TCP JSON-RPC 服务，支持配置和鉴权边界。
- **CLI** — 终端客户端，支持任务操作、服务器配置、认证与服务器信息查询。
- **Desktop** — Tauri 2 + React 桌面客户端，提供任务管理与服务器配置。
- **Browser** — Manifest V3 浏览器集成，可将可下载链接发送到 Nexum。
- **Plugin SDK** — 插件 Manifest、权限、Capability 与 SDK 基础。
- **Media** — 为后续媒体工作流提供基础媒体类型和探测结构。

## 开发状态

Phase 0–9 已在仓库中形成可工作的基础能力。Phase 10 已完成插件 Manifest、权限、Capability 与 SDK 骨架。媒体与自动化仍处于后续计划阶段。

当前重点能力：

- Core 任务生命周期与 Scheduler
- SQLite 持久化与重启恢复
- HTTP/HTTPS、Magnet、本地来源 Resolver
- InMemory 与 HTTP Engine Adapter
- JSON-RPC 2.0 任务 API、事件、版本协商与鉴权边界
- TCP Server 与 CLI
- Tauri 2 + React Desktop
- Manifest V3 Browser Extension
- Plugin Manifest、权限、Capability 与 SDK 骨架
- 基础媒体数据结构

查看：

- [架构设计](docs/ARCHITECTURE.zh-CN.md)
- [开发指南](docs/DEVELOPMENT.zh-CN.md)
- [路线图](ROADMAP.zh-CN.md)
- [贡献指南](CONTRIBUTING.zh-CN.md)
- [治理规则](GOVERNANCE.zh-CN.md)
- [开发计划](docs/DEVELOPMENT_PLAN.zh-CN.md)
- [变更日志](CHANGELOG.zh-CN.md)

## 许可证

Nexum 本身采用 [MIT License](LICENSE)。

第三方依赖按照各自适用的许可证使用。
