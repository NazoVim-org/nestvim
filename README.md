# nestvim

A minimal Vim-like TUI editor written in Rust.

## Features

- Vim modes: Normal, Insert, Visual, Command, Replace
- Vim operators: d, y, c, p, >, etc.
- Registers (yank/paste)
- Undo/Redo
- Syntax highlighting (syntect)
- Multi-language plugin support (Lua, Lisp, JavaScript, Rust, Nix)

## Install

```sh
cargo install nestvim
```

## Development

```sh
cargo build
cargo run -- [file]
cargo test
```

## License

MIT