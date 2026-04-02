# S8n (Katie) User Guide

## Getting Started

S8n is a unified system manager built for Lilith Linux. When you launch it, you'll see a main menu with three options:

1. **Package Manager** — Search, install, and manage software
2. **File Manager** — Browse and manage files
3. **Color Theme** — Change the application's appearance

Use **arrow keys** or **j/k** to navigate, **Enter** to select, and **q/Esc** to go back.

---

## Package Manager

### Searching for Packages

When you enter the package manager, you'll see a search prompt. Type a package name and press **Enter** to search across all available package managers on your system.

Results appear in a table with columns: `#`, `Name`, `Version`, `Source`, and `Description`.

### Installing Packages

1. Navigate to the package you want with **↑/↓** or **j/k**
2. Press **i** or **Enter** to start installation
3. Confirm with **Enter** on "Yes" (use **←/→** to switch between Yes/Cancel)
4. Watch the progress bar animate as the package installs

### Removing Packages

1. Navigate to the package you want to remove
2. Press **d** or **r**
3. Confirm the removal

### Viewing Installed Packages

Press **v** at any time to see all installed packages system-wide. This view shows:

- All packages from every available package manager
- Sorted alphabetically
- Filterable with **/** (type to search)
- Fuzzy searchable with **Ctrl+F** (requires `sk`/skim installed)
- Removable with **d** or **r**

Press **q** or **Esc** to return to the search/browse view.

### Filtering Results

Press **/** to enter search mode and type a filter. Results update as you search.

### Fuzzy Search

Press **Ctrl+F** to use skim for fuzzy searching. This requires the `sk` binary to be installed and in your PATH.

---

## File Manager

The file manager uses Miller-column navigation — selecting a directory shows its contents in the next column.

| Key | Action |
|-----|--------|
| **↑/↓** or **j/k** | Navigate files |
| **Enter** or **l** | Enter directory / select file |
| **h** or **Backspace** | Go up one level |
| **e** | Edit file in your `$EDITOR` |
| **r** | Rename file |
| **d** | Delete file |
| **m** | Move file |
| **q** / **Esc** | Go back / quit |

---

## Theme Picker

Change the application's appearance:

1. Select "Color Theme" from the main menu, or press **t** in the package manager
2. Use **↑/↓** or **j/k** to preview themes live
3. Press **Enter** to save your selection
4. Press **q** or **Esc** to return without saving

### Available Themes

| Theme | Palette |
|-------|---------|
| **Fire** (default) | Warm oranges and reds |
| **Ocean** | Cool blues and cyans |
| **Sunset** | Rich warm gradients |
| **Forest** | Natural greens |
| **Purple Dream** | Vibrant purples and pinks |

---

## Configuration

S8n stores configuration at `~/.config/s8n/theme.toml`:

```toml
theme = "Fire"
```

You can edit this file directly or use the built-in theme picker.

---

## Keyboard Quick Reference

### Global
- **q / Esc** — Go back or quit
- **↑ / k** — Move up
- **↓ / j** — Move down
- **Enter** — Select / confirm

### Package Manager
- **i / Enter** — Install selected package
- **d / r** — Remove selected package
- **v** — View installed packages
- **/** — Search / filter
- **Ctrl+F** — Fuzzy search (requires `sk`)
- **← / h** — Previous source or page
- **→ / l** — Next source or page
- **Tab** — Switch tab

### File Manager
- **Enter / l** — Enter directory
- **h / Backspace** — Go up
- **e** — Edit file
- **r** — Rename file
- **d** — Delete file
- **m** — Move file

---

## Troubleshooting

### "sk" command not found
Fuzzy search requires [skim](https://github.com/skim-rs/skim). Install it with:
```bash
cargo install skim
```
Or on Debian/Ubuntu:
```bash
sudo apt install skim
```

### No packages found
Ensure at least one package manager is installed on your system. S8n supports: apt, flatpak, snap, brew, npm, pip, pacstall, soar, and bun.

### Theme not persisting
Check that `~/.config/s8n/` exists and is writable. S8n creates this directory automatically if needed.
