# 变更日志

此处记录 Nexum 的所有重要变更。

## 未发布

### 新增

- **协议**：协议版本化（`ProtocolVersion` V1），通过 `server.version` 与 `server.auth` RPC 方法协商版本
- **协议**：`RpcRequest` 新增可选 `credential` 字段，用于未来鉴权（通过 `#[serde(default)]` 向后兼容）
- **协议**：从协议层重导出所有安全类型：`Credential`、`AuthenticationScheme`、`AuthenticationError`、`TlsConfig`、`RateLimit`、`PathPattern`
- **协议**：测试数从 8 增至 20，覆盖版本化、凭证解析、服务端方法
- **核心**：10 个集成测试（从 4 个增加），覆盖完整生命周期、并发限制、事件消费、SQLite 持久化、任务清理
- **引擎**：HTTP 重定向跟随（最多 5 级）、下载中进度上报、目标目录自动创建
- **服务端**：可配置 `--port`、`--data-dir`、`--max-connections`、`--require-auth`，配置文件解析，版本/帮助输出，凭证日志记录
- **客户端**：`config get-server` / `config set-server`、`auth set` / `auth clear`、`server ping` / `server version` / `server auth`、`--version` 标志、RPC 错误格式化
- **安全**：`Credential` 枚举（None、Bearer、ApiKey）、`AuthenticationScheme`、`TlsConfig`、`AuthenticationError`、`CredentialStore`、`RateLimit`
- **插件**：`Permission` 枚举、`PathPattern`、`Capability`、`PluginManifest` 含构建器模式
- **媒体**：`MediaType`、`Track`、带构建器的 `MediaProbe`、`MuxSpec`
- **桌面端**：Tauri 2.0 + React 19 基础架构，完整任务管理 UI（列表、创建、队列、暂停、恢复、删除）

## [0.1.0] - 2025-XX-XX

- 初始项目结构（工作区、文档、CI）

中文版本请参见 [CHANGELOG.zh-CN.md](CHANGELOG.zh-CN.md)。
