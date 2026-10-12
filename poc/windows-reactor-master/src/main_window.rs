use windows_reactor::*;

use crate::app::MainWindowInput;

#[derive(Clone, Copy)]
pub(crate) enum Message {
    CloseThisWindow,
    OpenChromeGrid,
}

pub(crate) struct MainWindow {
    open_chrome_grid: Callback<()>,
}

impl Component for MainWindow {
    type Input = MainWindowInput;
    type Message = Message;

    fn create(input: &Self::Input, _: &ComponentContext<Self>) -> Self {
        Self {
            open_chrome_grid: input.open_chrome_grid.clone(),
        }
    }

    fn input_changed(&mut self, input: &Self::Input, _: &ComponentContext<Self>) {
        self.open_chrome_grid = input.open_chrome_grid.clone();
    }

    fn update(&mut self, message: Self::Message, context: &ComponentContext<Self>) {
        match message {
            Message::CloseThisWindow => {
                let _ = context.close_window();
            }
            Message::OpenChromeGrid => self.open_chrome_grid.call(()),
        }
    }

    fn view(&self, input: &Self::Input, context: &mut ViewContext<Self>) -> View {
        context.window_visuals(WindowVisuals::new().client_size(470.0, 310.0));
        context.window_title("draweR · Reactor application lifetime");

        Border::new()
            .padding(Thickness::uniform(24.0))
            .content(
                StackPanel::new().spacing(12.0).children((
                    TextBlock::new().text("Commit-pinned windows-reactor Application POC"),
                    TextBlock::new()
                        .text("The tray is an application resource, independent of this window."),
                    Button::new()
                        .on_click(input.open_placement.clone())
                        .content("Open window placement test"),
                    Button::new()
                        .on_click(context.message(Message::OpenChromeGrid))
                        .content("Test borderless + Explorer icon grid"),
                    Button::new()
                        .on_click(context.message(Message::CloseThisWindow))
                        .content("Close this window (tray should remain)"),
                    TextBlock::new()
                        .text("Closing is not hiding: use the tray to reopen this keyed window."),
                )),
            )
            .into()
    }
}
