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
- **Nexum Protocol** — 客户端与核心之间的统一通信协议。
- **Engine Adapter** — 对接不同下载引擎。
- **Resolver** — 将 URL、Magnet 等输入解析为统一下载任务。
- **Server** — 提供远程下载能力。
- **CLI** — 面向终端与自动化场景。
- **Desktop** — 面向桌面用户的图形界面。
- **Browser** — 浏览器集成。
- **Plugin SDK** — 面向扩展与第三方能力。

## 项目结构

```text
nexum/
├── apps/
│   ├── desktop/
│   ├── server/
│   ├── cli/
│   └── extension/
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
├── packages/
│   ├── sdk/
│   ├── types/
│   └── ui/
├── plugins/
└── docs/
```

## 开发状态

Nexum 当前处于早期开发阶段。项目首先建立核心架构、协议与工程基础，再逐步实现桌面端、远程能力、扩展系统与媒体能力。

查看：

- [架构设计](docs/ARCHITECTURE.md)
- [开发指南](docs/DEVELOPMENT.md)
- [路线图](ROADMAP.md)
- [贡献指南](CONTRIBUTING.md)
- [治理规则](GOVERNANCE.md)

## 许可证

Nexum 本身采用 [MIT License](LICENSE)。

第三方依赖按照各自适用的许可证使用。
