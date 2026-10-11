# windows-reactor POC

An isolated experiment for the exact `windows-reactor = 0.100.0` release. It is deliberately not
a workspace member, so the root crate's default Cargo commands do not build this POC.

## Build and run

Requirements: Windows x64, Rust 1.95 or later with the `x86_64-pc-windows-msvc` target, MSVC C++
Build Tools, Windows SDK, and a compatible Windows App Runtime. `.cargo/config.toml` pins the target
triple. From this directory, run:

```powershell
cargo fmt --all -- --check
cargo check --all-targets
cargo test
cargo run
cargo tree -i windows-reactor
cargo tree -d
```

The executable intentionally retains its console while the preview APIs are being diagnosed.
`cargo run` opens a native window and remains running until its windows are closed. The POC uses the
framework-dependent mode; it does not add `windows-reactor-setup` or a packaging step.

## Implemented slice

- A Reactor `Component` main window opens a directory window and a settings window.
- The directory view enumerates one level on Reactor's Windows thread pool using
  `ComponentContext::spawn_background`. Refresh cancels the previous task and carries a generation
  number so an outdated completion cannot replace the current listing. Entries are sorted with
  directories first, then case-insensitive name order; empty and error states are shown.
- The settings window has an alpha slider and a pure-Rust `SettingsDraft`; its unit tests cover
  preview, Save, and Cancel. Alpha is applied to a content panel only, not to the native window.
- Closing the drawer/settings windows requests their own close. Task scope retirement is left to
  Reactor when its owning component window closes.

## Fixed-release API gap (blocks the full POC)

The POC guide suggests `windows-reactor = "=0.100.0"`, but the published 0.100.0 source and its
exported API do **not** contain `Application`, `ApplicationView`, tray/notify-icon menu APIs, or
`App::run_application`. Nor does its `WindowVisuals` expose the newer placement APIs shown by the
current `master` samples. Current upstream examples therefore cannot be copied into this pinned
crate without changing versions. This POC intentionally does not pull from `master`, add a private
HWND/tray implementation, or claim those requirements passed.

As a result, this slice cannot verify tray lifetime after the main window closes, stable keyed
window registration/reopen semantics, settings shared with the main window or persisted across
reopening, placement restoration, DPI/multi-monitor behavior, whole-window transparency,
Explorer-restart tray recovery, or clean-machine framework-dependent deployment. Those remain
manual follow-ups after choosing a release that publishes the required APIs (or recording an
approved platform adapter spike). The installed SDK/runtime on a developer machine is not evidence
that a clean deployment target can run the app.

## Environment / results

Recorded in the current workspace on 2026-10-11:

- Windows 10 Pro, x64 (host build `10.0.26300`); installed Windows SDK includes `10.0.26100.0`.
- `rustc 1.99.0 (b940084d7 2026-09-28)`, host `x86_64-pc-windows-msvc`.
- `cargo 1.99.0 (5f94df478 2026-08-27)`.
- Visual Studio Community 2026 `18.10.3`, MSVC toolset `14.51.36231`.
- Windows App Runtime packages are installed (including 1.8 and 2.5.1); this only records the
  development host, not the runtime version/installation requirement for a clean target machine.
- `cargo fmt --all -- --check`, `cargo check --all-targets`, and `cargo test` pass for this POC;
  both SettingsDraft tests pass. `cargo tree -i windows-reactor` shows only the isolated POC
  depends on Reactor, and `cargo tree -d` reports no duplicate package versions.
- The root crate's `cargo fmt --all -- --check` and `cargo test` also pass without building this
  isolated crate.
- The native executable has not been manually run through tray/window automation, DPI, opacity,
  or clean-machine deployment tests. A successful build or headless test is not a visual WinUI,
  tray, DPI, transparency, or deployment pass.