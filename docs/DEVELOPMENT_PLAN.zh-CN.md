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
- [x] 第一个真实引擎集成 (InMemory)
- [x] 第二个引擎集成 (HTTP 重定向跟随)
- [x] 可控测试环境下的集成测试

### Phase 6 — Nexum Protocol

- [x] Protocol Envelope
- [x] Request / Response Model
- [x] Task APIs (list, get, create, queue, start, pause, resume, remove)
- [x] Event Stream (事件信封，传输中立)
- [x] Error Codes (JSON-RPC 2.0 标准 + 自定义)
- [x] Versioning (ProtocolVersion V1, server.version RPC)
- [x] Authentication Boundary (Credential, AuthenticationScheme, server.auth RPC)
- [x] Compatibility Tests (版本协商、凭证传递)

### Phase 7 — Server 与 CLI

- [x] Server Process (TCP 监听器，多线程)
- [x] Local Server Mode (默认 127.0.0.1:39100)
- [x] CLI Task Commands (list, get, create, queue, start, pause, resume, remove)
- [x] CLI JSON-RPC Client (TCP 套接字，基于行的协议)
- [x] Remote Connection (配置服务器、超时处理)
- [x] CLI 创建任务 (CLI 参数传递 id, source, destination)
- [x] CLI 控制任务 (所有任务操作)
- [x] CLI 状态与日志 (RPC 错误格式化、服务器认证/版本)
- [x] Configuration (CLI 参数、配置文件解析、服务器配置)

### Phase 8 — Desktop

- [x] Tauri Shell (Tauri 2.0, 可配置窗口)
- [x] React Application (React 19, Vite)
- [x] Task List (完整 CRUD, 状态显示, 进度)
- [x] Task Detail (可展开行, 字节格式化)
- [x] 添加下载 (表单, 验证, RPC 创建)
- [x] 暂停 / 恢复 / 删除 (连接到服务器)
- [x] Settings (服务器配置)
- [x] Event-driven Updates (RPC 轮询, 错误处理)

### Phase 9 — Browser Integration

- [x] Browser Extension (Manifest v3)
- [x] Context Menu 集成 (链接右键 "发送到 Nexum")
- [x] Link Interception (悬停检测, 可下载 URL)
- [x] Send-to-Nexum 流程 (后台服务 Worker, 通知)
- [x] Server / Device Selection (chrome.storage.local, 弹窗配置)

### Phase 10 — Extensibility

- [x] Plugin Manifest (id, name, version, description, author, license, entry)
- [x] Permission Model (None, Read, Write, Network, Execute)
- [x] Capability API (name, version, features)
- [x] Plugin SDK 骨架 (PathPattern)
- [ ] 插件生命周期 (安装/卸载, 版本验证)
- [ ] 解析器插件 (自定义源类型处理)
- [ ] 引擎插件 (自定义下载引擎适配器)

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

**Phase 0–7：全部已完成。** Phase 8（桌面端）已完成 Tauri + React 基础。**Phase 9（浏览器扩展）已完成上下文菜单、链接拦截和发送流程。** 下一步：完善插件 SDK 生命周期、管理、扩展能力，以及媒体管道和自动化 API。
