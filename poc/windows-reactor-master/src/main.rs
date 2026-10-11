mod app;
mod main_window;
mod placement_window;

use app::TrayApp;
use windows_reactor::App;

fn main() {
    if let Err(error) = App::run_application::<TrayApp>(()) {
        eprintln!("windows-reactor master POC failed: {error}");
        std::process::exit(1);
    }
}
