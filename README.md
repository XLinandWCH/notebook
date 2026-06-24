# Notebook

A simple cross-platform text editor built with Rust, inspired by Windows 11 Notepad.

## Features

- File operations: New, Open, Save, Save As
- Edit operations: Cut, Copy, Paste, Select All
- View: Word Wrap toggle
- Status bar: Line/column position, file path, modification indicator
- About dialog
- Keyboard shortcuts

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+N | New |
| Ctrl+O | Open |
| Ctrl+S | Save |
| Ctrl+Shift+S | Save As |

## Building

### Prerequisites

- Rust 1.70+ (install via [rustup.rs](https://rustup.rs))
- A C++ compiler (for the native GUI dependencies)

### Build

```bash
cargo build --release
```

The binary will be at `target/release/notebook.exe` (Windows) or `target/release/notebook` (Linux/macOS).

### Run

```bash
cargo run --release
```

## Technology Stack

- **eframe / egui**: Cross-platform GUI framework
- **rfd**: Native file dialogs
