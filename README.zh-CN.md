# Nexum

[English](README.md) · [简体中文](README.zh-CN.md)

> 用于下载任务管理与客户端集成的 Rust Workspace。

Nexum 仍在开发中。目前可运行的主线是用于创建和管理任务的本地 TCP JSON-RPC 服务。Server 使用内存仓库与内存引擎：启动任务只会改变状态，**不会**将来源下载到目标路径。

[![CI](https://github.com/liveait/Nexum/actions/workflows/ci.yml/badge.svg)](https://github.com/liveait/Nexum/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## 当前状态

CLI 和早期 Tauri Desktop 客户端可调用本地 Server；Desktop 需要单独运行 Server。SQLite 仓库和 HTTP 引擎已有库实现，但 Server 尚未选用。Browser Extension 原型请求 HTTP `/jsonrpc`，而 Server 仅提供 TCP，因此扩展目前无法向它提交任务。

运行中的 Server 尚未启用认证、TLS、限流、可执行插件或真实媒体处理。代码边界与调用路径见[架构设计](docs/ARCHITECTURE.zh-CN.md)，后续集成工作见[开发计划](docs/DEVELOPMENT_PLAN.zh-CN.md)。

## 运行本地任务流程

需要支持 Rust 2024 edition 的 Rust（1.85 或更新版本）及 Cargo。在仓库根目录的一个终端启动 Server：

```bash
cargo run -p nexum-server
```

在另一个终端使用 CLI：

```bash
cargo run -p nexum-cli -- task create task-1 https://example.com/file.bin ./file.bin
cargo run -p nexum-cli -- task queue task-1
cargo run -p nexum-cli -- task start
cargo run -p nexum-cli -- task list
```

`task start` 会通过内存引擎把排队任务标记为 `Downloading`，不会创建 `./file.bin`；Server 退出后任务也会丢失。Server 默认绑定 `127.0.0.1:39100`，可用 `--port PORT` 修改端口，CLI 可用 `--server ADDR` 修改连接地址。即使设置 `--require-auth`，Server 也不会强制认证，因此不要将它暴露到不可信网络。客户端设置、完整 Rust 检查命令和平台依赖见[开发指南](docs/DEVELOPMENT.zh-CN.md)。

## 文档与贡献

- [架构设计](docs/ARCHITECTURE.zh-CN.md)
- [开发指南](docs/DEVELOPMENT.zh-CN.md)
- [开发计划](docs/DEVELOPMENT_PLAN.zh-CN.md)
- [变更日志](CHANGELOG.zh-CN.md)
- [贡献指南](CONTRIBUTING.zh-CN.md)
- [治理规则](GOVERNANCE.zh-CN.md)
- [English README](README.md)

Nexum 采用 [MIT License](LICENSE)。第三方依赖仍受其各自许可证约束。
