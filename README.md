# Notebook

A simple terminal-based text editor built with Rust, inspired by Windows 11 Notepad.

## Features

- File operations: New (Ctrl+N), Save (Ctrl+S), Quit (Ctrl+Q)
- Text editing: Insert, backspace, delete, newline, tab
- Navigation: Arrow keys, Home/End, Page Up/Down
- Status bar: Current line/column, file path, word wrap indicator
- Syntax highlighting for Rust code
- Line numbers with gutter

## Keyboard Shortcuts

| Shortcut     | Action       |
|--------------|--------------|
| Ctrl+N       | New file     |
| Ctrl+S       | Save         |
| Ctrl+Q       | Quit         |
| Ctrl+W       | Warn unsaved |
| Left/Right   | Move cursor  |
| Up/Down      | Move line    |
| Home/End     | Line start/end |
| PageUp/Down  | Page scroll  |
| Esc          | Quit         |

## Building

### Prerequisites

- Rust 1.70+
- A C++ compiler

### Build

```bash
cargo build --release
```

The binary will be at `target/release/notebook`.

### Run

```bash
cargo run --release
# or open a file:
cargo run --release /path/to/file.txt
```

## Technology Stack

- **ratatui**: Terminal UI framework
- **crossterm**: Terminal manipulation
- **dirs**: Home directory access
