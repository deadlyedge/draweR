use windows_reactor::*;

use crate::app::{SettingsDraft, Theme};

#[derive(Clone)]
pub(crate) enum Message {
    PreviewAlpha(f64),
    Save,
    Cancel,
}

pub(crate) struct SettingsWindow {
    draft: SettingsDraft,
    status: String,
}

impl Component for SettingsWindow {
    type Input = Theme;
    type Message = Message;

    fn create(theme: &Self::Input, _: &ComponentContext<Self>) -> Self {
        Self {
            draft: SettingsDraft::new(*theme),
            status: "Preview only; nothing has been saved.".to_string(),
        }
    }

    fn update(&mut self, message: Self::Message, context: &ComponentContext<Self>) {
        match message {
            Message::PreviewAlpha(alpha) => {
                self.draft
                    .preview_alpha(alpha.round().clamp(0.0, 255.0) as u8);
                self.status = "Preview updated (background alpha only).".to_string();
            }
            Message::Save => {
                self.draft.save();
                self.status = "Saved to this window's in-memory draft.".to_string();
            }
            Message::Cancel => {
                self.draft.cancel();
                let _ = context.window().request_close();
            }
        }
    }

    fn view(&self, _: &Self::Input, context: &mut ViewContext<Self>) -> View {
        context.window_visuals(WindowVisuals::new().client_size(420.0, 330.0));
        context.window_title("Settings draft");
        let alpha = self.draft.working().alpha;

        Border::new().padding(Thickness::uniform(22.0)).content(
            StackPanel::new().spacing(12.0).children((
                TextBlock::new().text("HSLA alpha preview"),
                TextBlock::new().text(format!("Background alpha: {alpha}/255")),
                Slider::new()
                    .minimum(0.0)
                    .maximum(255.0)
                    .value(f64::from(alpha))
                    .step_frequency(1.0)
                    .on_value_changed(context.callback(Message::PreviewAlpha)),
                Border::new()
                    .background(Color::argb(alpha, 45, 100, 180))
                    .padding(Thickness::uniform(18.0))
                    .content(TextBlock::new().text(
                        "This panel uses alpha; it does not make the whole window transparent.",
                    )),
                TextBlock::new().text(self.status.clone()),
                StackPanel::new()
                    .orientation(Orientation::Horizontal)
                    .spacing(8.0)
                    .children((
                        Button::new()
                            .on_click(context.message(Message::Save))
                            .content("Save draft"),
                        Button::new()
                            .on_click(context.message(Message::Cancel))
                            .content("Cancel and close"),
                    )),
            )),
        )
    }
}
