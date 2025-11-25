# Contributing to Token Status List

Thank you for your interest in contributing to Token Status List! This document provides guidelines and instructions for contributing.

## Code of Conduct

This project adheres to a Code of Conduct that all contributors are expected to follow. Please read [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) before contributing.

## How to Contribute

### Reporting Bugs

If you find a bug, please open an issue with:
- A clear description of the bug
- Steps to reproduce
- Expected behavior
- Actual behavior
- Your environment (Rust version, OS, etc.)

### Suggesting Enhancements

Enhancement suggestions are welcome! Please open an issue with:
- A clear description of the enhancement
- Use cases and motivation
- Proposed implementation approach (if you have one)

### Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Make your changes** following the coding standards below
3. **Add tests** for new functionality
4. **Ensure all tests pass**: `cargo test --all-features`
5. **Run clippy**: `cargo clippy -- -D warnings`
6. **Check formatting**: `cargo fmt --all -- --check`
7. **Update documentation** as needed
8. **Open a pull request** with a clear description

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Cargo

### Building

```bash
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test --all-features

# Run with output
cargo test --all-features -- --nocapture
```

### Linting and Formatting

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings
```

## Coding Standards

- Follow Rust naming conventions
- Use meaningful variable and function names
- Add documentation comments for public APIs
- Keep functions focused and small
- Handle errors explicitly (avoid unwrap in library code)
- Write tests for new functionality

## Testing Guidelines

- Unit tests should be in the same file as the code (in `#[cfg(test)]` modules)
- Integration tests should be in `src/tests.rs`
- Aim for high test coverage
- Test both success and error cases

## Commit Messages

- Use clear, descriptive commit messages
- Start with a verb in imperative mood (e.g., "Add", "Fix", "Update")
- Reference issue numbers when applicable

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.

