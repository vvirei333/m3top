# M3Top

A desktop system monitor for Linux, built with Rust and [egui](https://github.com/emilk/egui), styled after **Material 3 Expressive**: rounded cards, dynamic accent colors, and springy/bouncy motion.

![M3Top screenshot](docs/screenshot.png)

## Features

- **CPU** — total load + per-core percentages, with a live history graph and a level badge (normal / moderate / high).
- **RAM / Swap** — usage bar, GiB used/total, swap usage (or "Swap: none").
- **Temperatures** — real sensor readings from `/sys/class/hwmon` (e.g. `k10temp Tctl`, `nvme Composite`), prioritized over the often-unhelpful generic `acpitz` zone from `/sys/class/thermal`.
- **Network** — aggregate download/upload rate across interfaces (loopback excluded).
- **Processes** — searchable, sortable (click a column header) table: PID, name, CPU%, RAM, command. Filterable to just your own processes.
- **Material 3 Expressive UI** — pill-shaped chips and search bar, tonal surfaces instead of hard borders, spring-animated ("bouncy") resizing that never lets cards overlap their neighbors (excess spring energy turns into a visible squash instead).
- Dark theme, tight "shoulder to shoulder" layout, no network access required (everything is read locally from `/proc` and `/sys`).

## Project layout

```
crates/
  core/     — m3top-core: metric collection from /proc and /sys. No UI dependencies,
              pure data structures (CpuStats, MemStats, ProcessInfo, ThermalZone, ...).
  desktop/  — m3top: the egui/eframe GUI application.
```

`core` never panics on a missing `/proc` or `/sys` path — unavailable sources show up as `None` / an empty list, and the UI renders "unavailable" instead of crashing.

## Building & running

Requires a recent Rust toolchain (`rustup` recommended).

```bash
cargo build --release
cargo run -p m3top --release
```

Run the test suite:

```bash
cargo test --workspace
```

There's also a plain-text CLI example for the core crate (no GUI needed), useful for quick diagnostics:

```bash
cargo run -p m3top-core --example cli
```

## Installing a prebuilt package

Packages for Arch, Debian/Ubuntu, Fedora/openSUSE and a portable AppImage can
all be built with one script:

```bash
# one-time tool setup (needs network access)
cargo install cargo-deb cargo-generate-rpm
# download linuxdeploy + appimagetool continuous builds into packaging/tools/
# (see https://github.com/linuxdeploy/linuxdeploy and
#  https://github.com/AppImage/appimagetool releases), or just have them on PATH

./packaging/build-all.sh
```

This produces, in `dist/`:

| File | Install with |
|---|---|
| `m3top-<ver>-1-x86_64.pkg.tar.zst` | `sudo pacman -U dist/m3top-*.pkg.tar.zst` |
| `m3top_<ver>-1_amd64.deb` | `sudo apt install ./dist/m3top_*.deb` |
| `m3top-<ver>-1.x86_64.rpm` | `sudo dnf install ./dist/m3top-*.rpm` (or `rpm -i`/`zypper install`) |
| `M3Top-<ver>-x86_64.AppImage` | `chmod +x dist/M3Top-*.AppImage && ./dist/M3Top-*.AppImage` |

All four install a `m3top` launcher, a `.desktop` entry and icons at the
standard `hicolor` theme sizes (16–256px + scalable SVG). Individual steps
(`makepkg` in `packaging/arch/`, `cargo deb`, `cargo generate-rpm`) also work
standalone if you only need one format.

Note: `m3top` only hard-links `libc`/`libgcc`/`libm` — winit loads its
Wayland/X11/OpenGL backends with `dlopen` at runtime, so none of these
packages bundle (or can auto-detect) those libraries. The `.deb`/`.rpm`
packages declare them as regular dependencies; the AppImage relies on the
host system providing them, same as the raw binary.

## Platform notes

Currently developed and tested on Arch Linux, primarily targeting Wayland
compositors (e.g. Hyprland) with Material You-style theming. The core crate
only depends on standard `/proc` and `/sys` paths that are common across
Linux distributions, so other distros should work out of the box. A port to
Android is planned next.

## License

See individual dependency licenses. The bundled `NotoSans-Bold.ttf` font is licensed under [Apache License 2.0](https://openfontlicense.org) (Noto Sans by Google).
