use std::{
    fs,
    path::{Path, PathBuf},
};

use windows_reactor::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Entry {
    key: String,
    label: String,
    is_directory: bool,
}

#[derive(Clone)]
pub(crate) enum RefreshResult {
    Loaded(Vec<Entry>),
    Failed(String),
}

#[derive(Clone)]
pub(crate) enum Message {
    Refresh,
    Refreshed {
        generation: u64,
        result: RefreshResult,
    },
    Close,
}

pub(crate) struct DrawerWindow {
    path: PathBuf,
    entries: Vec<Entry>,
    status: String,
    generation: u64,
    refresh_task: Option<ComponentTask>,
}

impl Component for DrawerWindow {
    type Input = PathBuf;
    type Message = Message;

    fn create(path: &Self::Input, context: &ComponentContext<Self>) -> Self {
        let mut drawer = Self {
            path: path.clone(),
            entries: Vec::new(),
            status: "Loading directory…".to_string(),
            generation: 0,
            refresh_task: None,
        };
        drawer.start_refresh(context);
        drawer
    }

    fn update(&mut self, message: Self::Message, context: &ComponentContext<Self>) {
        match message {
            Message::Refresh => self.start_refresh(context),
            Message::Refreshed { generation, result } if generation == self.generation => {
                self.refresh_task = None;
                match result {
                    RefreshResult::Loaded(entries) if entries.is_empty() => {
                        self.entries = entries;
                        self.status = "This directory is empty.".to_string();
                    }
                    RefreshResult::Loaded(entries) => {
                        self.status = format!("{} item(s)", entries.len());
                        self.entries = entries;
                    }
                    RefreshResult::Failed(error) => {
                        self.entries.clear();
                        self.status = error;
                    }
                }
            }
            Message::Refreshed { .. } => {
                // An older request must never replace the active directory listing.
            }
            Message::Close => {
                let _ = context.window().request_close();
            }
        }
    }

    fn view(&self, _: &Self::Input, context: &mut ViewContext<Self>) -> View {
        context.window_title(format!("Drawer · {}", self.path.display()));
        context.window_visuals(WindowVisuals::new().client_size(520.0, 440.0));

        let rows = self.entries.iter().map(|entry| {
            let kind = if entry.is_directory { "📁" } else { "📄" };
            KeyedView::new(
                entry.key.clone(),
                TextBlock::new().text(format!("{kind}  {}", entry.label)),
            )
        });

        Border::new().padding(Thickness::uniform(18.0)).content(
            StackPanel::new().spacing(10.0).children((
                TextBlock::new().text(self.path.display().to_string()),
                TextBlock::new().text(self.status.clone()),
                StackPanel::new()
                    .orientation(Orientation::Horizontal)
                    .spacing(8.0)
                    .children((
                        Button::new()
                            .on_click(context.message(Message::Refresh))
                            .content("Refresh"),
                        Button::new()
                            .on_click(context.message(Message::Close))
                            .content("Close drawer"),
                    )),
                ScrollViewer::new()
                    .height(320.0)
                    .content(StackPanel::new().spacing(6.0).keyed_children(rows)),
            )),
        )
    }
}

impl DrawerWindow {
    fn start_refresh(&mut self, context: &ComponentContext<Self>) {
        if let Some(task) = self.refresh_task.take() {
            task.cancel();
        }

        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        let path = self.path.clone();
        self.status = "Loading directory…".to_string();
        self.entries.clear();

        self.refresh_task = Some(context.spawn_background(move |cancellation| {
            let result = read_directory(&path, &cancellation);
            Message::Refreshed { generation, result }
        }));
    }
}

fn read_directory(path: &Path, cancellation: &CancellationToken) -> RefreshResult {
    let result = (|| {
        let directory =
            fs::read_dir(path).map_err(|error| format!("Cannot read directory: {error}"))?;
        let mut entries = Vec::new();

        for item in directory {
            if cancellation.is_cancelled() {
                break;
            }

            let item = item.map_err(|error| format!("Cannot read directory entry: {error}"))?;
            let file_type = item
                .file_type()
                .map_err(|error| format!("Cannot inspect directory entry: {error}"))?;
            let label = item.file_name().to_string_lossy().into_owned();
            let key = item.path().to_string_lossy().into_owned();

            entries.push(Entry {
                key,
                label,
                is_directory: file_type.is_dir(),
            });
        }

        entries.sort_by(|left, right| {
            (!left.is_directory, left.label.to_lowercase(), &left.label).cmp(&(
                !right.is_directory,
                right.label.to_lowercase(),
                &right.label,
            ))
        });

        Ok(entries)
    })();

    match result {
        Ok(entries) => RefreshResult::Loaded(entries),
        Err(error) => RefreshResult::Failed(error),
    }
}
