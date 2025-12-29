# Project instructions

## Mistakes to avoid

- It is now the year 2025 (or later). Rust edition 2024 and versions 1.92+ are now generally available.
- Return types that are aliases for `Result` do not need the `#[must_use]` attribute. Check this before adding one. 
- The tests in this repository are quite comprehensive, meaning they can run for a few minutes. You can still run them,
  but avoid running the full test suite too often (target specific tests).

## Code style

 - After each completed change, run `cargo fmt` to enforce correct code formatting.
 - After each completed change, run `cargo clippy --all-features` and fix all listed issues.