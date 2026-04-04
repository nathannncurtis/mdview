# mdview

A fast, lightweight markdown viewer for Windows. No editing — just opens `.md` files and renders them with dark-mode formatting.

## Features

- Dark mode (GitHub-dark themed)
- Syntax highlighting in code blocks
- File watching — auto-reloads when the file changes on disk
- Drag & drop `.md` files onto the window
- Scroll position memory across sessions
- Register as default `.md` viewer via `--register` flag
- ~900 KB binary, instant startup

## Install

Download `mdview-setup.exe` or the standalone `mdview.exe` from [Releases](https://github.com/nathannncurtis/mdview/releases).

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

## Build from source

```
cargo build --release
```

Requires Rust and Visual Studio Build Tools.
