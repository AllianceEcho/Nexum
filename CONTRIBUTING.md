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

## Submission Standards

### Commit Format

Use Conventional Commits:

```
type: description
```

| type     | Description                      |
| -------- | -------------------------------- |
| `feat`   | New feature                      |
| `fix`    | Bug fix                          |
| `docs`   | Documentation change             |
| `style`  | Code formatting (no logic change) |
| `refactor` | Refactoring                    |
| `test`   | Test-related changes             |
| `chore`  | Build / toolchain / auxiliary    |

Type is followed by a space and colon. Description should be in English, no trailing period.

### Local CI

Run the following commands locally before submitting:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

All three must pass.

## Technical Discussion

Strong technical disagreement is welcome. Discuss code, design, evidence, and trade-offs rather than contributors.

## Security

Do not disclose unfixed security vulnerabilities in public Issues. Follow [SECURITY.md](SECURITY.md) for reporting guidance.

For the Chinese version, see [CONTRIBUTING.zh-CN.md](CONTRIBUTING.zh-CN.md).
