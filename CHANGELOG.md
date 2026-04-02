# Changelog

All notable changes to S8n (Katie) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-04-01

### Added
- **Installed Packages View** — Press `v` to see all system-wide installed packages from every available package manager, sorted alphabetically
- **Installed package indicators** — Search results now show a `✓` badge on already-installed packages with theme-aware coloring
- **Duplicate install prevention** — Attempting to install an already-installed package shows an informative message instead of proceeding
- **skim fuzzy search for installed packages** — `Ctrl+F` in the installed view filters only installed packages
- **cargo-deb support** — `.deb` package building via `cargo deb`
- **Comprehensive README** — Full documentation with keyboard reference, usage guide, and configuration docs
- **CHANGELOG.md** — Proper changelog tracking

### Fixed
- **Package installation flow** — Source selection is now properly tracked through the confirm dialog, fixing broken installs
- **Package removal flow** — Source-aware manager lookup for remove operations (was hardcoded to first manager)
- **Progress bar animation** — Now uses native ratatui `Span`/`Style` coloring with theme-based gradient stops that animate smoothly
- **Confirm dialog stability** — Fixed potential underflow panic on small terminals with `saturating_sub()`
- **Done view stability** — Same underflow fix applied
- **Search spinner overlap** — Spinner now positions after typed text instead of overlapping long queries
- **Grid table overflow** — Column widths now validate and scale proportionally when exceeding available space
- **Tab rendering artifacts** — Tab bar now fully clears its row before rendering, eliminating random character artifacts

### Changed
- **Version bump** to 0.2.0 reflecting the significant feature additions
- **Cargo.toml metadata** — Added keywords, categories, readme, documentation, and homepage fields
- **Package manager trait** — Added `list_installed()` method for collecting installed packages from all sources
- **All progress view elements** — Now use theme colors instead of hardcoded RGB values

## [0.1.3] - 2026-03-20

### Changed
- CI workflow upgraded to actions v4 with Node.js 24 environment

## [0.1.2] - 2026-03-15

### Added
- Release workflow for GitHub Releases with binary artifacts

## [0.1.1] - 2026-03-10

### Changed
- License updated to GPL-3.0
- Release profiling optimizations enabled

## [0.1.0] - 2026-03-01

### Added
- Initial release
- Unified package manager with multi-source search
- Miller-column file manager
- Dynamic theme picker with 5 palettes (Fire, Ocean, Sunset, Forest, Purple Dream)
- Animated braille spinners and gradient progress bars
- skim (fzf) integration for fuzzy search
- Main menu with mode selection
