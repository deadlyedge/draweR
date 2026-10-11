mod app;
mod drawer_window;
mod main_window;
mod settings_window;

use windows_reactor::App;

fn main() {
    if let Err(error) = App::run_component::<main_window::MainWindow>(()) {
        eprintln!("windows-reactor POC failed: {error}");
        std::process::exit(1);
    }
}
