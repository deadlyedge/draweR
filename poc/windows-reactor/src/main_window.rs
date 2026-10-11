use std::{env, path::PathBuf};

use windows_reactor::*;

use crate::{app::Theme, drawer_window::DrawerWindow, settings_window::SettingsWindow};

#[derive(Clone, Copy)]
pub(crate) enum Message {
    OpenDrawer,
    OpenSettings,
}

pub(crate) struct MainWindow;

impl Component for MainWindow {
    type Input = ();
    type Message = Message;

    fn create(_: &Self::Input, _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn update(&mut self, message: Self::Message, context: &ComponentContext<Self>) {
        match message {
            Message::OpenDrawer => {
                let path = desktop_directory();
                let _ = context.open_window(View::component::<DrawerWindow>(path));
            }
            Message::OpenSettings => {
                let _ = context.open_window(View::component::<SettingsWindow>(Theme::default()));
            }
        }
    }

    fn view(&self, _: &Self::Input, context: &mut ViewContext<Self>) -> View {
        context.window_visuals(WindowVisuals::new().client_size(380.0, 260.0));
        context.window_title("draweR · windows-reactor POC");
        Border::new().padding(Thickness::uniform(24.0)).content(
            StackPanel::new().spacing(12.0).children((
                TextBlock::new().text("Pinned windows-reactor 0.100.0"),
                TextBlock::new()
                    .text("Open a sample directory window or exercise the settings draft."),
                Button::new()
                    .on_click(context.message(Message::OpenDrawer))
                    .content("Open Desktop drawer"),
                Button::new()
                    .on_click(context.message(Message::OpenSettings))
                    .content("Open settings"),
                TextBlock::new()
                    .text("Tray/ApplicationView APIs are not present in the pinned release."),
            )),
        )
    }
}

fn desktop_directory() -> PathBuf {
    env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
        .join("Desktop")
}
