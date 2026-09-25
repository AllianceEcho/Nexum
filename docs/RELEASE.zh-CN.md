# 发布流程

## 当前状态

仓库目前没有自动发布工作流，当前检出也没有版本标签。`.github/workflows/ci.yml` 在推送和 Pull Request 上执行 Rust 格式、检查、测试和 Clippy；不会构建前端包、打包桌面安装程序或浏览器扩展，也不会发布 GitHub Release。各包与应用清单目前声明 `0.1.0`，而变更日志仍将该版本标为 TBD。

`Cargo.lock` 被 `.gitignore` 忽略，未纳入 Git；两个前端包也没有已跟踪的锁文件。因此，全新检出可能解析到不同的依赖版本。更新本地 `Cargo.lock` 不会改变发布标签中的内容。在宣称发布构建的依赖解析可复现之前，需要先确定并落实纳入版本控制的锁文件策略。

## 手动发布检查表

以下步骤说明当前源码树生成候选版本所需的工作。完成验证后，再人工决定打标签和发布。

1. 在 Rust 包清单（`crates/*/Cargo.toml`、`apps/server/Cargo.toml`、`apps/cli/Cargo.toml` 和 `apps/desktop/src-tauri/Cargo.toml`）中设置目标版本，并保持 `apps/desktop/package.json`、`apps/desktop/src-tauri/tauri.conf.json`、`apps/extension/package.json`、`apps/extension/manifest.json` 的版本一致。JSON-RPC 协议版本是独立的兼容性编号。
2. 将发布内容从 `Unreleased` 移至 `CHANGELOG.md` 和 `CHANGELOG.zh-CN.md` 中带日期的版本条目。
3. 在仓库根目录运行[开发指南](DEVELOPMENT.zh-CN.md#rust-检查)列出的四条 Rust CI 命令。
4. 在 `apps/desktop` 运行 `pnpm install && pnpm build`；在 `apps/extension` 运行 `pnpm install && pnpm lint && pnpm build`。这些 TypeScript 检查目前不在 CI 中。
5. 运行 `cargo build --release -p nexum-server -p nexum-cli` 构建 Rust 命令行产物；可执行文件为 `target/release/nexum-server` 和 `target/release/nexum-cli`。桌面安装包需先构建前端，再在每个目标平台使用 Tauri 2 CLI，从 `apps/desktop` 执行 `cargo tauri build`；Tauri 打包配置启用了该平台支持的全部目标。
6. 打包扩展时包含根目录 `manifest.json`、`icons/` 和生成的 `dist/`。清单同时引用后两处的文件，仅有 `dist/` 无法作为扩展加载。
7. 使用本地 Server 对可执行文件和原生桌面包进行冒烟测试。通过 HTTP 提供一个已知文件，将其 URL 加入队列并调用 `task start`；在 Worker 阻塞期间查询 `task.get` 或 `task.list`，确认能看到中间字节数；Worker 完成后核对目标文件字节和已持久化的 `Completed` 状态。确认失败传输保留已有目标文件，并通过任务 `error` 字段返回错误；使用同一 `--data-dir` 重启 Server，确认错误仍保留，下载中断的任务变为 `Queued`，再次调用 `task start` 会从零开始传输并清除旧错误。确认活跃的 HTTP 传输拒绝暂停、恢复和删除，会拒绝数据目录内和重叠的目标。浏览器扩展请求的 HTTP `/jsonrpc` 尚未在 Server 中实现，因此暂不能把扩展创建任务视为通过的发布检查。
8. 验证实际产物后，人工创建版本标签和 GitHub Release；在发布说明中记录目标平台及尚未完成的集成。

英文版见 [RELEASE.md](RELEASE.md)。
