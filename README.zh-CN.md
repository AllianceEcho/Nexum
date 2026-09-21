# Nexum

> **开放、本地优先的下载平台，覆盖 Desktop、Browser、CLI 与自动化场景。**

Nexum 是一个基于 Rust 构建的下载平台，以稳定的 **Core + Protocol** 边界为核心，将任务管理、调度、持久化、Resolver、下载引擎、TCP Server、CLI、桌面客户端、浏览器集成与插件基础能力组织在统一架构中。

[![CI](https://github.com/AllianceEcho/Nexum/actions/workflows/ci.yml/badge.svg)](https://github.com/AllianceEcho/Nexum/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> **状态：** Nexum 仍在积极开发中。当前仓库已经形成 Core、Protocol、Server、CLI、Desktop、Browser、Security、Plugin 与 Media 等基础能力；媒体工作流、自动化、可执行插件及更多下载引擎仍在持续开发。

## 为什么是 Nexum？

Nexum 围绕几条简单原则设计：

- **Core-first** — 任务生命周期、调度、状态、事件与编排集中在可测试的 Core。
- **Protocol-first** — 客户端通过稳定协议操作，而不是直接依赖 Core 内部实现。
- **Local-first** — 默认本地使用简单，同时为远程连接保留架构空间。
- **Engine-agnostic** — 下载引擎位于统一的 Adapter 边界之后。
- **Extensible** — Resolver、Engine、Plugin、Capability 与 SDK 都有明确扩展边界。
- **Recoverable** — 持久化任务状态与重启恢复是一等能力。
- **Open** — 源码、协议设计、文档与贡献规则均在仓库中公开维护。

## 当前包含什么？

| 组件 | 职责 | 当前状态 |
|---|---|---|
| **Nexum Core** | 任务生命周期、Scheduler、事件、编排 | Working |
| **Nexum Protocol** | JSON-RPC 2.0 API、事件、版本协商 | Working |
| **Storage** | SQLite 持久化、迁移、重启恢复 | Working |
| **Resolver** | HTTP/HTTPS、Magnet、本地来源 | Foundation working |
| **Engine Adapter** | 统一下载引擎边界 | Working |
| **HTTP Engine** | HTTP 下载执行与进度 | Working |
| **Server** | TCP JSON-RPC 服务 | Working |
| **CLI** | 任务控制、服务器配置、信息查询 | Working |
| **Desktop** | Tauri 2 + React 桌面客户端 | Foundation working |
| **Browser** | Manifest V3 发送任务集成 | Foundation working |
| **Security** | Credential、认证、TLS、Rate Limit 基础 | Foundation working |
| **Plugin SDK** | Manifest、权限、Capability、SDK 骨架 | Foundation working |
| **Media** | 媒体类型与探测基础结构 | Foundation working |

## 架构

```text
 Desktop ─────────┐
 Browser ─────────┤
 CLI ─────────────┤
                  ▼
           Nexum Protocol
                  │
                  ▼
             Nexum Core
       ┌──────────┼──────────┐
       │          │          │
    Storage    Resolver   Scheduler
                             │
                             ▼
                      Engine Adapter
                       /          \
                  InMemory         HTTP
```

当前 Server 与 CLI 使用 **基于 TCP 的按行 JSON-RPC**。Protocol 有意保持传输层中立，因此后续可以增加其他传输方式，而无需让 Core 与网络实现耦合。

### 任务生命周期

```text
Created → Queued → Downloading
                       ├── Paused
                       ├── Completed
                       └── Failed → Retrying → Queued
```

持久化任务元数据由 Storage 边界负责，运行时状态与事件由 Core 管理。

## 支持的来源

当前 Resolver 基础能力识别：

- **HTTP / HTTPS**
- **Magnet**
- **Local sources**

来源会先经过分类与校验，再进入下载任务：

```text
Input → Resolver → DownloadTask → Scheduler → Engine
```

## 客户端

### CLI

CLI 默认连接本地 Server，也可以连接已配置的远程 Server。

示例：

```bash
nexum task list
nexum task create task-1 https://example.com/file.bin ./file.bin
nexum task queue task-1
nexum task start
```

### Desktop

Desktop 使用 **Tauri 2 + React**，通过 Nexum Protocol 通信，而不是直接访问 Core 内部 API。

开发：

```bash
cd apps/desktop
pnpm install
pnpm dev
```

### Browser Extension

浏览器集成使用 **Manifest V3**，提供将可下载链接发送到 Nexum 的流程，并支持 Server / Device 选择。

构建：

```bash
cd apps/extension
pnpm install
pnpm build
```

生成的扩展可以作为 unpacked extension 加载到 Chromium 系浏览器中。

## 快速开始

### 环境要求

- Rust stable
- Cargo
- Node.js LTS
- pnpm
- 如果构建 Tauri Desktop，需要对应 Linux 桌面开发依赖

### 检查与测试 Workspace

在仓库根目录执行：

```bash
cargo check --workspace
cargo test --workspace
```

与 CI 接近的完整 Rust 校验：

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### 启动 Server

Server 默认监听：

```text
127.0.0.1:39100
```

具体启动命令和运行参数由 CLI 提供，开发流程请参考[开发指南](docs/DEVELOPMENT.zh-CN.md)。

## Protocol

Nexum Protocol 当前提供：

- JSON-RPC 2.0 请求 / 响应基础
- 任务创建、查询、排队、启动、暂停与控制
- 标准错误码与应用错误码
- 传输层中立的事件封装
- Protocol 版本协商
- 认证边界
- 兼容性测试

简化交互：

```text
Client
  │
  │ JSON-RPC request
  ▼
Server
  │
  ▼
Nexum Core
  │
  ├── validate / resolve
  ├── persist task
  ├── schedule
  └── execute through Engine Adapter
  │
  └── JSON-RPC response + events
  ▼
Client
```

Protocol 边界用于让 Desktop、CLI、Browser 以及未来的自动化客户端独立于 Core 内部实现细节。

## 持久化与恢复

Nexum 使用 SQLite 保存持久化任务元数据。

Storage 层提供：

- Schema 版本管理
- Migration
- 持久化任务状态
- 重启恢复
- 可替换的 Repository 边界，便于测试

Core 行为设计为无需网络即可测试；网络与 Engine 行为则进入受控集成测试。

## 安全边界

Nexum 已包含以下安全基础能力：

- Credential 处理
- Authentication
- TLS 配置
- Rate Limiting

默认 Server 仅绑定本地地址。若要开放远程访问，应将其视为明确的部署决策，并配置适当的认证与传输保护。

Nexum 当前定位仍是**下载平台基础设施**，不是已经完成公网加固的下载服务。暴露到不可信网络前，应结合当前文档与实现进行安全审查。

## 可扩展性

Nexum 有意拆分不同扩展点：

```text
Resolver ────────┐
Engine Adapter ─┤
Plugin SDK ─────┤
Capabilities ───┤──→ Nexum Core / Protocol
Media ──────────┘
```

Plugin 基础目前覆盖：

- Plugin Manifest
- Permissions
- Capabilities
- SDK structures

插件生命周期管理与可执行插件集成仍属于后续工作。

## Media 与自动化

Media 层当前提供基础媒体类型与探测结构。

以下内容仍属于 Roadmap：

- Media probing workflow
- Manifest parsing
- Track selection
- Segment scheduling
- Mux / post-processing
- Automation APIs
- AI / MCP integration
- Remote device management

因此，Nexum 当前应理解为一个**下载平台基础设施**，而不是已经完成的媒体自动化套件。

## 开发状态

仓库已经形成 **Phase 1–4** 的工作基础，并开始进入 Media & Automation 阶段。

当前已实现的基础能力包括：

- Rust Workspace 与 Domain Model
- Download Task 状态机
- Scheduler 与 Task Events
- SQLite 持久化与重启恢复
- Resolver Registry
- InMemory 与 HTTP Engine
- JSON-RPC 2.0 Protocol
- Protocol Version Negotiation
- Authentication Boundary
- TCP Server
- CLI Client 与任务管理
- Tauri 2 + React Desktop 基础
- Manifest V3 Browser 集成
- Security 基础能力
- Plugin Manifest / Permission / Capability / SDK 基础
- 基础 Media Structures

详细工作拆分以 [Roadmap](ROADMAP.zh-CN.md) 为准。

## 仓库结构

```text
Nexum/
├── crates/
│   ├── core/
│   ├── domain/
│   ├── task/
│   ├── scheduler/
│   ├── storage/
│   ├── resolver/
│   ├── protocol/
│   ├── plugin/
│   ├── security/
│   ├── media/
│   └── engine/
│
├── apps/
│   ├── server/
│   ├── cli/
│   ├── desktop/
│   └── extension/
│
├── docs/
├── ROADMAP.md
├── CHANGELOG.md
├── CONTRIBUTING.md
└── GOVERNANCE.md
```

## 文档

- [架构设计](docs/ARCHITECTURE.zh-CN.md)
- [开发指南](docs/DEVELOPMENT.zh-CN.md)
- [路线图](ROADMAP.zh-CN.md)
- [贡献指南](CONTRIBUTING.zh-CN.md)
- [治理规则](GOVERNANCE.zh-CN.md)
- [开发计划](docs/DEVELOPMENT_PLAN.zh-CN.md)
- [变更日志](CHANGELOG.zh-CN.md)
- [English README](README.md)

## Roadmap

下一阶段主要方向：

1. 完善 Plugin 生命周期与可执行扩展支持
2. 扩展 Resolver 与 Engine 集成
3. 构建 Media probing 与分段下载流水线
4. 增加 Automation API 与 AI/MCP 集成
5. 改进远程设备管理
6. 持续强化客户端、Protocol 兼容性与安全边界

Roadmap 用于表达方向，不构成固定版本承诺。

## 参与贡献

欢迎贡献代码、文档与设计建议。

进行较大的架构变更前，请先阅读贡献与治理文档；涉及公共边界的变化，建议先进行 RFC 讨论。

常用命令：

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
```

请确保文档与实际实现保持同步。

## 许可证

Nexum 本身采用 [MIT License](LICENSE)。

第三方依赖按照各自适用的许可证使用。
