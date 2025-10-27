# Repository Guidelines

## Project Structure & Module Organization
Source lives in `src/`, with `main.rs` bootstrapping the CLI, `cli.rs` wiring Clap commands, `config.rs` handling persistence, `server.rs` orchestrating SteamCMD interactions, and `steam.rs` wrapping Steam-facing utilities. Shared assets such as README and licensing sit at the repository root, while build artifacts go to `target/`. Create integration tests under `tests/` and sample server fixtures under `fixtures/` when needed.

## Build, Test, and Development Commands
Use `cargo fmt` before committing to normalize formatting. Run `cargo check` for a fast compile-time validation and `cargo clippy -- -D warnings` to enforce lint cleanliness. Execute `cargo test` for the full suite, and `cargo run -- --help` to smoke-test the CLI surface after changes to `cli.rs`.

## Coding Style & Naming Conventions
Adopt Rust 2021 defaults: four-space indentation, snake_case for functions and modules, CamelCase for types, SCREAMING_SNAKE_CASE for constants. Keep Clap command names kebab-cased (e.g., `server-backup`) to match existing subcommand patterns. Prefer explicit async boundaries; document non-obvious flows with short comments above the relevant block.

## Testing Guidelines
Unit tests belong adjacent to code via `#[cfg(test)]` modules; leverage `tokio::test` for async scenarios. Place scenario-driven integration tests in `tests/` using descriptive filenames like `tests/server_backup.rs`. Target meaningful coverage on CLI parsing, configuration serialization, and SteamCMD process handling, and ensure `cargo test` passes locally before opening a pull request.

## Commit & Pull Request Guidelines
The repository currently has no commit history; start with concise, imperative subject lines (e.g., `Add server backup workflow`) capped near 72 characters, followed by a short body when context helps. Reference GitHub issues using `Fixes #NN` when applicable. Pull requests should detail motivation, highlight risky areas, include manual test notes (commands run, configs touched), and add screenshots or logs when UX changes are involved.

## Configuration & Security Tips
Avoid committing real Steam credentials; rely on environment variables or `.env` entries excluded via `.gitignore`. Validate that SteamCMD is discoverable on PATH before invoking CLI commands and document platform-specific paths in the PR when they differ. Scrub server logs for sensitive tokens before sharing in issues or reviews.
