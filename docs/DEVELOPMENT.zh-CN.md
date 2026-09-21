# 开发指南

## 环境

Nexum Core 使用 Rust。客户端界面使用 TypeScript/React，桌面应用使用 Tauri 2。

建议安装：

- Rust stable
- Cargo
- Node.js LTS
- pnpm

## 工作区

Rust workspace 位于根目录：

```text
crates/
├── core/
├── domain/
├── task/
├── scheduler/
├── storage/
├── resolver/
├── protocol/
├── plugin/
├── security/
├── media/
└── engine/

apps/
├── server/
├── cli/
├── desktop/
└── extension/
```

## 开发原则

- 保持模块边界清晰。
- 公共 API 必须有文档和测试。
- Core 状态变化必须可测试。
- 不要让 UI 逻辑进入 Core。
- 不要让具体 Engine 污染 Domain Model。
- 破坏性架构变化先提交 RFC。
- 实现行为变化后同步更新文档。

## 常用命令

```bash
cargo check --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets
```

与 CI 等价的本地检查：

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 运行 Server

Server 默认监听 `127.0.0.1:39100`。服务器地址和运行时配置可以通过 Server 命令行参数配置。

## 运行 CLI

CLI 默认连接本地 Server，也可以通过 Server 参数选择其他服务器。

示例：

```bash
nexum task list
nexum task create task-1 https://example.com/file.bin ./file.bin
nexum task queue task-1
nexum task start
```

## Desktop 与 Browser

Desktop 位于 `apps/desktop`，使用 Tauri 2 + React。

Browser 集成位于 `apps/extension`，使用 Manifest V3。

## 测试

Core 行为应保持无需网络即可测试。网络和 Engine 行为应放在受控的集成测试中。
