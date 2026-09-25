# Nexum

[English](README.md) · [简体中文](README.zh-CN.md)

> 用于下载任务管理与客户端集成的 Rust Workspace。

Nexum 仍在开发中。目前可运行的主线是用于创建和管理任务的本地 TCP JSON-RPC 服务。Server 使用 SQLite 保存任务。启动已排队的 HTTP/HTTPS 任务会通过后台 Worker 下载到目标路径。

[![CI](https://github.com/liveait/Nexum/actions/workflows/ci.yml/badge.svg)](https://github.com/liveait/Nexum/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## 当前状态

CLI 和早期 Tauri Desktop 客户端可调用本地 Server；Desktop 需要单独运行 Server。Server 在数据目录（默认 `./data`）下打开 `nexum.sqlite`，启动时恢复已保存的任务。`task.start` 对 HTTP/HTTPS 任务执行真实下载。Magnet 和本地文件来源可以创建任务，但尚无传输路径。Browser Extension 原型请求 HTTP `/jsonrpc`，而 Server 仅提供 TCP，因此扩展目前无法向它提交任务。

运行中的 Server 尚未启用认证、TLS、限流、可执行插件或真实媒体处理。代码边界与调用路径见[架构设计](docs/ARCHITECTURE.zh-CN.md)，后续集成工作见[开发计划](docs/DEVELOPMENT_PLAN.zh-CN.md)。

## 运行本地任务流程

需要支持 Rust 2024 edition 的 Rust（1.85 或更新版本）及 Cargo。在仓库根目录的一个终端启动 Server：

```bash
cargo run -p nexum-server
```

在另一个终端使用 CLI：

```bash
cargo run -p nexum-cli -- task create task-1 https://example.com/ ./example.html
cargo run -p nexum-cli -- task queue task-1
cargo run -p nexum-cli -- task start
cargo run -p nexum-cli -- task list
```

`task start` 在启动 HTTP Worker 后即返回。Worker 每写入一个响应块就报告进度；Server 自上次写入起累计至少 1 MiB 或经过 250 ms 时持久化一次中间快照，并在任务标记为 `Completed` 前刷新最终快照。成功后会写入 `./example.html` 并记录最终下载字节数；此期间 `task list` 可能显示 `Downloading`。`task.get` 和 `task.list` 返回持久化进度，以及表示最近一次传输错误的 `error` 字段。完整响应先写入临时 `.part` 文件，完成后才替换目标文件。传输失败会保留已有目标文件、记录 Server 错误、将错误保存到任务，并在重试策略允许时重新排队；默认策略允许重试三次，但排队的重试仍需再次调用 `task start` 才会执行。领取新一轮传输时进度会重置，旧错误也会清除。活跃的 HTTP 传输仍不能暂停、恢复或取消。Server 重启后任务仍在，但中断的 `Downloading` 任务会重置为 `Queued`，再次调用 `task start` 才会从头下载。Server 默认绑定 `127.0.0.1:39100`，可用 `--port PORT` 修改端口，CLI 可用 `--server ADDR` 修改连接地址。即使设置 `--require-auth`，Server 也不会强制认证，因此不要将它暴露到不可信网络。客户端设置、完整 Rust 检查命令和平台依赖见[开发指南](docs/DEVELOPMENT.zh-CN.md)。

## 文档与贡献

- [架构设计](docs/ARCHITECTURE.zh-CN.md)
- [开发指南](docs/DEVELOPMENT.zh-CN.md)
- [开发计划](docs/DEVELOPMENT_PLAN.zh-CN.md)
- [变更日志](CHANGELOG.zh-CN.md)
- [贡献指南](CONTRIBUTING.zh-CN.md)
- [治理规则](GOVERNANCE.zh-CN.md)
- [English README](README.md)

Nexum 采用 [MIT License](LICENSE)。第三方依赖仍受其各自许可证约束。
