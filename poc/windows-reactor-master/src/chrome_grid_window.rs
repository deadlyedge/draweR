use windows_reactor::*;
use windows_sys::Win32::{
    errhandlingapi::{GetLastError, SetLastError},
    windef::HWND,
    winuser::{
        GWL_STYLE, GetWindowLongPtrW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        SWP_NOZORDER, SetWindowLongPtrW, SetWindowPos, WS_CAPTION, WS_THICKFRAME,
    },
};

const FRAME_STYLE_BITS: isize = (WS_CAPTION | WS_THICKFRAME) as isize;
const WINDOW_TITLE: &str = "draweR · frameless / Explorer-like icon grid";

#[derive(Clone)]
pub(crate) enum Message {
    ToggleFrame,
    FrameChanged(Result<bool, String>),
    SelectionChanged(Option<usize>),
    Close,
}

pub(crate) struct ChromeGridWindow {
    frame_removed: bool,
    selected: Option<usize>,
    status: String,
}

impl Component for ChromeGridWindow {
    type Input = ();
    type Message = Message;

    fn create(_: &Self::Input, _: &ComponentContext<Self>) -> Self {
        Self {
            frame_removed: false,
            selected: None,
            status: "Native caption and resize frame are enabled.".into(),
        }
    }

    fn update(&mut self, message: Self::Message, context: &ComponentContext<Self>) {
        match message {
            Message::ToggleFrame => {
                let remove = !self.frame_removed;
                if !context.run_window(move |window| {
                    let result = set_native_frame(window.as_raw().cast(), !remove).map(|()| remove);
                    Message::FrameChanged(result)
                }) {
                    self.status = "Window style update was not queued.".into();
                } else {
                    self.status = "Updating native frame style…".into();
                }
            }
            Message::FrameChanged(Ok(removed)) => {
                self.frame_removed = removed;
                self.status = if removed {
                    "WS_CAPTION and WS_THICKFRAME removed; native resize is disabled."
                } else {
                    "Native caption and resize frame restored."
                }
                .into();
            }
            Message::FrameChanged(Err(error)) => {
                self.status = format!("Could not update native window frame: {error}");
            }
            Message::SelectionChanged(selected) => self.selected = selected,
            Message::Close => {
                let _ = context.close_window();
            }
        }
    }

    fn view(&self, _: &Self::Input, context: &mut ViewContext<Self>) -> View {
        context.window_visuals(WindowVisuals::new().client_size(820.0, 720.0).constraints(
            WindowConstraints {
                min_width: Some(560.0),
                min_height: Some(480.0),
                ..Default::default()
            },
        ));
        context.window_title(WINDOW_TITLE);

        let items = icon_items();
        let title_bar = TitleBar::new()
            .preferred_height(WindowTitleBarHeight::Tall)
            .title("draweR")
            .subtitle("Custom WinUI title bar / frameless HWND toggle")
            .left_header(TextBlock::new().text("POC"))
            .right_header(
                Button::new()
                    .on_click(context.message(Message::ToggleFrame))
                    .content(if self.frame_removed {
                        "Restore frame"
                    } else {
                        "Toggle frameless"
                    }),
            );

        Border::new().padding(Thickness::uniform(14.0)).content(
            StackPanel::new().spacing(10.0).children((
                title_bar,
                TextBlock::new().text(format!(
                    "{} items · fixed-size tiles · responsive horizontal wrap",
                    items.len()
                )),
                TextBlock::new().text(
                    "Local sample images only; this does not test Shell icon extraction.",
                ),
                StackPanel::new()
                    .orientation(Orientation::Horizontal)
                    .spacing(8.0)
                    .children((
                        Button::new()
                            .on_click(context.message(Message::Close))
                            .content("Close test window"),
                        TextBlock::new().text(self.status.clone()),
                    )),
                GridView::new()
                    .height(550.0)
                    .horizontal_alignment(HorizontalAlignment::Stretch)
                    .selected_index(self.selected)
                    .on_selection_changed(context.callback(Message::SelectionChanged))
                    .keyed_items(items.into_iter().map(|item| {
                        KeyedView::new(
                            item.key.clone(),
                            GridViewItem::new().tag(item.key).content(
                                Border::new()
                                    .width(116.0)
                                    .height(132.0)
                                    .padding(Thickness::uniform(8.0))
                                    .corner_radius(6.0)
                                    .background(Color::rgb(248, 246, 240))
                                    .content(
                                        StackPanel::new()
                                            .spacing(6.0)
                                            .horizontal_alignment(HorizontalAlignment::Center)
                                            .children((
                                                Image::new().source_data(EncodedImage::from_static(
                                                    if item.is_folder {
                                                        include_bytes!("../../../assets/icons/folder_icon.png")
                                                    } else {
                                                        include_bytes!("../../../assets/icons/question_unknown.png")
                                                    },
                                                ))
                                                .width(64.0)
                                                .height(64.0)
                                                .stretch(Stretch::Uniform),
                                                TextBlock::new()
                                                    .text(item.name)
                                                    .width(96.0)
                                                    .height(38.0)
                                                    .text_wrapping(TextWrapping::Wrap)
                                                    .text_trimming(TextTrimming::CharacterEllipsis)
                                                    .horizontal_alignment(HorizontalAlignment::Center),
                                            )),
                                    ),
                            ),
                        )
                    })),
            )),
        )
        .into()
    }
}

struct IconItem {
    key: String,
    name: String,
    is_folder: bool,
}

fn icon_items() -> Vec<IconItem> {
    (0..48)
        .map(|index| {
            let is_folder = index % 4 != 3;
            let name = if is_folder {
                match index % 8 {
                    0 => format!("Projects\n{}", index / 4),
                    1 => format!("Long folder name that wraps {}", index + 1),
                    2 => format!("Screenshots & images {}", index + 1),
                    _ => format!("Archive {}", index + 1),
                }
            } else {
                format!("Document with a long filename {}.txt", index + 1)
            };
            IconItem {
                key: format!("item-{index:02}"),
                name,
                is_folder,
            }
        })
        .collect()
}

fn set_native_frame(hwnd: HWND, enabled: bool) -> Result<(), String> {
    if hwnd.is_null() {
        return Err("Reactor supplied a null HWND".into());
    }

    // SAFETY: run_window lends Reactor's live HWND synchronously. This callback does not retain
    // or destroy it, and completes the style update before handing control back to Reactor.
    unsafe {
        SetLastError(0);
        let previous = GetWindowLongPtrW(hwnd, GWL_STYLE);
        let error = GetLastError();
        if previous == 0 && error != 0 {
            return Err(format!("GetWindowLongPtrW failed with Win32 error {error}"));
        }

        let frame_bits = FRAME_STYLE_BITS;
        let updated = if enabled {
            previous | frame_bits
        } else {
            previous & !frame_bits
        };
        SetLastError(0);
        let replaced = SetWindowLongPtrW(hwnd, GWL_STYLE, updated);
        let error = GetLastError();
        if replaced == 0 && error != 0 {
            return Err(format!("SetWindowLongPtrW failed with Win32 error {error}"));
        }

        SetLastError(0);
        if SetWindowPos(
            hwnd,
            core::ptr::null_mut(),
            0,
            0,
            0,
            0,
            (SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED) as u32,
        ) == 0
        {
            let error = GetLastError();
            SetLastError(0);
            let _ = SetWindowLongPtrW(hwnd, GWL_STYLE, previous);
            let _ = SetWindowPos(
                hwnd,
                core::ptr::null_mut(),
                0,
                0,
                0,
                0,
                (SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED) as u32,
            );
            return Err(format!(
                "SetWindowPos(SWP_FRAMECHANGED) failed with Win32 error {error}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use windows_sys::Win32::winuser::{WS_CAPTION, WS_THICKFRAME};

    fn toggle_frame(style: isize, enabled: bool) -> isize {
        let bits = (WS_CAPTION | WS_THICKFRAME) as isize;
        if enabled { style | bits } else { style & !bits }
    }

    #[test]
    fn frame_toggle_removes_and_restores_only_native_frame_bits() {
        let bits = (WS_CAPTION | WS_THICKFRAME) as isize;
        let original = 0x0008_0000_isize | bits;
        let frameless = toggle_frame(original, false);

        assert_eq!(frameless & bits, 0);
        assert_eq!(frameless & 0x0008_0000, 0x0008_0000);
        assert_eq!(toggle_frame(frameless, true), original);
    }
}
