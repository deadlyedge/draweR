# Commit-pinned windows-reactor Application POC

This is an isolated experiment. It does not modify or replace `poc/windows-reactor` (the crates.io
`0.100.0` POC) and is not a root workspace member.

## Exact source pin

```text
Repository:  https://github.com/microsoft/windows-rs.git
Commit:      d5b6a1f68f4fdd0c79495954d162901ea90da4f4
Commit date: 2026-10-10T13:55:23-05:00
Subject:     windows-reactor unify native event dispatch (#5061)
Package:     windows-reactor 0.100.0, Rust 1.95, edition 2024
```

`Cargo.toml` pins both `windows-reactor` and the direct `windows-sys` interop dependency to this
full Git revision. Keep `Cargo.lock` with the POC; neither dependency may float to a branch/tag or
silently fall back to the crates.io package.

## Scope

- `App::run_application::<TrayApp>` owns application state separately from component windows.
- The application declares stable `main`, `placement`, and `chrome-grid` window keys plus a tray
  resource. The `chrome-grid` test can be opened from the main-window button or tray menu.
- Tray activation/menu commands request the corresponding keyed window; Exit calls `context.exit()`.
- The main window can be closed independently while the application/tray remains alive.
- The placement window observes `WindowPlacement`; the application holds the latest value in memory
  and supplies it as the initial input on a later open.
- The chrome-grid window is a visual test surface, not product directory browsing.

## Visual test — borderless chrome and Explorer-like icon grid (2026-10-12)

- **Custom WinUI title bar:** Reactor's `TitleBar` maps to WinUI
  `SetExtendsContentIntoTitleBar(true)` / `SetTitleBar(element)`. It customizes title-bar content but
  does not itself remove the native HWND caption/frame.
- **Native frame toggle:** a button uses `windows-sys` at the same pinned revision to remove/restore
  `WS_CAPTION | WS_THICKFRAME`, then calls `SetWindowPos(SWP_FRAMECHANGED)`. UI Automation observed
  native Minimize/Maximize/Close controls disappear in frameless mode and return after restoring the
  frame. Removing these styles also disables native resize. Custom drag hit-testing, resize behavior,
  and replacement caption buttons are not implemented or verified.
- **Icon grid:** `GridView` / `GridViewItem` display 48 fixed-size (116 × 132 DIP), local
  image-backed folder/document examples with wrapping/truncated names. UI Automation observed 48
  Image controls and representative labels. On this host, resizing the committed test window from
  1666 px to 1346 px changed the observed layout from 6 to 5 columns. These are sample items, not
  filesystem entries or proof of pixel parity with Explorer.
- The images are the repository's `folder_icon.png` / `question_unknown.png`. Shell icon extraction,
  directory enumeration, opening items, Explorer selection/keyboard parity, virtualization under
  load, and theme fidelity remain untested.
- The executable uses the Windows GUI subsystem (`windows_subsystem = "windows"`). The built PE
  subsystem was verified as GUI (2), and startup produced the app window without a separate console.
- The keyed test window is owned by `ApplicationView`. Closing it destroys its component state;
  reopening creates a new component. This is not the product's state-preserving Hide/Show behavior.

These are interactive observations from this Windows host, not CI visual assertions. A clean-machine
deployment, Explorer parity, drag/resize, DPI, and multi-monitor behavior have not been established.

## Window placement and lifecycle limits

- `WindowPlacement` is outer-window position/size in physical screen pixels (negative coordinates
  are valid); initial client size remains DIPs. Placement is captured by the placement message
  callback, not a shutdown callback.
- Reactor reads `WindowVisuals::initial_placement` only when the component/window is first declared.
  Supply saved placement before `show_window`; changing it does not move an already-open window.
  Fully off-screen bounds are adjusted by Windows, bounds/size are validated, and minimized state is
  excluded from observation. Values in this POC are in-memory only.
- This revision has `show_window` and `exit`, but no first-class hide-window command. Closing a
  keyed window destroys the component instance; reopening creates it again and does not retain its
  local component state.
- The native notify icon registers `TaskbarCreated` and re-registers itself after recovery according
  to source/API inspection. Explorer restart behavior has not been tested on this host.
- This POC targets x64 only. DPI, negative-coordinate/multiple-monitor behavior, and display removal
  still require live testing. Reactor remains preview; a Git SHA improves reproducibility but is not
  a production API stability promise.

## Build and manual test

Requirements: Windows x64, Rust 1.95+, MSVC C++ Build Tools, Windows SDK, compatible Windows App
Runtime, and an interactive Explorer desktop for tray/UI tests. `.cargo/config.toml` pins
`x86_64-pc-windows-msvc`.

```powershell
cargo fmt --all -- --check
cargo check --locked --offline --all-targets
cargo clippy --locked --offline --all-targets -- -D warnings
cargo test --locked --offline
cargo build --locked --offline
cargo tree --locked --offline -i windows-reactor
cargo tree --locked --offline -d
```

Run the built executable (or `cargo run --locked --offline --bin windows-reactor-master-poc`) on an
interactive desktop. Use the main-window button or tray menu item **Borderless / icon grid test**.
Check the image count, resize/reflow, and frame toggle/restore. When frameless, native caption
buttons and native resize are absent; use the in-window Restore frame button before closing if
needed. Separately test tray Show/Exit, Explorer restart recovery, placement/DPI/multiple monitors,
and clean-machine deployment. UI automation or headless tests do not replace these checks.

## Results (2026-10-12)

```text
Source checkout: windows-rs revision d5b6a1f68f4fdd0c79495954d162901ea90da4f4.
rustc: 1.99.0 (b940084d7 2026-09-28), x86_64-pc-windows-msvc.
cargo: 1.99.0 (5f94df478 2026-08-27).
MSVC: Visual Studio Community 2026 18.10.3, toolset 14.51.36231.
Windows SDK: 10.0.26100.0.
fmt/check/clippy/test/build: passed using --locked --offline; clippy used -D warnings.
Unit tests: 1 passed (native frame-style mask removes/restores only the requested bits).
Dependency tree: cargo tree -d reported no duplicate packages; windows-sys 0.100.0 uses the pinned SHA.
Executable: PE subsystem 2 (GUI); interactive startup had no separate console window.
UI Automation: main window launched the keyed chrome-grid test; 48 image controls were present.
Frame toggle: native Minimize/Maximize/Close controls disappeared and all three returned on restore.
Grid resize: 1666 px showed 6 columns; 1346 px showed 5 columns.
Prior placement test: (1020, 448), 1306 x 791 physical pixels, maximized=false on this host.
Owner observation: tray icon and right-click menu reportedly work; this is not automated evidence.
Not verified: each tray action/double-click/Exit cleanup, Explorer restart recovery, product Hide,
  drag/custom resize, Shell icon extraction, filesystem browsing, DPI/multi-monitor, clean deployment.
```
