# Changelog

[Română](CHANGELOG.md) · **English** · [Русский](CHANGELOG.ru.md)

The notable changes in each version. Versions follow
[Semantic Versioning](https://semver.org/).

## [3.0.2] — 2026-10-23

### Added

- The **Format** button (Ctrl+Shift+F): re-indents the program and puts the
  command after `IF … THEN` and `ELSE` on lines of their own. Nothing is
  indented between `BEGIN` and `END`. The cursor stays next to the same word,
  and Ctrl+Z undoes the formatting in one go.
- Suggestions while typing: after `wh` you get `WHILE NOT_BORDER DO …` and
  `WHILE BORDER DO …`; the names of the procedures in the program are offered
  too, and `END` suggests the end of the open block first. Pick one with ↑/↓
  and Enter or Tab, or with the mouse; Esc closes the list.
- Enter keeps the block's indentation; `END REPEAT`, `END WHILE`,
  `END PROCEDURE` and `ELSE` line up with the start of their block by
  themselves.

### Changed

- Command buttons: the command gets the indentation of the block it lands in,
  the example inside it (`DOWN`, `UP`) stays selected so that the next command
  replaces it, and after a move the cursor goes to a new line.
- `IF` is inserted on several lines, together with `ELSE`; after the command
  under `THEN`, the one under `ELSE` gets selected.
- The examples are formatted.

### Fixed

- Copy and paste (Ctrl+C, Ctrl+X, Ctrl+V) work with other programs too.
- The cursor no longer disappears from the editor after clicking the command
  buttons or **Fix**.
- Clicking in the problem list keeps the misspelled word selected.

## [3.0.1] — 2026-10-22

### Added

- An `.msi` installer for 64- and 32-bit Windows 10/11: Start menu entry,
  `.ant` file association, upgrades over the old version.
- README in English and Russian.

### Changed

- The command buttons are on two rows: the moves first, then the other
  commands.

## [3.0.0] — 2026-10-20

The first version rewritten in Rust, based on the original “Furnica” program
(Ant v2.00, 1999). `.ant` files from the old version open and run unchanged.

- Interface in Romanian, Russian and English, with a light and a dark theme.
- A vector-drawn ant that turns smoothly and moves its legs as it walks.
- An editor with line numbers, syntax highlighting and buttons that insert
  commands.
- Animated execution with the current line highlighted and the ant's path
  drawn on the table; step by step, pause, adjustable speed or instant.
- The analyzer checks the program as you type and suggests fixes.
- Blocks can be nested freely.
- The table is edited directly on it, from 3 × 3 to 20 × 20 cells.
- Packages for Windows (including Windows 7), Linux and macOS.

[3.0.2]: https://github.com/nazdravan/furnica/compare/v3.0.1...v3.0.2
[3.0.1]: https://github.com/nazdravan/furnica/compare/v3.0.0...v3.0.1
[3.0.0]: https://github.com/nazdravan/furnica/releases/tag/v3.0.0
