use std::{
    cell::RefCell,
    env, fs,
    path::{Path, PathBuf},
    rc::Rc,
    time::Duration,
};

use anyhow::{Context, Result, anyhow, bail};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use slint::{Color, ComponentHandle, ModelRc, SharedString, StyledText, VecModel};

use crate::{
    canvas::{export::export_canvas_png, parser::parse_canvas_file, view_model::CanvasViewModel},
    cli::LaunchDocument,
};

slint::include_modules!();

struct AppState {
    current_path: Option<PathBuf>,
    recent_path: Option<PathBuf>,
    watcher: Option<RecommendedWatcher>,
}

impl AppState {
    fn new() -> Self {
        Self {
            current_path: None,
            recent_path: load_recent_file(),
            watcher: None,
        }
    }
}

/// Launch the RipCanvas Slint viewer.
///
/// # Errors
/// Returns an error when the native Slint window cannot be created or run.
pub fn run(document: LaunchDocument) -> Result<()> {
    let (initial_path, view_model) = match document {
        LaunchDocument::Empty => (None, CanvasViewModel::empty()),
        LaunchDocument::Loaded { path, canvas } => {
            (Some(path), CanvasViewModel::from_canvas(&canvas))
        }
    };

    let window = AppWindow::new()?;
    window.set_app_font_family(preferred_font_family());
    let state = Rc::new(RefCell::new(AppState::new()));

    apply_view_model(&window, view_model);
    set_status(&window, "idle", "Idle", "No canvas loaded");
    sync_state_to_ui(&window, &state.borrow());

    if let Some(path) = initial_path {
        let mut state = state.borrow_mut();
        state.current_path = Some(path.clone());
        state.recent_path = Some(path.clone());
        save_recent_file(&path);
        match start_watcher(&window, &path) {
            Ok(watcher) => {
                state.watcher = Some(watcher);
                set_status(
                    &window,
                    "success",
                    "Watching",
                    format!("Loaded {}", path.display()),
                );
            }
            Err(error) => {
                state.watcher = None;
                set_status(
                    &window,
                    "warning",
                    "Loaded",
                    format!("Watcher unavailable: {}", short_error(&error)),
                );
            }
        }
        sync_state_to_ui(&window, &state);
    }

    wire_callbacks(&window, Rc::clone(&state));

    window.show()?;
    window.window().set_maximized(true);
    slint::run_event_loop()?;
    Ok(())
}

fn wire_callbacks(window: &AppWindow, state: Rc<RefCell<AppState>>) {
    let weak = window.as_weak();
    let state_for_open = Rc::clone(&state);
    window.on_request_open_file(move || {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Obsidian Canvas", &["canvas"])
            .pick_file()
        else {
            return;
        };

        if let Some(window) = weak.upgrade() {
            let result = open_canvas_path(&window, &mut state_for_open.borrow_mut(), path);
            if let Err(error) = result {
                set_status(
                    &window,
                    "error",
                    "Error",
                    format!("Open failed: {}", short_error(&error)),
                );
            }
            sync_state_to_ui(&window, &state_for_open.borrow());
        }
    });

    let weak = window.as_weak();
    let state_for_recent = Rc::clone(&state);
    window.on_request_open_recent(move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let Some(path) = state_for_recent.borrow().recent_path.clone() else {
            set_status(&window, "warning", "Idle", "No recent canvas");
            return;
        };

        let result = open_canvas_path(&window, &mut state_for_recent.borrow_mut(), path);
        if let Err(error) = result {
            set_status(
                &window,
                "error",
                "Error",
                format!("Recent open failed: {}", short_error(&error)),
            );
        }
        sync_state_to_ui(&window, &state_for_recent.borrow());
    });

    let weak = window.as_weak();
    let state_for_refresh = Rc::clone(&state);
    window.on_request_refresh(move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let Some(path) = state_for_refresh.borrow().current_path.clone() else {
            set_status(&window, "warning", "Idle", "No canvas to refresh");
            return;
        };

        set_status(&window, "idle", "Reloading", "Refreshing canvas");
        match load_canvas_path(
            &window,
            &path,
            "success",
            "Reloaded",
            format!("Refreshed {}", path.display()),
        ) {
            Ok(()) => {}
            Err(error) => set_status(
                &window,
                "error",
                "Reload failed",
                format!("Keeping last view: {}", short_error(&error)),
            ),
        }
    });

    let weak = window.as_weak();
    let state_for_export = Rc::clone(&state);
    window.on_request_export_image(move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let Some(path) = state_for_export.borrow().current_path.clone() else {
            set_status(&window, "warning", "Idle", "No canvas to export");
            return;
        };
        let Some(output) = rfd::FileDialog::new()
            .add_filter("PNG image", &["png"])
            .set_file_name(default_export_file_name(&path))
            .save_file()
        else {
            return;
        };

        match export_canvas_path(&path, &output) {
            Ok(()) => set_status(
                &window,
                "success",
                "Exported",
                format!("Exported PNG to {}", output.display()),
            ),
            Err(error) => set_status(
                &window,
                "error",
                "Error",
                format!("Export failed: {}", short_error(&error)),
            ),
        }
    });

    let weak = window.as_weak();
    window.on_request_copy_text(move |text| match copy_to_clipboard(text.as_str()) {
        Ok(()) => {
            if let Some(window) = weak.upgrade() {
                set_status(&window, "success", "Copied", "Copied to clipboard");
            }
        }
        Err(error) => {
            if let Some(window) = weak.upgrade() {
                set_status(
                    &window,
                    "error",
                    "Error",
                    format!("Copy failed: {}", short_error(&error)),
                );
            }
        }
    });
}

fn open_canvas_path(window: &AppWindow, state: &mut AppState, path: PathBuf) -> Result<()> {
    let path = validate_canvas_path(path)?;
    load_canvas_path(
        window,
        &path,
        "success",
        "Loaded",
        format!("Loaded {}", path.display()),
    )?;

    state.current_path = Some(path.clone());
    state.recent_path = Some(path.clone());
    save_recent_file(&path);
    match start_watcher(window, &path) {
        Ok(watcher) => {
            state.watcher = Some(watcher);
            set_status(
                window,
                "success",
                "Watching",
                format!("Loaded {}", path.display()),
            );
        }
        Err(error) => {
            state.watcher = None;
            set_status(
                window,
                "warning",
                "Loaded",
                format!("Watcher unavailable: {}", short_error(&error)),
            );
        }
    }
    Ok(())
}

fn load_canvas_path(
    window: &AppWindow,
    path: &Path,
    status_kind: &str,
    status_label: &str,
    operation: impl Into<String>,
) -> Result<()> {
    let canvas = parse_canvas_file(path)
        .with_context(|| format!("Failed to load canvas file: {}", path.display()))?;
    apply_view_model(window, CanvasViewModel::from_canvas(&canvas));
    set_status(window, status_kind, status_label, operation);
    Ok(())
}

fn export_canvas_path(path: &Path, output: &Path) -> Result<()> {
    let canvas = parse_canvas_file(path)
        .with_context(|| format!("Failed to load canvas file: {}", path.display()))?;
    export_canvas_png(&canvas, output)
}

fn apply_view_model(window: &AppWindow, view_model: CanvasViewModel) {
    window.set_canvas_width(view_model.width);
    window.set_canvas_height(view_model.height);
    window.set_node_count(view_model.nodes.len() as i32);
    window.set_edge_count(view_model.edges.len() as i32);
    window.set_selected_index(-1);
    window.set_edges(ModelRc::new(Rc::new(VecModel::from(
        view_model
            .edges
            .into_iter()
            .map(|edge| UiCanvasEdge {
                from_id: SharedString::from(edge.from_id),
                to_id: SharedString::from(edge.to_id),
                from_node_x: edge.from_node_x,
                from_node_y: edge.from_node_y,
                from_node_width: edge.from_node_width,
                from_node_height: edge.from_node_height,
                to_node_x: edge.to_node_x,
                to_node_y: edge.to_node_y,
                to_node_width: edge.to_node_width,
                to_node_height: edge.to_node_height,
                from_x: edge.from_x,
                from_y: edge.from_y,
                control_1_x: edge.control_1_x,
                control_1_y: edge.control_1_y,
                control_2_x: edge.control_2_x,
                control_2_y: edge.control_2_y,
                to_x: edge.to_x,
                to_y: edge.to_y,
                from_color: color_from_hex(&edge.from_color),
                to_color: color_from_hex(&edge.to_color),
            })
            .collect::<Vec<UiCanvasEdge>>(),
    ))));
    window.set_nodes(ModelRc::new(Rc::new(VecModel::from(
        view_model
            .nodes
            .into_iter()
            .map(|node| UiCanvasNode {
                id: SharedString::from(node.id),
                is_group: node.kind == "group",
                title: SharedString::from(node.title),
                x: node.x,
                y: node.y,
                width: node.width,
                height: node.height,
                label: SharedString::from(node.label),
                markdown: styled_text_from_markdown(&node.markdown),
                kind: SharedString::from(node.kind),
                source: SharedString::from(node.source),
                geometry: SharedString::from(node.geometry),
                geometry_x: SharedString::from(node.geometry_x),
                geometry_y: SharedString::from(node.geometry_y),
                geometry_w: SharedString::from(node.geometry_w),
                geometry_h: SharedString::from(node.geometry_h),
                color: color_from_hex(&node.color),
                color_raw: SharedString::from(node.color_raw),
                text_color: color_from_hex(&node.text_color),
            })
            .collect::<Vec<UiCanvasNode>>(),
    ))));
}

fn color_from_hex(value: &str) -> Color {
    let hex = value.strip_prefix('#').unwrap_or(value);
    if hex.len() != 6 {
        return Color::from_rgb_u8(255, 255, 255);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    Color::from_rgb_u8(r, g, b)
}

fn styled_text_from_markdown(markdown: &str) -> StyledText {
    StyledText::from_markdown(markdown).unwrap_or_else(|_| StyledText::from_plain_text(markdown))
}

fn start_watcher(window: &AppWindow, path: &Path) -> Result<RecommendedWatcher> {
    let watched_path = path.to_path_buf();
    let weak = window.as_weak();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
        let Ok(event) = event else {
            return;
        };
        if !is_canvas_reload_event(&event, &watched_path) {
            return;
        }

        let weak = weak.clone();
        let path = watched_path.clone();
        let status_path = path.clone();
        let _ = weak.upgrade_in_event_loop(move |window| {
            set_status(
                &window,
                "idle",
                "Reloading",
                format!("Reloading {}", status_path.display()),
            );
        });
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(250));
            let result = parse_canvas_file(&path)
                .map(|canvas| CanvasViewModel::from_canvas(&canvas))
                .map_err(|error| format!("{error:#}"));
            let _ = weak.upgrade_in_event_loop(move |window| match result {
                Ok(view_model) => {
                    apply_view_model(&window, view_model);
                    set_status(
                        &window,
                        "success",
                        "Reloaded",
                        format!("Reloaded {}", path.display()),
                    );
                }
                Err(error) => set_status(
                    &window,
                    "error",
                    "Reload failed",
                    format!("Keeping last view: {}", short_message(&error)),
                ),
            });
        });
    })?;

    match watcher.watch(path, RecursiveMode::NonRecursive) {
        Ok(()) => Ok(watcher),
        Err(file_error) => {
            let Some(parent) = path.parent() else {
                return Err(file_error.into());
            };
            watcher
                .watch(parent, RecursiveMode::NonRecursive)
                .with_context(|| format!("Failed to watch {}", path.display()))?;
            Ok(watcher)
        }
    }
}

fn is_canvas_reload_event(event: &Event, watched_path: &Path) -> bool {
    let is_relevant_kind = matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    );
    is_relevant_kind
        && event
            .paths
            .iter()
            .any(|path| paths_match(path, watched_path))
}

fn paths_match(path: &Path, watched_path: &Path) -> bool {
    path == watched_path
        || (path.file_name().is_some() && path.file_name() == watched_path.file_name())
}

fn validate_canvas_path(path: PathBuf) -> Result<PathBuf> {
    if !path.exists() {
        bail!("Canvas file not found: {}", path.display());
    }
    if !path.is_file() {
        bail!("Canvas path is not a file: {}", path.display());
    }
    match path.extension().and_then(std::ffi::OsStr::to_str) {
        Some("canvas") => {}
        Some(_) | None => bail!("Expected a .canvas file: {}", path.display()),
    }

    Ok(path.canonicalize().unwrap_or(path))
}

fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().map_err(|error| anyhow!(error.to_string()))?;
    clipboard
        .set_text(text.to_owned())
        .map_err(|error| anyhow!(error.to_string()))
}

fn sync_state_to_ui(window: &AppWindow, state: &AppState) {
    window.set_has_open_file(state.current_path.is_some());
    window.set_has_recent_file(state.recent_path.is_some());
    window.set_current_file_name(SharedString::from(
        state
            .current_path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("No canvas"),
    ));
    window.set_current_file_path(SharedString::from(
        state
            .current_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "No canvas".to_owned()),
    ));
    window.set_recent_file_label(SharedString::from(
        state
            .recent_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "No recent file".to_owned()),
    ));
}

fn set_status(window: &AppWindow, kind: &str, label: &str, operation: impl Into<String>) {
    let operation = operation.into();
    window.set_status_kind(SharedString::from(kind));
    window.set_status_label(SharedString::from(label));
    window.set_status_text(SharedString::from(operation.clone()));
    window.set_operation_text(SharedString::from(operation));
    if kind == "success" {
        schedule_success_status_settle(window, label);
    }
}

fn schedule_success_status_settle(window: &AppWindow, expected_label: &str) {
    let weak = window.as_weak();
    let expected_label = expected_label.to_owned();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(1800));
        let _ = weak.upgrade_in_event_loop(move |window| {
            if window.get_status_kind().as_str() == "success"
                && window.get_status_label().as_str() == expected_label
            {
                if window.get_has_open_file() {
                    window.set_status_kind(SharedString::from("idle"));
                    window.set_status_label(SharedString::from("Watching"));
                } else {
                    window.set_status_kind(SharedString::from("idle"));
                    window.set_status_label(SharedString::from("Idle"));
                }
            }
        });
    });
}

fn short_error(error: &anyhow::Error) -> String {
    short_message(&format!("{error:#}"))
}

fn short_message(message: &str) -> String {
    let mut line = message.lines().next().unwrap_or(message).trim().to_owned();
    const MAX_LEN: usize = 160;
    if line.len() > MAX_LEN {
        line.truncate(MAX_LEN);
        line.push_str("...");
    }
    line
}

fn preferred_font_family() -> SharedString {
    const FAMILIES: &[&str] = &[
        "Microsoft YaHei UI",
        "Microsoft YaHei",
        "Segoe UI",
        "Noto Sans CJK SC",
        "Noto Sans SC",
        "Arial",
    ];

    let mut database = fontdb::Database::new();
    database.load_system_fonts();

    FAMILIES
        .iter()
        .find(|family| {
            database.faces().any(|face| {
                face.families
                    .iter()
                    .any(|(candidate, _)| candidate.eq_ignore_ascii_case(family))
            })
        })
        .copied()
        .unwrap_or("sans-serif")
        .into()
}

fn recent_store_path() -> PathBuf {
    if let Some(appdata) = env::var_os("APPDATA") {
        return PathBuf::from(appdata).join("RipCanvas").join("recent.txt");
    }
    if let Some(home) = env::var_os("USERPROFILE").or_else(|| env::var_os("HOME")) {
        return PathBuf::from(home)
            .join(".config")
            .join("ripcanvas")
            .join("recent.txt");
    }
    PathBuf::from("ripcanvas-recent.txt")
}

fn load_recent_file() -> Option<PathBuf> {
    let raw = fs::read_to_string(recent_store_path()).ok()?;
    let path = PathBuf::from(raw.trim());
    path.exists().then_some(path)
}

fn save_recent_file(path: &Path) {
    let store_path = recent_store_path();
    if let Some(parent) = store_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(store_path, path.display().to_string());
}

fn default_export_file_name(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("canvas");
    format!("{stem}.png")
}
