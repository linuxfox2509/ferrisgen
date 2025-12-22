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

## Build everything & Packaging (Windows)

This repository includes a helper PowerShell script to build the release binaries (CLI, GUI, and installer) and package them into a single ZIP in `dist\` for distribution or manual download.

Build all release artifacts (workspace-wide):

```powershell
# Build every workspace member in release mode
cargo build --workspace --release

# Or build only the items we package (faster)
cargo build --release -p ferrisgen-cli -p ferrisgen-gui -p ferrisgen-installer
```

Package into a ZIP (Windows only):

```powershell
# From the workspace root
> .\tools\package.ps1
# Or explicitly (useful in CI)
> powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\package.ps1
```

What the script does:
- Builds the release binaries (if missing).
- Copies `ferrisgen-cli.exe`, `ferrisgen-gui.exe`, and `ferrisgen-installer.exe` plus `README.md` and `LICENSE` into a temporary bundle.
- Generates a `SHA256SUMS.txt` file inside the bundle with SHA-256 checksums for every included file.
- Zips the bundle into `dist\FerrisGen-Windows-<timestamp>.zip` and writes a `<zipname>.sha256` file next to the ZIP containing the ZIP's SHA-256 checksum.

Verify checksums locally (PowerShell):

```powershell
# Verify a file's SHA-256
Get-FileHash -Algorithm SHA256 <path-to-file>

# Verify ZIP SHA by comparing to the .sha256 file (or use certutil)
certutil -hashfile <path-to-zip> SHA256
```

Using the bundle:
- Users can run `ferrisgen-installer.exe` from the unzipped folder to perform an interactive install (choose install directory and components, add PATH and Start Menu shortcuts). The installer is Windows-only.
- Or the user can simply run the binaries directly from the unzipped folder (no install required).

---


The script also generates checksums:

- A `SHA256SUMS.txt` file is included inside the bundle listing SHA-256 checksums for every file in the bundle.
- A `<zipname>.sha256` file is created in `dist\` containing the SHA-256 checksum of the ZIP file (useful for quick verification after download).

Verify with PowerShell:

```powershell
Get-FileHash -Algorithm SHA256 <file>
```

Or using `certutil`:

```powershell
certutil -hashfile <file> SHA256
```
