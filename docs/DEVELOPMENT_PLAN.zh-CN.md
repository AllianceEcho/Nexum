# Nexum 开发计划

本计划区分“代码中已有基础能力”和“当前 Server/客户端可端到端使用的功能”。勾选表示仓库中已有对应实现；未勾选表示仍需实现或接线。计划项目仅说明方向，不承诺固定的发布时间。当前调用路径见[架构设计](ARCHITECTURE.zh-CN.md)，工程和提交规范见[贡献指南](../CONTRIBUTING.zh-CN.md)。

## 1. 开发阶段

### Phase 0 - 项目基础

- [x] 仓库结构、MIT License、贡献与治理文档
- [x] 英文和简体中文文档
- [x] 包含 Server、CLI、Desktop Tauri crate 和 Core crates 的 Rust workspace
- [x] GitHub CI 工作流已配置格式检查、workspace check/test 和 Clippy

### Phase 1 - Domain 与 Task Core

- [x] Domain 值类型和下载任务模型
- [x] 带校验的任务状态机
- [x] 内存任务服务与 Task Events
- [x] 任务生命周期和状态转换单元测试

### Phase 2 - Scheduler

- [x] 优先级队列与任务并发上限
- [x] 重试策略和暂停/恢复操作
- [x] 带宽策略接口与限速值计算
- [x] Scheduler Events 与受控单元测试
- [ ] 将计算出的带宽限制应用到传输
- [ ] 自动派发排队任务和重试；HTTP Worker 目前需要显式调用 `task.start`

### Phase 3 - Storage

- [x] `TaskRepository` 与内存实现
- [x] 带 Schema Version 和 Migration 的 SQLite 任务元数据/进度 Repository
- [x] 重建排队任务并持久化恢复状态的 `Core::recover`
- [x] 在 Server 启动路径打开 SQLite 并执行恢复
- [x] 验证真实 Server 重启后任务连续性

### Phase 4 - Resolver

- [x] Resolver 请求/结果/错误模型与 Registry
- [x] HTTP/HTTPS 校验、Magnet `xt=urn:btih:` 参数存在性检查、已有本地路径校验
- [x] Resolver 测试与 Core 创建任务时的来源校验
- [x] 从 Server 的 `task.start` 将排队的 HTTP/HTTPS 任务路由到 HTTP Worker
- [ ] 为 Magnet 和本地文件来源接入兼容的传输 Engine
- [ ] 增加真实 Magnet 与本地来源传输路径

### Phase 5 - Engine Adapter

- [x] Adapter Capability、Task Mapping 与 Engine Registry
- [x] 模拟 InMemory Engine 和支持重定向的阻塞式 HTTP GET Engine
- [x] 受控 Engine 测试
- [x] 从 Server 的 `task.start` 运行 HTTP Engine；CLI 与 Desktop 使用同一个 RPC
- [x] 将 HTTP 下载暂存到 `.part` 文件，仅在响应完整后重命名到目标路径
- [x] 为真实传输增加增量进度报告与持久化
- [ ] 为真实传输增加取消与可用的暂停/恢复能力

### Phase 6 - Nexum Protocol 与 Security

- [x] JSON-RPC 2.0 请求/响应与错误对象
- [x] Task 创建/查询/列表/排队/启动/暂停/恢复/删除，以及 Server 信息查询方法
- [x] V1 版本类型、请求字段和 `server.version` 方法
- [x] Task/Scheduler 事件信封与缓存
- [x] Credential、TLS、限流类型及 Protocol/Security 单元测试
- [ ] 在信封校验之外执行 Protocol 兼容性检查
- [ ] 通过 Server 传输发布事件，并让客户端接收
- [ ] 校验 Credential，并按配置执行认证、TLS 与限流

### Phase 7 - Server 与 CLI

- [x] 使用按行 JSON-RPC 的本地回环 TCP Server
- [x] 支持任务控制、地址配置和 Server 信息查询的 CLI TCP 客户端
- [x] Server 命令行选项与 key-value 配置解析
- [ ] 执行 `require_auth` 和 `max_connections`；当前只解析这两项
- [x] 利用 `data_dir` 实现 SQLite 持久化与重启恢复
- [x] 让普通 `task.start` 对支持的 HTTP/HTTPS 来源启动真实下载
- [x] 持久化传输错误并通过任务视图返回，而不只写入 Server 日志
- [ ] 自动派发排队中的重试

### Phase 8 - Desktop

- [x] Tauri 2 + React 应用与 TCP JSON-RPC 命令桥接
- [x] Task 列表、添加、排队/启动、暂停/恢复与删除 UI
- [x] 可编辑 Server 地址，操作或地址变更后刷新
- [ ] 增加定时或事件驱动更新，反映进度和外部任务变化
- [ ] 持久化 Server 设置，可靠呈现连接与操作失败

### Phase 9 - Browser 集成

- [x] Manifest V3 扩展骨架、链接右键菜单和可下载链接标记启发式逻辑
- [x] 保存单个 Server 地址的 Popup 字段
- [ ] 使用受支持的传输完成 Send-to-Nexum；扩展向 HTTP `/jsonrpc` 发请求，而 Server 只支持 TCP
- [ ] 在后台脚本使用已保存的 `server` 地址；当前读的是 `address` 字段
- [ ] 处理 Content Script 的发送消息，并完成端到端任务创建测试
- [ ] 若多设备投递仍是产品需求，增加设备选择

### Phase 10 - 可扩展性

- [x] Plugin Manifest、Permission 和 Capability 数据模型
- [x] `PluginManager` 状态转换与测试
- [x] `EngineProvider` 和 `ResolverProvider` Trait
- [ ] 调用插件生命周期实现并加载可执行插件入口
- [ ] 向 Core 注册插件提供的 Engine/Resolver；当前初始化只改变 Manager 状态
- [ ] 执行 Permission 约束，并定义稳定的 SDK/Runtime 合约

### Phase 11 - Media 与 Automation

- [x] Media、Job、Workflow 与 MCP Request 基础类型
- [x] Workflow 依赖排序和模拟的内存 Job API
- [ ] 探测真实媒体并解析 Manifest
- [ ] 实现 Track Selection、Segment Scheduling、Mux 与后处理
- [ ] 执行并持久化真实 Job/Workflow，而不是构造模拟完成结果
- [ ] 对外提供 Automation 端点，并按需集成 AI/MCP
- [ ] 实现远程设备管理

## 2. 基于当前代码的实施顺序

1. 补齐 HTTP 运行路径：取消或可用的暂停/恢复，以及自动派发重试。
2. 补齐 Server/客户端合约：按配置认证的传输、可观察的事件，以及 Browser 可用的端点或桥接。
3. 接入 Plugin Provider 并执行其声明的权限。
4. 用真实处理替换模拟的 Media 操作，再对外提供 Automation 与远程设备工作流。

## 3. 当前重点

Phase 0-9 各自具有不同程度的脚手架和库级覆盖。运行中的 Server 使用 SQLite 持久化任务并在重启后恢复，`task.start` 会启动真实的 HTTP/HTTPS Worker。Server 会节流持久化中间进度，通过任务视图返回最近一次传输错误，并在传输成功后写入最终进度与完成状态。活跃的 HTTP 传输不能暂停、恢复或删除；当前没有取消能力，也不会自动派发重试。Magnet 和本地文件传输仍不受支持。Plugin 与 Media crates 已包含数据类型之外的代码，但 Provider 回调和真实处理尚未接入产品路径。
