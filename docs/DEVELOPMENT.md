# 开发指南

## 环境

Nexum 的核心使用 Rust，客户端界面使用 TypeScript/React，桌面应用使用 Tauri。

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
```

## 开发原则

- 优先保持模块边界清晰。
- 公共 API 必须有文档和测试。
- 核心状态变化必须可测试。
- 不要让 UI 逻辑进入 Core。
- 不要让具体 Engine 污染 Domain Model。
- 破坏性架构变化先提交 RFC。

## 常用命令

```bash
cargo check --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets
```

随着项目进入可运行阶段，会在这里补充完整的本地开发、调试和测试流程。
