# Enlisted Stats Desktop

A native desktop GUI for [enlisted-stats-viewer](https://github.com/GuitaristRin/enlisted-stats-viewer), letting you generate Enlisted player stat cards without using the command line.

## Features

- Input a player ID and choose card style (landscape / portrait) and language
- Saves the generated PNG via a native file dialog with a timestamped default filename
- Self-contained binary — no runtime dependencies, no system pollution

## Requirements

- A display server (X11 or Wayland on Linux; built-in on Windows/macOS)
- Internet access to reach [enlistedrollcall.com](https://enlistedrollcall.com)

## Build

```bash
git clone --recurse-submodules https://github.com/GuitaristRin/enlisted-stats-desktop
cd enlisted-stats-desktop
cargo build --release
./target/release/enlisted-stats-desktop
```

If you already cloned without `--recurse-submodules`:

```bash
git submodule update --init
```

## License

MIT
