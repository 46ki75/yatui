use std::{cell::Cell, collections::VecDeque, num::NonZeroUsize};

use arborui::{
    Color, Element, EventPhase, KeyAction, Modifier, Style, UiEvent, UiKey,
    layout::{Dimension, FlexDirection, LayoutStyle},
    prelude::{
        Application, Block, Command, Invalidation, Point, UpdateContext, column, list, scroll_view,
    },
    widgets::text,
};

use crate::{FixedHeightProvider, VisibleRange};

const LOG_OVERSCAN_ROWS: usize = 2;
const HELP: &str = "Up/Down/Page/Home/End scroll | a appends | q quit";
const SOURCES: [&str; 6] = ["api", "worker", "Zürich", "München", "cache", "Δelta"];
const MESSAGES: [&str; 6] = [
    "request completed",
    "checkpoint saved",
    "café queue drained",
    "retry scheduled",
    "health check passed",
    "payload indexed",
];

/// One stable record retained by the bounded scrolling-log experiment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogRecord {
    key: u64,
    text: String,
}

impl LogRecord {
    /// Returns the monotonic stable record key.
    #[must_use]
    pub const fn key(&self) -> u64 {
        self.key
    }

    /// Returns the preformatted display line.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// One deterministic action accepted by the shared scrolling-log model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogAction {
    /// Scroll upward by one line and pause follow-tail.
    Up,
    /// Scroll downward by one line.
    Down,
    /// Scroll upward by one viewport and pause follow-tail.
    PageUp,
    /// Scroll downward by one viewport.
    PageDown,
    /// Move to the oldest retained record and pause follow-tail.
    Home,
    /// Move to the newest retained record and resume follow-tail.
    End,
    /// Apply a signed wheel delta.
    Scrolled(Point),
    /// Change the complete terminal dimensions.
    Resize {
        /// Complete terminal width.
        width: u16,
        /// Complete terminal height.
        height: u16,
    },
    /// Append a deterministic producer batch.
    Append {
        /// Number of records in the batch.
        count: usize,
        /// Monotonic producer generation.
        generation: u64,
    },
    /// Request orderly shutdown from the interactive application.
    Quit,
}

/// Framework-neutral state and transition policy for the scrolling-log workload.
pub struct LogModel {
    records: VecDeque<LogRecord>,
    history_limit: usize,
    provider: FixedHeightProvider,
    terminal_width: u16,
    terminal_height: u16,
    viewport_height: usize,
    scroll: usize,
    follow_tail: bool,
    next_key: u64,
    generation: u64,
}

impl LogModel {
    /// Creates generated history with an explicit bound and terminal dimensions.
    #[must_use]
    pub fn new(initial_count: usize, history_limit: usize, width: u16, height: u16) -> Self {
        let history_limit = history_limit.max(1);
        let retained_count = initial_count.min(history_limit);
        let first_key = initial_count.saturating_sub(retained_count);
        let records = (first_key..initial_count)
            .map(|index| make_record(u64::try_from(index).unwrap_or(u64::MAX)))
            .collect();
        let viewport_height = log_viewport_height(height);
        let provider =
            FixedHeightProvider::new(retained_count, NonZeroUsize::MIN, LOG_OVERSCAN_ROWS);
        let scroll = provider.max_scroll(viewport_height);
        Self {
            records,
            history_limit,
            provider,
            terminal_width: width,
            terminal_height: height,
            viewport_height,
            scroll,
            follow_tail: true,
            next_key: u64::try_from(initial_count).unwrap_or(u64::MAX),
            generation: 0,
        }
    }

    /// Applies one action and returns whether it requests shutdown.
    pub fn apply(&mut self, action: LogAction) -> bool {
        match action {
            LogAction::Up => self.scroll_up(1),
            LogAction::Down => self.scroll_down(1),
            LogAction::PageUp => self.scroll_up(self.viewport_height),
            LogAction::PageDown => self.scroll_down(self.viewport_height),
            LogAction::Home => {
                self.scroll = 0;
                self.follow_tail = false;
            }
            LogAction::End => {
                self.scroll = self.max_scroll();
                self.follow_tail = true;
            }
            LogAction::Scrolled(delta) => {
                if delta.y.is_negative() {
                    self.scroll_up(delta.y.unsigned_abs() as usize);
                } else {
                    self.scroll_down(delta.y as usize);
                }
            }
            LogAction::Resize { width, height } => self.resize(width, height),
            LogAction::Append { count, generation } => self.append(count, generation),
            LogAction::Quit => return true,
        }
        false
    }

    /// Returns all retained records in chronological order.
    #[must_use]
    pub const fn records(&self) -> &VecDeque<LogRecord> {
        &self.records
    }

    /// Returns the configured maximum retained history.
    #[must_use]
    pub const fn history_limit(&self) -> usize {
        self.history_limit
    }

    /// Returns whether new records currently keep the viewport at the tail.
    #[must_use]
    pub const fn follows_tail(&self) -> bool {
        self.follow_tail
    }

    /// Returns the controlled row scroll offset.
    #[must_use]
    pub const fn scroll_offset(&self) -> usize {
        self.scroll
    }

    /// Returns the data viewport height.
    #[must_use]
    pub const fn viewport_height(&self) -> usize {
        self.viewport_height
    }

    /// Returns the latest accepted producer generation.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Returns the complete terminal dimensions.
    #[must_use]
    pub const fn terminal_size(&self) -> (u16, u16) {
        (self.terminal_width, self.terminal_height)
    }

    /// Returns the current visible and overscanned record range.
    #[must_use]
    pub fn visible_range(&self) -> VisibleRange {
        self.provider
            .visible_range(self.scroll, self.viewport_height)
    }

    fn max_scroll(&self) -> usize {
        self.provider.max_scroll(self.viewport_height)
    }

    fn scroll_up(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_sub(amount);
        self.follow_tail = false;
    }

    fn scroll_down(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_add(amount).min(self.max_scroll());
        self.follow_tail = self.scroll == self.max_scroll();
    }

    fn resize(&mut self, width: u16, height: u16) {
        self.terminal_width = width;
        self.terminal_height = height;
        self.viewport_height = log_viewport_height(height);
        self.scroll = if self.follow_tail {
            self.max_scroll()
        } else {
            self.scroll.min(self.max_scroll())
        };
    }

    fn append(&mut self, count: usize, generation: u64) {
        for _ in 0..count {
            self.records.push_back(make_record(self.next_key));
            self.next_key = self.next_key.saturating_add(1);
        }
        let evicted = self.records.len().saturating_sub(self.history_limit);
        self.records.drain(..evicted);
        self.provider =
            FixedHeightProvider::new(self.records.len(), NonZeroUsize::MIN, LOG_OVERSCAN_ROWS);
        self.scroll = if self.follow_tail {
            self.max_scroll()
        } else {
            self.scroll.saturating_sub(evicted).min(self.max_scroll())
        };
        self.generation = self.generation.max(generation);
    }
}

/// Returns the log-row viewport for a complete terminal height.
#[must_use]
pub const fn log_viewport_height(terminal_height: u16) -> usize {
    let height = terminal_height.saturating_sub(4);
    if height == 0 { 1 } else { height as usize }
}

/// Facade-only ArborUI adapter for the shared scrolling-log model.
pub struct LogLab {
    model: LogModel,
    constructed_rows: Cell<usize>,
}

impl LogLab {
    /// Creates a bounded scrolling-log application at explicit terminal dimensions.
    #[must_use]
    pub fn new(initial_count: usize, history_limit: usize, width: u16, height: u16) -> Self {
        Self {
            model: LogModel::new(initial_count, history_limit, width, height),
            constructed_rows: Cell::new(0),
        }
    }

    /// Returns the shared application model.
    #[must_use]
    pub const fn model(&self) -> &LogModel {
        &self.model
    }

    /// Returns the number of row elements built by the latest view.
    #[must_use]
    pub fn constructed_rows(&self) -> usize {
        self.constructed_rows.get()
    }
}

impl Application for LogLab {
    type Message = LogAction;

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
        let range = self.model.visible_range();
        self.constructed_rows.set(range.len());
        let full_width = Dimension::percent(100);
        let rows = self
            .model
            .records()
            .iter()
            .skip(range.start())
            .take(range.len())
            .map(|record| {
                (
                    record.key(),
                    text(record.text()).layout(LayoutStyle {
                        width: full_width,
                        height: Dimension::cells(1),
                        flex_shrink: 0,
                        ..LayoutStyle::default()
                    }),
                )
            });
        let content = list(rows).layout(LayoutStyle {
            width: full_width,
            height: Dimension::cells(u16::try_from(range.content_height()).unwrap_or(u16::MAX)),
            direction: FlexDirection::Column,
            flex_shrink: 0,
            ..LayoutStyle::default()
        });
        let local_offset = i32::try_from(range.local_offset()).unwrap_or(i32::MAX);
        let append_generation = self.model.generation().saturating_add(1);
        let body = scroll_view(Point::new(0, local_offset), content)
            .on_scroll(LogAction::Scrolled)
            .layout(LayoutStyle::new().size(
                full_width,
                Dimension::cells(u16::try_from(self.model.viewport_height()).unwrap_or(u16::MAX)),
            ))
            .build()
            .key("log")
            .focusable(true)
            .focus_style(Style::new().add_modifiers(Modifier::REVERSED))
            .on_event(EventPhase::Target, move |event, context| {
                let message = match event {
                    UiEvent::Key(key)
                        if matches!(key.action, KeyAction::Press | KeyAction::Repeat) =>
                    {
                        match key.key {
                            UiKey::Up => Some(LogAction::Up),
                            UiKey::Down => Some(LogAction::Down),
                            UiKey::Home => Some(LogAction::Home),
                            UiKey::End => Some(LogAction::End),
                            UiKey::PageUp => Some(LogAction::PageUp),
                            UiKey::PageDown => Some(LogAction::PageDown),
                            UiKey::Character('a') => Some(LogAction::Append {
                                count: 1,
                                generation: append_generation,
                            }),
                            UiKey::Character('q') | UiKey::Escape => Some(LogAction::Quit),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                if let Some(message) = message {
                    context.emit(message);
                    context.mark_handled();
                    context.prevent_default();
                }
            });
        let panel = Block::new(body)
            .title("Bounded scrolling log")
            .border_style(Style::new().foreground(Color::BrightCyan))
            .layout(
                LayoutStyle::new().size(
                    full_width,
                    Dimension::cells(
                        u16::try_from(self.model.viewport_height().saturating_add(2))
                            .unwrap_or(u16::MAX),
                    ),
                ),
            )
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
                    context.emit(LogAction::Resize {
                        width: size.width,
                        height: size.height,
                    });
                }
            })
    }
}

fn make_record(key: u64) -> LogRecord {
    let index = usize::try_from(key).unwrap_or(usize::MAX);
    let level = match index % 4 {
        0 => "INFO ",
        1 => "DEBUG",
        2 => "WARN ",
        _ => "TRACE",
    };
    LogRecord {
        key,
        text: format!(
            "{key:06} {level} {:<7} {} ({:03} ms)",
            SOURCES[index % SOURCES.len()],
            MESSAGES[index % MESSAGES.len()],
            5 + index % 995,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paused_append_preserves_anchor_across_eviction() {
        let mut model = LogModel::new(20, 20, 48, 12);
        model.apply(LogAction::Home);
        model.apply(LogAction::Down);
        let anchor = model.records()[1].key();

        model.apply(LogAction::Append {
            count: 1,
            generation: 3,
        });

        assert!(!model.follows_tail());
        assert_eq!(model.records()[model.scroll_offset()].key(), anchor);
        assert_eq!(model.records().len(), 20);
        assert_eq!(model.generation(), 3);
    }
}
