# RipCanvas

RipCanvas is a fast, read-only desktop viewer for Obsidian Canvas files.

The command-line entry point is `rocv`. It is built for workflows where agents,
scripts, or other tools generate `.canvas` files and you want a lightweight
viewer open for quick inspection.

## Features

- Open `.canvas` files from the command line or the app toolbar.
- Render Obsidian Canvas nodes, groups, colors, and curved edges.
- Zoom, pan, fit to view, and reset the viewport.
- Inspect selected node metadata, including id, type, source, color, and geometry.
- Copy node identifiers and source labels for prompts or scripts.
- Reopen the most recent file.
- Watch the opened file and reload after external changes.
- Preserve the last good view when refresh parsing fails.

RipCanvas is a viewer, not an editor. It does not save files or mutate canvas
node positions.

## Usage

Launch with no file:

```powershell
rocv
```

Open a canvas file:

```powershell
rocv path\to\architecture.canvas
```

Inside the app, use the toolbar to open a file, refresh, reopen the recent file,
fit the canvas, and inspect nodes.

## Build From Source

Requirements:

- Rust stable with Cargo
- Windows, macOS, or Linux supported by Slint

Build and run:

```powershell
cargo run --bin rocv
cargo run --bin rocv -- tests\fixtures\basic.canvas
```

Run checks:

```powershell
cargo fmt --all -- --check
cargo check --all-targets
cargo test
```

## Windows Packaging

Create a portable Windows zip for the current machine's common target:

```powershell
.\scripts\package-windows.ps1 -ZipOnly
```

Create Windows 11 packages for supported architectures:

```powershell
.\scripts\package-windows.ps1 -ZipOnly -Arch x64
.\scripts\package-windows.ps1 -ZipOnly -Arch x86
.\scripts\package-windows.ps1 -ZipOnly -Arch arm
```

Create the configured installer package with `cargo-packager`:

```powershell
cargo install cargo-packager --locked
.\scripts\package-windows.ps1
```

Generated packages are written under `dist/`.

## CI/CD

GitHub Actions runs on `master`, pull requests to `master`, tags matching `v*`,
and manual dispatches.

The workflow:

- validates formatting, compilation, and tests;
- creates source archives as `.zip` and `.tar.gz`;
- builds Windows 11 portable packages for `x64`, `x86`, and `arm`;
- publishes a GitHub Release automatically for tags like `v0.1.0`.

Release artifacts are named:

```text
ripcanvas-source-<version>.zip
ripcanvas-source-<version>.tar.gz
ripcanvas-windows11-x64-<version>.zip
ripcanvas-windows11-x86-<version>.zip
ripcanvas-windows11-arm-<version>.zip
```

## Project Layout

```text
src/
  main.rs                 Binary entry point for rocv.
  cli.rs                  CLI argument parsing and path validation.
  app.rs                  Slint app startup, UI binding, refresh, recent file, and clipboard logic.
  canvas/                 Obsidian Canvas model, parser, and view-model mapping.
ui/
  app-window.slint        Main desktop UI.
tests/
  fixtures/               Sample .canvas files.
scripts/
  package-windows.ps1     Windows packaging helper.
assets/
  icon and toolbar assets.
```

## License

RipCanvas is available under the MIT License. See [LICENSE](LICENSE).
