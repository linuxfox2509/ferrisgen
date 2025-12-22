FerrisGen Installer

Interactive CLI installer. Usage:

1. Build the project releases you want to install (e.g. `cargo build --release -p ferrisgen-cli -p ferrisgen-gui`)
2. Run the installer from the workspace root: `cargo run -p ferrisgen-installer --release`

The installer will prompt for an install path and whether to install All (A), GUI only (G) or CLI only (C). A is the default.

It will copy the built release executables into the chosen folder. On Windows it can optionally add the install directory to the *user* PATH and create Start Menu shortcuts. These options are presented during installation.
