//! Pilot task queue and focus timer using only the `arborui` facade.

use std::time::Duration;

use arborui::{Modifier, prelude::*};

const DEFAULT_FOCUS_SECONDS: u32 = 25 * 60;
const TICK_INTERVAL: Duration = Duration::from_secs(1);

/// Messages accepted by [`FocusQueue`].
pub enum Message {
    /// Replace the controlled new-task input.
    DraftChanged(TextBuffer),
    /// Add the current non-empty draft to the queue.
    AddTask,
    /// Toggle a task's completion state.
    ToggleTask(u64),
    /// Delete a task from the queue.
    DeleteTask(u64),
    /// Move the controlled queue viewport.
    Scrolled(Point),
    /// Start or resume the focus timer.
    StartTimer,
    /// Pause the focus timer.
    PauseTimer,
    /// Restore the timer to its configured duration.
    ResetTimer,
    /// Process a scheduled timer tick if it is still current.
    TimerTick(u64),
    /// Request orderly application shutdown.
    Quit,
}

struct Task {
    id: u64,
    title: String,
    completed: bool,
}

/// A small task queue paired with a deterministic focus timer.
pub struct FocusQueue {
    draft: TextBuffer,
    tasks: Vec<Task>,
    next_task_id: u64,
    scroll_y: i32,
    focus_seconds: u32,
    remaining_seconds: u32,
    timer_running: bool,
    timer_generation: u64,
    timer_label: String,
    summary_label: String,
}

impl Default for FocusQueue {
    fn default() -> Self {
        Self::with_focus_seconds(DEFAULT_FOCUS_SECONDS)
    }
}

impl FocusQueue {
    /// Creates an empty queue with a focus timer of at least one second.
    #[must_use]
    pub fn with_focus_seconds(focus_seconds: u32) -> Self {
        let focus_seconds = focus_seconds.max(1);
        let mut queue = Self {
            draft: TextBuffer::default(),
            tasks: Vec::new(),
            next_task_id: 1,
            scroll_y: 0,
            focus_seconds,
            remaining_seconds: focus_seconds,
            timer_running: false,
            timer_generation: 0,
            timer_label: String::new(),
            summary_label: String::new(),
        };
        queue.refresh_labels();
        queue
    }

    /// Returns the controlled draft text.
    #[must_use]
    pub fn draft(&self) -> &str {
        self.draft.text()
    }

    /// Returns the number of queued tasks.
    #[must_use]
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Returns a task title by queue position.
    #[must_use]
    pub fn task_title(&self, index: usize) -> Option<&str> {
        self.tasks.get(index).map(|task| task.title.as_str())
    }

    /// Returns whether a task is complete by queue position.
    #[must_use]
    pub fn task_completed(&self, index: usize) -> Option<bool> {
        self.tasks.get(index).map(|task| task.completed)
    }

    /// Returns the timer's remaining whole seconds.
    #[must_use]
    pub const fn remaining_seconds(&self) -> u32 {
        self.remaining_seconds
    }

    /// Returns whether the focus timer is running.
    #[must_use]
    pub const fn timer_running(&self) -> bool {
        self.timer_running
    }

    /// Returns the vertical queue offset.
    #[must_use]
    pub const fn scroll_y(&self) -> i32 {
        self.scroll_y
    }

    fn add_task(&mut self) -> bool {
        let title = self.draft.text().trim().to_owned();
        if title.is_empty() {
            return false;
        }
        let Some(next_task_id) = self.next_task_id.checked_add(1) else {
            return false;
        };
        self.tasks.push(Task {
            id: self.next_task_id,
            title,
            completed: false,
        });
        self.next_task_id = next_task_id;
        self.draft = TextBuffer::default();
        self.refresh_labels();
        true
    }

    fn remove_task(&mut self, id: u64) -> bool {
        let previous_len = self.tasks.len();
        self.tasks.retain(|task| task.id != id);
        if self.tasks.len() == previous_len {
            return false;
        }
        self.scroll_y = self.scroll_y.min(self.max_scroll_y());
        self.refresh_labels();
        true
    }

    fn toggle_task(&mut self, id: u64) -> bool {
        let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) else {
            return false;
        };
        task.completed = !task.completed;
        self.refresh_labels();
        true
    }

    fn max_scroll_y(&self) -> i32 {
        i32::try_from(self.tasks.len().saturating_sub(1)).unwrap_or(i32::MAX)
    }

    fn refresh_labels(&mut self) {
        let minutes = self.remaining_seconds / 60;
        let seconds = self.remaining_seconds % 60;
        self.timer_label = format!("{minutes:02}:{seconds:02}");

        let completed = self.tasks.iter().filter(|task| task.completed).count();
        let open = self.tasks.len().saturating_sub(completed);
        self.summary_label = format!("{open} open / {completed} complete");
    }

    fn next_timer_generation(&mut self) -> u64 {
        self.timer_generation = self.timer_generation.wrapping_add(1);
        self.timer_generation
    }
}

impl Application for FocusQueue {
    type Message = Message;

    fn update(
        &mut self,
        message: Self::Message,
        context: &mut UpdateContext<Self::Message>,
    ) -> Command<Self::Message> {
        match message {
            Message::DraftChanged(draft) => {
                self.draft = draft;
                context.invalidate(Invalidation::Layout);
                Command::none()
            }
            Message::AddTask => {
                if self.add_task() {
                    context.invalidate(Invalidation::Recompose);
                }
                Command::none()
            }
            Message::ToggleTask(id) => {
                if self.toggle_task(id) {
                    context.invalidate(Invalidation::Paint);
                }
                Command::none()
            }
            Message::DeleteTask(id) => {
                if self.remove_task(id) {
                    context.invalidate(Invalidation::Recompose);
                }
                Command::none()
            }
            Message::Scrolled(delta) => {
                let next = self
                    .scroll_y
                    .saturating_add(delta.y)
                    .clamp(0, self.max_scroll_y());
                if next != self.scroll_y {
                    self.scroll_y = next;
                    context.invalidate(Invalidation::Layout);
                }
                Command::none()
            }
            Message::StartTimer => {
                if self.timer_running {
                    return Command::none();
                }
                if self.remaining_seconds == 0 {
                    self.remaining_seconds = self.focus_seconds;
                    self.refresh_labels();
                }
                self.timer_running = true;
                let generation = self.next_timer_generation();
                context.invalidate(Invalidation::Layout);
                Command::after(TICK_INTERVAL, Message::TimerTick(generation))
            }
            Message::PauseTimer => {
                if self.timer_running {
                    self.timer_running = false;
                    self.next_timer_generation();
                    context.invalidate(Invalidation::Layout);
                }
                Command::none()
            }
            Message::ResetTimer => {
                self.timer_running = false;
                self.next_timer_generation();
                self.remaining_seconds = self.focus_seconds;
                self.refresh_labels();
                context.invalidate(Invalidation::Layout);
                Command::none()
            }
            Message::TimerTick(generation) => {
                if !self.timer_running || generation != self.timer_generation {
                    return Command::none();
                }
                self.remaining_seconds = self.remaining_seconds.saturating_sub(1);
                self.refresh_labels();
                if self.remaining_seconds == 0 {
                    self.timer_running = false;
                    context.invalidate(Invalidation::Layout);
                    Command::none()
                } else {
                    context.invalidate(Invalidation::Paint);
                    Command::after(TICK_INTERVAL, Message::TimerTick(generation))
                }
            }
            Message::Quit => Command::quit(),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let border_style = Style::new().foreground(Color::BrightCyan);
        let button_style = Style::new().foreground(Color::BrightYellow);
        let full_width = Dimension::percent(100);

        let input = text_input(&self.draft, Message::DraftChanged)
            .on_submit(|| Message::AddTask)
            .style(Style::new().foreground(Color::BrightWhite))
            .layout(LayoutStyle::new().flex(1, 1))
            .focus_order(0)
            .build()
            .key("new-task");
        let add = button("Add", || Message::AddTask)
            .label_style(button_style)
            .focus_order(1)
            .build()
            .key("add-task");
        let input_row = row_with_gap([input, add], 1).layout(LayoutStyle {
            width: full_width,
            direction: FlexDirection::Row,
            gap: 1,
            ..LayoutStyle::default()
        });
        let input_panel = Block::new(input_row)
            .title("New task")
            .border_style(border_style)
            .layout(LayoutStyle::new().size(full_width, Dimension::cells(3)))
            .build();

        let task_rows = self.tasks.iter().map(|task| {
            let id = task.id;
            let marker = if task.completed { "[x]" } else { "[ ]" };
            let title_style = if task.completed {
                Style::new()
                    .foreground(Color::BrightBlack)
                    .add_modifiers(Modifier::DIM | Modifier::CROSSED_OUT)
            } else {
                Style::new().foreground(Color::White)
            };
            let row_layout = LayoutStyle {
                width: full_width,
                height: Dimension::cells(1),
                direction: FlexDirection::Row,
                flex_shrink: 0,
                gap: 1,
                ..LayoutStyle::default()
            };
            let row = row_with_gap(
                [
                    button(marker, move || Message::ToggleTask(id))
                        .label_style(button_style)
                        .build()
                        .key(format!("task-{id}-toggle")),
                    text(&task.title)
                        .style(title_style)
                        .layout(LayoutStyle::new().flex(1, 1)),
                    button("Delete", move || Message::DeleteTask(id))
                        .label_style(Style::new().foreground(Color::BrightRed))
                        .build()
                        .key(format!("task-{id}-delete")),
                ],
                1,
            )
            .layout(row_layout);
            (id, row)
        });
        let queue_content = if self.tasks.is_empty() {
            text("No tasks yet. Type one above and press Enter.")
                .style(Style::new().foreground(Color::BrightBlack))
        } else {
            let content_height = u16::try_from(self.tasks.len()).unwrap_or(u16::MAX);
            list(task_rows).layout(LayoutStyle {
                width: full_width,
                height: Dimension::cells(content_height),
                direction: FlexDirection::Column,
                flex_shrink: 0,
                ..LayoutStyle::default()
            })
        };
        let queue = scroll_view(Point::new(0, self.scroll_y), queue_content)
            .on_scroll(Message::Scrolled)
            .layout(LayoutStyle::new().size(full_width, Dimension::percent(100)))
            .build();
        let queue_panel = Block::new(queue)
            .title("Focus queue")
            .border_style(border_style)
            .layout(LayoutStyle {
                width: full_width,
                min_height: Dimension::cells(5),
                flex_grow: 1,
                flex_shrink: 1,
                ..LayoutStyle::default()
            })
            .build();

        let timer_style = Style::new()
            .foreground(if self.timer_running {
                Color::BrightGreen
            } else {
                Color::BrightYellow
            })
            .add_modifiers(Modifier::BOLD);
        let timer_action = if self.timer_running {
            button("Pause", || Message::PauseTimer)
        } else {
            button("Start", || Message::StartTimer)
        }
        .label_style(button_style)
        .build()
        .key("timer-toggle");
        let timer_panel = Block::new(row_with_gap(
            [
                text(&self.timer_label).style(timer_style),
                text(&self.summary_label),
                flexible_spacer(),
                timer_action,
                button("Reset", || Message::ResetTimer)
                    .label_style(button_style)
                    .build()
                    .key("timer-reset"),
            ],
            2,
        ))
        .title("Focus timer")
        .border_style(border_style)
        .layout(LayoutStyle::new().size(full_width, Dimension::cells(3)))
        .build();

        let footer = row_with_gap(
            [
                text("Tab focus | Enter activate | Wheel scroll")
                    .style(Style::new().foreground(Color::BrightBlack)),
                flexible_spacer(),
                button("Quit", || Message::Quit)
                    .label_style(Style::new().foreground(Color::BrightMagenta))
                    .build()
                    .key("quit"),
            ],
            1,
        )
        .layout(LayoutStyle::new().size(full_width, Dimension::cells(1)));

        column_with_gap([input_panel, queue_panel, timer_panel, footer], 1).layout(LayoutStyle {
            width: full_width,
            height: Dimension::percent(100),
            direction: FlexDirection::Column,
            gap: 1,
            ..LayoutStyle::default()
        })
    }
}
