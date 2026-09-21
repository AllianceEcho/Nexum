# Nexum

**An open download platform.**

Nexum 是一个开放、现代、可扩展的下载平台。

## 愿景

Nexum 希望提供一个统一的下载核心，以及可以运行在不同环境中的客户端与工具。

核心原则：

- **Core-first**：下载能力集中在稳定、可测试的核心层。
- **Protocol-first**：客户端、服务端、浏览器与自动化工具通过统一协议协作。
- **Local-first**：本地使用简单直接，同时为远程设备与服务器保留扩展空间。
- **Extensible**：通过 Engine Adapter、Resolver、Plugin 与 SDK 扩展能力。
- **Open**：源码、协议与贡献流程保持开放透明。

## 架构

```text
                        Desktop
                           │
              Web ─────────┼──────── CLI
                           │
                       Browser
                           │
                       AI Agent
                           │
                    Nexum Protocol
                           │
                      Nexum Core
                           │
                    Engine Adapter
                     /     │      \
                  curl    aria2   libtorrent
```

## 主要组件

- **Nexum Core** — 下载任务、调度、状态机、事件与生命周期。
- **Nexum Protocol** — 客户端与核心之间的统一通信协议（JSON-RPC 2.0、版本协商、鉴权边界）。
- **Engine Adapter** — 对接不同下载引擎（InMemory 完整生命周期、HTTP 重定向跟随）。
- **Resolver** — 将 URL、Magnet 等输入解析为统一下载任务。
- **Server** — 提供远程下载服务（TCP、可配置、优雅关闭）。
- **CLI** — 面向终端与自动化场景（JSON-RPC 客户端、配置管理、认证）。
- **Desktop** — Tauri 2.0 + React 19 桌面应用（完整任务管理）。
- **Browser** — 浏览器集成（计划中）。
- **Plugin SDK** — 面向扩展与第三方能力（清单、权限、能力骨架）。

## 开发状态

**Phase 0–7：核心已完成。** Phase 8（桌面端）：Tauri + React 完整任务管理。Phase 9–11：浏览器扩展、插件 SDK、媒体与自动化。

- 核心任务生命周期，包含状态机（Created → Queued → Downloading → Completed/Failed/Paused/Retrying）
- JSON-RPC 2.0 协议，包含版本协商与鉴权边界
- 调度器：优先级、并发限制、重试策略、带宽控制
- 存储：内存与 SQLite，包含 schema 迁移与恢复
- 解析器：HTTP/HTTPS、Magnet、本地文件分类
- 引擎适配器：InMemory（完整生命周期）、HTTP（重定向跟随、进度上报）
- 服务端：TCP、可配置、凭证支持、优雅关闭
- 客户端：完整的任务 CRUD、配置管理、认证、服务器命令
- 桌面端：Tauri 2.0 + React 19，完整任务列表、创建、队列、暂停/恢复、删除

查看：

- [架构设计](docs/ARCHITECTURE.md)
- [开发指南](docs/DEVELOPMENT.md)
- [路线图](ROADMAP.md)
- [贡献指南](CONTRIBUTING.md)
- [治理规则](GOVERNANCE.md)

## 许可证

Nexum 本身采用 [MIT License](LICENSE)。

第三方依赖按照各自适用的许可证使用。
