//! Downstream pilot tests using only the public application and test facades.

use std::time::Duration;

use arborui::{Point, TextBuffer};
use arborui_example_focus_queue::{FocusQueue, Message};
use arborui_test::{
    Key, KeyCode, KeyModifiers, MouseEvent, MouseEventKind, Size, TestApp, TestCellContent,
};

#[test]
fn first_typed_character_is_visible_in_the_input() {
    let mut app = TestApp::new(FocusQueue::default(), Size::new(60, 16));

    app.key(KeyCode::Character('m'));

    assert_eq!(app.application().draft(), "m");
    assert!(matches!(
        app.frame().cell(Point::new(1, 1)).map(|cell| &cell.content),
        Some(TestCellContent::Grapheme { text, .. }) if text.as_ref() == "m"
    ));
}

#[test]
fn tasks_are_added_and_completed_through_widgets() {
    let mut app = TestApp::new(FocusQueue::with_focus_seconds(60), Size::new(72, 18));

    assert_eq!(app.focused_key(), Some(Key::from("new-task")));
    app.paste("Write pilot notes");
    app.key(KeyCode::Enter);

    assert_eq!(app.application().task_count(), 1);
    assert_eq!(app.application().task_title(0), Some("Write pilot notes"));
    assert_eq!(app.application().draft(), "");
    assert!(app.frame().characters().contains("Write pilot notes"));

    app.key(KeyCode::Tab);
    app.key(KeyCode::Tab);
    assert_eq!(
        app.focused_key(),
        Some(Key::from("task-1-toggle")),
        "the task toggle follows the input and Add button"
    );
    app.key(KeyCode::Enter);

    assert_eq!(app.application().task_completed(0), Some(true));
    assert!(app.frame().characters().contains("[x] Write pilot notes"));
    assert!(app.frame().characters().contains("0 open / 1 complete"));

    app.key(KeyCode::Tab);
    assert_eq!(app.focused_key(), Some(Key::from("task-1-delete")));
    app.key(KeyCode::Enter);
    assert_eq!(app.application().task_count(), 0);
    assert!(app.frame().characters().contains("No tasks yet"));
}

#[test]
fn timer_pause_invalidates_an_already_scheduled_tick() {
    let mut app = TestApp::new(FocusQueue::with_focus_seconds(3), Size::new(60, 16));

    app.send(Message::StartTimer);
    app.advance(Duration::from_secs(1));
    assert_eq!(app.application().remaining_seconds(), 2);

    app.send(Message::PauseTimer);
    app.advance(Duration::from_secs(5));
    assert_eq!(app.application().remaining_seconds(), 2);

    app.send(Message::StartTimer);
    app.advance(Duration::from_secs(2));
    assert_eq!(app.application().remaining_seconds(), 1);
    assert!(app.application().timer_running());

    app.advance(Duration::from_secs(1));
    assert_eq!(app.application().remaining_seconds(), 0);
    assert!(!app.application().timer_running());
    assert!(app.frame().characters().contains("00:00"));
}

#[test]
fn wheel_scrolling_is_controlled_and_clamped_by_the_model() {
    let mut app = TestApp::new(FocusQueue::default(), Size::new(60, 16));
    for index in 1..=8 {
        app.send(Message::DraftChanged(TextBuffer::new(format!(
            "Task {index}"
        ))));
        app.send(Message::AddTask);
    }

    let scroll_down = MouseEvent {
        kind: MouseEventKind::ScrollDown,
        position: Point::new(2, 6),
        modifiers: KeyModifiers::NONE,
    };
    app.mouse(scroll_down);
    app.mouse(scroll_down);
    assert_eq!(app.application().scroll_y(), 2);

    for _ in 0..20 {
        app.mouse(scroll_down);
    }
    assert_eq!(app.application().scroll_y(), 7);
    assert!(
        app.frame().characters().contains("Task 8"),
        "{}",
        app.frame().characters()
    );
}
