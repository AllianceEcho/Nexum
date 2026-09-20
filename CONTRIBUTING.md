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

## Technical Discussion

Strong technical disagreement is welcome. Discuss code, design, evidence, and trade-offs rather than contributors.

## Security

Do not disclose unfixed security vulnerabilities in public Issues. Follow [SECURITY.md](SECURITY.md) for reporting guidance.

For the Chinese version, see [CONTRIBUTING.zh-CN.md](CONTRIBUTING.zh-CN.md).
