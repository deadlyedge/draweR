/// Minimal theme state kept independent of Reactor controls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Theme {
    pub(crate) alpha: u8,
}

impl Default for Theme {
    fn default() -> Self {
        Self { alpha: 220 }
    }
}

/// Settings-window working copy. Cancel restores the value from when the
/// settings window was opened; Save commits the current draft in memory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SettingsDraft {
    baseline: Theme,
    working: Theme,
    committed: Theme,
}

impl SettingsDraft {
    pub(crate) fn new(theme: Theme) -> Self {
        Self {
            baseline: theme,
            working: theme,
            committed: theme,
        }
    }

    pub(crate) fn working(&self) -> Theme {
        self.working
    }

    #[cfg(test)]
    pub(crate) fn committed(&self) -> Theme {
        self.committed
    }

    pub(crate) fn preview_alpha(&mut self, alpha: u8) {
        self.working.alpha = alpha;
    }

    pub(crate) fn save(&mut self) {
        self.committed = self.working;
        self.baseline = self.working;
    }

    pub(crate) fn cancel(&mut self) {
        self.working = self.baseline;
    }
}

#[cfg(test)]
mod tests {
    use super::{SettingsDraft, Theme};

    #[test]
    fn cancel_restores_theme_that_was_active_when_draft_opened() {
        let original = Theme { alpha: 200 };
        let mut draft = SettingsDraft::new(original);

        draft.preview_alpha(80);
        assert_eq!(draft.working().alpha, 80);
        assert_eq!(draft.committed(), original);

        draft.cancel();

        assert_eq!(draft.working(), original);
        assert_eq!(draft.committed(), original);
    }

    #[test]
    fn save_commits_the_previewed_theme() {
        let original = Theme { alpha: 200 };
        let mut draft = SettingsDraft::new(original);

        draft.preview_alpha(128);
        draft.save();

        let saved = Theme { alpha: 128 };
        assert_eq!(draft.working(), saved);
        assert_eq!(draft.committed(), saved);
    }
}
