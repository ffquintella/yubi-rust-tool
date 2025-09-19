# Repository Guidelines

## Project Structure & Module Organization
- Root files: `Cargo.toml` (crate metadata), `Cargo.lock`, `LICENSE`.
- Source code lives in `src/` (entrypoint: `src/main.rs`).
- Build artifacts go to `target/` (ignored by Git).
- Add unit tests beside code; put integration tests in `tests/` (one file per feature).
- Naming: modules/files use `snake_case`; types/enums use `CamelCase`; constants use `SCREAMING_SNAKE_CASE`.

## Build, Test, and Development Commands
- Build: `cargo build` — compiles the project in debug mode.
- Run: `cargo run -- <args>` — runs the binary locally.
- Check: `cargo check` — fast type-check without producing binaries.
- Test: `cargo test` — runs unit and integration tests.
- Format: `cargo fmt --all` — formats code with rustfmt.
- Lint: `cargo clippy --all-targets --all-features -D warnings` — static analysis; fix or justify lints.

## Coding Style & Conventions
- Follow rustfmt defaults (4 spaces, no tabs); run format before committing.
- Prefer small, focused modules; keep functions short and pure when possible.
- Error handling: return `Result<T, E>` and propagate with `?` instead of `unwrap()`/`expect()`.
- Document public items with `///` and modules with `//!`.
- CLI args, logs, and features should be additive and backward compatible.

## Testing Guidelines
- Unit tests: add `#[cfg(test)] mod tests { ... }` at the bottom of source files.
- Integration tests: create `tests/<feature>_test.rs`; use the public API only.
- Name tests descriptively: `does_<thing>_when_<condition>()`.
- Run the full suite and ensure determinism (no network, no time-based flakiness).

## Commit & Pull Request Guidelines
- Use Conventional Commits: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`.
- Keep commits scoped and atomic; include rationale in the body when helpful.
- Before opening a PR: run `cargo fmt`, `cargo clippy -D warnings`, and `cargo test`.
- PRs should include: concise description, linked issues (`Closes #123`), and example commands/output when relevant.

## Security & Configuration Tips
- Do not commit secrets or tokens; prefer environment variables.
- Validate all external input; avoid panics in production paths.
- Review dependencies before adding; keep minimal and up to date.
