# 贡献指南

感谢你参与 Nexum。

Nexum 目前处于早期开发阶段。为了保持核心架构清晰，我们采用一个简单的原则：

> **小改动直接 PR，大改动先讨论，架构改动必须 RFC。**

## 提交前

1. 阅读 [开发指南](docs/DEVELOPMENT.zh-CN.md)。
2. 确认 Issue、Discussion 或 RFC 中已经记录了相关背景（如果适用）。
3. 保持改动范围清晰，避免把无关重构混入功能或修复。
4. 为行为变化补充测试和文档。

## 什么情况需要 RFC

以下类型的改动应先提出 RFC：

- 核心架构调整
- Nexum Protocol 的新增或破坏性修改
- 数据模型或数据库迁移
- 核心任务状态机变化
- 插件权限模型变化
- Engine Adapter 抽象变化
- 可能影响多个模块的 Breaking Change

## Pull Request

PR 应说明：

- 改动解决了什么问题
- 为什么采用当前方案
- 是否存在兼容性影响
- 如何验证
- 是否需要更新文档

保持提交小而聚焦。维护者会优先关注正确性、可维护性、测试覆盖和长期兼容性。

## 提交规范

### 格式

提交信息使用 Conventional Commits 格式：

```
type: description
```

| 类型 | 说明 |
| ---- | ---- |
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `docs` | 文档变更 |
| `style` | 代码格式（不影响逻辑） |
| `refactor` | 重构 |
| `test` | 测试相关 |
| `chore` | 构建 / 工具链 / 其他辅助变更 |

类型后加空格和冒号，描述使用中文。

### 本地 CI 检查

提交前必须在本地跑通以下命令：

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

三项全部通过方可提交。

## 技术讨论

可以直接、充分地讨论技术方案，也可以提出反对意见。请针对代码、设计和证据讨论，不针对贡献者本人。

## 安全问题

不要在公开 Issue 中披露尚未修复的安全漏洞。请按照 [SECURITY.md](SECURITY.md) 中的流程报告。
