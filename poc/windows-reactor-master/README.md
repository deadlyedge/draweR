# Commit-pinned windows-reactor Application POC

This is an isolated second experiment. It does not modify or replace
`poc/windows-reactor` (the crates.io `0.100.0` POC) and is not a root workspace member.

## Exact source pin

```text
Repository: https://github.com/microsoft/windows-rs.git
Commit:     d5b6a1f68f4fdd0c79495954d162901ea90da4f4
Commit date: 2026-10-10T13:55:23-05:00
Subject:    `windows-reactor` unify native event dispatch (#5061)
Package:    windows-reactor 0.100.0, Rust 1.95, edition 2024
```

`Cargo.toml` uses the full immutable Git revision, not a branch or tag. Keep this crate's
`Cargo.lock` with the POC. The icon is the existing `assets/drawer.icon.4.ico` in this checkout;
there is no generated asset or build script.

## Scope

- `App::run_application::<TrayApp>` owns application state separate from component windows.
- The application declares stable `main` and `placement` window keys plus a `tray` resource.
- Tray activation and menu commands request `show_window` for either key; Exit calls `context.exit()`.
- The main window has a test action that closes only that window. The declared tray resource should
  keep the process alive so the keyed window can be reopened from the tray.
- The placement window observes `WindowPlacement`; the application stores the latest value in
  memory and passes it as the next initial input. Close then reopen tests an in-process restore.
- `WindowPlacement` is outer-window position/size in physical screen pixels (negative coordinates
  are valid). Initial client size remains DIPs. Placement is captured before closure in the
  placement message callback, not in a shutdown callback.

## Build/run

Requirements: Windows x64, Rust 1.95+, MSVC C++ Build Tools, Windows SDK, compatible Windows App
Runtime, and an interactive Explorer desktop for tray tests. `.cargo/config.toml` pins
`x86_64-pc-windows-msvc`.

```powershell
cargo fmt --all -- --check
cargo check --all-targets
cargo test
cargo run --bin windows-reactor-master-poc
cargo tree -i windows-reactor
cargo tree -d
```

When running, manually test: close main via its button and via the title-bar X; confirm process and
tray remain; tray activate / Show main reopens it; open placement, move/resize/maximize, close it,
and reopen via tray; test secondary monitor with negative origin and DPI scaling; Exit removes tray
and ends the process. Restart Explorer and check tray recovery. UI automation or headless tests are
not substitutes for these checks.

## API interpretation / limits

- Application docs state application state outlives individual windows and the app exits when no
  user windows or notification icons remain and lifecycle work has settled. `ApplicationView` is
  declarative, not an invisible window. A declared keyed window is initially closed; closing it
  drops that component state; `show_window` creates/reopens it without duplicate instances.
- This revision has `ApplicationContext::show_window` and `exit`, but **no first-class hide-window
  command**. `close_window()` closes/destroys the component instance; it is not hide. This POC tests
  close/reopen, and deliberately makes no hide-preserves-component-state claim.
- Native notify icon implementation registers `TaskbarCreated`, produces a `Recover` event, and
  re-registers the icon. The application adapter forwards recovery failures as application errors.
  This is API/source evidence, not a test that Explorer restart works on this host.
- `WindowVisuals::initial_placement` is read only from the initial component declaration. Supply
  the saved value before calling `show_window`; later changes do not move an already open window.
  Fully off-screen bounds are adjusted by Windows; bounds/size are validated. Minimized state is
  excluded from observation. Values are stored in memory only, so a process restart returns to the
  default unless persistence is added.
- This project targets x64 only. DPI / multiple-monitor behavior and negative coordinates still
  require live testing; code comments and upstream implementation are not empirical results.
- Reactor remains preview. A Git SHA makes this experiment reproducible, not a production API
  stability promise.

## Results (2026-10-11)

```text
Source: local windows-rs checkout verified at d5b6a1f68f4fdd0c79495954d162901ea90da4f4.
rustc: 1.99.0 (b940084d7 2026-09-28), x86_64-pc-windows-msvc.
cargo: 1.99.0 (5f94df478 2026-08-27).
MSVC: Visual Studio Community 2026 18.10.3, toolset 14.51.36231.
Windows SDK: 10.0.26100.0 installed.
Build/check/test: fmt check, `cargo check --locked --offline --all-targets`, `cargo build --locked
  --offline`, and `cargo test --locked --offline` passed with the POC's actual Git-revision manifest
  and lockfile. Cargo's Git URL was temporarily rewritten to the already-present checkout when
  generating the lockfile; no network data was downloaded. `cargo test` had 0 test cases.
  `cargo tree` showed one Reactor dependency and no duplicate package versions.
Native startup: passed; UI Automation found the main window and rendered controls.
Open placement window: passed through UI Automation. Placement report showed
  (1020, 448), 1306 x 791 physical pixels, maximized=false on this host.
Close placement window: passed using its native Close button; the app and main window remained.
Close main with its button: passed; main HWND/title disappeared, process remained alive.
Tray icon visibility and right-click menu: manually observed working by the project owner (user report).
This is owner-observed evidence, not an automated UIA assertion; UI Automation could not enumerate
the tray item. Individual Show / Settings / Exit actions, tray double-click activation, reopening
each window via the tray, and Exit cleanup: not independently verified here.
Explorer restart / TaskbarCreated recovery: not verified.
DPI / negative-coordinate monitor / placement restoration after moving: not verified.
Explicit Hide semantics: no hide command in this revision; close destroys the component instance.
Exact immutable Git source in Cargo.toml and Cargo.lock: committed as full SHA above. The Cargo
  lockfile was generated with a local Git URL rewrite to the already-present exact-SHA checkout;
  `cargo check/test/tree/build --locked --offline` then all passed from the POC directory.
Additional lifecycle smoke check: after closing the main window by its POC button, the process
  stayed alive as expected while the tray resource remained declared. The project owner reports
  the icon and right-click interaction are working; the automated UIA tree did not expose the tray
  entry (possibly overflow/non-enumerable). Explorer restart/recovery and automated icon activation
  remain unverified.
```