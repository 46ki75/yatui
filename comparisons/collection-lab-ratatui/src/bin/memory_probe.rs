//! Isolated heap measurement process for one matched Collection Lab case.

use std::{error::Error, hint::black_box};

use arborui_comparison_collection_lab_ratatui::{
    ComparisonAction, RatatuiCollectionLab, RatatuiLogLab, RatatuiOverlayLab, RatatuiTableLab,
    draw_test_log_terminal, draw_test_overlay_terminal, draw_test_table_terminal,
    draw_test_terminal,
};
use arborui_example_collection_lab::{
    CollectionLab, CollectionMode, LogAction, LogLab, Message, OverlayAction, OverlayLab,
    TableAction, TableLab,
};
use arborui_test::{KeyCode, Size, TestApp};
use ratatui::{Terminal, backend::TestBackend};

const WIDTH: u16 = 48;
const HEIGHT: u16 = 12;
const RESIZED_HEIGHT: u16 = 16;
const OVERLAY_WIDTH: u16 = 40;
const OVERLAY_HEIGHT: u16 = 12;
const OVERLAY_RESIZED_WIDTH: u16 = 44;
const OVERLAY_RESIZED_HEIGHT: u16 = 14;

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

#[derive(Clone, Copy)]
enum Framework {
    Arborui,
    Ratatui,
}

#[derive(Clone, Copy)]
enum Workload {
    Collection(CollectionMode),
    Table,
    Log,
    Overlay,
}

#[derive(Clone, Copy)]
enum Scenario {
    Model,
    Cold,
    InitialRender,
    PageDown,
    Resize,
    Selection,
    Reverse,
    UnchangedRedraw,
    VisibleUpdate,
    OffscreenUpdate,
    PageUp,
    AppendFollowing,
    AppendPaused,
    Open,
    FocusNext,
    Cancel,
    Confirm,
    BackgroundActivation,
    ResizeOpen,
}

struct Metrics {
    total_blocks: u64,
    total_bytes: u64,
    max_blocks: usize,
    max_bytes: usize,
    curr_blocks: usize,
    curr_bytes: usize,
    end_blocks: usize,
    end_bytes: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = std::env::args().collect::<Vec<_>>();
    let [_, framework, workload, scenario, item_count] = arguments.as_slice() else {
        return Err(
            "usage: memory_probe <arborui|ratatui> <fixed|variable|table|log|overlay> <scenario> <items>"
                .into(),
        );
    };
    let framework = parse_framework(framework)?;
    let workload = parse_workload(workload)?;
    let scenario = parse_scenario(scenario)?;
    let item_count = item_count.parse::<usize>()?;
    if matches!(workload, Workload::Overlay) && item_count != 1 {
        return Err("overlay items must be 1 (the workload has no item count)".into());
    }

    let metrics = match framework {
        Framework::Arborui => measure_arborui(workload, scenario, item_count),
        Framework::Ratatui => measure_ratatui(workload, scenario, item_count),
    };
    println!(
        "total_blocks={} total_bytes={} max_blocks={} max_bytes={} curr_blocks={} curr_bytes={} end_blocks={} end_bytes={}",
        metrics.total_blocks,
        metrics.total_bytes,
        metrics.max_blocks,
        metrics.max_bytes,
        metrics.curr_blocks,
        metrics.curr_bytes,
        metrics.end_blocks,
        metrics.end_bytes
    );
    Ok(())
}

fn measure_arborui(workload: Workload, scenario: Scenario, item_count: usize) -> Metrics {
    match workload {
        Workload::Collection(mode) => measure_arborui_collection(mode, scenario, item_count),
        Workload::Table => measure_arborui_table(scenario, item_count),
        Workload::Log => measure_arborui_log(scenario, item_count),
        Workload::Overlay => measure_arborui_overlay(scenario),
    }
}

fn measure_arborui_collection(
    mode: CollectionMode,
    scenario: Scenario,
    item_count: usize,
) -> Metrics {
    match scenario {
        Scenario::Model => {
            measure(|| CollectionLab::new(mode, item_count, viewport_height(HEIGHT)))
        }
        Scenario::Cold => measure(|| {
            let application = TestApp::new(
                CollectionLab::new(mode, item_count, viewport_height(HEIGHT)),
                base_size(),
            );
            assert_bounded(application.application().constructed_rows());
            application
        }),
        Scenario::InitialRender => {
            let model = CollectionLab::new(mode, item_count, viewport_height(HEIGHT));
            measure(move || {
                let application = TestApp::new(model, base_size());
                assert_bounded(application.application().constructed_rows());
                application
            })
        }
        Scenario::PageDown
        | Scenario::Resize
        | Scenario::Selection
        | Scenario::Reverse
        | Scenario::UnchangedRedraw => {
            let mut application = TestApp::new(
                CollectionLab::new(mode, item_count, viewport_height(HEIGHT)),
                base_size(),
            );
            if matches!(scenario, Scenario::Selection) {
                application.send(Message::Down);
                application.send(Message::SelectActive);
                application.send(Message::Home);
            }
            measure(move || {
                match scenario {
                    Scenario::PageDown => {
                        application.send(Message::PageDown);
                    }
                    Scenario::Resize => {
                        application.resize(Size::new(WIDTH, RESIZED_HEIGHT));
                    }
                    Scenario::Selection => {
                        application.send(Message::SelectActive);
                    }
                    Scenario::Reverse => {
                        application.send(Message::Reverse);
                    }
                    Scenario::UnchangedRedraw => {
                        application.send(Message::Home);
                    }
                    Scenario::Model | Scenario::Cold | Scenario::InitialRender => {
                        unreachable!("setup scenarios are handled separately")
                    }
                    Scenario::VisibleUpdate
                    | Scenario::OffscreenUpdate
                    | Scenario::PageUp
                    | Scenario::AppendFollowing
                    | Scenario::AppendPaused
                    | Scenario::Open
                    | Scenario::FocusNext
                    | Scenario::Cancel
                    | Scenario::Confirm
                    | Scenario::BackgroundActivation
                    | Scenario::ResizeOpen => {
                        unreachable!("other workload scenarios are handled separately")
                    }
                }
                assert_bounded(application.application().constructed_rows());
                black_box(application)
            })
        }
        Scenario::VisibleUpdate
        | Scenario::OffscreenUpdate
        | Scenario::PageUp
        | Scenario::AppendFollowing
        | Scenario::AppendPaused
        | Scenario::Open
        | Scenario::FocusNext
        | Scenario::Cancel
        | Scenario::Confirm
        | Scenario::BackgroundActivation
        | Scenario::ResizeOpen => {
            panic!("scenario belongs to another workload")
        }
    }
}

fn measure_arborui_table(scenario: Scenario, item_count: usize) -> Metrics {
    match scenario {
        Scenario::Model => measure(|| TableLab::new(item_count, WIDTH, HEIGHT)),
        Scenario::Cold => measure(|| {
            let application = TestApp::new(TableLab::new(item_count, WIDTH, HEIGHT), base_size());
            assert_bounded(application.application().constructed_rows());
            application
        }),
        Scenario::InitialRender => {
            let model = TableLab::new(item_count, WIDTH, HEIGHT);
            measure(move || {
                let application = TestApp::new(model, base_size());
                assert_bounded(application.application().constructed_rows());
                application
            })
        }
        Scenario::PageDown
        | Scenario::Resize
        | Scenario::Selection
        | Scenario::UnchangedRedraw
        | Scenario::VisibleUpdate
        | Scenario::OffscreenUpdate => {
            let mut application =
                TestApp::new(TableLab::new(item_count, WIDTH, HEIGHT), base_size());
            measure(move || {
                match scenario {
                    Scenario::PageDown => {
                        application.send(TableAction::PageDown);
                    }
                    Scenario::Resize => {
                        application.resize(Size::new(WIDTH, RESIZED_HEIGHT));
                    }
                    Scenario::Selection => {
                        application.send(TableAction::SelectActive);
                    }
                    Scenario::UnchangedRedraw => {
                        application.send(TableAction::Home);
                    }
                    Scenario::VisibleUpdate => {
                        application.send(TableAction::BackgroundUpdate {
                            key: 0,
                            revision: 1,
                        });
                    }
                    Scenario::OffscreenUpdate => {
                        application.send(TableAction::BackgroundUpdate {
                            key: u64::try_from(item_count.saturating_sub(1)).unwrap_or(u64::MAX),
                            revision: 1,
                        });
                    }
                    Scenario::Model
                    | Scenario::Cold
                    | Scenario::InitialRender
                    | Scenario::Reverse
                    | Scenario::PageUp
                    | Scenario::AppendFollowing
                    | Scenario::AppendPaused
                    | Scenario::Open
                    | Scenario::FocusNext
                    | Scenario::Cancel
                    | Scenario::Confirm
                    | Scenario::BackgroundActivation
                    | Scenario::ResizeOpen => unreachable!("scenario is handled separately"),
                }
                assert_bounded(application.application().constructed_rows());
                black_box(application)
            })
        }
        Scenario::Reverse
        | Scenario::PageUp
        | Scenario::AppendFollowing
        | Scenario::AppendPaused
        | Scenario::Open
        | Scenario::FocusNext
        | Scenario::Cancel
        | Scenario::Confirm
        | Scenario::BackgroundActivation
        | Scenario::ResizeOpen => panic!("scenario is not a table scenario"),
    }
}

fn measure_arborui_log(scenario: Scenario, item_count: usize) -> Metrics {
    let history_limit = item_count.saturating_mul(2).max(1);
    match scenario {
        Scenario::Model => measure(|| LogLab::new(item_count, history_limit, WIDTH, HEIGHT)),
        Scenario::Cold => measure(|| {
            let application = TestApp::new(
                LogLab::new(item_count, history_limit, WIDTH, HEIGHT),
                base_size(),
            );
            assert_bounded(application.application().constructed_rows());
            application
        }),
        Scenario::InitialRender => {
            let model = LogLab::new(item_count, history_limit, WIDTH, HEIGHT);
            measure(move || {
                let application = TestApp::new(model, base_size());
                assert_bounded(application.application().constructed_rows());
                application
            })
        }
        Scenario::PageUp
        | Scenario::Resize
        | Scenario::AppendFollowing
        | Scenario::AppendPaused
        | Scenario::UnchangedRedraw => {
            let mut application = TestApp::new(
                LogLab::new(item_count, history_limit, WIDTH, HEIGHT),
                base_size(),
            );
            if matches!(scenario, Scenario::AppendPaused) {
                application.send(LogAction::PageUp);
            }
            measure(move || {
                match scenario {
                    Scenario::PageUp => {
                        application.send(LogAction::PageUp);
                    }
                    Scenario::Resize => {
                        application.resize(Size::new(WIDTH, RESIZED_HEIGHT));
                    }
                    Scenario::AppendFollowing | Scenario::AppendPaused => {
                        application.send(LogAction::Append {
                            count: 1,
                            generation: 1,
                        });
                    }
                    Scenario::UnchangedRedraw => {
                        application.send(LogAction::End);
                    }
                    Scenario::Model
                    | Scenario::Cold
                    | Scenario::InitialRender
                    | Scenario::PageDown
                    | Scenario::Selection
                    | Scenario::Reverse
                    | Scenario::VisibleUpdate
                    | Scenario::OffscreenUpdate
                    | Scenario::Open
                    | Scenario::FocusNext
                    | Scenario::Cancel
                    | Scenario::Confirm
                    | Scenario::BackgroundActivation
                    | Scenario::ResizeOpen => {
                        unreachable!("scenario is handled separately")
                    }
                }
                assert_bounded(application.application().constructed_rows());
                black_box(application)
            })
        }
        Scenario::PageDown
        | Scenario::Selection
        | Scenario::Reverse
        | Scenario::VisibleUpdate
        | Scenario::OffscreenUpdate
        | Scenario::Open
        | Scenario::FocusNext
        | Scenario::Cancel
        | Scenario::Confirm
        | Scenario::BackgroundActivation
        | Scenario::ResizeOpen => panic!("scenario is not a scrolling-log scenario"),
    }
}

fn measure_arborui_overlay(scenario: Scenario) -> Metrics {
    match scenario {
        Scenario::Model => measure(|| OverlayLab::new(OVERLAY_WIDTH, OVERLAY_HEIGHT)),
        Scenario::Cold => measure(|| {
            let application = TestApp::new(
                OverlayLab::new(OVERLAY_WIDTH, OVERLAY_HEIGHT),
                overlay_size(),
            );
            assert_arborui_overlay_bounded(&application);
            application
        }),
        Scenario::InitialRender => {
            let model = OverlayLab::new(OVERLAY_WIDTH, OVERLAY_HEIGHT);
            measure(move || {
                let application = TestApp::new(model, overlay_size());
                assert_arborui_overlay_bounded(&application);
                application
            })
        }
        Scenario::Open
        | Scenario::FocusNext
        | Scenario::Cancel
        | Scenario::Confirm
        | Scenario::BackgroundActivation
        | Scenario::ResizeOpen => {
            let mut application = arborui_overlay_fixture(scenario);
            measure(move || {
                match scenario {
                    Scenario::Open | Scenario::Confirm | Scenario::BackgroundActivation => {
                        application.key(KeyCode::Enter);
                    }
                    Scenario::FocusNext => {
                        application.key(KeyCode::Tab);
                    }
                    Scenario::Cancel => {
                        application.key(KeyCode::Escape);
                    }
                    Scenario::ResizeOpen => {
                        application
                            .resize(Size::new(OVERLAY_RESIZED_WIDTH, OVERLAY_RESIZED_HEIGHT));
                    }
                    _ => unreachable!("setup scenarios are handled separately"),
                }
                assert_arborui_overlay_bounded(&application);
                black_box(application)
            })
        }
        _ => panic!("scenario is not an overlay scenario"),
    }
}

fn measure_ratatui(workload: Workload, scenario: Scenario, item_count: usize) -> Metrics {
    match workload {
        Workload::Collection(mode) => measure_ratatui_collection(mode, scenario, item_count),
        Workload::Table => measure_ratatui_table(scenario, item_count),
        Workload::Log => measure_ratatui_log(scenario, item_count),
        Workload::Overlay => measure_ratatui_overlay(scenario),
    }
}

fn measure_ratatui_collection(
    mode: CollectionMode,
    scenario: Scenario,
    item_count: usize,
) -> Metrics {
    match scenario {
        Scenario::Model => measure(|| RatatuiCollectionLab::new(mode, item_count, WIDTH, HEIGHT)),
        Scenario::Cold => measure(|| {
            let mut application = RatatuiCollectionLab::new(mode, item_count, WIDTH, HEIGHT);
            let mut terminal =
                Terminal::new(TestBackend::new(WIDTH, HEIGHT)).expect("test terminal must open");
            draw_test_terminal(&mut terminal, &mut application).expect("initial frame must draw");
            assert_bounded(application.semantic_state().constructed_rows);
            (application, terminal)
        }),
        Scenario::InitialRender => {
            let mut application = RatatuiCollectionLab::new(mode, item_count, WIDTH, HEIGHT);
            measure(move || {
                let mut terminal = Terminal::new(TestBackend::new(WIDTH, HEIGHT))
                    .expect("test terminal must open");
                draw_test_terminal(&mut terminal, &mut application)
                    .expect("initial frame must draw");
                assert_bounded(application.semantic_state().constructed_rows);
                (application, terminal)
            })
        }
        Scenario::PageDown
        | Scenario::Resize
        | Scenario::Selection
        | Scenario::Reverse
        | Scenario::UnchangedRedraw => {
            let (mut application, mut terminal) = ratatui_fixture(mode, item_count);
            if matches!(scenario, Scenario::Selection) {
                for action in [
                    ComparisonAction::Down,
                    ComparisonAction::SelectActive,
                    ComparisonAction::Home,
                ] {
                    application.apply(action);
                    draw_test_terminal(&mut terminal, &mut application)
                        .expect("selection fixture must draw");
                }
            }
            measure(move || {
                match scenario {
                    Scenario::PageDown => application.apply(ComparisonAction::PageDown),
                    Scenario::Resize => {
                        terminal.backend_mut().resize(WIDTH, RESIZED_HEIGHT);
                        application.apply(ComparisonAction::Resize {
                            width: WIDTH,
                            height: RESIZED_HEIGHT,
                        });
                    }
                    Scenario::Selection => application.apply(ComparisonAction::SelectActive),
                    Scenario::Reverse => application.apply(ComparisonAction::Reverse),
                    Scenario::UnchangedRedraw => application.apply(ComparisonAction::Home),
                    Scenario::Model | Scenario::Cold | Scenario::InitialRender => {
                        unreachable!("setup scenarios are handled separately")
                    }
                    Scenario::VisibleUpdate
                    | Scenario::OffscreenUpdate
                    | Scenario::PageUp
                    | Scenario::AppendFollowing
                    | Scenario::AppendPaused
                    | Scenario::Open
                    | Scenario::FocusNext
                    | Scenario::Cancel
                    | Scenario::Confirm
                    | Scenario::BackgroundActivation
                    | Scenario::ResizeOpen => {
                        unreachable!("other workload scenarios are handled separately")
                    }
                }
                draw_test_terminal(&mut terminal, &mut application).expect("frame must draw");
                assert_bounded(application.semantic_state().constructed_rows);
                black_box((application, terminal))
            })
        }
        Scenario::VisibleUpdate
        | Scenario::OffscreenUpdate
        | Scenario::PageUp
        | Scenario::AppendFollowing
        | Scenario::AppendPaused
        | Scenario::Open
        | Scenario::FocusNext
        | Scenario::Cancel
        | Scenario::Confirm
        | Scenario::BackgroundActivation
        | Scenario::ResizeOpen => {
            panic!("scenario belongs to another workload")
        }
    }
}

fn measure_ratatui_table(scenario: Scenario, item_count: usize) -> Metrics {
    match scenario {
        Scenario::Model => measure(|| RatatuiTableLab::new(item_count, WIDTH, HEIGHT)),
        Scenario::Cold => measure(|| {
            let mut application = RatatuiTableLab::new(item_count, WIDTH, HEIGHT);
            let mut terminal =
                Terminal::new(TestBackend::new(WIDTH, HEIGHT)).expect("test terminal must open");
            draw_test_table_terminal(&mut terminal, &mut application)
                .expect("initial table frame must draw");
            assert_bounded(application.semantic_state().constructed_rows);
            (application, terminal)
        }),
        Scenario::InitialRender => {
            let mut application = RatatuiTableLab::new(item_count, WIDTH, HEIGHT);
            measure(move || {
                let mut terminal = Terminal::new(TestBackend::new(WIDTH, HEIGHT))
                    .expect("test terminal must open");
                draw_test_table_terminal(&mut terminal, &mut application)
                    .expect("initial table frame must draw");
                assert_bounded(application.semantic_state().constructed_rows);
                (application, terminal)
            })
        }
        Scenario::PageDown
        | Scenario::Resize
        | Scenario::Selection
        | Scenario::UnchangedRedraw
        | Scenario::VisibleUpdate
        | Scenario::OffscreenUpdate => {
            let mut application = RatatuiTableLab::new(item_count, WIDTH, HEIGHT);
            let mut terminal =
                Terminal::new(TestBackend::new(WIDTH, HEIGHT)).expect("test terminal must open");
            draw_test_table_terminal(&mut terminal, &mut application)
                .expect("initial table frame must draw");
            measure(move || {
                match scenario {
                    Scenario::PageDown => application.apply(TableAction::PageDown),
                    Scenario::Resize => {
                        terminal.backend_mut().resize(WIDTH, RESIZED_HEIGHT);
                        application.apply(TableAction::Resize {
                            width: WIDTH,
                            height: RESIZED_HEIGHT,
                        });
                    }
                    Scenario::Selection => application.apply(TableAction::SelectActive),
                    Scenario::UnchangedRedraw => application.apply(TableAction::Home),
                    Scenario::VisibleUpdate => application.apply(TableAction::BackgroundUpdate {
                        key: 0,
                        revision: 1,
                    }),
                    Scenario::OffscreenUpdate => {
                        application.apply(TableAction::BackgroundUpdate {
                            key: u64::try_from(item_count.saturating_sub(1)).unwrap_or(u64::MAX),
                            revision: 1,
                        });
                    }
                    Scenario::Model
                    | Scenario::Cold
                    | Scenario::InitialRender
                    | Scenario::Reverse
                    | Scenario::PageUp
                    | Scenario::AppendFollowing
                    | Scenario::AppendPaused
                    | Scenario::Open
                    | Scenario::FocusNext
                    | Scenario::Cancel
                    | Scenario::Confirm
                    | Scenario::BackgroundActivation
                    | Scenario::ResizeOpen => unreachable!("scenario is handled separately"),
                }
                draw_test_table_terminal(&mut terminal, &mut application)
                    .expect("table frame must draw");
                assert_bounded(application.semantic_state().constructed_rows);
                black_box((application, terminal))
            })
        }
        Scenario::Reverse
        | Scenario::PageUp
        | Scenario::AppendFollowing
        | Scenario::AppendPaused
        | Scenario::Open
        | Scenario::FocusNext
        | Scenario::Cancel
        | Scenario::Confirm
        | Scenario::BackgroundActivation
        | Scenario::ResizeOpen => panic!("scenario is not a table scenario"),
    }
}

fn measure_ratatui_log(scenario: Scenario, item_count: usize) -> Metrics {
    let history_limit = item_count.saturating_mul(2).max(1);
    match scenario {
        Scenario::Model => measure(|| RatatuiLogLab::new(item_count, history_limit, WIDTH, HEIGHT)),
        Scenario::Cold => measure(|| {
            let mut application = RatatuiLogLab::new(item_count, history_limit, WIDTH, HEIGHT);
            let mut terminal =
                Terminal::new(TestBackend::new(WIDTH, HEIGHT)).expect("test terminal must open");
            draw_test_log_terminal(&mut terminal, &mut application)
                .expect("initial scrolling-log frame must draw");
            assert_bounded(application.semantic_state().constructed_rows);
            (application, terminal)
        }),
        Scenario::InitialRender => {
            let mut application = RatatuiLogLab::new(item_count, history_limit, WIDTH, HEIGHT);
            measure(move || {
                let mut terminal = Terminal::new(TestBackend::new(WIDTH, HEIGHT))
                    .expect("test terminal must open");
                draw_test_log_terminal(&mut terminal, &mut application)
                    .expect("initial scrolling-log frame must draw");
                assert_bounded(application.semantic_state().constructed_rows);
                (application, terminal)
            })
        }
        Scenario::PageUp
        | Scenario::Resize
        | Scenario::AppendFollowing
        | Scenario::AppendPaused
        | Scenario::UnchangedRedraw => {
            let mut application = RatatuiLogLab::new(item_count, history_limit, WIDTH, HEIGHT);
            let mut terminal =
                Terminal::new(TestBackend::new(WIDTH, HEIGHT)).expect("test terminal must open");
            draw_test_log_terminal(&mut terminal, &mut application)
                .expect("initial scrolling-log frame must draw");
            if matches!(scenario, Scenario::AppendPaused) {
                application.apply(LogAction::PageUp);
                draw_test_log_terminal(&mut terminal, &mut application)
                    .expect("paused scrolling-log baseline must draw");
            }
            measure(move || {
                match scenario {
                    Scenario::PageUp => application.apply(LogAction::PageUp),
                    Scenario::Resize => {
                        terminal.backend_mut().resize(WIDTH, RESIZED_HEIGHT);
                        application.apply(LogAction::Resize {
                            width: WIDTH,
                            height: RESIZED_HEIGHT,
                        });
                    }
                    Scenario::AppendFollowing | Scenario::AppendPaused => {
                        application.apply(LogAction::Append {
                            count: 1,
                            generation: 1,
                        });
                    }
                    Scenario::UnchangedRedraw => application.apply(LogAction::End),
                    Scenario::Model
                    | Scenario::Cold
                    | Scenario::InitialRender
                    | Scenario::PageDown
                    | Scenario::Selection
                    | Scenario::Reverse
                    | Scenario::VisibleUpdate
                    | Scenario::OffscreenUpdate
                    | Scenario::Open
                    | Scenario::FocusNext
                    | Scenario::Cancel
                    | Scenario::Confirm
                    | Scenario::BackgroundActivation
                    | Scenario::ResizeOpen => {
                        unreachable!("scenario is handled separately")
                    }
                }
                draw_test_log_terminal(&mut terminal, &mut application)
                    .expect("scrolling-log frame must draw");
                assert_bounded(application.semantic_state().constructed_rows);
                black_box((application, terminal))
            })
        }
        Scenario::PageDown
        | Scenario::Selection
        | Scenario::Reverse
        | Scenario::VisibleUpdate
        | Scenario::OffscreenUpdate
        | Scenario::Open
        | Scenario::FocusNext
        | Scenario::Cancel
        | Scenario::Confirm
        | Scenario::BackgroundActivation
        | Scenario::ResizeOpen => panic!("scenario is not a scrolling-log scenario"),
    }
}

fn measure_ratatui_overlay(scenario: Scenario) -> Metrics {
    match scenario {
        Scenario::Model => measure(|| RatatuiOverlayLab::new(OVERLAY_WIDTH, OVERLAY_HEIGHT)),
        Scenario::Cold => measure(|| {
            let application = RatatuiOverlayLab::new(OVERLAY_WIDTH, OVERLAY_HEIGHT);
            let mut terminal = Terminal::new(TestBackend::new(OVERLAY_WIDTH, OVERLAY_HEIGHT))
                .expect("test terminal must open");
            draw_test_overlay_terminal(&mut terminal, &application)
                .expect("initial overlay frame must draw");
            assert_ratatui_overlay_bounded(&application, &terminal);
            (application, terminal)
        }),
        Scenario::InitialRender => {
            let application = RatatuiOverlayLab::new(OVERLAY_WIDTH, OVERLAY_HEIGHT);
            measure(move || {
                let mut terminal = Terminal::new(TestBackend::new(OVERLAY_WIDTH, OVERLAY_HEIGHT))
                    .expect("test terminal must open");
                draw_test_overlay_terminal(&mut terminal, &application)
                    .expect("initial overlay frame must draw");
                assert_ratatui_overlay_bounded(&application, &terminal);
                (application, terminal)
            })
        }
        Scenario::Open
        | Scenario::FocusNext
        | Scenario::Cancel
        | Scenario::Confirm
        | Scenario::BackgroundActivation
        | Scenario::ResizeOpen => {
            let (mut application, mut terminal) = ratatui_overlay_fixture(scenario);
            measure(move || {
                match scenario {
                    Scenario::Open => {
                        application.apply(OverlayAction::Open);
                    }
                    Scenario::FocusNext => application.focus_next(),
                    Scenario::Cancel => {
                        application.apply(OverlayAction::Cancel);
                    }
                    Scenario::Confirm => {
                        application.apply(OverlayAction::Confirm);
                    }
                    Scenario::BackgroundActivation => {
                        application.apply(OverlayAction::ActivateBackground);
                    }
                    Scenario::ResizeOpen => {
                        terminal
                            .backend_mut()
                            .resize(OVERLAY_RESIZED_WIDTH, OVERLAY_RESIZED_HEIGHT);
                        application.apply(OverlayAction::Resize {
                            width: OVERLAY_RESIZED_WIDTH,
                            height: OVERLAY_RESIZED_HEIGHT,
                        });
                    }
                    _ => unreachable!("setup scenarios are handled separately"),
                }
                draw_test_overlay_terminal(&mut terminal, &application)
                    .expect("overlay frame must draw");
                assert_ratatui_overlay_bounded(&application, &terminal);
                black_box((application, terminal))
            })
        }
        _ => panic!("scenario is not an overlay scenario"),
    }
}

fn measure<T>(operation: impl FnOnce() -> T) -> Metrics {
    let profiler = dhat::Profiler::builder()
        .testing()
        .trim_backtraces(Some(4))
        .build();
    let value = operation();
    let live = dhat::HeapStats::get();
    drop(value);
    let end = dhat::HeapStats::get();
    drop(profiler);
    Metrics {
        total_blocks: live.total_blocks,
        total_bytes: live.total_bytes,
        max_blocks: live.max_blocks,
        max_bytes: live.max_bytes,
        curr_blocks: live.curr_blocks,
        curr_bytes: live.curr_bytes,
        end_blocks: end.curr_blocks,
        end_bytes: end.curr_bytes,
    }
}

fn ratatui_fixture(
    mode: CollectionMode,
    item_count: usize,
) -> (RatatuiCollectionLab, Terminal<TestBackend>) {
    let mut application = RatatuiCollectionLab::new(mode, item_count, WIDTH, HEIGHT);
    let mut terminal =
        Terminal::new(TestBackend::new(WIDTH, HEIGHT)).expect("test terminal must open");
    draw_test_terminal(&mut terminal, &mut application).expect("initial frame must draw");
    (application, terminal)
}

fn arborui_overlay_fixture(scenario: Scenario) -> TestApp<OverlayLab> {
    let mut application = TestApp::new(
        OverlayLab::new(OVERLAY_WIDTH, OVERLAY_HEIGHT),
        overlay_size(),
    );
    match scenario {
        Scenario::FocusNext | Scenario::Cancel | Scenario::Confirm | Scenario::ResizeOpen => {
            application.key(KeyCode::Enter);
        }
        Scenario::BackgroundActivation => {
            application.key(KeyCode::Tab);
        }
        Scenario::Open => {}
        _ => unreachable!("only overlay actions use an overlay fixture"),
    }
    assert_arborui_overlay_bounded(&application);
    application
}

fn ratatui_overlay_fixture(scenario: Scenario) -> (RatatuiOverlayLab, Terminal<TestBackend>) {
    let mut application = RatatuiOverlayLab::new(OVERLAY_WIDTH, OVERLAY_HEIGHT);
    let mut terminal = Terminal::new(TestBackend::new(OVERLAY_WIDTH, OVERLAY_HEIGHT))
        .expect("test terminal must open");
    draw_test_overlay_terminal(&mut terminal, &application)
        .expect("initial overlay frame must draw");
    match scenario {
        Scenario::FocusNext | Scenario::Cancel | Scenario::Confirm | Scenario::ResizeOpen => {
            application.apply(OverlayAction::Open);
            draw_test_overlay_terminal(&mut terminal, &application)
                .expect("open overlay baseline must draw");
        }
        Scenario::BackgroundActivation => {
            application.focus_next();
            draw_test_overlay_terminal(&mut terminal, &application)
                .expect("focused background baseline must draw");
        }
        Scenario::Open => {}
        _ => unreachable!("only overlay actions use an overlay fixture"),
    }
    assert_ratatui_overlay_bounded(&application, &terminal);
    (application, terminal)
}

fn assert_arborui_overlay_bounded(application: &TestApp<OverlayLab>) {
    let frame = application.frame();
    let (width, height) = application.application().model().terminal_size();
    assert_eq!(frame.size(), Size::new(width, height));
    assert_eq!(
        frame.cells().len(),
        usize::from(width) * usize::from(height)
    );
    assert!(frame.cells().len() <= overlay_max_cells());
}

fn assert_ratatui_overlay_bounded(
    application: &RatatuiOverlayLab,
    terminal: &Terminal<TestBackend>,
) {
    let buffer = terminal.backend().buffer();
    let (width, height) = application.terminal_size();
    assert_eq!((buffer.area.width, buffer.area.height), (width, height));
    assert_eq!(
        buffer.content().len(),
        usize::from(width) * usize::from(height)
    );
    assert!(buffer.content().len() <= overlay_max_cells());
}

const fn overlay_max_cells() -> usize {
    OVERLAY_RESIZED_WIDTH as usize * OVERLAY_RESIZED_HEIGHT as usize
}

fn assert_bounded(constructed_rows: usize) {
    assert!(constructed_rows > 0);
    assert!(constructed_rows <= usize::from(RESIZED_HEIGHT));
}

fn parse_framework(value: &str) -> Result<Framework, Box<dyn Error>> {
    match value {
        "arborui" => Ok(Framework::Arborui),
        "ratatui" => Ok(Framework::Ratatui),
        _ => Err(format!("unknown framework: {value}").into()),
    }
}

fn parse_workload(value: &str) -> Result<Workload, Box<dyn Error>> {
    match value {
        "fixed" => Ok(Workload::Collection(CollectionMode::Fixed)),
        "variable" => Ok(Workload::Collection(CollectionMode::Variable)),
        "table" => Ok(Workload::Table),
        "log" => Ok(Workload::Log),
        "overlay" => Ok(Workload::Overlay),
        _ => Err(format!("unknown workload: {value}").into()),
    }
}

fn parse_scenario(value: &str) -> Result<Scenario, Box<dyn Error>> {
    match value {
        "model" => Ok(Scenario::Model),
        "cold" => Ok(Scenario::Cold),
        "initial-render" => Ok(Scenario::InitialRender),
        "page-down" => Ok(Scenario::PageDown),
        "resize" => Ok(Scenario::Resize),
        "selection" => Ok(Scenario::Selection),
        "reverse" => Ok(Scenario::Reverse),
        "unchanged-redraw" => Ok(Scenario::UnchangedRedraw),
        "visible-update" => Ok(Scenario::VisibleUpdate),
        "offscreen-update" => Ok(Scenario::OffscreenUpdate),
        "page-up" => Ok(Scenario::PageUp),
        "append-following" => Ok(Scenario::AppendFollowing),
        "append-paused" => Ok(Scenario::AppendPaused),
        "open" => Ok(Scenario::Open),
        "focus-next" => Ok(Scenario::FocusNext),
        "cancel" => Ok(Scenario::Cancel),
        "confirm" => Ok(Scenario::Confirm),
        "background-activation" => Ok(Scenario::BackgroundActivation),
        "resize-open" => Ok(Scenario::ResizeOpen),
        _ => Err(format!("unknown scenario: {value}").into()),
    }
}

const fn base_size() -> Size {
    Size::new(WIDTH, HEIGHT)
}

const fn overlay_size() -> Size {
    Size::new(OVERLAY_WIDTH, OVERLAY_HEIGHT)
}

fn viewport_height(terminal_height: u16) -> usize {
    usize::from(terminal_height.saturating_sub(4).max(1))
}
