use windows_reactor::*;

use crate::app::PlacementWindowInput;

pub(crate) struct PlacementWindow;

impl Component for PlacementWindow {
    type Input = PlacementWindowInput;
    type Message = ();

    fn create(_: &Self::Input, _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, input: &Self::Input, context: &mut ViewContext<Self>) -> View {
        let mut visuals = WindowVisuals::new().client_size(640.0, 360.0);
        if let Some(saved) = input.saved {
            visuals = visuals.initial_placement(saved);
        } else {
            visuals = visuals.initial_position(ScreenPoint::new(100, 100));
        }
        context.window_visuals(visuals);
        context.on_window_placement(input.on_placement.clone());

        let status = input.saved.map_or_else(
            || "No placement observed yet; initial top-left is (100, 100) physical pixels.".into(),
            |placement| {
                format!(
                    "Saved bounds: ({}, {}) {} × {} physical px; maximized={}",
                    placement.x,
                    placement.y,
                    placement.width,
                    placement.height,
                    placement.maximized,
                )
            },
        );

        context.window_frame(
            "Reactor window placement test",
            StackPanel::new().margin(24.0).spacing(12.0).children((
                "Move / resize / maximize, then close and reopen from the tray.",
                TextBlock::new().text(status),
                "Placement is retained in Application state for this process only.",
                "Coordinates are physical virtual-screen pixels; negative monitor origins are valid.",
                "Minimized placements are not reported. Whole-off-screen restore is adjusted by Windows.",
            )),
        )
    }
}
