# RipCanvas Design

RipCanvas is a fast, read-only desktop viewer for Obsidian Canvas files. The command-line entry point is `rocv`, and the app is designed for architecture work where coding CLI agents generate or modify `.canvas` files while the user keeps a lightweight viewer open for immediate visual feedback.

## Product Scope

RipCanvas is a viewer, not an editor. The application should make it cheap to open `.canvas` files from any folder, inspect the graph, zoom and pan quickly, and refresh when agent-written files change on disk.

The source of truth remains the `.canvas` file. RipCanvas should not save changes, mutate nodes, or compete with Obsidian as a full editing environment.

## Main Features

### File Opening

- Open a `.canvas` file from the command line:
  ```powershell
  rocv path\to\architecture.canvas
  ```
- Launch with no file and show an empty state.
- Open a file from inside the app through a toolbar action.
- Reject missing paths and non-`.canvas` files with clear errors.
- Keep a recent-file path for fast reopen in a later slice.

### Canvas Browsing

- Render Obsidian Canvas nodes in a read-only viewport.
- Support common node types:
  - `text`
  - `file`
  - `link` / URL-like nodes
  - unknown node types with a safe fallback label
- Render basic node labels without full Markdown preview in the first phase.
- Render edges as simple connections once node rendering and viewport behavior are stable.

### Navigation

- Mouse wheel zoom.
- Drag-to-pan canvas movement.
- Fit-to-view command for quickly framing the whole canvas.
- Reset view command for returning to the default zoom and pan.
- Keep negative-coordinate canvases visible by normalizing source coordinates with padding.

### Async Refresh

- Watch the opened `.canvas` file for external changes.
- Debounce filesystem events to avoid parsing half-written files.
- Reload in the background so the UI remains responsive.
- Apply successful parses on the UI thread.
- Preserve the last good canvas when a refresh fails.
- Show parse or read errors without clearing the current view.

### Inspector

- Select a node to inspect metadata.
- Show node id, type, source label, file path or URL, and geometry.
- Support copying node id or path for use in coding-agent prompts.

## Non-Goals

- No manual node editing.
- No saving `.canvas` files.
- No drag-to-save node positions.
- No edge creation or deletion.
- No full Markdown rendering in the first version.
- No Obsidian vault indexing.
- No plugin integration.
- No installer or auto-update flow until the viewer behavior is solid.

## Architecture

```text
rocv CLI
  -> validate optional path
  -> parse launch request
  -> start Slint app

Slint UI
  -> toolbar/status
  -> canvas viewport
  -> node layer
  -> edge layer
  -> inspector panel

Canvas domain
  -> serde model for Obsidian .canvas JSON
  -> parser from disk/string into typed structures
  -> view-model mapping for UI coordinates and labels

Watcher
  -> filesystem event subscription
  -> debounce
  -> background reload
  -> UI-thread update
```

## Module Layout

```text
src/
  main.rs                 Binary entry point for rocv.
  lib.rs                  Public module wiring for tests and the binary.
  cli.rs                  CLI arguments and launch validation.
  app.rs                  Slint app startup and UI model binding.
  canvas/
    mod.rs                Canvas module exports.
    model.rs              Typed Obsidian Canvas JSON structures.
    parser.rs             File/string parsing boundary.
    view_model.rs         UI-facing nodes, edges, labels, and coordinate normalization.
  watch/
    mod.rs                Future file-watching implementation.

ui/
  app-window.slint        Main RipCanvas window and visual layout.

tests/
  canvas_parser.rs        Parser and view-model behavior tests.
  cli.rs                  CLI behavior tests.
  fixtures/               Sample .canvas files.
```

## Technology Stack

### Language And Build

- Rust 2024 edition.
- Cargo package name: `ripcanvas`.
- Binary command name: `rocv`.
- `build.rs` compiles Slint UI files.

### UI

- `slint` for native desktop UI.
- `.slint` markup for the main window.
- Rust-side `VecModel` / `ModelRc` for dynamic node data.
- UI updates must happen on the Slint event loop when background work is added.

### Parsing

- `serde` for typed data structures.
- `serde_json` for Obsidian `.canvas` JSON parsing.
- Required geometry fields stay required so malformed canvas files fail clearly.
- Optional node payload fields use `Option<T>`.

### CLI And Errors

- `clap` derive for command-line parsing.
- `anyhow` for binary-level error context.
- CLI validates path existence, file type, and `.canvas` extension before app launch.

### Future File Watching

- `notify` should be added when live refresh is implemented.
- Refresh should use a debounce timer and background parse worker.
- Parsed data should cross into Slint through `upgrade_in_event_loop` or `invoke_from_event_loop`.

### Testing

- `cargo test` for unit and integration tests.
- `assert_cmd` and `predicates` for CLI tests.
- Committed fixtures under `tests/fixtures` for stable canvas examples.
- GUI smoke checks should launch `target/debug/rocv.exe` with and without a fixture.

## Implementation Phases

### Phase 1: Basic Viewer

- Launch Slint window.
- Parse `.canvas` files.
- Render basic nodes.
- Support CLI path input.
- Include tests and fixtures.

Current status: implemented.

### Phase 2: Core Navigation

- Add in-app open-file action.
- Add zoom controls.
- Add mouse wheel zoom.
- Add drag-to-pan.
- Add fit-to-view and reset-view actions.

### Phase 3: Better Canvas Fidelity

- Render edges.
- Improve node type styling.
- Add node selection.
- Add inspector panel.
- Add copy node id/path actions.

### Phase 4: Live Refresh

- Add filesystem watching.
- Debounce writes.
- Reload asynchronously.
- Preserve the last good canvas on parse errors.
- Show refresh status and error details.

### Phase 5: Daily-Use Polish

- Recent file support.
- Keyboard shortcuts.
- Better large-canvas performance.
- Packaging for local Windows use.

## Verification Commands

Run these from the project root:

```powershell
cargo fmt --all -- --check
cargo check --all-targets
cargo test
cargo run --bin rocv -- --help
cargo run --bin rocv
cargo run --bin rocv -- tests\fixtures\basic.canvas
```

## Design Principles

- Keep the app read-only.
- Make opening external `.canvas` files faster than using Obsidian.
- Prefer clear failure messages over silent fallback behavior.
- Keep parsing and view-model logic testable without launching the UI.
- Add live refresh only after viewport interactions are stable.
- Optimize for coding-agent workflows: inspect, copy identifiers, refresh quickly, and never corrupt the source file.
