# Nexum

**An open download platform.**

**One core. Any client. Anywhere.**

Nexum 是一个开放、现代、可扩展的下载平台，以 Rust-first 核心为基础，将下载编排、客户端和传输层解耦，让同一套核心能力可以服务于 Desktop、Browser、CLI、Server 以及未来的自动化工具。

## 为什么是 Nexum

Nexum 围绕以下架构原则构建：

- **Core-first**：下载能力集中在稳定、可测试的核心层。
- **Protocol-first**：客户端、服务端、浏览器与自动化工具通过统一协议边界协作。
- **Local-first**：本地使用保持简单，同时为远程设备与服务器保留扩展空间。
- **Extensible**：通过 Engine Adapter、Resolver、Plugin 与 SDK 提供明确的扩展边界。
- **Transport-neutral**：协议类型与底层网络传输方式解耦。
- **Open**：源码、文档、协议与贡献流程保持开放透明。

## Nexum 提供什么

| 层 | 职责 |
| --- | --- |
| **Core** | 任务生命周期、调度、持久化协调、恢复、编排与事件 |
| **Protocol** | JSON-RPC 2.0 API、版本协商、鉴权边界与事件表示 |
| **Engine Adapter** | Core 与具体下载引擎之间的稳定接口 |
| **Resolver** | 来源分类与校验 |
| **Server** | 本地/远程 TCP JSON-RPC 服务 |
| **Clients** | CLI、Desktop、Browser 客户端 |
| **Extensibility** | 插件 Manifest、权限、Capability 与 SDK 基础 |
| **Media** | 基础媒体模型与探测结构 |

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

**Core** 负责下载状态与核心编排，客户端不需要了解具体下载引擎的实现细节。

**Protocol** 保持传输层中立。目前 Server / CLI 基础实现使用基于 TCP 的按行 JSON-RPC；未来可以在不改变 Core API 边界的情况下增加其他传输方式。

## 主要组件

### Nexum Core

Rust 核心目前提供：

- 任务创建与生命周期管理
- 队列与优先级调度
- 并发限制
- 重试策略
- 带宽策略抽象
- Task Event 与 Scheduler Event
- SQLite 持久化
- 重启恢复
- 来源解析
- Engine 协调

### Nexum Protocol

协议目前提供：

- JSON-RPC 2.0 请求/响应处理
- 任务操作 API
- 事件表示
- 协议版本信息
- 鉴权边界
- 结构化错误
- 与传输方式无关的协议类型

Protocol 的目标是让客户端不需要依赖 Core 的内部实现细节。

### Engine Adapter

Engine Adapter 定义任务编排与实际下载执行之间的边界。

当前基础引擎包括：

- **InMemory Engine**：用于核心流程和测试的确定性引擎。
- **HTTP Engine**：支持 HTTP/HTTPS 下载与进度跟踪。

未来可以通过 Adapter 边界接入更多引擎，而不需要改变任务模型。

### Resolver

Resolver 负责将输入来源分类并转换为经过校验的资源。

当前基础能力包括：

- HTTP
- HTTPS
- Magnet
- 本地来源

Resolver 使用 Registry 组织实现，因此后续可以独立增加新的来源类型。

### Server

Server 通过 TCP 提供 Nexum Protocol 服务，为以下场景提供基础：

- 本地 Desktop 客户端
- CLI 客户端
- 远程客户端
- 带鉴权的部署
- 后续传输方式扩展

Server 与 Core 分离，因此同一套 Core 既可以嵌入应用，也可以由独立 Server 承载。

### Clients

**CLI**

命令行客户端支持任务操作、服务器配置、认证以及服务器信息查询。

**Desktop**

Desktop 使用 **Tauri 2 + React + TypeScript**，提供任务管理、创建下载、服务器配置以及状态更新。

**Browser**

Browser 使用 **Manifest V3**，可以将可下载链接发送到配置好的 Nexum Server。

## 扩展能力

当前插件基础包括：

- Plugin Manifest
- 插件身份与元数据
- 权限模型
- Capability 模型
- Plugin SDK 骨架

后续计划包括：

- 插件生命周期管理
- Resolver Plugin
- Engine Plugin
- 更完整的 Automation API
- AI/MCP 集成
- 远程设备管理

插件模型采用 Capability 导向，未来插件可以只申请自身真正需要的能力。

## Media 与自动化

Media crate 当前提供未来媒体工作流所需的基础数据结构。

后续计划包括：

- Media Probe
- Manifest 解析
- Track Selection
- Segment Scheduling
- Mux 与后处理
- Automation API
- AI/MCP 集成

这些能力会逐步建设，不会与最初的下载核心强耦合。

## 仓库结构

```text
Nexum/
├── apps/
│   ├── cli/             # 命令行客户端
│   ├── desktop/         # Tauri 桌面应用
│   ├── server/          # TCP JSON-RPC Server
│   └── extension/       # 浏览器扩展
├── crates/
│   ├── core/            # Core 编排
│   ├── domain/          # 共享领域类型
│   ├── engine/          # Engine Adapter 与引擎
│   ├── media/           # Media 基础能力
│   ├── plugin/          # Plugin 基础
│   ├── protocol/        # JSON-RPC Protocol
│   ├── resolver/        # 来源解析
│   ├── scheduler/       # Scheduler
│   ├── security/        # 鉴权与安全基础
│   ├── storage/         # 持久化
│   └── task/             # Task 生命周期
├── docs/                # 架构与开发文档
└── .github/             # CI 与贡献模板
```

## 快速开始

### 环境要求

Rust 开发需要当前稳定版 Rust 工具链和 Cargo。

如果需要开发 Desktop，还需要安装 Tauri 2 对应操作系统的系统依赖。

### 构建整个 Workspace

```bash
cargo check --workspace --all-targets
```

### 运行测试

```bash
cargo test --workspace
```

### 检查格式

```bash
cargo fmt --all -- --check
```

### 运行 Clippy

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

仓库 CI 会将这些 Workspace 校验组合执行。具体 CI 是否通过，应以当前 GitHub Actions 结果为准，而不是根据 README 推断。

## 开发状态

仓库目前已经形成主要架构阶段的工作基础：

- **Phase 0 — Project Foundation：** ✓
- **Phase 1 — Domain & Task Core：** ✓
- **Phase 2 — Scheduler：** ✓
- **Phase 3 — Storage：** ✓
- **Phase 4 — Resolver：** ✓
- **Phase 5 — Engine Adapter：** ✓
- **Phase 6 — Nexum Protocol：** ✓ 基础能力
- **Phase 7 — Server & CLI：** ✓ 基础能力
- **Phase 8 — Desktop：** ✓ 基础能力
- **Phase 9 — Browser Integration：** ✓ 基础能力
- **Phase 10 — Extensibility：** ✓ 基础能力，生命周期与运行时插件待完成
- **Phase 11 — Media & Automation：** 进行中

需要注意：标记为“基础能力”并不代表该阶段的所有计划功能都已经完成。具体实现状态以 Development Plan 与 Roadmap 为准。

## 项目文档

- [架构设计](docs/ARCHITECTURE.zh-CN.md)
- [开发指南](docs/DEVELOPMENT.zh-CN.md)
- [开发计划](docs/DEVELOPMENT_PLAN.zh-CN.md)
- [路线图](ROADMAP.zh-CN.md)
- [贡献指南](CONTRIBUTING.zh-CN.md)
- [治理规则](GOVERNANCE.zh-CN.md)
- [安全策略](SECURITY.zh-CN.md)
- [支持](SUPPORT.zh-CN.md)
- [变更日志](CHANGELOG.zh-CN.md)
- [RFC](docs/RFC/README.md)
- [架构决策](docs/decisions/README.md)

英文版请见 [README.md](README.md)。

## 参与贡献

Nexum 欢迎代码、文档、测试、工具与设计方面的贡献。

当前贡献规则保持简单：

- 小改动可以直接提交 Pull Request
- 较大的功能应先讨论
- 架构改动必须 RFC
- Protocol 改动必须 RFC
- 数据库迁移需要 RFC 或架构决策
- 安全问题应按照安全报告流程提交

完整流程请查看 [CONTRIBUTING.zh-CN.md](CONTRIBUTING.zh-CN.md)。

## 许可证

Nexum 本身采用 [MIT License](LICENSE)。

第三方依赖按照各自适用的许可证使用。
