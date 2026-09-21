# Nexum 开发计划

## 1. 开发原则

Nexum 从核心向外开发。

1. 先定义领域模型，再构建 UI。
2. 先定义任务生命周期，再实现调度器。
3. 围绕稳定的 Core 设计 Protocol。
4. 通过明确接口隔离持久化实现。
5. 核心抽象经过测试后，再接入具体下载引擎。
6. 每个里程碑完成后，仓库都必须保持可构建、可测试。

## 2. 开发阶段

### Phase 0 — 项目基础

- [x] 仓库结构
- [x] MIT License
- [x] 贡献与治理文档
- [x] 中英文文档体系
- [x] Rust workspace
- [x] GitHub CI
- [ ] CI 上验证 workspace 完整构建通过
- [ ] 正式的格式化与 lint 规范

### Phase 1 — Domain 与 Task Core

- [x] Domain Model
- [x] Task 状态机
- [x] 状态转换验证
- [x] Task Service
- [x] Task Events
- [x] 单元测试

### Phase 2 — Scheduler

- [x] Queue 抽象
- [x] 并发限制
- [x] 优先级
- [x] Retry Policy
- [x] 暂停 / 恢复调度
- [x] 带宽策略抽象
- [x] Scheduler Events

### Phase 3 — Storage

- [x] Repository Trait
- [x] InMemory Repository
- [x] SQLite 实现
- [x] Schema Versioning
- [x] Migration
- [x] Task 持久化
- [x] 重启恢复

### Phase 4 — Resolver

- [x] Resolver Trait 与 Request / Result Model
- [x] HTTP/HTTPS 分类与校验
- [x] Magnet 分类与校验
- [x] 本地来源处理
- [x] Resolver Registry
- [x] Resolver Error Model
- [x] Resolver 测试

### Phase 5 — Engine Adapter

- [x] Adapter Trait
- [x] Engine Capabilities
- [x] Task / Progress Mapping
- [x] Pause / Resume / Remove Mapping
- [x] InMemory Engine
- [x] HTTP Engine 与重定向跟随
- [x] 受控 Engine 测试

### Phase 6 — Nexum Protocol

- [x] JSON-RPC 2.0 Envelope
- [x] Request / Response Model
- [x] Task APIs
- [x] 传输中立事件信封
- [x] Error Codes
- [x] Protocol 版本协商
- [x] 鉴权边界
- [x] Compatibility Tests

### Phase 7 — Server 与 CLI

- [x] TCP Server
- [x] Local Server Mode
- [x] CLI JSON-RPC Client
- [x] 任务创建与控制
- [x] 可配置 Server 连接
- [x] 认证与服务器信息命令
- [x] Server Configuration

### Phase 8 — Desktop

- [x] Tauri 2 Shell
- [x] React Application
- [x] Task List / Detail
- [x] 添加下载
- [x] 暂停 / 恢复 / 删除
- [x] Server Settings
- [x] 事件驱动 / 轮询更新

### Phase 9 — Browser Integration

- [x] Manifest V3 Extension
- [x] Context Menu
- [x] 可下载链接拦截
- [x] Send-to-Nexum
- [x] Server / Device Selection

### Phase 10 — Extensibility

- [x] Plugin Manifest
- [x] Permission Model
- [x] Capability API
- [x] Plugin SDK 骨架
- [ ] Plugin 生命周期
- [ ] Resolver Plugins
- [ ] Engine Plugins

### Phase 11 — Media & Automation

- [x] 基础媒体数据结构
- [ ] 媒体探测工作流
- [ ] Manifest Parsing
- [ ] Track Selection
- [ ] Segment Scheduling
- [ ] Mux / Post-processing
- [ ] Automation API
- [ ] AI / MCP Integration
- [ ] Remote Device Management

## 3. 实施顺序

Foundation → Domain → Task State Machine → Task Service → Scheduler → Storage → Resolver → Engine Adapter → Protocol → Server/CLI → Desktop/Browser → Extensibility → Media/Automation。

## 4. 工程规则

### 测试

Core 行为必须可以在无网络环境下测试。网络和 Engine 行为应放在受控集成测试层。

### 依赖

优先选择小而明确的依赖。每个依赖都应该解决具体问题。

### API 稳定性

任何跨 crate 暴露的 API 都应视为 API 边界。Breaking Change 必须经过明确讨论并记录。

### 错误处理

错误应保留可操作的上下文，不能在子系统边界被简单压缩成无上下文字符串。

### 可观测性

Core 操作应使用结构化事件，并逐步采用结构化日志，而不是依赖零散输出。

### 兼容性

Protocol、Storage Schema 和 Plugin API 在成为稳定公共接口前，都需要明确版本策略。

## 5. Git 工作流

- main 保持可构建。
- 功能开发使用聚焦的分支。
- 小修复可以直接进入 PR。
- 架构变化必须先 RFC。
- 每个里程碑应由可审查的提交组成。
- Release 从已知可用的 commit 打标签。

## 6. 当前重点

Phase 0–9 已在仓库中形成可工作的基础能力。Phase 10 已完成 Plugin Manifest、Permission、Capability 与 SDK 骨架。下一阶段重点是 Transport 层和 Plugin 生命周期，然后进入 Phase 11 的媒体与自动化工作。
