//! Downstream pilot tests using only the public application and test facades.

use std::time::Duration;

use arborui::{CursorShape, CursorVisibility, Modifier, Point, TextBuffer};
use arborui_example_focus_queue::{FocusQueue, Message};
use arborui_test::{
    Key, KeyCode, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind, Size, TestApp,
    TestCellContent, TestFrame,
};

fn focused_label(frame: &TestFrame) -> String {
    frame
        .cells()
        .iter()
        .filter(|cell| cell.style.modifiers.contains(Modifier::REVERSED))
        .filter_map(|cell| match &cell.content {
            TestCellContent::Grapheme { text, .. } => Some(text.as_ref()),
            TestCellContent::Empty | TestCellContent::Continuation { .. } => None,
        })
        .collect()
}

#[test]
fn representative_frames_match_snapshots() {
    let empty = TestApp::new(FocusQueue::default(), Size::new(60, 16));
    assert_eq!(empty.application().task_count(), 0);
    insta::assert_snapshot!("focus_queue_empty", empty.frame());

    let mut task = TestApp::new(FocusQueue::default(), Size::new(60, 16));
    task.paste("Write pilot notes");
    task.key(KeyCode::Enter);
    assert_eq!(task.application().task_count(), 1);
    insta::assert_snapshot!("focus_queue_task_added", task.frame());

    task.key(KeyCode::Tab);
    task.key(KeyCode::Tab);
    task.key(KeyCode::Enter);
    assert_eq!(task.application().task_completed(0), Some(true));
    insta::assert_snapshot!("focus_queue_task_completed", task.frame());

    task.key(KeyCode::Tab);
    task.key(KeyCode::Enter);
    assert_eq!(task.application().editing_task_id(), Some(1));
    assert_eq!(task.focused_key(), Some(Key::from("edit-title")));
    insta::assert_snapshot!("focus_queue_edit_dialog", task.frame());

    let mut scrolled = TestApp::new(FocusQueue::default(), Size::new(60, 16));
    for index in 1..=8 {
        scrolled.send(Message::DraftChanged(TextBuffer::new(format!(
            "Task {index}"
        ))));
        scrolled.send(Message::AddTask);
    }
    let scroll_down = MouseEvent {
        kind: MouseEventKind::ScrollDown,
        position: Point::new(2, 6),
        modifiers: KeyModifiers::NONE,
    };
    scrolled.mouse(scroll_down);
    scrolled.mouse(scroll_down);
    assert_eq!(scrolled.application().scroll_y(), 2);
    insta::assert_snapshot!("focus_queue_scrolled", scrolled.frame());
}

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
fn tab_focus_is_visible_on_buttons() {
    let mut app = TestApp::new(FocusQueue::default(), Size::new(60, 16));

    let input_cursor = app.frame().cursor();
    assert_eq!(input_cursor.visibility, CursorVisibility::Visible);
    assert_eq!(input_cursor.shape, CursorShape::Bar);

    app.key(KeyCode::Tab);

    assert_eq!(app.focused_key(), Some(Key::from("add-task")));
    assert_eq!(app.frame().cursor().visibility, CursorVisibility::Hidden);
    assert_eq!(focused_label(app.frame()), "Add");

    app.key(KeyCode::Tab);

    assert_eq!(app.focused_key(), Some(Key::from("timer-toggle")));
    assert_eq!(focused_label(app.frame()), "Start");
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
    assert_eq!(app.focused_key(), Some(Key::from("task-1-edit")));
    app.key(KeyCode::Tab);
    assert_eq!(app.focused_key(), Some(Key::from("task-1-delete")));
    app.key(KeyCode::Enter);
    assert_eq!(app.application().task_count(), 0);
    assert!(app.frame().characters().contains("No tasks yet"));
}

#[test]
fn edit_dialog_traps_focus_cancels_and_restores_the_origin() {
    let mut app = TestApp::new(FocusQueue::default(), Size::new(72, 18));
    app.paste("Original task");
    app.key(KeyCode::Enter);
    app.key(KeyCode::Tab);
    app.key(KeyCode::Tab);
    app.key(KeyCode::Tab);
    assert_eq!(app.focused_key(), Some(Key::from("task-1-edit")));

    app.key(KeyCode::Enter);
    assert_eq!(app.application().editing_task_id(), Some(1));
    assert_eq!(app.focused_key(), Some(Key::from("edit-title")));

    app.key(KeyCode::Tab);
    assert_eq!(app.focused_key(), Some(Key::from("edit-completed")));
    app.key(KeyCode::Tab);
    assert_eq!(app.focused_key(), Some(Key::from("edit-save")));
    app.key(KeyCode::Tab);
    assert_eq!(app.focused_key(), Some(Key::from("edit-cancel")));
    app.key(KeyCode::Tab);
    assert_eq!(app.focused_key(), Some(Key::from("edit-title")));
    app.key_with(KeyCode::Tab, KeyModifiers::SHIFT, KeyEventKind::Press);
    assert_eq!(app.focused_key(), Some(Key::from("edit-cancel")));

    app.key(KeyCode::Escape);
    assert_eq!(app.application().editing_task_id(), None);
    assert_eq!(app.application().task_title(0), Some("Original task"));
    assert_eq!(app.focused_key(), Some(Key::from("task-1-edit")));
}

#[test]
fn edit_dialog_saves_unicode_and_checkbox_state() {
    let mut app = TestApp::new(FocusQueue::default(), Size::new(72, 18));
    app.paste("Original task");
    app.key(KeyCode::Enter);
    app.send(Message::OpenEdit(1));

    app.key_with(
        KeyCode::Character('a'),
        KeyModifiers::CONTROL,
        KeyEventKind::Press,
    );
    app.paste("a👩‍💻界");
    app.key(KeyCode::Home);
    app.key(KeyCode::Right);
    app.key(KeyCode::Delete);
    assert_eq!(app.application().edit_title(), Some("a界"));

    app.key(KeyCode::Tab);
    app.key(KeyCode::Enter);
    assert_eq!(app.application().edit_completed(), Some(true));
    app.key(KeyCode::Tab);
    app.key(KeyCode::Enter);

    assert_eq!(app.application().editing_task_id(), None);
    assert_eq!(app.application().task_title(0), Some("a界"));
    assert_eq!(app.application().task_completed(0), Some(true));
    insta::assert_snapshot!("focus_queue_unicode_saved", app.frame());
}

#[test]
fn edit_dialog_scrim_blocks_the_background() {
    let mut app = TestApp::new(FocusQueue::default(), Size::new(72, 18));
    app.paste("Keep this task");
    app.key(KeyCode::Enter);
    app.send(Message::OpenEdit(1));

    app.click(Point::new(70, 6));

    assert_eq!(app.application().editing_task_id(), Some(1));
    assert_eq!(app.application().task_count(), 1);
    assert_eq!(app.application().task_title(0), Some("Keep this task"));
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
