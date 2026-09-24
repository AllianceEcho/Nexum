# Contributing

Thank you for contributing to Nexum.

Nexum is in early development. To keep the architecture clear, we follow a simple rule:

**Small changes can go directly to a PR. Large changes should be discussed first. Architectural changes require an RFC.**

## Before Contributing

1. Read the [Development Guide](docs/DEVELOPMENT.md).
2. Check whether the relevant context is already documented in an Issue, Discussion, or RFC.
3. Keep each change focused and avoid unrelated refactoring.
4. Add tests and documentation for behavioral changes.

## When an RFC Is Required

Use an RFC for:

- Core architecture changes
- New or breaking Protocol changes
- Data model or database migrations
- Core task state-machine changes
- Plugin permission model changes
- Engine Adapter abstraction changes
- Breaking changes affecting multiple modules

## Pull Requests

A PR should explain:

- What problem it solves
- Why the proposed approach was chosen
- Compatibility impact
- How it was validated
- Whether documentation needs to change

Keep commits focused. Reviewers will prioritize correctness, maintainability, test coverage, and long-term compatibility.

Keep `main` buildable and Core behavior testable without network access; use controlled integration tests for network and engine behavior. Add dependencies for concrete needs, preserve actionable error context, and use structured task/scheduler events as runtime behavior expands. Treat public crate APIs and protocol, storage, and plugin contracts as compatibility boundaries; establish versioning before declaring them stable and document deliberate breaking changes.

## Submission Standards

### Commit Format

Use Conventional Commits:

```
type: description
```

| Type | Description |
| ---- | ----------- |
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation change |
| `style` | Code formatting (no logic change) |
| `refactor` | Refactoring |
| `test` | Test-related changes |
| `chore` | Build / toolchain / auxiliary change |

Place a colon and space after the type. Write the description in English without a trailing period, following the existing commit history.

### Local CI

Before submitting, run the Rust checks in the [Development Guide](docs/DEVELOPMENT.md#rust-checks); they match [CI](.github/workflows/ci.yml). The Rust workspace includes the Tauri crate and needs its platform build dependencies. For frontend or extension changes, also run the applicable package checks in the Development Guide; CI does not run them yet.

## Technical Discussion

Strong technical disagreement is welcome. Discuss code, design, evidence, and trade-offs rather than contributors.

## Security

Do not disclose unfixed security vulnerabilities in public Issues. Follow [SECURITY.md](SECURITY.md) for reporting guidance.

For the Chinese version, see [CONTRIBUTING.zh-CN.md](CONTRIBUTING.zh-CN.md).
