//! Matched semantic and character-frame contracts.

use arborui_example_collection_lab::{
    CollectionLab, CollectionMode, Message, TableAction, TableLab,
};
use arborui_test::{Size, TestApp};
use ratatui::{Terminal, backend::TestBackend};

use arborui_comparison_collection_lab_ratatui::{
    ComparisonAction, CountingBackend, RatatuiCollectionLab, RatatuiTableLab, SemanticState,
    TableSemanticState, draw_terminal, draw_test_frame, draw_test_table_frame,
};

#[test]
fn canonical_table_trace_matches_semantics_and_characters() {
    let mut arborui = TestApp::new(TableLab::new(100_000, 48, 12), Size::new(48, 12));
    let mut ratatui = RatatuiTableLab::new(100_000, 48, 12);
    let mut terminal = Terminal::new(TestBackend::new(48, 12)).expect("test terminal must open");

    assert_table_frame(&arborui, &mut ratatui, &mut terminal);
    for action in [
        TableAction::PageDown,
        TableAction::Down,
        TableAction::SelectActive,
        TableAction::BackgroundUpdate {
            key: 8,
            revision: 1,
        },
        TableAction::BackgroundUpdate {
            key: 99_999,
            revision: 2,
        },
        TableAction::Resize {
            width: 34,
            height: 9,
        },
        TableAction::Resize {
            width: 64,
            height: 15,
        },
    ] {
        apply_arborui_table(&mut arborui, action);
        ratatui.apply(action);
        if let TableAction::Resize { width, height } = action {
            terminal.backend_mut().resize(width, height);
        }
        assert_table_frame(&arborui, &mut ratatui, &mut terminal);
    }

    assert_eq!(ratatui.semantic_state().selected_key, Some(8));
    assert_eq!(ratatui.semantic_state().generation, 2);
    assert_eq!(ratatui.model().rows()[8].revision(), 1);
    assert_eq!(ratatui.model().rows()[99_999].revision(), 2);
}

#[test]
fn table_construction_is_bounded_and_unicode_is_visible() {
    let arborui = TestApp::new(TableLab::new(1_000_000, 64, 12), Size::new(64, 12));
    let mut ratatui = RatatuiTableLab::new(1_000_000, 64, 12);
    let mut terminal = Terminal::new(TestBackend::new(64, 12)).expect("test terminal must open");
    let frame =
        draw_test_table_frame(&mut terminal, &mut ratatui).expect("Ratatui table frame must draw");

    assert_eq!(arborui.application().constructed_rows(), 9);
    assert_eq!(ratatui.semantic_state().constructed_rows, 9);
    assert_eq!(arborui_table_state(&arborui), ratatui.semantic_state());
    assert_eq!(arborui.frame().characters(), frame);
    assert!(frame.contains("München"));
}

#[test]
fn canonical_variable_trace_has_matching_semantics_and_characters() {
    let size = Size::new(38, 11);
    let mut arborui = TestApp::new(
        CollectionLab::new(CollectionMode::Variable, 100_000, 7),
        size,
    );
    let mut ratatui = RatatuiCollectionLab::new(CollectionMode::Variable, 100_000, 38, 11);
    let mut terminal = Terminal::new(TestBackend::new(38, 11)).expect("test terminal must open");

    for action in [
        ComparisonAction::PageDown,
        ComparisonAction::Down,
        ComparisonAction::SelectActive,
    ] {
        apply_arborui(&mut arborui, action);
        ratatui.apply(action);
    }
    let ratatui_frame =
        draw_test_frame(&mut terminal, &mut ratatui).expect("Ratatui frame must draw");

    assert_eq!(arborui_state(&arborui), ratatui.semantic_state());
    assert_eq!(arborui.frame().characters(), ratatui_frame);
}

#[test]
fn isolated_scenarios_have_matching_semantics_and_characters() {
    for mode in [CollectionMode::Fixed, CollectionMode::Variable] {
        for action in [
            ComparisonAction::PageDown,
            ComparisonAction::End,
            ComparisonAction::SelectActive,
            ComparisonAction::Reverse,
            ComparisonAction::Resize {
                width: 48,
                height: 16,
            },
        ] {
            assert_isolated_scenario(mode, action);
        }
    }
}

#[test]
fn construction_is_bounded_at_one_million_rows() {
    let size = Size::new(48, 12);
    let arborui = TestApp::new(
        CollectionLab::new(CollectionMode::Fixed, 1_000_000, 8),
        size,
    );
    let mut ratatui = RatatuiCollectionLab::new(CollectionMode::Fixed, 1_000_000, 48, 12);
    let mut terminal = Terminal::new(TestBackend::new(48, 12)).expect("test terminal must open");

    draw_test_frame(&mut terminal, &mut ratatui).expect("Ratatui frame must draw");

    assert_eq!(arborui.application().constructed_rows(), 10);
    assert_eq!(ratatui.semantic_state().constructed_rows, 10);
    assert_eq!(arborui_state(&arborui), ratatui.semantic_state());
}

#[test]
fn stable_identity_survives_unmount_and_reverse() {
    let size = Size::new(40, 10);
    let mut arborui = TestApp::new(CollectionLab::new(CollectionMode::Fixed, 100, 6), size);
    let mut ratatui = RatatuiCollectionLab::new(CollectionMode::Fixed, 100, 40, 10);
    let mut terminal = Terminal::new(TestBackend::new(40, 10)).expect("test terminal must open");

    for action in [
        ComparisonAction::SelectActive,
        ComparisonAction::End,
        ComparisonAction::Reverse,
        ComparisonAction::Home,
    ] {
        apply_arborui(&mut arborui, action);
        ratatui.apply(action);
    }
    draw_test_frame(&mut terminal, &mut ratatui).expect("Ratatui frame must draw");

    assert_eq!(arborui_state(&arborui), ratatui.semantic_state());
    assert_eq!(ratatui.semantic_state().active_key, Some(99));
    assert_eq!(ratatui.semantic_state().selected_key, Some(0));
}

#[test]
fn unchanged_redraw_has_no_logical_output_and_idle_does_no_work() {
    let mut arborui = TestApp::new(
        CollectionLab::new(CollectionMode::Fixed, 1_000_000, 8),
        Size::new(48, 12),
    );
    let mut application = RatatuiCollectionLab::new(CollectionMode::Fixed, 1_000_000, 48, 12);
    let mut terminal =
        Terminal::new(CountingBackend::new(48, 12)).expect("counting terminal must open");
    draw_terminal(&mut terminal, &mut application).expect("initial frame must draw");
    terminal.backend_mut().reset_counts();
    let patch_count = arborui.frame_patches().len();

    arborui.send(Message::Home);
    application.apply(ComparisonAction::Home);
    draw_terminal(&mut terminal, &mut application).expect("unchanged frame must draw");

    assert_eq!(arborui.frame_patches().len(), patch_count);
    assert_eq!(terminal.backend().changed_cells(), 0);
    assert_eq!(terminal.backend().draws(), 1);
    assert_eq!(terminal.backend().flushes(), 1);
    terminal.backend_mut().reset_counts();

    assert_eq!(terminal.backend().changed_cells(), 0);
    assert_eq!(terminal.backend().draws(), 0);
    assert_eq!(terminal.backend().flushes(), 0);
}

#[test]
fn one_row_navigation_reports_logical_output_work() {
    let mut arborui = TestApp::new(
        CollectionLab::new(CollectionMode::Fixed, 1_000_000, 8),
        Size::new(48, 12),
    );
    let mut ratatui = RatatuiCollectionLab::new(CollectionMode::Fixed, 1_000_000, 48, 12);
    let mut terminal =
        Terminal::new(CountingBackend::new(48, 12)).expect("counting terminal must open");
    draw_terminal(&mut terminal, &mut ratatui).expect("initial frame must draw");
    terminal.backend_mut().reset_counts();

    let patch_count = arborui.frame_patches().len();
    arborui.send(Message::Down);
    ratatui.apply(ComparisonAction::Down);
    draw_terminal(&mut terminal, &mut ratatui).expect("updated frame must draw");

    let patch = &arborui.frame_patches()[patch_count];
    let arborui_cells = patch.runs.iter().map(|run| run.cells.len()).sum::<usize>();
    assert!(arborui_cells > 0);
    assert!(terminal.backend().changed_cells() > 0);
    assert_eq!(terminal.backend().draws(), 1);
    assert_eq!(terminal.backend().flushes(), 1);
}

#[test]
fn resize_recomputes_the_same_window() {
    let mut arborui = TestApp::new(
        CollectionLab::new(CollectionMode::Fixed, 10_000, 4),
        Size::new(40, 8),
    );
    let mut ratatui = RatatuiCollectionLab::new(CollectionMode::Fixed, 10_000, 40, 8);
    let mut terminal = Terminal::new(TestBackend::new(40, 8)).expect("test terminal must open");
    draw_test_frame(&mut terminal, &mut ratatui).expect("initial frame must draw");

    apply_arborui(
        &mut arborui,
        ComparisonAction::Resize {
            width: 40,
            height: 14,
        },
    );
    ratatui.apply(ComparisonAction::Resize {
        width: 40,
        height: 14,
    });
    terminal.backend_mut().resize(40, 14);
    let ratatui_frame =
        draw_test_frame(&mut terminal, &mut ratatui).expect("resized frame must draw");

    assert_eq!(arborui_state(&arborui), ratatui.semantic_state());
    assert_eq!(arborui.frame().characters(), ratatui_frame);
}

fn assert_isolated_scenario(mode: CollectionMode, action: ComparisonAction) {
    let initial_size = Size::new(48, 12);
    let mut arborui = TestApp::new(CollectionLab::new(mode, 100_000, 8), initial_size);
    let mut ratatui = RatatuiCollectionLab::new(mode, 100_000, 48, 12);
    let mut terminal = Terminal::new(TestBackend::new(48, 12)).expect("test terminal must open");
    let initial_ratatui =
        draw_test_frame(&mut terminal, &mut ratatui).expect("initial frame must draw");

    assert_eq!(arborui_state(&arborui), ratatui.semantic_state());
    assert_eq!(arborui.frame().characters(), initial_ratatui);

    apply_arborui(&mut arborui, action);
    ratatui.apply(action);
    if let ComparisonAction::Resize { width, height } = action {
        terminal.backend_mut().resize(width, height);
    }
    let ratatui_frame =
        draw_test_frame(&mut terminal, &mut ratatui).expect("scenario frame must draw");

    assert_eq!(arborui_state(&arborui), ratatui.semantic_state());
    assert_eq!(arborui.frame().characters(), ratatui_frame);
}

fn arborui_state(app: &TestApp<CollectionLab>) -> SemanticState {
    SemanticState {
        active_key: app.application().active_key(),
        selected_key: app.application().selected_key(),
        scroll_offset: app.application().scroll_offset(),
        viewport_height: app.application().viewport_height(),
        visible_range: app.application().visible_range(),
        constructed_rows: app.application().constructed_rows(),
    }
}

fn apply_arborui(app: &mut TestApp<CollectionLab>, action: ComparisonAction) {
    let message = match action {
        ComparisonAction::Up => Message::Up,
        ComparisonAction::Down => Message::Down,
        ComparisonAction::Home => Message::Home,
        ComparisonAction::End => Message::End,
        ComparisonAction::PageUp => Message::PageUp,
        ComparisonAction::PageDown => Message::PageDown,
        ComparisonAction::SelectActive => Message::SelectActive,
        ComparisonAction::ToggleMode => Message::ToggleMode,
        ComparisonAction::Reverse => Message::Reverse,
        ComparisonAction::Resize { width, height } => {
            app.resize(Size::new(width, height));
            return;
        }
    };
    app.send(message);
}

fn assert_table_frame(
    arborui: &TestApp<TableLab>,
    ratatui: &mut RatatuiTableLab,
    terminal: &mut Terminal<TestBackend>,
) {
    let frame = draw_test_table_frame(terminal, ratatui).expect("Ratatui table frame must draw");
    assert_eq!(arborui_table_state(arborui), ratatui.semantic_state());
    assert_eq!(arborui.frame().characters(), frame);
}

fn arborui_table_state(app: &TestApp<TableLab>) -> TableSemanticState {
    TableSemanticState {
        active_key: app.application().model().active_key(),
        selected_key: app.application().model().selected_key(),
        scroll_offset: app.application().model().scroll_offset(),
        viewport_height: app.application().model().viewport_height(),
        visible_range: app.application().model().visible_range(),
        constructed_rows: app.application().constructed_rows(),
        generation: app.application().model().generation(),
    }
}

fn apply_arborui_table(app: &mut TestApp<TableLab>, action: TableAction) {
    if let TableAction::Resize { width, height } = action {
        app.resize(Size::new(width, height));
    } else {
        app.send(action);
    }
}
