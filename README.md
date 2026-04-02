# S8n System Manager


[![CI](https://github.com/BlancoBAM/S8n-System/actions/workflows/ci.yml/badge.svg)](https://github.com/BlancoBAM/S8n-System/actions/workflows/ci.yml)
[![Release](https://github.com/BlancoBAM/S8n-System/actions/workflows/release.yml/badge.svg)](https://github.com/BlancoBAM/S8n-System/actions/workflows/release.yml)
[![Crates.io](https://img.shields.io/crates/v/s8n.svg)](https://crates.io/crates/s8n)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)

S8n (pronounced "system") is a fast, visually stunning System Manager, Unified Package Manager, and File Manager built in Rust. It provides native terminal gradients, Miller-column file browsing, and cross-package-manager fuzzy search — all from a single, beautiful terminal interface.

---

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage Guide](#usage-guide)
  - [Main Menu](#main-menu)
  - [Package Manager](#package-manager)
  - [Installed Packages](#installed-packages)
  - [File Manager](#file-manager)
  - [Theme Picker](#theme-picker)
- [Keyboard Reference](#keyboard-reference)
- [Supported Package Managers](#supported-package-managers)
- [Configuration](#configuration)
- [Building from Source](#building-from-source)
- [Contributing](#contributing)
- [Inspirations & Credits](#inspirations--credits)
- [License](#license)

---

## Features

- **Unified Package Manager** — Search, install, and remove packages across 10+ package managers from a single interface (apt, flatpak, snap, brew, npm, pip, and more)
- **Installed Packages View** — See all system-wide installed packages at a glance, sorted alphabetically, with filtering and fuzzy search
- **Animated Progress Displays** — Theme-aware gradient progress bars and braille spinners during install/remove operations
- **Miller-Column File Manager** — Safe directory traversal with drill-down exploration, file editing (`$EDITOR`), moves, renames, and deletions
- **Dynamic Theming Engine** — Switch the aesthetic of the entire application live with 5 built-in themes: Fire, Ocean, Sunset, Forest, and Purple Dream
- **Fuzzy Search** — Integration with [skim](https://github.com/skim-rs/skim) (`sk`) for fast fuzzy filtering of packages
- **Keyboard-Native** — Vim-style navigation (`j/k/h/l`) alongside arrow keys, designed for efficiency

---

## Quick Start

```bash
# Install (if binary is available)
s8n

# Or run directly
~/.local/bin/s8n
```

Navigate the main menu with arrow keys or `j/k`. Press `Enter` to select a mode. Press `q` or `Esc` to go back or quit.

---

## Installation

### Binary Release (Recommended)

Download the pre-built binary from [GitHub Releases](https://github.com/BlancoBAM/S8n-System/releases):

```bash
# Download the latest release
curl -L -o s8n https://github.com/BlancoBAM/S8n-System/releases/latest/download/s8n-linux-amd64
chmod +x s8n
sudo mv s8n /usr/local/bin/
```

Or install via `.deb` package (available on the releases page):

```bash
sudo dpkg -i s8n_*.deb
```

### From crates.io

```bash
cargo install s8n
```

### From Source

```bash
git clone https://github.com/BlancoBAM/S8n-System.git
cd S8n-System
cargo build --release
cp target/release/s8n ~/.local/bin/s8n
```

### Prerequisites

- **Rust toolchain** (for building from source): `rustc` and `cargo`
- **skim** (optional, for fuzzy search): Install `sk` and ensure it's in your `PATH`

---

## Usage Guide

### Main Menu

Launch with `s8n` to see the main menu with three options:

| Option | Description |
|--------|-------------|
| **Package Manager** | Search, install, and manage packages across multiple sources |
| **File Manager** | Browse and manage files with Miller-column navigation |
| **Color Theme** | Preview and select a visual theme for the application |

### Package Manager

The package manager is the core feature. Upon entering, you'll see a search prompt.

**Basic workflow:**

1. **Search** — Type a package name and press `Enter` to search across all available package managers
2. **Browse** — Use `↑/↓` or `j/k` to navigate results. Use `←/→` or `h/l` to switch between sources or pages
3. **Install** — Select a package and press `i` or `Enter` to install. Confirm with the dialog
4. **Remove** — Select a package and press `d` or `r` to remove it
5. **Filter** — Press `/` to re-enter search mode and refine results

**Source tabs:** When a package is available from multiple sources, tabs appear at the top showing each source. Use `←/→` to switch between them.

### Installed Packages

Press `v` at any time in the package manager to view **all installed packages** system-wide:

- Packages are collected from all available package managers
- Sorted alphabetically for easy browsing
- Press `/` to filter the list by name, source, or version
- Press `Ctrl+F` for fuzzy search with skim
- Press `d` or `r` on any package to remove it
- Press `q` or `Esc` to return to search/browse mode

Installed packages in search results are marked with a **✓** indicator and displayed in the theme's accent color.

### File Manager

The Miller-column file manager lets you browse directories safely:

- **Navigate** — Use arrow keys or `j/k` to move through files
- **Drill down** — Press `Enter` or `l` to enter a directory
- **Go back** — Press `h` or `Backspace` to go up a level
- **Edit** — Press `e` to open a file in `$EDITOR`
- **Rename** — Press `r` to rename a file
- **Delete** — Press `d` to delete a file (with confirmation)
- **Move** — Press `m` to move a file to another location

### Theme Picker

Access the theme picker from the main menu or by pressing `t` in the package manager:

- Use `↑/↓` or `j/k` to preview themes live
- Press `Enter` to save the selected theme
- Press `q` or `Esc` to return without saving

**Available themes:**

| Theme | Description |
|-------|-------------|
| **Fire** | Warm oranges and reds — the default Lilith Linux aesthetic |
| **Ocean** | Cool blues and cyans for a calm, deep-sea feel |
| **Sunset** | Rich warm gradients reminiscent of a golden hour |
| **Forest** | Natural greens and earth tones |
| **Purple Dream** | Vibrant purples and pinks — the original lipgloss vibe |

---

## Keyboard Reference

### Global

| Key | Action |
|-----|--------|
| `q` / `Esc` | Go back or quit |
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Enter` | Select / confirm |

### Package Manager — Browse Mode

| Key | Action |
|-----|--------|
| `i` / `Enter` | Install selected package |
| `d` / `r` | Remove selected package |
| `/` | Search / filter |
| `v` | View all installed packages |
| `Ctrl+F` | Fuzzy search with skim |
| `←` / `h` | Previous source or page |
| `→` / `l` | Next source or page |
| `Tab` | Switch tab / filter mode |

### Installed Packages View

| Key | Action |
|-----|--------|
| `d` / `r` | Remove selected package |
| `/` | Filter installed packages |
| `Ctrl+F` | Fuzzy search with skim |
| `q` / `Esc` | Return to browse mode |

### File Manager

| Key | Action |
|-----|--------|
| `Enter` / `l` | Enter directory |
| `h` / `Backspace` | Go up |
| `e` | Edit file in `$EDITOR` |
| `r` | Rename file |
| `d` | Delete file |
| `m` | Move file |

---

## Supported Package Managers

S8n automatically detects which package managers are available on your system:

| Manager | Binary | Search | Install | Remove | List Installed |
|---------|--------|--------|---------|--------|----------------|
| **apt** | `apt` | ✓ | ✓ | ✓ | ✓ |
| **flatpak** | `flatpak` | ✓ | ✓ | ✓ | ✓ |
| **snap** | `snap` | ✓ | ✓ | ✓ | ✓ |
| **brew** | `brew` | ✓ | ✓ | ✓ | ✓ |
| **npm** | `npm` | ✓ | ✓ | ✓ | ✓ |
| **pip** | `pip` | ✓ | ✓ | ✓ | ✓ |
| **pacstall** | `pacstall` | ✓ | ✓ | ✓ | ✓ |
| **soar** | `soar` | ✓ | ✓ | ✓ | ✓ |
| **bun** | `bun` | — | ✓ | ✓ | ✓ |
| **topgrade** | `topgrade` | — | — | — | — |

---

## Configuration

S8n stores its configuration at `~/.config/s8n/`:

```
~/.config/s8n/
└── theme.toml    # Current theme selection
```

The theme file is a simple TOML file:

```toml
theme = "Fire"
```

You can edit this file manually, or use the built-in theme picker.

---

## Building from Source

### Requirements

- Rust 1.70+ (edition 2021)
- `cargo`

### Build

```bash
# Debug build (faster compilation, larger binary)
cargo build

# Release build (optimized, smaller binary)
cargo build --release
```

### Run tests

```bash
cargo test
```

### Lint

```bash
cargo clippy
cargo fmt -- --check
```

---

## Contributing

Contributions are welcome! Please feel free to:

- Report bugs via [GitHub Issues](https://github.com/BlancoBAM/S8n-System/issues)
- Submit pull requests
- Request features or improvements

All contributions should follow the project's code style and pass `cargo clippy` and `cargo fmt`.

---

## Inspirations & Credits

The user interface and aesthetics for S8n were heavily inspired by the incredible ecosystem of [Charmbracelet Labs](https://charm.sh/).

Special thanks to the following Rust libraries and their authors:

- [ratatui](https://github.com/ratatui-org/ratatui) — Sleek terminal UIs in Rust
- [bubbletea-rs](https://github.com/whit3rabbit/bubbletea-rs) by @whit3rabbit — Reference for smooth TUI package manager experiences and gradient progress bars
- [lipgloss-rs](https://github.com/whit3rabbit/lipgloss-rs) by @whit3rabbit — Gradient generation and heat-mapping, shaping S8n's Fire aesthetic

---

## License

This software is released under the [GNU General Public License v3.0](LICENSE).

**Built specifically for Lilith Linux and Katie (aka S8n)**



