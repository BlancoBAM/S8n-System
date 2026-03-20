# S8n System Manager

S8n is a fast, visually stunning System Manager, Unified Package Manager, and File Manager built in Rust. It utilizes `ratatui` to provide native terminal gradients, miller-column file browsing, and cross-package-manager fuzzy search capabilities directly in your console.

## Features

- **Unified Package Manager**: Search, install, and track progress for system packages across PyPI, Cargo, Node/NPM, and more. 
- **Animated Progress Displays**: Native braille spinners and heat-gradient task loading bars as each package installs.
- **Miller-Column File Manager**: Traverse through your local directories safely with drill-down exploration, dynamic deletion, editing (`$EDITOR`), moves, and file previews.
- **Dynamic Theming Engine**: Switch the aesthetic of the *entire application* live using the built-in Color Picker. Try `Fire`, `Ocean`, `Sunset`, `Forest`, or `Purple Dream` instantly.

## Installation

### Binary Release (Recommended)
You can directly download the fully compiled static binaries for Linux via the [GitHub Releases](../../releases) tab. Simply drop the `s8n` binary into `~/.local/bin/s8n` or `/usr/local/bin` and you're good to go.

### From Source
Ensure you have `cargo` and `rustc` installed. Build the application and copy it manually to your system's binaries folder:

```bash
git clone https://github.com/BlancoBAM/S8n-System.git
cd S8n-System
cargo build --release
cp target/release/s8n ~/.local/bin/s8n
```

*Note: S8n occasionally utilizes `skim` (the Rust version of fzf) for pipelining its package searches. Please make sure `sk` is in your environment PATH for the optimal PackMan experience.*

## Usage

Start the main menu straight from anywhere via terminal:

```bash
s8n
```

Navigate with Arrow Keys, `[j/k/h/l]`, and `Enter`.
Hit `q` or `Esc` to jump back to the previous screen or close out.

## Inspirations & Credits

The user interface and aesthetics for S8n were heavily inspired by the incredible ecosystem of the [Charmbracelet Labs](https://charm.sh/). 
Special shout-out to the following incredible Rust libraries and references:
- [ratatui](https://github.com/ratatui-org/ratatui) - An exceptional library for cooking up sleek terminal UIs in Rust.
- [bubbletea-rs](https://github.com/whit3rabbit/bubbletea-rs) by @whit3rabbit - Reference examples for building smooth TUI package manager experiences and gradient progress bars in Rust.
- [lipgloss-rs](https://github.com/whit3rabbit/lipgloss-rs) by @whit3rabbit - The reference definitions for gradient generation and heat-mapping, helping shape `s8n`'s Fire aesthetic mappings.

## License

This software is released openly on GitHub. Feel free to track issues, clone, fork, and request improvements.
