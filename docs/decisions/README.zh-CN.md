# Architecture Decision Records

ADR 用于记录已经作出的重要技术决策，以及决策发生时的背景。

建议记录：

- 背景
- 决策
- 原因
- 影响
- 被考虑的替代方案

当前已记录的首个决策是[0001：持久化传输进度与错误](0001-persist-transfer-progress-and-errors.zh-CN.md)。后续决策请使用递增的编号文件名：

```text
docs/decisions/
├── 0001-use-rust-core.md
├── 0002-use-sqlite-storage.md
└── 0003-protocol-versioning.md
```
