# 架构设计

本文描述仓库中已经存在的行为。crate 中定义了类型或 trait，并不代表 Server 或客户端已经接入该能力。

## 运行边界

```text
CLI ───────────────┐
Desktop (Tauri) ───┼── 基于 TCP 的按行 JSON-RPC 2.0
                   ▼
               本地 Server
                   │
              RpcDispatcher
                   │
                   ▼
                 Core
       ┌───────────┼────────────┐
  TaskService   Scheduler   TaskRepository
       │            │              │
  Task Events   Queue/Events  InMemory (Server)
                    │           SQLite (可注入)
                    ▼
              EngineRegistry
             /              \
       InMemoryEngine     HttpEngine

Browser 扩展 ── HTTP POST /jsonrpc（Server 尚无对应端点）
```

Server 默认监听 `127.0.0.1:39100`。当前实现始终绑定回环地址，`--port` 只改变端口。CLI 可以连接配置的 TCP 地址，Desktop UI 可以填写 Server 地址。Browser 扩展发送的 HTTP 请求与当前仅支持 TCP 行协议的 Server 不兼容。

## Crate 职责

| Crate | 已实现职责 |
| --- | --- |
| `domain` | Task ID、来源、目标路径和进度值类型。 |
| `task` | 内存中的任务服务、经过校验的状态转换和 Task Events。 |
| `scheduler` | 显式队列操作、优先级、并发上限、重试策略、带宽策略计算和 Scheduler Events。不执行网络 I/O，也不会自动启动排队任务。 |
| `storage` | `TaskRepository` 的内存与 SQLite 实现；SQLite 包含 Schema Version 和 Migration。 |
| `resolver` | 通过 Registry 对 HTTP/HTTPS、Magnet 和本地来源分类、校验。 |
| `engine` | Adapter/Registry API、模拟的 InMemory Engine 和阻塞式 HTTP GET Engine。 |
| `core` | 协调 Task、Scheduler、Resolver、Engine、Repository 与 Plugin Manager 状态。 |
| `security` | Credential、TLS 配置、限流配置和 Credential Store 类型。 |
| `protocol` | JSON-RPC 请求/响应分发、Task/Server 方法、错误对象与事件信封。 |
| `plugin` | Manifest/Permission/Capability 类型、Provider Trait 和 Plugin Manager 状态机。 |
| `media` | 媒体与工作流数据类型、依赖排序、模拟的内存 Job。 |

## 任务路径

`Core::create_task` 经 `ResolverRegistry` 校验来源，创建 `DownloadTask` 并写入注入的 Repository。`queue_task` 将任务加入 Scheduler。`start_next` 显式取出下一个任务，默认选用 `InMemoryEngine`；`start_next_with_engine("http")` 只是库 API，不是 JSON-RPC 方法或 CLI 选项。因此默认 Server 调用 `task.start` 时不会下载文件。

任务状态机允许：

```text
Created     → Queued
Queued      → Downloading
Downloading → Paused | Completed | Failed
Paused      → Queued | Downloading
Completed   → Queued
Failed      → Retrying → Queued
```

Scheduler 在调用 `start_next` 或 `resume` 时检查并发数，并按重试策略重新入队失败任务。带宽策略目前只计算限速值，HTTP 传输并未应用。Task 和 Scheduler Events 在 Core 中收集、可取出，但 Server 没有向客户端发布。

## Engine 与来源

Resolver 接受 HTTP/HTTPS URL、包含 `xt=urn:btih:` 参数的 Magnet URI，以及存在的本地路径；它不会校验 Magnet Hash 本身。解析来源不会自动选择对应 Engine。`HttpEngine` 可以将阻塞式 GET 响应写入目标文件，最多跟随五次重定向；它不支持暂停/恢复，在调用阻塞期间也不提供增量进度。InMemory Engine 只模拟生命周期，不传输字节。当前没有 Magnet 或本地文件传输 Engine。

## 持久化与恢复

`Core<R>` 接受 `TaskRepository`。SQLite Repository 持久化任务元数据和进度，`Core::recover` 恢复已保存任务；原先执行中、暂停或重试中的任务会重新排队。这些路径有库级测试。Server 当前创建的是使用 `InMemoryRepository` 的 `Core::new`，也没有调用 `recover`；其 `data_dir` 设置只用于创建目录。重启 Server 后任务不会保留。

## Protocol 与客户端

Server 每次从 TCP 连接读取一行 JSON-RPC 请求，对带 `id` 的请求写回一行响应。Dispatcher 支持 `task.get`、`task.list`、`task.create`、`task.queue`、`task.start`、`task.pause`、`task.resume`、`task.remove`、`server.version` 和 `server.auth`，没有通用的 Task Update 方法。Protocol 包含 V1 版本字段和版本查询方法，但没有协商功能集合；除 JSON-RPC `2.0` 信封校验外，也没有版本强制校验。crate 中已有事件信封转换和缓存，Server 尚无订阅或推送通道。

CLI 通过 TCP 协议管理任务、查询 Server。Tauri 2 + React Desktop 通过 Tauri 命令调用 TCP JSON-RPC，包含任务列表、添加与控制视图。它在操作后或 Server 地址变更时刷新，没有定时轮询或事件推送；Server 地址只保存在组件状态。Manifest V3 Browser 扩展有右键菜单和链接标记 UI，但发送流程向 `/jsonrpc` 发 HTTP 请求，目前没有兼容端点。Popup 写入 Storage 的 `server` 键，后台脚本却读取 `address` 字段，因此保存的地址不会生效。扩展也没有设备选择。

Protocol 请求可以携带 Credential，`server.auth` 目前只返回 `none`。Server 不校验 Credential；`require_auth` 与 `max_connections` 配置虽可解析，但未执行限制。TLS 和限流类型也未接入 Server。

## 扩展与 Media 边界

`PluginManager` 可以注册 Manifest 并记录加载、启动、停止状态，crate 定义了 `EngineProvider` 与 `ResolverProvider` Trait。Core 未调用插件实现或注册插件提供的 Adapter；`init_plugins` 只推进 Manager 状态。Permission 声明目前只是元数据，没有沙箱约束。动态加载和可执行插件集成尚未实现。

Media crate 定义了 Probe、Manifest、Track、Segment、Mux、Pipeline、Job、Workflow 和 MCP Request 类型。Workflow 依赖排序与内存 Job 管理已经实现。`MediaProcessor::probe` 返回默认元数据，Job 执行会构造模拟的完成结果；真实文件探测、Manifest 解析、分段处理、Mux、外部自动化端点和 MCP 集成均未接入。
