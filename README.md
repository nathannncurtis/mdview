# mdview

> **Note:** This is the original Rust/WebView2 implementation. A rewrite using Zig + DirectWrite is available at [mdview-zig](https://github.com/nathannncurtis/mdview-zig) — same features, ~240 KB binary, no webview dependency.

A fast, lightweight markdown viewer for Windows. No editing — just opens `.md` files and renders them with dark-mode formatting.

## Features

- Dark mode (GitHub-dark themed)
- Borderless floating window
- Syntax highlighting in code blocks
- File watching — auto-reloads when the file changes on disk
- Drag & drop `.md` files onto the window
- Alt+drag to move the window
- Ctrl+Q to quit
- Scroll position memory across sessions
- Links open in default browser
- Register as default `.md` viewer via `--register` flag
- ~1.1 MB binary, instant startup

## Install

Download `mdview-setup.exe` or the standalone `mdview.exe` from [Releases](https://github.com/nathannncurtis/mdview/releases).

The installer supports per-user (AppData) or system-wide (Program Files) installation and optionally associates `.md` files with mdview.

### Set as default viewer

Either run the installer (handles file association automatically) or:

```
mdview.exe --register
```

## Usage

```
mdview README.md
```

Or just double-click any `.md` file after registering.

## Keybindings

| Key | Action |
|-----|--------|
| Ctrl+Q | Quit |
| Alt+Drag | Move window |

## Logging

Logs are written to `%LOCALAPPDATA%\mdview\mdview.log`.

## Build from source

```
cargo build --release
```

Requires Rust and Visual Studio Build Tools.

## License

[MIT](LICENSE)
