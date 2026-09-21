# Roadmap

Nexum 的路线图会随着架构验证和社区反馈持续调整。

## Phase 1 — Core

- [x] Rust workspace
- [x] Domain model
- [x] 下载任务状态机
- [x] 调度器
- [x] 持久化存储
- [x] 事件总线
- [x] 引擎适配器接口

## Phase 2 — Protocol

- [x] Nexum Protocol
- [x] 任务 CRUD
- [x] 任务控制
- [x] 进度事件
- [x] 鉴权模型

## Phase 3 — Clients

- [x] CLI 基础（JSON-RPC 客户端、配置管理、认证）
- [x] 服务端基础（TCP、可配置、优雅关闭）
- [x] 桌面端基础（Tauri 2.0 + React 19）
- [ ] 浏览器集成

## Phase 4 — Extensibility

- [x] 插件清单（id、name、version、description、author、license）
- [x] 权限模型（None、Read、Write、Network、Execute）
- [x] 插件 SDK 骨架（Capability、PathPattern）
- [ ] 解析器扩展
- [ ] 引擎扩展

## Phase 5 — Media & Automation

- [ ] 媒体管道（MediaProbe、Track、MuxSpec）
- [ ] 自动化 API
- [ ] AI/MCP 集成
- [ ] 远程设备管理

> Roadmap 项目仅为方向性说明，不代表固定发布计划的承诺。
