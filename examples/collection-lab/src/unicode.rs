use arborui::{
    Color, Element, EventPhase, KeyAction, Modifier, Style, UiEvent, UiKey,
    layout::{Dimension, FlexDirection, LayoutStyle},
    prelude::{
        Application, Block, Command, Invalidation, Point, UpdateContext, column, scroll_view,
    },
    widgets::text,
};

const CONTENT_WIDTH: u16 = 56;
const PANEL_HEIGHT: u16 = 9;
const ROW_COUNT: u16 = 7;
const HELP: &str = "Arrows shift | r replace | q quit";
const WIDE_REPLACEMENT: &str = "07 replace   | before 界 after | marker";
const NARROW_REPLACEMENT: &str = "07 replace   | before x after | marker";

/// One deterministic action accepted by the shared Unicode workload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnicodeAction {
    /// Shift the content one terminal cell to the right.
    ShiftRight,
    /// Shift the content one terminal cell to the left.
    ShiftLeft,
    /// Toggle one row between a width-two glyph and width-one ASCII.
    ReplaceWide,
    /// Change the complete terminal dimensions.
    Resize {
        /// Complete terminal width.
        width: u16,
        /// Complete terminal height.
        height: u16,
    },
    /// Request orderly shutdown from the interactive application.
    Quit,
}

/// Framework-neutral state for matched Unicode clipping workloads.
pub struct UnicodeModel {
    rows: Vec<String>,
    offset: usize,
    replacement_is_wide: bool,
    terminal_width: u16,
    terminal_height: u16,
}

impl UnicodeModel {
    /// Creates the deterministic Unicode rows at explicit terminal dimensions.
    #[must_use]
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            rows: unicode_rows(true),
            offset: 0,
            replacement_is_wide: true,
            terminal_width: width,
            terminal_height: height,
        }
    }

    /// Applies one action and returns whether it requests shutdown.
    pub fn apply(&mut self, action: UnicodeAction) -> bool {
        match action {
            UnicodeAction::ShiftRight => {
                self.offset = self.offset.saturating_add(1).min(self.max_offset());
            }
            UnicodeAction::ShiftLeft => self.offset = self.offset.saturating_sub(1),
            UnicodeAction::ReplaceWide => {
                self.replacement_is_wide = !self.replacement_is_wide;
                if let Some(row) = self.rows.last_mut() {
                    *row = if self.replacement_is_wide {
                        WIDE_REPLACEMENT
                    } else {
                        NARROW_REPLACEMENT
                    }
                    .to_owned();
                }
            }
            UnicodeAction::Resize { width, height } => {
                self.terminal_width = width;
                self.terminal_height = height;
                self.offset = self.offset.min(self.max_offset());
            }
            UnicodeAction::Quit => return true,
        }
        false
    }

    /// Returns the controlled horizontal cell offset.
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// Returns whether the designated row currently contains its width-two glyph.
    #[must_use]
    pub const fn replacement_is_wide(&self) -> bool {
        self.replacement_is_wide
    }

    /// Returns the complete terminal dimensions.
    #[must_use]
    pub const fn terminal_size(&self) -> (u16, u16) {
        (self.terminal_width, self.terminal_height)
    }

    /// Returns the deterministic rows in display order.
    #[must_use]
    pub fn rows(&self) -> &[String] {
        &self.rows
    }

    fn max_offset(&self) -> usize {
        usize::from(CONTENT_WIDTH.saturating_sub(self.terminal_width.saturating_sub(2).max(1)))
    }
}

/// Facade-only ArborUI adapter for the shared Unicode clipping model.
pub struct UnicodeLab {
    model: UnicodeModel,
}

impl UnicodeLab {
    /// Creates a Unicode clipping application at explicit terminal dimensions.
    #[must_use]
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            model: UnicodeModel::new(width, height),
        }
    }

    /// Returns the shared application model.
    #[must_use]
    pub const fn model(&self) -> &UnicodeModel {
        &self.model
    }
}

impl Application for UnicodeLab {
    type Message = UnicodeAction;

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
        let full_width = Dimension::percent(100);
        let rows = self.model.rows().iter().map(|row| {
            text(row).layout(LayoutStyle {
                width: Dimension::cells(CONTENT_WIDTH),
                height: Dimension::cells(1),
                flex_shrink: 0,
                ..LayoutStyle::default()
            })
        });
        let content = column(rows).layout(LayoutStyle {
            width: Dimension::cells(CONTENT_WIDTH),
            height: Dimension::cells(ROW_COUNT),
            direction: FlexDirection::Column,
            flex_shrink: 0,
            ..LayoutStyle::default()
        });
        let offset = i32::try_from(self.model.offset()).unwrap_or(i32::MAX);
        let body = scroll_view(Point::new(offset, 0), content)
            .layout(LayoutStyle::new().size(full_width, Dimension::cells(ROW_COUNT)))
            .build()
            .key("unicode-panel")
            .focusable(true)
            .focus_style(Style::new().add_modifiers(Modifier::REVERSED))
            .on_event(EventPhase::Target, |event, context| {
                let action = match event {
                    UiEvent::Key(key)
                        if matches!(key.action, KeyAction::Press | KeyAction::Repeat) =>
                    {
                        match key.key {
                            UiKey::Right => Some(UnicodeAction::ShiftRight),
                            UiKey::Left => Some(UnicodeAction::ShiftLeft),
                            UiKey::Character('r') => Some(UnicodeAction::ReplaceWide),
                            UiKey::Character('q') | UiKey::Escape => Some(UnicodeAction::Quit),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                if let Some(action) = action {
                    context.emit(action);
                    context.mark_handled();
                    context.prevent_default();
                }
            });
        let panel = Block::new(body)
            .title("Unicode cell clipping")
            .border_style(Style::new().foreground(Color::BrightCyan))
            .layout(LayoutStyle::new().size(full_width, Dimension::cells(PANEL_HEIGHT)))
            .build();

        column([text(HELP), panel])
            .layout(LayoutStyle {
                width: full_width,
                height: Dimension::percent(100),
                direction: FlexDirection::Column,
                ..LayoutStyle::default()
            })
            .on_event(EventPhase::Capture, |event, context| {
                if let UiEvent::Resize(size) = event {
                    context.emit(UnicodeAction::Resize {
                        width: size.width,
                        height: size.height,
                    });
                }
            })
    }
}

fn unicode_rows(wide_replacement: bool) -> Vec<String> {
    [
        "01 combining | cafe\u{301} | marker",
        "02 CJK       | 東京界 | marker",
        "03 ZWJ emoji | 👩\u{200d}💻 | marker",
        "04 flag      | 🇯🇵 | marker",
        "05 VS heart  | ❤\u{fe0f} | marker",
        "06 ambiguous | A·B | marker",
        if wide_replacement {
            WIDE_REPLACEMENT
        } else {
            NARROW_REPLACEMENT
        },
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_shifts_replaces_and_clamps_after_resize() {
        let mut model = UnicodeModel::new(36, 10);
        let original_rows = model.rows().to_vec();

        assert!(!model.apply(UnicodeAction::ShiftRight));
        assert_eq!(model.offset(), 1);
        assert!(!model.apply(UnicodeAction::ReplaceWide));
        assert!(!model.replacement_is_wide());
        assert_eq!(model.rows()[..6], original_rows[..6]);
        assert_eq!(model.rows()[6], NARROW_REPLACEMENT);

        for _ in 0..64 {
            model.apply(UnicodeAction::ShiftRight);
        }
        assert_eq!(model.offset(), 22);

        model.apply(UnicodeAction::Resize {
            width: 48,
            height: 12,
        });
        assert_eq!(model.offset(), 10);
        assert_eq!(model.terminal_size(), (48, 12));
        assert!(model.apply(UnicodeAction::Quit));
    }
}
