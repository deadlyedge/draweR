use windows_reactor::*;

use crate::{main_window::MainWindow, placement_window::PlacementWindow};

const MAIN_WINDOW_KEY: &str = "main";
const PLACEMENT_WINDOW_KEY: &str = "placement";
const TRAY_ICON_KEY: &str = "tray";

#[derive(Clone)]
pub(crate) enum Message {
    ShowMain,
    ShowPlacement,
    Menu(Key),
    PlacementChanged(WindowPlacement),
}

#[derive(Default)]
pub(crate) struct TrayApp {
    saved_placement: Option<WindowPlacement>,
}

impl Application for TrayApp {
    type Input = ();
    type Message = Message;

    fn create(_: &(), context: &ApplicationContext<Self>) -> Self {
        context.show_window(MAIN_WINDOW_KEY);
        Self::default()
    }

    fn update(&mut self, message: Self::Message, context: &ApplicationContext<Self>) {
        match message {
            Message::ShowMain => context.show_window(MAIN_WINDOW_KEY),
            Message::ShowPlacement => context.show_window(PLACEMENT_WINDOW_KEY),
            Message::Menu(key) => match key.as_str() {
                Some("main") => context.show_window(MAIN_WINDOW_KEY),
                Some("placement") => context.show_window(PLACEMENT_WINDOW_KEY),
                Some("exit") => context.exit(),
                _ => {}
            },
            Message::PlacementChanged(placement) => {
                self.saved_placement = Some(placement);
            }
        }
    }

    fn view(&self, _: &(), context: &ApplicationViewContext<Self>) -> ApplicationView {
        let icon_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "\\..\\..\\assets\\drawer.icon.4.ico"
        );

        ApplicationView::new()
            .window::<MainWindow>(
                MAIN_WINDOW_KEY,
                MainWindowInput {
                    open_placement: context.callback(|()| Message::ShowPlacement),
                },
            )
            .window::<PlacementWindow>(
                PLACEMENT_WINDOW_KEY,
                PlacementWindowInput {
                    saved: self.saved_placement,
                    on_placement: context.callback(Message::PlacementChanged),
                },
            )
            .notify_icon(
                TRAY_ICON_KEY,
                NotifyIcon::new(icon_path)
                    .tooltip("draweR windows-reactor master POC")
                    .on_activate(context.callback(|_| Message::ShowMain))
                    .menu(Menu::new(
                        [
                            MenuItem::item("main", "Show main window"),
                            MenuItem::item("placement", "Show placement test"),
                            MenuItem::separator("separator"),
                            MenuItem::item("exit", "Exit"),
                        ],
                        context.callback(Message::Menu),
                    )),
            )
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct MainWindowInput {
    pub(crate) open_placement: Callback<()>,
}

#[derive(Clone, PartialEq)]
pub(crate) struct PlacementWindowInput {
    pub(crate) saved: Option<WindowPlacement>,
    pub(crate) on_placement: Callback<WindowPlacement>,
}
