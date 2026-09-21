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
- [ ] CI 上完整通过 workspace 构建
- [ ] 基础格式化与 lint 规范

**完成标准：** 全新 checkout 后，可以按照开发文档成功运行 Rust 检查。

### Phase 1 — Domain 与 Task Core

目标：定义“下载任务是什么”以及“下载任务如何运行”。

#### 1A — Domain Model

- [x] Task ID
- [x] 下载源模型
- [x] 目标路径模型
- [x] 基础任务元数据
- [x] 进度模型
- [x] 错误模型

#### 1B — Task 状态机

- [ ] Created
- [ ] Queued
- [ ] Downloading
- [ ] Paused
- [ ] Completed
- [ ] Failed
- [ ] Retry
- [ ] 明确的状态转换验证
- [ ] 有效与无效状态转换单元测试

#### 1C — Task Service

- [ ] 创建任务
- [ ] 加入队列
- [ ] 暂停 / 恢复
- [ ] 完成 / 失败
- [ ] 重试
- [ ] 删除
- [ ] Task Events

**完成标准：** 不依赖真实下载引擎，任务行为完全可预测并有完整单元测试。

### Phase 2 — Scheduler

- [x] Queue 抽象
- [x] 并发限制
- [x] 优先级
- [x] Retry Policy
- [x] 暂停 / 恢复调度
- [x] 带宽策略抽象
- [x] Scheduler Events

### Phase 3 — Storage

Scheduler Phase 2 已完成，下一阶段进入 Storage。存储层将通过 Repository trait 与 Core 解耦，并先保留 InMemory 实现作为测试基础。

- [x] Repository traits
- [x] SQLite 实现
- [x] Schema versioning
- [x] Migration 机制
- [x] Task 持久化
- [x] 重启恢复

### Phase 4 — Resolver

- [x] Resolver Trait 与 Request / Result Model
- [x] HTTP/HTTPS Source 分类
- [x] Magnet Source 分类
- [x] HTTP/HTTPS URL Resolver
- [x] Magnet Resolver
- [x] 本地 Source 校验
- [x] Resolver Registry
- [x] Resolver Error Model
- [x] Resolver 测试

### Phase 5 — Engine Adapter

- [x] Adapter Trait
- [x] Engine Capabilities
- [x] Task Mapping
- [x] Progress Mapping
- [x] Pause / Resume / Remove Mapping
- [x] 第一个真实引擎集成
- [ ] 可控测试环境下的集成测试

### Phase 6 — Nexum Protocol

- [ ] Protocol Envelope
- [ ] Request / Response Model
- [ ] Task APIs
- [ ] Event Stream
- [ ] Error Codes
- [ ] Versioning
- [ ] Authentication Boundary
- [ ] Compatibility Tests

### Phase 7 — Server 与 CLI

- [ ] Server Process
- [ ] Local Server Mode
- [ ] Remote Connection
- [ ] CLI 创建任务
- [ ] CLI 控制任务
- [ ] CLI 状态与日志
- [ ] Configuration

### Phase 8 — Desktop

- [ ] Tauri Shell
- [ ] React Application
- [ ] Task List
- [ ] Task Detail
- [ ] 添加下载
- [ ] 暂停 / 恢复 / 删除
- [ ] Settings
- [ ] Event-driven Updates

### Phase 9 — Browser Integration

- [ ] Browser Extension
- [ ] Context Menu
- [ ] Link Interception
- [ ] Send-to-Nexum
- [ ] Server / Device Selection

### Phase 10 — Extensibility

- [ ] Plugin Manifest
- [ ] Permission Model
- [ ] Capability API
- [ ] Plugin SDK
- [ ] Resolver Plugins
- [ ] Engine Plugins
- [ ] Plugin Lifecycle

### Phase 11 — Media & Automation

- [ ] Media Probe
- [ ] Manifest Parsing
- [ ] Track Selection
- [ ] Segment Scheduling
- [ ] Mux / Post-processing
- [ ] Automation API
- [ ] AI / MCP Integration

## 3. 实施顺序

Foundation → Domain → Task State Machine → Task Service → Scheduler → Storage → Resolver → Engine Adapter → Protocol → Server/CLI → Desktop/Browser。

在 Core 和 Protocol 形成稳定边界之前，不开始桌面端开发。

## 4. 工程规则

### 测试

Core 行为必须可以在没有网络的情况下测试。网络和真实引擎测试放在 Integration Test 层。

### 依赖

优先选择小而职责明确的依赖。一个依赖应该解决明确的问题。

### API 稳定性

任何跨 crate 暴露的接口都应视为 API。Breaking Change 必须经过明确设计并留下文档记录。

### 错误处理

错误必须保留足够上下文，不能在子系统边界简单退化成没有上下文的字符串。

### 可观测性

Core 最终应使用结构化 Event 和结构化 Log，而不是临时的打印语句。

### 兼容性

Protocol、Storage Schema 和 Plugin API 在正式公开前都必须建立明确的版本策略。

## 5. Git 工作流

- main 始终保持可构建。
- 功能开发使用独立分支。
- 小型修复通过 PR。
- 架构变化必须 RFC。
- 每个里程碑形成易于审查的提交。
- Release 从经过验证的 commit 创建 tag。

## 6. 当前立即执行

**Phase 2 — Scheduler 已完成。** **Phase 3 — Storage 已完成。** Repository、SQLite、Schema versioning、Migration、Task 持久化与初步重启恢复均已接入。**Phase 4 — Resolver 已完成基础实现，并已接入 Core 任务创建流程。Phase 5 — Engine Adapter 已启动，完成 Adapter 边界、Capabilities 与 Task Mapping。下一步加强 HTTP Engine 集成测试，并开始 Nexum Protocol 边界。**
