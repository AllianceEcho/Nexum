# 开发指南

## 环境

Nexum 是使用 Rust 2024 edition 的工作区，桌面客户端采用 Tauri 2。桌面前端使用 TypeScript 和 React；浏览器扩展使用 TypeScript。

- Rust stable（1.85 或更新版本）、Cargo、`rustfmt` 和 Clippy
- Node.js LTS 和 pnpm，用于两个前端包
- 运行或打包原生桌面应用时所需的 Tauri 2 系统依赖及 Tauri CLI

Ubuntu CI 在检查 Rust 工作区前安装 `libgtk-3-dev` 和 `libwebkit2gtk-4.1-dev`。本地构建还需满足对应平台的 Tauri 依赖要求。

## 工作区

根目录 `Cargo.toml` 包含下列全部 crate 和 `apps/desktop/src-tauri`。浏览器扩展是独立的前端包。

```text
crates/
  core/ domain/ task/ scheduler/ storage/ resolver/
  protocol/ plugin/ security/ media/ engine/
apps/
  server/                 # nexum-server 可执行文件
  cli/                    # nexum-cli 可执行文件
  desktop/                # React/Vite 和 src-tauri/
  extension/              # Manifest V3 扩展
```

保持 UI 逻辑与 Core 分离，避免将具体 Engine 的实现细节放入 Domain Model。行为变化应补充针对性测试，并同步更新受影响文档的中英文版本。需要 RFC 的变更类型见[贡献指南](../CONTRIBUTING.zh-CN.md)。

## Rust 检查

在仓库根目录运行与 `.github/workflows/ci.yml` 相同的命令：

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

CI 在 Ubuntu 上执行这些 Rust 检查。目前未构建或检查 TypeScript 包，也不生成发布产物。

## Server 与 CLI

在一个终端启动本地 Server：

```bash
cargo run -p nexum-server -- --port 39100
```

默认绑定 `127.0.0.1:39100`，通过 TCP 提供以换行符分隔的 JSON-RPC 2.0 服务。在另一个终端运行 CLI：

```bash
cargo run -p nexum-cli -- task list
cargo run -p nexum-cli -- task create task-1 https://example.com/ ./example.html
cargo run -p nexum-cli -- task queue task-1
cargo run -p nexum-cli -- task start
```

编译出的可执行文件分别名为 `nexum-server` 和 `nexum-cli`。使用 `cargo run -p nexum-server -- --help` 和 `cargo run -p nexum-cli -- --help` 查看当前参数与命令。CLI 默认连接 `127.0.0.1:39100`；如需覆盖地址，将 `--server ADDR` 放在 `task` 或 `server` 前面。

Server 将任务元数据和进度保存到 `--data-dir` 下的 `nexum.sqlite`（默认 `./data`，相对于 Server 工作目录）。启动时会按需创建目录，并在退出前一直持有同目录下 `nexum.lock` 的锁，因此同一数据目录只能由一个 Server 使用。Server 在监听前打开数据库并恢复任务；目录、锁、数据库或恢复失败会阻止启动。重启后，原先下载中、暂停或重试中的任务会在内存和 SQLite 中变为排队状态，但 Server 不会自动启动它们。`--max-connections` 仅显示配置值，不限制连接数；`--require-auth` 不会启用身份验证，`server.auth` 返回 `none`。

`task start` 选取排队中的 HTTP/HTTPS 任务，启动 Worker 后返回任务 ID。Worker 每写入一个响应块就报告进度；Server 在新增至少 1 MiB 或经过 250 ms 时持久化中间快照，并在任务标记为 `Completed` 前刷新最终快照。传输异步完成，可再次执行 `task list` 或 `task get ID` 查看字节数和完成状态。Server 会拒绝数据目录内的目标、符号链接目标，以及与活动传输重叠的目标。Worker 会先将完整响应写入目标目录的 `.part` 文件，再重命名到目标路径。传输失败会保留已有目标文件，并将错误文本保存到任务视图的 `error` 字段；在重试策略允许时重新排队，但不会自动派发，仍需再次调用 `task start`。默认策略允许重试三次。领取新一轮任务时进度会重置，旧错误也会清除。活跃的 HTTP 传输不能暂停、恢复或删除。Magnet 和本地文件来源仍无传输 Engine。重启后会从零开始传输。

## Desktop

在 `apps/desktop` 启动 Vite 前端：

```bash
cd apps/desktop
pnpm install
pnpm dev
```

Vite 使用 `1420` 端口。前端调用 Tauri 命令，因此测试完整应用还需要运行中的 Nexum Server 和原生 Tauri 窗口。安装 Tauri 2 CLI 后，保持 Vite 运行，并在另一个终端从 `apps/desktop` 执行 `cargo tauri dev`。`pnpm build` 会执行 TypeScript 编译和 Vite 构建；这是 Rust CI 之外的检查。

## 浏览器扩展

在 `apps/extension` 执行：

```bash
cd apps/extension
pnpm install
pnpm lint
pnpm build
```

扩展的 `pnpm dev` 会监听文件变化并重新构建。以 `apps/extension` 为目录加载未打包扩展：根目录的 `manifest.json` 引用 `dist/` 下的构建产物和 `icons/` 下的图标。

扩展当前向 `/jsonrpc` 发送 HTTP 请求，而 `nexum-server` 只接受 TCP JSON-RPC。在增加 HTTP 桥接或统一传输方式之前，扩展的发送到 Nexum 操作无法通过当前 Server 创建任务。

英文版见 [DEVELOPMENT.md](DEVELOPMENT.md)。
