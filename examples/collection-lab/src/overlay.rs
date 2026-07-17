use arborui::{
    Color, Element, EventPhase, KeyAction, Modifier, PointerButton, PointerEventKind, Style,
    UiEvent, UiKey,
    layout::{Dimension, FlexDirection, LayoutStyle},
    prelude::{
        Application, Block, Command, Insets, Invalidation, UpdateContext, column, dialog,
        row_with_gap, spacer, stack,
    },
    widgets::text,
};

/// Stable keys used by the overlay workload's focus contract.
pub const OVERLAY_OPEN_KEY: &str = "overlay-open";
/// Stable key for the covered background action.
pub const OVERLAY_BACKGROUND_KEY: &str = "overlay-background";
/// Stable key for the dialog's confirm action.
pub const OVERLAY_CONFIRM_KEY: &str = "overlay-confirm";
/// Stable key for the dialog's cancel action.
pub const OVERLAY_CANCEL_KEY: &str = "overlay-cancel";

/// One deterministic action accepted by the shared overlay model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayAction {
    /// Open the confirmation dialog.
    Open,
    /// Confirm and close the dialog.
    Confirm,
    /// Cancel and close the dialog.
    Cancel,
    /// Activate the background control.
    ActivateBackground,
    /// Change the complete terminal dimensions.
    Resize {
        /// Complete terminal width.
        width: u16,
        /// Complete terminal height.
        height: u16,
    },
    /// Request application exit.
    Quit,
}

/// Framework-neutral state for the matched overlay workload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverlayModel {
    dialog_open: bool,
    confirmations: u64,
    background_activations: u64,
    terminal_width: u16,
    terminal_height: u16,
}

impl OverlayModel {
    /// Creates a closed overlay workload at explicit terminal dimensions.
    #[must_use]
    pub const fn new(width: u16, height: u16) -> Self {
        Self {
            dialog_open: false,
            confirmations: 0,
            background_activations: 0,
            terminal_width: width,
            terminal_height: height,
        }
    }

    /// Applies one deterministic action and returns whether it requests exit.
    pub const fn apply(&mut self, action: OverlayAction) -> bool {
        match action {
            OverlayAction::Open => self.dialog_open = true,
            OverlayAction::Confirm => {
                self.confirmations = self.confirmations.saturating_add(1);
                self.dialog_open = false;
            }
            OverlayAction::Cancel => self.dialog_open = false,
            OverlayAction::ActivateBackground => {
                self.background_activations = self.background_activations.saturating_add(1);
            }
            OverlayAction::Resize { width, height } => {
                self.terminal_width = width;
                self.terminal_height = height;
            }
            OverlayAction::Quit => return true,
        }
        false
    }

    /// Returns whether the modal dialog is present.
    #[must_use]
    pub const fn dialog_open(&self) -> bool {
        self.dialog_open
    }

    /// Returns the number of confirmed actions.
    #[must_use]
    pub const fn confirmations(&self) -> u64 {
        self.confirmations
    }

    /// Returns the number of background control activations.
    #[must_use]
    pub const fn background_activations(&self) -> u64 {
        self.background_activations
    }

    /// Returns the complete terminal dimensions.
    #[must_use]
    pub const fn terminal_size(&self) -> (u16, u16) {
        (self.terminal_width, self.terminal_height)
    }
}

/// Facade-only ArborUI adapter for the shared overlay model.
pub struct OverlayLab {
    model: OverlayModel,
}

impl OverlayLab {
    /// Creates an overlay application at explicit terminal dimensions.
    #[must_use]
    pub const fn new(width: u16, height: u16) -> Self {
        Self {
            model: OverlayModel::new(width, height),
        }
    }

    /// Returns the shared application model.
    #[must_use]
    pub const fn model(&self) -> &OverlayModel {
        &self.model
    }
}

impl Application for OverlayLab {
    type Message = OverlayAction;

    fn update(
        &mut self,
        message: Self::Message,
        context: &mut UpdateContext<Self::Message>,
    ) -> Command<Self::Message> {
        if self.model.apply(message) {
            return Command::quit();
        }
        context.invalidate(Invalidation::Recompose);
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let full = Dimension::percent(100);
        let background = column([
            text("Persistent workspace"),
            spacer(1, 1),
            action_label("[ Open dialog ]", OVERLAY_OPEN_KEY, 0, OverlayAction::Open),
            action_label(
                "[ Background action ]",
                OVERLAY_BACKGROUND_KEY,
                1,
                OverlayAction::ActivateBackground,
            ),
            spacer(1, 1),
            text("Confirmation and background counts are tracked semantically."),
        ])
        .layout(LayoutStyle {
            width: full,
            height: full,
            direction: FlexDirection::Column,
            ..LayoutStyle::default()
        });
        let application = Block::new(background)
            .title("Overlay composition")
            .padding(Insets::all(1))
            .border_style(Style::new().foreground(Color::BrightCyan))
            .layout(LayoutStyle::new().size(full, full))
            .build();
        let overlay_layout = LayoutStyle::new().size(full, full);
        let root = if self.model.dialog_open {
            let actions = row_with_gap(
                [
                    action_label(
                        "[ Confirm ]",
                        OVERLAY_CONFIRM_KEY,
                        0,
                        OverlayAction::Confirm,
                    ),
                    action_label("[ Cancel ]", OVERLAY_CANCEL_KEY, 1, OverlayAction::Cancel),
                ],
                2,
            );
            let content = column([text("Delete selected item?"), spacer(1, 1), actions]).layout(
                LayoutStyle {
                    width: full,
                    height: full,
                    direction: FlexDirection::Column,
                    ..LayoutStyle::default()
                },
            );
            let panel = Block::new(content)
                .title("Confirm action")
                .padding(Insets::all(1))
                .style(
                    Style::new()
                        .foreground(Color::BrightWhite)
                        .background(Color::Black),
                )
                .border_style(Style::new().foreground(Color::BrightCyan))
                .layout(LayoutStyle::new().size(Dimension::cells(26), Dimension::cells(7)))
                .build();
            let modal = dialog(panel, || OverlayAction::Cancel)
                .scrim_style(Style::new().background(Color::Black))
                .build()
                .key("overlay-dialog");
            stack([application, modal]).layout(overlay_layout)
        } else {
            stack([application]).layout(overlay_layout)
        };

        root.on_event(EventPhase::Capture, |event, context| match event {
            UiEvent::Resize(size) => context.emit(OverlayAction::Resize {
                width: size.width,
                height: size.height,
            }),
            UiEvent::Key(key)
                if matches!(key.action, KeyAction::Press | KeyAction::Repeat)
                    && key.key == UiKey::Character('q') =>
            {
                context.emit(OverlayAction::Quit);
                context.mark_handled();
            }
            _ => {}
        })
    }
}

fn action_label<'a>(
    label: &'a str,
    key: &'static str,
    order: i32,
    action: OverlayAction,
) -> Element<'a, OverlayAction> {
    text(label)
        .focusable(true)
        .focus_order(order)
        .focus_style(Style::new().add_modifiers(Modifier::REVERSED))
        .interactive(true)
        .on_event(EventPhase::Target, move |event, context| {
            let activated = matches!(
                event,
                UiEvent::Key(key_event)
                    if key_event.action == KeyAction::Press
                        && matches!(key_event.key, UiKey::Enter | UiKey::Character(' '))
            ) || matches!(
                event,
                UiEvent::Pointer(pointer)
                    if pointer.kind == PointerEventKind::Down(PointerButton::Primary)
            );
            if activated {
                context.emit(action);
                context.mark_handled();
                context.prevent_default();
            }
        })
        .key(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_model_tracks_overlay_transitions() {
        let mut model = OverlayModel::new(40, 12);
        assert!(!model.dialog_open());

        assert!(!model.apply(OverlayAction::Open));
        assert!(model.dialog_open());
        assert!(!model.apply(OverlayAction::Confirm));
        assert_eq!(model.confirmations(), 1);
        assert!(!model.dialog_open());
        assert!(!model.apply(OverlayAction::ActivateBackground));
        assert_eq!(model.background_activations(), 1);
        assert!(model.apply(OverlayAction::Quit));
    }
}
