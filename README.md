# nestvim

A minimal Vim-like TUI text editor built with Bun and TypeScript.

## Features

- **Modal Editing**: Normal, Insert, and Command modes with Vim-like keybindings (h/j/k/l navigation, `i` to enter Insert mode, `Esc` to return to Normal mode)
- **Syntax Highlighting**: Powered by Tree-sitter with support for multiple languages (Lua, Markdown, TypeScript, Python, Rust, etc.)
- **Plugin System**: Extend functionality via Lua or TypeScript plugins, with event hooks, custom keymaps, and commands
- **Text Buffer Management**: Uses `@codemirror/state` for efficient text manipulation
- **Cross-Platform Support**: Nix flake for reproducible development environments and packaging

## Prerequisites

- [Bun](https://bun.sh) (v1.0+ recommended)
- (Optional) [Nix](https://nixos.org) for development shell and packaging

## Installation

### Quick Start
```bash
git clone https://github.com/NazoVim-org/nestvim.git
cd nestvim
bun install
```

### Nix
Enter the development shell:
```bash
nix develop
```

Build the package:
```bash
nix build
./result/bin/nestvim
```

## Usage

Start the editor with an optional file path:
```bash
bun run start [filepath]
# Or if installed via Nix:
nestvim [filepath]
```

### Keybindings

| Mode       | Key          | Action                                  |
|------------|--------------|-----------------------------------------|
| Normal     | `h`/`j`/`k`/`l` | Move cursor left/down/up/right       |
| Normal     | `i`          | Enter Insert mode                       |
| Normal     | `:`          | Enter Command mode                      |
| Insert     | `Esc`        | Return to Normal mode                   |
| Insert     | `Backspace`  | Delete character before cursor (merges lines if at start of line) |
| Insert     | `Enter`      | Insert newline                         |
| Command    | `:w` + Enter | Save file                               |
| Command    | `:q` + Enter | Quit editor                             |
| Command    | `:wq` + Enter| Save and quit                           |
| Command    | `Esc`        | Cancel command input                    |

## Project Structure

```
src/
├── main.ts                 # Entry point
├── editor.ts               # Core editor logic, mode management, event loop
├── terminal.ts             # Terminal raw mode, ANSI escape code control
├── buffer.ts               # Text buffer management using @codemirror/state
├── renderer.ts             # Screen rendering, status bar
├── types.ts                # Shared type definitions
├── highlight/              # Syntax highlighting with Tree-sitter
│   ├── highlighter.ts
│   ├── detector.ts
│   ├── theme.ts
│   └── languages/
├── plugin/                 # Plugin system
│   ├── manager.ts
│   ├── api.ts
│   ├── keymaps.ts
│   ├── commands.ts
│   ├── events.ts
│   ├── loaders/            # Plugin loaders for Lua and TypeScript
│   └── types.ts
└── plugins/                # Example plugins
    ├── hello.ts
    └── hello.lua
```

## Development

Start the development server with hot reload:
```bash
bun run dev
```

Check Tree-sitter grammar availability:
```bash
bun run scripts/check-treesitter.ts
```

## License

MIT License. See [LICENSE](LICENSE) for details.
