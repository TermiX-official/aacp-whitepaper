# Contributing to AACP

Thank you for your interest in contributing to the Agent Autonomous Commerce Protocol.

## How to Contribute

### Whitepaper Feedback

- Open an [Issue](../../issues) for questions, suggestions, or inconsistencies
- Use the `whitepaper` label for feedback on the protocol specification
- Reference specific section numbers (e.g., "Section 5.2 zkVM Evaluation")

### Protocol Design

- Major protocol changes should be proposed as a GitHub Issue first
- Include motivation, specification changes, and impact analysis
- Economic model changes require simulation results or formal analysis

### Verification Program Examples

We welcome example verification programs in `examples/`:

- Write in Rust (recommended) targeting RISC-V
- Include a README explaining what the program verifies
- Include test inputs (pass and fail cases)
- Follow the I/O contract: stdin for input, exit code 0/1 for pass/fail

### Smart Contract Review

Reference contract interfaces live in `contracts/`:

- Review for security vulnerabilities
- Suggest gas optimizations
- Propose interface improvements

### Translations

- Create a new directory: `translations/<language-code>/`
- Translate `whitepaper.md` and `README.md`
- Keep the same section numbering for cross-referencing

## Style Guide

- Use clear, precise language
- Define technical terms on first use
- Include concrete examples for abstract concepts
- Keep diagrams in ASCII art (for maximum compatibility)
- Reference specific EIPs and papers by number/name

## Code of Conduct

Be respectful, constructive, and focused on improving the protocol. We are building infrastructure for autonomous agent commerce — contributions should serve that goal.
