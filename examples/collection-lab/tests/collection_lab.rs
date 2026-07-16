//! Application evidence through the public ArborUI facades.

use arborui_example_collection_lab::{CollectionLab, CollectionMode, Message};
use arborui_test::{Key, KeyCode, Point, Size, TestApp};

#[test]
fn fixed_construction_and_tree_size_are_independent_of_item_count() {
    let small = TestApp::new(
        CollectionLab::new(CollectionMode::Fixed, 100, 8),
        Size::new(48, 12),
    );
    let large = TestApp::new(
        CollectionLab::new(CollectionMode::Fixed, 1_000_000, 8),
        Size::new(48, 12),
    );

    assert_eq!(small.application().constructed_rows(), 10);
    assert_eq!(large.application().constructed_rows(), 10);
    assert_eq!(small.ui_tree().len(), large.ui_tree().len());
    assert_eq!(
        large.focused_key(),
        Some(Key::from("collection")),
        "frame:\n{}",
        large.frame()
    );
}

#[test]
fn selection_and_focus_survive_unmounting_and_reorder_by_stable_key() {
    let mut app = TestApp::new(
        CollectionLab::new(CollectionMode::Fixed, 100, 6),
        Size::new(40, 10),
    );

    app.key(KeyCode::Enter);
    assert_eq!(app.application().selected_key(), Some(0));
    app.key(KeyCode::End);
    assert_eq!(app.application().active_key(), Some(99));
    assert_eq!(app.application().selected_key(), Some(0));
    assert!(app.application().scroll_offset() > 0);
    app.send(Message::Reverse);
    assert_eq!(app.application().active_key(), Some(99));
    assert_eq!(app.application().selected_key(), Some(0));
    assert_eq!(app.focused_key(), Some(Key::from("collection")));
    app.key(KeyCode::Home);
    assert_eq!(app.application().active_key(), Some(99));
    assert_eq!(app.application().selected_key(), Some(0));
}

#[test]
fn resize_recomputes_the_bounded_window() {
    let mut app = TestApp::new(
        CollectionLab::new(CollectionMode::Fixed, 10_000, 4),
        Size::new(40, 8),
    );
    assert_eq!(app.application().constructed_rows(), 6);

    app.resize(Size::new(40, 14));

    assert_eq!(app.application().viewport_height(), 10);
    assert_eq!(app.application().constructed_rows(), 12);
}

#[test]
fn pointer_selection_maps_a_visible_row_to_its_stable_key() {
    let mut app = TestApp::new(
        CollectionLab::new(CollectionMode::Fixed, 10_000, 6),
        Size::new(40, 10),
    );

    app.click(Point::new(3, 4));

    assert_eq!(app.application().selected_key(), Some(2));
    assert_eq!(app.application().active_key(), Some(2));
    assert_eq!(app.focused_key(), Some(Key::from("collection")));
}

#[test]
fn variable_rows_render_from_cached_measurements() {
    let mut app = TestApp::new(
        CollectionLab::new(CollectionMode::Variable, 100_000, 7),
        Size::new(38, 11),
    );

    app.key(KeyCode::PageDown);
    app.key(KeyCode::Down);
    app.key(KeyCode::Enter);

    assert_eq!(
        app.application().selected_key(),
        app.application().active_key()
    );
    assert!(app.application().constructed_rows() < 10);
    insta::assert_snapshot!("collection_lab_variable", app.frame());
}
