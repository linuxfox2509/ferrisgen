# ferrisgen
Password Generator coded in Rust

## Overview

This workspace contains three crates:

- `ferrisgen` — core library for generating passwords
- `ferrisgen-cli` — command-line interface
- `ferrisgen-gui` — simple desktop GUI based on `eframe`/`egui`

## Features

- Generate passwords 6–25 characters long
- Select which character classes to include: lowercase, uppercase, numbers, special
- Optionally exclude ambiguous characters (O/0, I/l, 1)
- Cross-platform clipboard copy (Windows, macOS, Linux)
- GUI shows an estimated password **strength indicator** and an **Auto-copy on generate** toggle

---

## Building

Ensure you have a recent Rust toolchain installed (Rust 1.70+ recommended).

Build CLI:

```bash
cargo build -p ferrisgen-cli --release
```

Build GUI:

```bash
cargo build -p ferrisgen-gui --release
```

Run CLI directly with cargo:

```bash
cargo run -p ferrisgen-cli -- --length 16 -l -u -n -s --no-ambiguous
```

If no include flags (`-l`, `-u`, `-n`, `-s`) are supplied, the CLI defaults to using lowercase + uppercase + numbers.

Run GUI in development:

```bash
cargo run -p ferrisgen-gui
```

## Examples

Generate a 16-character password with lowercase, uppercase and numbers, copy to clipboard:

```bash
cargo run -p ferrisgen-cli -- --length 16 -l -u -n --copy
```

Generate with no ambiguous characters:

```bash
cargo run -p ferrisgen-cli -- --length 12 -a
```

## Notes

- The GUI uses `eframe` and may require additional system dependencies on some Linux distributions (standard desktop libs).
- If building for macOS, ensure you have the proper toolchain and SDK.
- The GUI shows an info tooltip on the Strength progress bar (hover the ℹ icon): "Estimated entropy = length × log2(pool size). Pool size counts selected character classes; ambiguous characters are excluded if selected. This is an approximate indicator used for guidance only." 

Behavior notes:

- Per your request, the generator will not use the following punctuation characters at all: `.` `,` `(` `)` `[` `]` `{` `}`. These are removed from the special character set.
- When possible, generated passwords will not start with a special character; if only special characters are selected the generator will still produce a valid password (the first char may be special since no non-special characters are available).

## Testing

Run unit tests with:

```bash
cargo test -p ferrisgen
```

Run CLI integration tests with:

```bash
cargo test -p ferrisgen-cli
```

---

Contributions and improvements welcome.
