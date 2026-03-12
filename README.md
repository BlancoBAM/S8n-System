# S8n-Rx-PackMan 

![s8n](https://img.shields.io/badge/Language-Rust-orange?style=for-the-badge&logo=rust) ![License](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)

`s8n` is a universal package manager wrapper written in Rust, designed for Lilith Linux. It unifies operations across multiple diverse package managers while providing an elegant, inline Terminal UI inspired by the Charmbracelet `bubbletea` framework.

## ✨ Features

- **Unified Interface**: Use a single set of commands to interact with multiple package managers.
- **Beautiful TUI**: Features a sleek inline spinner and progress tracking interface powered by `ratatui`.
- **Intelligent Routing**: Automatically routes URL installations to `soar` and delegates system upgrades to `topgrade`.
- **Supported Backends**:
  - `apt` / `pacstall`
  - `flatpak` / `snap` / `appimage` (via soar)
  - `brew`
  - `npm` / `bun`
  - `pip` / `pypi`
  - `soar`
  - `topgrade`

## 🚀 Installation

Ensure you have Rust and Cargo installed, then clone and build the project:

```bash
git clone https://github.com/BlancoBAM/S8n-Rx-PackMan.git
cd S8n-Rx-PackMan
cargo build --release
sudo cp target/release/s8n /usr/local/bin/
```

## 💻 Usage

`s8n` relies on 4 simple intuitive commands:

### Search for Packages
Search across all integrated package managers:
```bash
s8n search <query>
```

### Install Packages
Install packages using the native system package manager, or install directly from a URL via `soar`:
```bash
s8n stall <package_name>
s8n stall https://github.com/example/release.appimage
```

### Remove Packages
Remove installed packages from your system:
```bash
s8n burn <package_name>
```

### Update System
Upgrade all system packages across all connected package managers. Utilizes `topgrade` if installed:
```bash
s8n upd8
```

## 🏗️ Architecture

The codebase is structured to be extensible:
- **`src/main.rs`**: Core CLI routing and entrypoint.
- **`src/ui.rs`**: The inline `ratatui` Terminal User Interface used for displaying progress states.
- **`src/pm/mod.rs`**: Trait definitions for generic package manager backends.
- **`src/pm/builtin.rs`**: Built-in shell execution strategies for all natively wrapped package managers.
- **`src/config.rs`**: Future-proof configuration loader for dynamic package manager additions.

## 🤝 Contributing
Contributions are welcome! Please feel free to submit a Pull Request.
