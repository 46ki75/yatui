//! Production ANSI serialization measurements for matched application frames.

use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};

use arborui::{
    Capabilities, CrosstermBackend as ArboruiCrosstermBackend, FramePatch, TerminalBackend,
};
use arborui_comparison_collection_lab_ratatui::{
    ComparisonAction, RatatuiCollectionLab, RatatuiLogLab, RatatuiTableLab, draw_test_log_terminal,
    draw_test_table_terminal, draw_test_terminal,
};
use arborui_example_collection_lab::{
    CollectionLab, CollectionMode, LogAction, LogLab, Message, TableAction, TableLab,
};
use arborui_test::{Size as ArboruiSize, TestApp};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend as RatatuiCrosstermBackend, TestBackend},
    buffer::Buffer,
    layout::Rect,
};

const ITEM_COUNT: usize = 100_000;
const WIDTH: u16 = 48;
const HEIGHT: u16 = 12;
const RESIZED_HEIGHT: u16 = 16;

#[derive(Clone, Copy, Debug)]
enum Scenario {
    InitialRender,
    PageDown,
    End,
    Resize,
    Selection,
    Reverse,
    UnchangedRedraw,
}

#[derive(Clone, Copy, Debug)]
enum TableScenario {
    InitialRender,
    PageDown,
    Resize,
    Selection,
    VisibleUpdate,
    OffscreenUpdate,
}

#[derive(Clone, Copy, Debug)]
enum LogScenario {
    InitialRender,
    PageUp,
    Resize,
    AppendFollowing,
    AppendPaused,
}

impl LogScenario {
    const ALL: [Self; 5] = [
        Self::InitialRender,
        Self::PageUp,
        Self::Resize,
        Self::AppendFollowing,
        Self::AppendPaused,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::InitialRender => "initial-render",
            Self::PageUp => "page-up",
            Self::Resize => "resize",
            Self::AppendFollowing => "append-following",
            Self::AppendPaused => "append-paused",
        }
    }
}

impl TableScenario {
    const ALL: [Self; 6] = [
        Self::InitialRender,
        Self::PageDown,
        Self::Resize,
        Self::Selection,
        Self::VisibleUpdate,
        Self::OffscreenUpdate,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::InitialRender => "initial-render",
            Self::PageDown => "page-down",
            Self::Resize => "resize",
            Self::Selection => "selection",
            Self::VisibleUpdate => "visible-update",
            Self::OffscreenUpdate => "offscreen-update",
        }
    }
}

impl Scenario {
    const ALL: [Self; 7] = [
        Self::InitialRender,
        Self::PageDown,
        Self::End,
        Self::Resize,
        Self::Selection,
        Self::Reverse,
        Self::UnchangedRedraw,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::InitialRender => "initial-render",
            Self::PageDown => "page-down",
            Self::End => "end",
            Self::Resize => "resize",
            Self::Selection => "selection",
            Self::Reverse => "reverse",
            Self::UnchangedRedraw => "unchanged-redraw",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OutputMetrics {
    bytes: usize,
    writes: usize,
    flushes: usize,
}

#[derive(Clone, Default)]
struct CountingWriter {
    metrics: Arc<Mutex<OutputMetrics>>,
}

impl CountingWriter {
    fn metrics(&self) -> OutputMetrics {
        *self.metrics.lock().expect("metrics lock must be available")
    }
}

impl Write for CountingWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let mut metrics = self
            .metrics
            .lock()
            .map_err(|_| io::Error::other("metrics lock poisoned"))?;
        metrics.bytes = metrics.bytes.saturating_add(buffer.len());
        metrics.writes = metrics.writes.saturating_add(1);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut metrics = self
            .metrics
            .lock()
            .map_err(|_| io::Error::other("metrics lock poisoned"))?;
        metrics.flushes = metrics.flushes.saturating_add(1);
        Ok(())
    }
}

#[test]
fn reports_production_ansi_output_metrics() {
    println!("| Mode | Scenario | Framework | ANSI bytes | Writer calls | Flushes |");
    println!("| --- | --- | --- | ---: | ---: | ---: |");

    for mode in [CollectionMode::Fixed, CollectionMode::Variable] {
        for scenario in Scenario::ALL {
            let arborui = arborui_metrics(mode, scenario);
            let ratatui = ratatui_metrics(mode, scenario);
            print_metrics(mode, scenario, "ArborUI", arborui);
            print_metrics(mode, scenario, "Ratatui", ratatui);

            if matches!(scenario, Scenario::UnchangedRedraw) {
                assert_eq!(arborui, OutputMetrics::default());
                assert!(ratatui.bytes > 0);
                assert_eq!(ratatui.flushes, 1);
            } else {
                assert!(arborui.bytes > 0);
                assert!(ratatui.bytes > 0);
                assert_eq!(arborui.flushes, 1);
                let expected_ratatui_flushes = if matches!(scenario, Scenario::Resize) {
                    2
                } else {
                    1
                };
                assert_eq!(ratatui.flushes, expected_ratatui_flushes);
            }
        }
    }
}

#[test]
fn reports_table_ansi_output_metrics() {
    println!("| Scenario | Framework | ANSI bytes | Writer calls | Flushes |");
    println!("| --- | --- | ---: | ---: | ---: |");

    for scenario in TableScenario::ALL {
        let arborui = arborui_table_metrics(scenario);
        let ratatui = ratatui_table_metrics(scenario);
        println!(
            "| {} | ArborUI | {} | {} | {} |",
            scenario.name(),
            arborui.bytes,
            arborui.writes,
            arborui.flushes
        );
        println!(
            "| {} | Ratatui | {} | {} | {} |",
            scenario.name(),
            ratatui.bytes,
            ratatui.writes,
            ratatui.flushes
        );

        if matches!(scenario, TableScenario::OffscreenUpdate) {
            assert_eq!(arborui, OutputMetrics::default());
        } else {
            assert!(arborui.bytes > 0);
            assert!(ratatui.bytes > 0);
        }
    }
}

#[test]
fn reports_scrolling_log_ansi_output_metrics() {
    println!("| Scenario | Framework | ANSI bytes | Writer calls | Flushes |");
    println!("| --- | --- | ---: | ---: | ---: |");

    for scenario in LogScenario::ALL {
        let arborui = arborui_log_metrics(scenario);
        let ratatui = ratatui_log_metrics(scenario);
        println!(
            "| {} | ArborUI | {} | {} | {} |",
            scenario.name(),
            arborui.bytes,
            arborui.writes,
            arborui.flushes
        );
        println!(
            "| {} | Ratatui | {} | {} | {} |",
            scenario.name(),
            ratatui.bytes,
            ratatui.writes,
            ratatui.flushes
        );

        if matches!(scenario, LogScenario::AppendPaused) {
            assert_eq!(arborui, OutputMetrics::default());
        } else {
            assert!(arborui.bytes > 0);
            assert!(ratatui.bytes > 0);
        }
    }
}

fn arborui_metrics(mode: CollectionMode, scenario: Scenario) -> OutputMetrics {
    let mut application = TestApp::new(
        CollectionLab::new(mode, ITEM_COUNT, usize::from(HEIGHT - 4)),
        ArboruiSize::new(WIDTH, HEIGHT),
    );
    let patch = match scenario {
        Scenario::InitialRender => application.last_frame_patch().cloned(),
        Scenario::PageDown => patch_after(&mut application, |app| {
            app.send(Message::PageDown);
        }),
        Scenario::End => patch_after(&mut application, |app| {
            app.send(Message::End);
        }),
        Scenario::Resize => patch_after(&mut application, |app| {
            app.resize(ArboruiSize::new(WIDTH, RESIZED_HEIGHT));
        }),
        Scenario::Selection => {
            application.send(Message::Down);
            application.send(Message::SelectActive);
            application.send(Message::Home);
            patch_after(&mut application, |app| {
                app.send(Message::SelectActive);
            })
        }
        Scenario::Reverse => patch_after(&mut application, |app| {
            app.send(Message::Reverse);
        }),
        Scenario::UnchangedRedraw => patch_after(&mut application, |app| {
            app.send(Message::Home);
        }),
    };
    patch.map_or_else(OutputMetrics::default, |patch| serialize_arborui(&patch))
}

fn patch_after(
    application: &mut TestApp<CollectionLab>,
    action: impl FnOnce(&mut TestApp<CollectionLab>),
) -> Option<FramePatch> {
    let previous_count = application.frame_patches().len();
    action(application);
    application.frame_patches().get(previous_count).cloned()
}

fn serialize_arborui(patch: &FramePatch) -> OutputMetrics {
    let writer = CountingWriter::default();
    let metrics = writer.clone();
    let mut backend = ArboruiCrosstermBackend::new(writer)
        .expect("ArborUI production backend must open")
        .with_capabilities(Capabilities::default());
    backend
        .write_patch(patch)
        .expect("ArborUI patch must serialize");
    metrics.metrics()
}

fn arborui_table_metrics(scenario: TableScenario) -> OutputMetrics {
    let mut application = TestApp::new(
        TableLab::new(ITEM_COUNT, WIDTH, HEIGHT),
        ArboruiSize::new(WIDTH, HEIGHT),
    );
    let patch = match scenario {
        TableScenario::InitialRender => application.last_frame_patch().cloned(),
        TableScenario::PageDown => patch_after_table(&mut application, TableAction::PageDown),
        TableScenario::Resize => {
            let previous_count = application.frame_patches().len();
            application.resize(ArboruiSize::new(WIDTH, RESIZED_HEIGHT));
            application.frame_patches().get(previous_count).cloned()
        }
        TableScenario::Selection => patch_after_table(&mut application, TableAction::SelectActive),
        TableScenario::VisibleUpdate => patch_after_table(
            &mut application,
            TableAction::BackgroundUpdate {
                key: 0,
                revision: 1,
            },
        ),
        TableScenario::OffscreenUpdate => patch_after_table(
            &mut application,
            TableAction::BackgroundUpdate {
                key: u64::try_from(ITEM_COUNT - 1).unwrap_or(u64::MAX),
                revision: 1,
            },
        ),
    };
    patch.map_or_else(OutputMetrics::default, |patch| serialize_arborui(&patch))
}

fn patch_after_table(
    application: &mut TestApp<TableLab>,
    action: TableAction,
) -> Option<FramePatch> {
    let previous_count = application.frame_patches().len();
    application.send(action);
    application.frame_patches().get(previous_count).cloned()
}

fn arborui_log_metrics(scenario: LogScenario) -> OutputMetrics {
    let mut application = TestApp::new(
        LogLab::new(ITEM_COUNT, ITEM_COUNT.saturating_mul(2), WIDTH, HEIGHT),
        ArboruiSize::new(WIDTH, HEIGHT),
    );
    if matches!(scenario, LogScenario::AppendPaused) {
        application.send(LogAction::PageUp);
    }
    let patch = match scenario {
        LogScenario::InitialRender => application.last_frame_patch().cloned(),
        LogScenario::PageUp => patch_after_log(&mut application, LogAction::PageUp),
        LogScenario::Resize => {
            let previous_count = application.frame_patches().len();
            application.resize(ArboruiSize::new(WIDTH, RESIZED_HEIGHT));
            application.frame_patches().get(previous_count).cloned()
        }
        LogScenario::AppendFollowing | LogScenario::AppendPaused => patch_after_log(
            &mut application,
            LogAction::Append {
                count: 1,
                generation: 1,
            },
        ),
    };
    patch.map_or_else(OutputMetrics::default, |patch| serialize_arborui(&patch))
}

fn patch_after_log(application: &mut TestApp<LogLab>, action: LogAction) -> Option<FramePatch> {
    let previous_count = application.frame_patches().len();
    application.send(action);
    application.frame_patches().get(previous_count).cloned()
}

fn ratatui_metrics(mode: CollectionMode, scenario: Scenario) -> OutputMetrics {
    let mut application = RatatuiCollectionLab::new(mode, ITEM_COUNT, WIDTH, HEIGHT);
    let mut terminal =
        Terminal::new(TestBackend::new(WIDTH, HEIGHT)).expect("test terminal must open");
    draw_test_terminal(&mut terminal, &mut application).expect("initial frame must draw");

    if matches!(scenario, Scenario::InitialRender) {
        let initial = terminal.backend().buffer().clone();
        let blank = Buffer::empty(Rect::new(0, 0, WIDTH, HEIGHT));
        return serialize_ratatui(&blank, &initial, false);
    }

    if matches!(scenario, Scenario::Selection) {
        for action in [
            ComparisonAction::Down,
            ComparisonAction::SelectActive,
            ComparisonAction::Home,
        ] {
            application.apply(action);
            draw_test_terminal(&mut terminal, &mut application)
                .expect("selection baseline must draw");
        }
    }
    let previous = terminal.backend().buffer().clone();

    match scenario {
        Scenario::PageDown => application.apply(ComparisonAction::PageDown),
        Scenario::End => application.apply(ComparisonAction::End),
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
        Scenario::InitialRender => unreachable!("initial render returned above"),
    }
    draw_test_terminal(&mut terminal, &mut application).expect("scenario frame must draw");
    let current = terminal.backend().buffer().clone();
    if matches!(scenario, Scenario::Resize) {
        let blank = Buffer::empty(Rect::new(0, 0, WIDTH, RESIZED_HEIGHT));
        serialize_ratatui(&blank, &current, true)
    } else {
        serialize_ratatui(&previous, &current, false)
    }
}

fn serialize_ratatui(previous: &Buffer, current: &Buffer, clear: bool) -> OutputMetrics {
    let writer = CountingWriter::default();
    let metrics = writer.clone();
    let mut backend = RatatuiCrosstermBackend::new(writer);
    if clear {
        backend.clear().expect("Ratatui backend must clear");
    }
    backend
        .draw(previous.diff_iter(current))
        .expect("Ratatui diff must serialize");
    Backend::flush(&mut backend).expect("Ratatui output must flush");
    metrics.metrics()
}

fn ratatui_table_metrics(scenario: TableScenario) -> OutputMetrics {
    let mut application = RatatuiTableLab::new(ITEM_COUNT, WIDTH, HEIGHT);
    let mut terminal =
        Terminal::new(TestBackend::new(WIDTH, HEIGHT)).expect("test terminal must open");
    draw_test_table_terminal(&mut terminal, &mut application)
        .expect("initial table frame must draw");

    if matches!(scenario, TableScenario::InitialRender) {
        let initial = terminal.backend().buffer().clone();
        let blank = Buffer::empty(Rect::new(0, 0, WIDTH, HEIGHT));
        return serialize_ratatui(&blank, &initial, false);
    }
    let previous = terminal.backend().buffer().clone();
    let action = match scenario {
        TableScenario::PageDown => TableAction::PageDown,
        TableScenario::Resize => {
            terminal.backend_mut().resize(WIDTH, RESIZED_HEIGHT);
            TableAction::Resize {
                width: WIDTH,
                height: RESIZED_HEIGHT,
            }
        }
        TableScenario::Selection => TableAction::SelectActive,
        TableScenario::VisibleUpdate => TableAction::BackgroundUpdate {
            key: 0,
            revision: 1,
        },
        TableScenario::OffscreenUpdate => TableAction::BackgroundUpdate {
            key: u64::try_from(ITEM_COUNT - 1).unwrap_or(u64::MAX),
            revision: 1,
        },
        TableScenario::InitialRender => unreachable!("initial render returned above"),
    };
    application.apply(action);
    draw_test_table_terminal(&mut terminal, &mut application).expect("table frame must draw");
    let current = terminal.backend().buffer().clone();
    if matches!(scenario, TableScenario::Resize) {
        let blank = Buffer::empty(Rect::new(0, 0, WIDTH, RESIZED_HEIGHT));
        serialize_ratatui(&blank, &current, true)
    } else {
        serialize_ratatui(&previous, &current, false)
    }
}

fn ratatui_log_metrics(scenario: LogScenario) -> OutputMetrics {
    let mut application =
        RatatuiLogLab::new(ITEM_COUNT, ITEM_COUNT.saturating_mul(2), WIDTH, HEIGHT);
    let mut terminal =
        Terminal::new(TestBackend::new(WIDTH, HEIGHT)).expect("test terminal must open");
    draw_test_log_terminal(&mut terminal, &mut application)
        .expect("initial scrolling-log frame must draw");

    if matches!(scenario, LogScenario::InitialRender) {
        let initial = terminal.backend().buffer().clone();
        let blank = Buffer::empty(Rect::new(0, 0, WIDTH, HEIGHT));
        return serialize_ratatui(&blank, &initial, false);
    }
    if matches!(scenario, LogScenario::AppendPaused) {
        application.apply(LogAction::PageUp);
        draw_test_log_terminal(&mut terminal, &mut application)
            .expect("paused scrolling-log baseline must draw");
    }
    let previous = terminal.backend().buffer().clone();
    let action = match scenario {
        LogScenario::PageUp => LogAction::PageUp,
        LogScenario::Resize => {
            terminal.backend_mut().resize(WIDTH, RESIZED_HEIGHT);
            LogAction::Resize {
                width: WIDTH,
                height: RESIZED_HEIGHT,
            }
        }
        LogScenario::AppendFollowing | LogScenario::AppendPaused => LogAction::Append {
            count: 1,
            generation: 1,
        },
        LogScenario::InitialRender => unreachable!("initial render returned above"),
    };
    application.apply(action);
    draw_test_log_terminal(&mut terminal, &mut application).expect("scrolling-log frame must draw");
    let current = terminal.backend().buffer().clone();
    if matches!(scenario, LogScenario::Resize) {
        let blank = Buffer::empty(Rect::new(0, 0, WIDTH, RESIZED_HEIGHT));
        serialize_ratatui(&blank, &current, true)
    } else {
        serialize_ratatui(&previous, &current, false)
    }
}

fn print_metrics(
    mode: CollectionMode,
    scenario: Scenario,
    framework: &str,
    metrics: OutputMetrics,
) {
    let mode = match mode {
        CollectionMode::Fixed => "Fixed",
        CollectionMode::Variable => "Variable",
    };
    println!(
        "| {mode} | {} | {framework} | {} | {} | {} |",
        scenario.name(),
        metrics.bytes,
        metrics.writes,
        metrics.flushes
    );
}
