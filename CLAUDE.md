# CLAUDE.md

## Build Commands

- Build: `cargo build` (debug) / `cargo build --release`
- Run: `cargo run -- [file]`
- Test: `cargo test`
- Lint: `rustup component add clippy && cargo clippy -- -D warnings`
- Format: `cargo fmt` (auto) / `cargo fmt --check`

## Project Structure

- Entry: `src/main.rs`
- Core: `src/editor.rs`, `src/buffer.rs`, `src/renderer.rs`, `src/terminal.rs`
- Plugins: `src/plugin/`
- Highlight: `src/highlight/`

## Constraints

- No Bun/Node.js - this is a Rust project.
- Use cargo for all build operations.