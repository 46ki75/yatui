//! ArborUI phase attribution for the matched Collection Lab scenarios.

use std::time::{Duration, Instant};

use arborui::{
    AppRunner, Application, Capabilities, HeadlessRenderOutcome, KeyAction, KeyModifiers,
    RenderTimings, Renderer, Size, UiEvent, UiKey, UiKeyEvent,
};
use arborui_comparison_collection_lab_ratatui::{
    OVERLAY_RESIZE_STORM, STANDARD_RESIZE_STORM, UNICODE_RESIZE_STORM, UNICODE_RESIZE_STORM_OFFSET,
};
use arborui_example_collection_lab::{
    CollectionLab, CollectionMode, LogAction, LogLab, Message, OverlayAction, OverlayLab,
    TableAction, TableLab, UnicodeAction, UnicodeLab,
};

const ITEM_COUNT: usize = 100_000;
const WIDTH: u16 = 48;
const HEIGHT: u16 = 12;
const RESIZED_HEIGHT: u16 = 16;
const OVERLAY_WIDTH: u16 = 40;
const OVERLAY_HEIGHT: u16 = 12;
const OVERLAY_RESIZED_WIDTH: u16 = 44;
const OVERLAY_RESIZED_HEIGHT: u16 = 14;
const UNICODE_WIDTH: u16 = 36;
const UNICODE_HEIGHT: u16 = 10;
const NARROW_UNICODE_WIDTH: u16 = 30;
const SHIFT_BOUNDARY_OFFSET: usize = 15;
const SAMPLES: u32 = 100;
const INITIAL_SAMPLES: u32 = 20;

#[derive(Clone, Copy)]
enum Scenario {
    PageDown,
    End,
    Resize,
    Selection,
    Reverse,
    UnchangedRedraw,
}

#[derive(Clone, Copy)]
enum TableScenario {
    PageDown,
    Resize,
    Selection,
    VisibleUpdate,
    OffscreenUpdate,
}

#[derive(Clone, Copy)]
enum LogScenario {
    PageUp,
    Resize,
    AppendFollowing,
    AppendPaused,
}

#[derive(Clone, Copy)]
enum OverlayScenario {
    Open,
    FocusNext,
    Cancel,
    Confirm,
    BackgroundActivation,
    ResizeOpen,
}

#[derive(Clone, Copy)]
enum UnicodeScenario {
    ShiftBoundary,
    ReplaceWide,
    ResizeNarrow,
}

impl UnicodeScenario {
    const ALL: [Self; 3] = [Self::ShiftBoundary, Self::ReplaceWide, Self::ResizeNarrow];

    const fn name(self) -> &'static str {
        match self {
            Self::ShiftBoundary => "shift-boundary",
            Self::ReplaceWide => "replace-wide",
            Self::ResizeNarrow => "resize-narrow",
        }
    }
}

impl OverlayScenario {
    const ALL: [Self; 6] = [
        Self::Open,
        Self::FocusNext,
        Self::Cancel,
        Self::Confirm,
        Self::BackgroundActivation,
        Self::ResizeOpen,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::FocusNext => "focus-next",
            Self::Cancel => "cancel",
            Self::Confirm => "confirm",
            Self::BackgroundActivation => "background-activation",
            Self::ResizeOpen => "resize-open",
        }
    }
}

impl LogScenario {
    const ALL: [Self; 4] = [
        Self::PageUp,
        Self::Resize,
        Self::AppendFollowing,
        Self::AppendPaused,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::PageUp => "page-up",
            Self::Resize => "resize",
            Self::AppendFollowing => "append-following",
            Self::AppendPaused => "append-paused",
        }
    }
}

impl TableScenario {
    const ALL: [Self; 5] = [
        Self::PageDown,
        Self::Resize,
        Self::Selection,
        Self::VisibleUpdate,
        Self::OffscreenUpdate,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::PageDown => "page-down",
            Self::Resize => "resize",
            Self::Selection => "selection",
            Self::VisibleUpdate => "visible-update",
            Self::OffscreenUpdate => "offscreen-update",
        }
    }
}

impl Scenario {
    const ALL: [Self; 6] = [
        Self::PageDown,
        Self::End,
        Self::Resize,
        Self::Selection,
        Self::Reverse,
        Self::UnchangedRedraw,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::PageDown => "page-down",
            Self::End => "end",
            Self::Resize => "resize",
            Self::Selection => "selection",
            Self::Reverse => "reverse",
            Self::UnchangedRedraw => "unchanged-redraw",
        }
    }
}

#[derive(Default)]
struct Totals {
    update: Duration,
    render: RenderTimings,
}

#[test]
#[ignore = "runs the optimized phase measurement matrix"]
fn reports_arborui_render_phase_metrics() {
    println!(
        "| Mode | Scenario | Update ns | View ns | Stage/reconcile ns | Layout ns | Paint ns | Diff ns | Commit ns | Post-commit ns | Render total ns |"
    );
    println!("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");

    for mode in [CollectionMode::Fixed, CollectionMode::Variable] {
        let initial = measure_initial_render(mode);
        print_totals(mode, "initial-render", initial, INITIAL_SAMPLES);
        for scenario in Scenario::ALL {
            let totals = measure_scenario(mode, scenario);
            print_totals(mode, scenario.name(), totals, SAMPLES);
        }
    }
}

#[test]
#[ignore = "runs the optimized table phase measurement matrix"]
fn reports_arborui_table_phase_metrics() {
    println!(
        "| Workload | Scenario | Update ns | View ns | Stage/reconcile ns | Layout ns | Paint ns | Diff ns | Commit ns | Post-commit ns | Render total ns |"
    );
    println!("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");

    print_table_totals(
        "initial-render",
        measure_table_initial_render(),
        INITIAL_SAMPLES,
    );
    for scenario in TableScenario::ALL {
        print_table_totals(scenario.name(), measure_table_scenario(scenario), SAMPLES);
    }
}

#[test]
#[ignore = "runs the optimized scrolling-log phase measurement matrix"]
fn reports_arborui_scrolling_log_phase_metrics() {
    println!(
        "| Workload | Scenario | Update ns | View ns | Stage/reconcile ns | Layout ns | Paint ns | Diff ns | Commit ns | Post-commit ns | Render total ns |"
    );
    println!("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");

    print_workload_totals(
        "Scrolling log",
        "initial-render",
        measure_log_initial_render(),
        INITIAL_SAMPLES,
    );
    for scenario in LogScenario::ALL {
        print_workload_totals(
            "Scrolling log",
            scenario.name(),
            measure_log_scenario(scenario),
            SAMPLES,
        );
    }
}

#[test]
#[ignore = "runs the optimized overlay phase measurement matrix"]
fn reports_arborui_overlay_phase_metrics() {
    println!(
        "| Workload | Scenario | Update ns | View ns | Stage/reconcile ns | Layout ns | Paint ns | Diff ns | Commit ns | Post-commit ns | Render total ns |"
    );
    println!("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");

    print_workload_totals(
        "Overlay",
        "initial-render",
        measure_overlay_initial_render(),
        INITIAL_SAMPLES,
    );
    for scenario in OverlayScenario::ALL {
        print_workload_totals(
            "Overlay",
            scenario.name(),
            measure_overlay_scenario(scenario),
            SAMPLES,
        );
    }
}

#[test]
#[ignore = "runs the optimized Unicode phase measurement matrix"]
fn reports_arborui_unicode_phase_metrics() {
    println!(
        "| Workload | Scenario | Update ns | View ns | Stage/reconcile ns | Layout ns | Paint ns | Diff ns | Commit ns | Post-commit ns | Render total ns |"
    );
    println!("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");

    print_workload_totals(
        "Unicode",
        "initial-render",
        measure_unicode_initial_render(),
        INITIAL_SAMPLES,
    );
    for scenario in UnicodeScenario::ALL {
        print_workload_totals(
            "Unicode",
            scenario.name(),
            measure_unicode_scenario(scenario),
            SAMPLES,
        );
    }
}

#[test]
#[ignore = "runs the optimized resize-storm phase measurement matrix"]
fn reports_arborui_resize_storm_phase_metrics() {
    println!(
        "| Workload | Scenario | Update ns | View ns | Stage/reconcile ns | Layout ns | Paint ns | Diff ns | Commit ns | Post-commit ns | Render total ns |"
    );
    println!("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");

    for mode in [CollectionMode::Fixed, CollectionMode::Variable] {
        let mut runner = new_runner(CollectionLab::new(
            mode,
            ITEM_COUNT,
            viewport_height(HEIGHT),
        ));
        prepare_resize_storm_runner(&mut runner);
        send(&mut runner, Message::SelectActive);
        render_reset(&mut runner);
        print_totals(
            mode,
            "resize-storm",
            measure_resize_storm(&mut runner, &STANDARD_RESIZE_STORM),
            SAMPLES,
        );
    }

    let mut table = new_table_runner(TableLab::new(ITEM_COUNT, WIDTH, HEIGHT));
    prepare_resize_storm_runner(&mut table);
    send_table(&mut table, TableAction::SelectActive);
    render_table_reset(&mut table);
    print_workload_totals(
        "Table",
        "resize-storm",
        measure_resize_storm(&mut table, &STANDARD_RESIZE_STORM),
        SAMPLES,
    );

    let mut log = new_log_runner(LogLab::new(
        ITEM_COUNT,
        ITEM_COUNT.saturating_mul(2),
        WIDTH,
        HEIGHT,
    ));
    prepare_resize_storm_runner(&mut log);
    send_log(&mut log, LogAction::PageUp);
    render_log_reset(&mut log);
    print_workload_totals(
        "Scrolling log paused",
        "resize-storm",
        measure_resize_storm(&mut log, &STANDARD_RESIZE_STORM),
        SAMPLES,
    );

    let mut overlay = new_overlay_runner();
    prepare_resize_storm_runner(&mut overlay);
    send_overlay(&mut overlay, OverlayAction::Open);
    render_overlay_reset(&mut overlay);
    dispatch_overlay_key(&mut overlay, UiKey::Tab);
    render_overlay_reset(&mut overlay);
    print_workload_totals(
        "Overlay open",
        "resize-storm",
        measure_resize_storm(&mut overlay, &OVERLAY_RESIZE_STORM),
        SAMPLES,
    );

    let mut unicode = new_unicode_runner();
    prepare_resize_storm_runner(&mut unicode);
    for _ in 0..UNICODE_RESIZE_STORM_OFFSET {
        send_unicode(&mut unicode, UnicodeAction::ShiftRight);
    }
    render_unicode_reset(&mut unicode);
    print_workload_totals(
        "Unicode",
        "resize-storm",
        measure_resize_storm(&mut unicode, &UNICODE_RESIZE_STORM),
        SAMPLES,
    );
}

fn prepare_resize_storm_runner<A: Application>(runner: &mut AppRunner<A>) {
    assert_eq!(
        runner
            .render_headless()
            .expect("initial resize-storm frame must render"),
        HeadlessRenderOutcome::Committed
    );
}

fn measure_resize_storm<A: Application>(runner: &mut AppRunner<A>, trace: &[(u16, u16)]) -> Totals {
    let mut totals = Totals::default();
    for _ in 0..SAMPLES {
        for &(width, height) in trace {
            let update_started = Instant::now();
            runner
                .dispatch_ui_event(UiEvent::Resize(Size::new(width, height)))
                .expect("resize-storm event must dispatch");
            runner.process_pending();
            totals.update = totals.update.saturating_add(update_started.elapsed());
            let rendered = runner
                .render_headless_timed()
                .expect("resize-storm frame must render");
            assert_eq!(rendered.outcome, HeadlessRenderOutcome::Committed);
            add_timings(
                &mut totals.render,
                rendered.timings.expect("render must include timings"),
            );
        }
    }
    totals
}

fn measure_unicode_initial_render() -> Totals {
    let mut totals = Totals::default();
    for _ in 0..INITIAL_SAMPLES {
        let mut runner = new_unicode_runner();
        let rendered = runner
            .render_headless_timed()
            .expect("initial Unicode frame must render");
        assert_eq!(rendered.outcome, HeadlessRenderOutcome::Committed);
        add_timings(
            &mut totals.render,
            rendered.timings.expect("render must include timings"),
        );
    }
    totals
}

fn measure_unicode_scenario(scenario: UnicodeScenario) -> Totals {
    let mut runner = new_unicode_runner();
    assert_eq!(
        runner
            .render_headless()
            .expect("initial Unicode frame must render"),
        HeadlessRenderOutcome::Committed
    );
    if matches!(scenario, UnicodeScenario::ShiftBoundary) {
        prepare_unicode_shift_boundary(&mut runner);
    }

    let mut totals = Totals::default();
    for _ in 0..SAMPLES {
        let update_started = Instant::now();
        apply_unicode_scenario(&mut runner, scenario);
        totals.update = totals.update.saturating_add(update_started.elapsed());
        let rendered = runner
            .render_headless_timed()
            .expect("Unicode scenario frame must render");
        assert_eq!(rendered.outcome, HeadlessRenderOutcome::Committed);
        add_timings(
            &mut totals.render,
            rendered.timings.expect("render must include timings"),
        );
        reset_unicode_scenario(&mut runner, scenario);
    }
    totals
}

fn apply_unicode_scenario(runner: &mut AppRunner<UnicodeLab>, scenario: UnicodeScenario) {
    match scenario {
        UnicodeScenario::ShiftBoundary => send_unicode(runner, UnicodeAction::ShiftRight),
        UnicodeScenario::ReplaceWide => send_unicode(runner, UnicodeAction::ReplaceWide),
        UnicodeScenario::ResizeNarrow => {
            resize_unicode(runner, NARROW_UNICODE_WIDTH, UNICODE_HEIGHT)
        }
    }
}

fn reset_unicode_scenario(runner: &mut AppRunner<UnicodeLab>, scenario: UnicodeScenario) {
    match scenario {
        UnicodeScenario::ShiftBoundary => {
            for _ in 0..=SHIFT_BOUNDARY_OFFSET {
                send_unicode(runner, UnicodeAction::ShiftLeft);
            }
            prepare_unicode_shift_boundary(runner);
        }
        UnicodeScenario::ReplaceWide => {
            send_unicode(runner, UnicodeAction::ReplaceWide);
            render_unicode_reset(runner);
        }
        UnicodeScenario::ResizeNarrow => {
            resize_unicode(runner, UNICODE_WIDTH, UNICODE_HEIGHT);
            render_unicode_reset(runner);
        }
    }
}

fn prepare_unicode_shift_boundary(runner: &mut AppRunner<UnicodeLab>) {
    for _ in 0..SHIFT_BOUNDARY_OFFSET {
        send_unicode(runner, UnicodeAction::ShiftRight);
    }
    render_unicode_reset(runner);
}

fn send_unicode(runner: &mut AppRunner<UnicodeLab>, action: UnicodeAction) {
    runner.enqueue(action);
    runner.process_pending();
}

fn resize_unicode(runner: &mut AppRunner<UnicodeLab>, width: u16, height: u16) {
    runner
        .dispatch_ui_event(UiEvent::Resize(Size::new(width, height)))
        .expect("Unicode resize event must dispatch");
    runner.process_pending();
}

fn render_unicode_reset(runner: &mut AppRunner<UnicodeLab>) {
    assert_eq!(
        runner
            .render_headless()
            .expect("Unicode reset frame must render"),
        HeadlessRenderOutcome::Committed
    );
}

fn new_unicode_runner() -> AppRunner<UnicodeLab> {
    let size = Size::new(UNICODE_WIDTH, UNICODE_HEIGHT);
    AppRunner::new(
        UnicodeLab::new(UNICODE_WIDTH, UNICODE_HEIGHT),
        size,
        Renderer::new(size, Capabilities::default().width_policy),
    )
}

fn measure_overlay_initial_render() -> Totals {
    let mut totals = Totals::default();
    for _ in 0..INITIAL_SAMPLES {
        let mut runner = new_overlay_runner();
        let rendered = runner
            .render_headless_timed()
            .expect("initial overlay frame must render");
        assert_eq!(rendered.outcome, HeadlessRenderOutcome::Committed);
        add_timings(
            &mut totals.render,
            rendered.timings.expect("render must include timings"),
        );
    }
    totals
}

fn measure_overlay_scenario(scenario: OverlayScenario) -> Totals {
    let mut runner = new_overlay_runner();
    assert_eq!(
        runner
            .render_headless()
            .expect("initial overlay frame must render"),
        HeadlessRenderOutcome::Committed
    );
    if matches!(
        scenario,
        OverlayScenario::FocusNext
            | OverlayScenario::Cancel
            | OverlayScenario::Confirm
            | OverlayScenario::ResizeOpen
    ) {
        send_overlay(&mut runner, OverlayAction::Open);
        render_overlay_reset(&mut runner);
    }

    let mut totals = Totals::default();
    for _ in 0..SAMPLES {
        let update_started = Instant::now();
        apply_overlay_scenario(&mut runner, scenario);
        totals.update = totals.update.saturating_add(update_started.elapsed());
        let rendered = runner
            .render_headless_timed()
            .expect("overlay scenario frame must render");
        assert_eq!(rendered.outcome, HeadlessRenderOutcome::Committed);
        add_timings(
            &mut totals.render,
            rendered.timings.expect("render must include timings"),
        );
        reset_overlay_scenario(&mut runner, scenario);
    }
    totals
}

fn apply_overlay_scenario(runner: &mut AppRunner<OverlayLab>, scenario: OverlayScenario) {
    match scenario {
        OverlayScenario::Open => send_overlay(runner, OverlayAction::Open),
        OverlayScenario::FocusNext => dispatch_overlay_key(runner, UiKey::Tab),
        OverlayScenario::Cancel => dispatch_overlay_key(runner, UiKey::Escape),
        OverlayScenario::Confirm => dispatch_overlay_key(runner, UiKey::Enter),
        OverlayScenario::BackgroundActivation => {
            send_overlay(runner, OverlayAction::ActivateBackground);
        }
        OverlayScenario::ResizeOpen => {
            resize_overlay(runner, OVERLAY_RESIZED_WIDTH, OVERLAY_RESIZED_HEIGHT)
        }
    }
}

fn reset_overlay_scenario(runner: &mut AppRunner<OverlayLab>, scenario: OverlayScenario) {
    match scenario {
        OverlayScenario::Open => {
            send_overlay(runner, OverlayAction::Cancel);
            render_overlay_reset(runner);
        }
        OverlayScenario::FocusNext => {
            dispatch_overlay_key(runner, UiKey::Tab);
            render_overlay_reset(runner);
        }
        OverlayScenario::Cancel | OverlayScenario::Confirm => {
            send_overlay(runner, OverlayAction::Open);
            render_overlay_reset(runner);
        }
        OverlayScenario::ResizeOpen => {
            resize_overlay(runner, OVERLAY_WIDTH, OVERLAY_HEIGHT);
            render_overlay_reset(runner);
        }
        OverlayScenario::BackgroundActivation => {}
    }
}

fn send_overlay(runner: &mut AppRunner<OverlayLab>, action: OverlayAction) {
    runner.enqueue(action);
    runner.process_pending();
}

fn dispatch_overlay_key(runner: &mut AppRunner<OverlayLab>, key: UiKey) {
    runner
        .dispatch_ui_event(UiEvent::Key(UiKeyEvent {
            key,
            modifiers: KeyModifiers::NONE,
            action: KeyAction::Press,
        }))
        .expect("overlay key event must dispatch");
    runner.process_pending();
}

fn resize_overlay(runner: &mut AppRunner<OverlayLab>, width: u16, height: u16) {
    runner
        .dispatch_ui_event(UiEvent::Resize(Size::new(width, height)))
        .expect("overlay resize event must dispatch");
    runner.process_pending();
}

fn render_overlay_reset(runner: &mut AppRunner<OverlayLab>) {
    assert_eq!(
        runner
            .render_headless()
            .expect("overlay reset frame must render"),
        HeadlessRenderOutcome::Committed
    );
}

fn new_overlay_runner() -> AppRunner<OverlayLab> {
    let size = Size::new(OVERLAY_WIDTH, OVERLAY_HEIGHT);
    AppRunner::new(
        OverlayLab::new(OVERLAY_WIDTH, OVERLAY_HEIGHT),
        size,
        Renderer::new(size, Capabilities::default().width_policy),
    )
}

fn measure_log_initial_render() -> Totals {
    let mut totals = Totals::default();
    for _ in 0..INITIAL_SAMPLES {
        let mut runner = new_log_runner(LogLab::new(
            ITEM_COUNT,
            ITEM_COUNT.saturating_mul(2),
            WIDTH,
            HEIGHT,
        ));
        let rendered = runner
            .render_headless_timed()
            .expect("initial scrolling-log frame must render");
        assert_eq!(rendered.outcome, HeadlessRenderOutcome::Committed);
        add_timings(
            &mut totals.render,
            rendered.timings.expect("render must include timings"),
        );
    }
    totals
}

fn measure_log_scenario(scenario: LogScenario) -> Totals {
    let mut runner = new_log_runner(LogLab::new(
        ITEM_COUNT,
        ITEM_COUNT.saturating_mul(2),
        WIDTH,
        HEIGHT,
    ));
    assert_eq!(
        runner
            .render_headless()
            .expect("initial scrolling-log frame must render"),
        HeadlessRenderOutcome::Committed
    );
    if matches!(scenario, LogScenario::AppendPaused) {
        send_log(&mut runner, LogAction::PageUp);
        render_log_reset(&mut runner);
    }

    let mut totals = Totals::default();
    for generation in 1..=SAMPLES {
        let update_started = Instant::now();
        apply_log_scenario(&mut runner, scenario, u64::from(generation));
        totals.update = totals.update.saturating_add(update_started.elapsed());
        let rendered = runner
            .render_headless_timed()
            .expect("scrolling-log scenario frame must render");
        assert_eq!(rendered.outcome, HeadlessRenderOutcome::Committed);
        add_timings(
            &mut totals.render,
            rendered.timings.expect("render must include timings"),
        );
        reset_log_scenario(&mut runner, scenario);
    }
    totals
}

fn apply_log_scenario(runner: &mut AppRunner<LogLab>, scenario: LogScenario, generation: u64) {
    match scenario {
        LogScenario::PageUp => send_log(runner, LogAction::PageUp),
        LogScenario::Resize => resize_log(runner, RESIZED_HEIGHT),
        LogScenario::AppendFollowing | LogScenario::AppendPaused => send_log(
            runner,
            LogAction::Append {
                count: 1,
                generation,
            },
        ),
    }
}

fn reset_log_scenario(runner: &mut AppRunner<LogLab>, scenario: LogScenario) {
    match scenario {
        LogScenario::PageUp => {
            send_log(runner, LogAction::End);
            render_log_reset(runner);
        }
        LogScenario::Resize => {
            resize_log(runner, HEIGHT);
            render_log_reset(runner);
        }
        LogScenario::AppendFollowing | LogScenario::AppendPaused => {}
    }
}

fn send_log(runner: &mut AppRunner<LogLab>, action: LogAction) {
    runner.enqueue(action);
    runner.process_pending();
}

fn resize_log(runner: &mut AppRunner<LogLab>, height: u16) {
    runner
        .dispatch_ui_event(UiEvent::Resize(Size::new(WIDTH, height)))
        .expect("scrolling-log resize event must dispatch");
    runner.process_pending();
}

fn render_log_reset(runner: &mut AppRunner<LogLab>) {
    assert_eq!(
        runner
            .render_headless()
            .expect("scrolling-log reset frame must render"),
        HeadlessRenderOutcome::Committed
    );
}

fn new_log_runner(application: LogLab) -> AppRunner<LogLab> {
    let size = Size::new(WIDTH, HEIGHT);
    AppRunner::new(
        application,
        size,
        Renderer::new(size, Capabilities::default().width_policy),
    )
}

fn measure_table_initial_render() -> Totals {
    let mut totals = Totals::default();
    for _ in 0..INITIAL_SAMPLES {
        let mut runner = new_table_runner(TableLab::new(ITEM_COUNT, WIDTH, HEIGHT));
        let rendered = runner
            .render_headless_timed()
            .expect("initial table frame must render");
        assert_eq!(rendered.outcome, HeadlessRenderOutcome::Committed);
        add_timings(
            &mut totals.render,
            rendered.timings.expect("render must include timings"),
        );
    }
    totals
}

fn measure_table_scenario(scenario: TableScenario) -> Totals {
    let mut runner = new_table_runner(TableLab::new(ITEM_COUNT, WIDTH, HEIGHT));
    assert_eq!(
        runner
            .render_headless()
            .expect("initial table frame must render"),
        HeadlessRenderOutcome::Committed
    );
    if matches!(scenario, TableScenario::Selection) {
        reset_table_scenario(&mut runner, scenario);
    }

    let mut totals = Totals::default();
    for revision in 1..=SAMPLES {
        let update_started = Instant::now();
        apply_table_scenario(&mut runner, scenario, u64::from(revision));
        totals.update = totals.update.saturating_add(update_started.elapsed());
        let rendered = runner
            .render_headless_timed()
            .expect("table scenario frame must render");
        assert_eq!(rendered.outcome, HeadlessRenderOutcome::Committed);
        add_timings(
            &mut totals.render,
            rendered.timings.expect("render must include timings"),
        );
        reset_table_scenario(&mut runner, scenario);
    }
    totals
}

fn apply_table_scenario(runner: &mut AppRunner<TableLab>, scenario: TableScenario, revision: u64) {
    match scenario {
        TableScenario::PageDown => send_table(runner, TableAction::PageDown),
        TableScenario::Resize => resize_table(runner, RESIZED_HEIGHT),
        TableScenario::Selection => send_table(runner, TableAction::SelectActive),
        TableScenario::VisibleUpdate => {
            send_table(runner, TableAction::BackgroundUpdate { key: 0, revision })
        }
        TableScenario::OffscreenUpdate => send_table(
            runner,
            TableAction::BackgroundUpdate {
                key: u64::try_from(ITEM_COUNT - 1).unwrap_or(u64::MAX),
                revision,
            },
        ),
    }
}

fn reset_table_scenario(runner: &mut AppRunner<TableLab>, scenario: TableScenario) {
    match scenario {
        TableScenario::PageDown => {
            send_table(runner, TableAction::Home);
            render_table_reset(runner);
        }
        TableScenario::Resize => {
            resize_table(runner, HEIGHT);
            render_table_reset(runner);
        }
        TableScenario::Selection => {
            for action in [
                TableAction::Down,
                TableAction::SelectActive,
                TableAction::Home,
            ] {
                send_table(runner, action);
                render_table_reset(runner);
            }
        }
        TableScenario::VisibleUpdate | TableScenario::OffscreenUpdate => {}
    }
}

fn send_table(runner: &mut AppRunner<TableLab>, action: TableAction) {
    runner.enqueue(action);
    runner.process_pending();
}

fn resize_table(runner: &mut AppRunner<TableLab>, height: u16) {
    runner
        .dispatch_ui_event(UiEvent::Resize(Size::new(WIDTH, height)))
        .expect("table resize event must dispatch");
    runner.process_pending();
}

fn render_table_reset(runner: &mut AppRunner<TableLab>) {
    assert_eq!(
        runner
            .render_headless()
            .expect("table reset frame must render"),
        HeadlessRenderOutcome::Committed
    );
}

fn new_table_runner(application: TableLab) -> AppRunner<TableLab> {
    let size = Size::new(WIDTH, HEIGHT);
    AppRunner::new(
        application,
        size,
        Renderer::new(size, Capabilities::default().width_policy),
    )
}

fn measure_initial_render(mode: CollectionMode) -> Totals {
    let mut totals = Totals::default();
    for _ in 0..INITIAL_SAMPLES {
        let model = CollectionLab::new(mode, ITEM_COUNT, viewport_height(HEIGHT));
        let mut runner = new_runner(model);
        let rendered = runner
            .render_headless_timed()
            .expect("initial frame must render");
        assert_eq!(rendered.outcome, HeadlessRenderOutcome::Committed);
        add_timings(
            &mut totals.render,
            rendered.timings.expect("render must include timings"),
        );
    }
    totals
}

fn measure_scenario(mode: CollectionMode, scenario: Scenario) -> Totals {
    let mut runner = new_runner(CollectionLab::new(
        mode,
        ITEM_COUNT,
        viewport_height(HEIGHT),
    ));
    assert_eq!(
        runner.render_headless().expect("initial frame must render"),
        HeadlessRenderOutcome::Committed
    );
    prepare_selection(&mut runner, scenario);

    let mut totals = Totals::default();
    for _ in 0..SAMPLES {
        let update_started = Instant::now();
        apply_scenario(&mut runner, scenario);
        totals.update = totals.update.saturating_add(update_started.elapsed());
        let rendered = runner
            .render_headless_timed()
            .expect("scenario frame must render");
        assert_eq!(rendered.outcome, HeadlessRenderOutcome::Committed);
        add_timings(
            &mut totals.render,
            rendered.timings.expect("render must include timings"),
        );
        reset_scenario(&mut runner, scenario);
    }
    totals
}

fn apply_scenario(runner: &mut AppRunner<CollectionLab>, scenario: Scenario) {
    match scenario {
        Scenario::PageDown => send(runner, Message::PageDown),
        Scenario::End => send(runner, Message::End),
        Scenario::Resize => resize(runner, RESIZED_HEIGHT),
        Scenario::Selection => send(runner, Message::SelectActive),
        Scenario::Reverse => send(runner, Message::Reverse),
        Scenario::UnchangedRedraw => send(runner, Message::Home),
    }
}

fn reset_scenario(runner: &mut AppRunner<CollectionLab>, scenario: Scenario) {
    match scenario {
        Scenario::PageDown | Scenario::End => {
            send(runner, Message::Home);
            render_reset(runner);
        }
        Scenario::Resize => {
            resize(runner, HEIGHT);
            render_reset(runner);
        }
        Scenario::Selection => {
            for message in [Message::Down, Message::SelectActive, Message::Home] {
                send(runner, message);
                render_reset(runner);
            }
        }
        Scenario::Reverse => {
            send(runner, Message::Reverse);
            render_reset(runner);
        }
        Scenario::UnchangedRedraw => {}
    }
}

fn prepare_selection(runner: &mut AppRunner<CollectionLab>, scenario: Scenario) {
    if matches!(scenario, Scenario::Selection) {
        reset_scenario(runner, scenario);
    }
}

fn send(runner: &mut AppRunner<CollectionLab>, message: Message) {
    runner.enqueue(message);
    runner.process_pending();
}

fn resize(runner: &mut AppRunner<CollectionLab>, height: u16) {
    runner
        .dispatch_ui_event(UiEvent::Resize(Size::new(WIDTH, height)))
        .expect("resize event must dispatch");
    runner.process_pending();
}

fn render_reset(runner: &mut AppRunner<CollectionLab>) {
    assert_eq!(
        runner.render_headless().expect("reset frame must render"),
        HeadlessRenderOutcome::Committed
    );
}

fn new_runner(application: CollectionLab) -> AppRunner<CollectionLab> {
    let size = Size::new(WIDTH, HEIGHT);
    AppRunner::new(
        application,
        size,
        Renderer::new(size, Capabilities::default().width_policy),
    )
}

fn add_timings(total: &mut RenderTimings, sample: RenderTimings) {
    total.total = total.total.saturating_add(sample.total);
    total.view_construction = total
        .view_construction
        .saturating_add(sample.view_construction);
    total.staging_reconciliation = total
        .staging_reconciliation
        .saturating_add(sample.staging_reconciliation);
    total.layout = total.layout.saturating_add(sample.layout);
    total.paint = total.paint.saturating_add(sample.paint);
    total.diff = total.diff.saturating_add(sample.diff);
    total.commit = Some(
        total
            .commit
            .unwrap_or_default()
            .saturating_add(sample.commit.unwrap_or_default()),
    );
    total.post_commit = Some(
        total
            .post_commit
            .unwrap_or_default()
            .saturating_add(sample.post_commit.unwrap_or_default()),
    );
}

fn print_totals(mode: CollectionMode, scenario: &str, totals: Totals, samples: u32) {
    println!(
        "| {} | {scenario} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
        mode_name(mode),
        average(totals.update, samples),
        average(totals.render.view_construction, samples),
        average(totals.render.staging_reconciliation, samples),
        average(totals.render.layout, samples),
        average(totals.render.paint, samples),
        average(totals.render.diff, samples),
        average(totals.render.commit.unwrap_or_default(), samples),
        average(totals.render.post_commit.unwrap_or_default(), samples),
        average(totals.render.total, samples),
    );
}

fn print_table_totals(scenario: &str, totals: Totals, samples: u32) {
    print_workload_totals("Table", scenario, totals, samples);
}

fn print_workload_totals(workload: &str, scenario: &str, totals: Totals, samples: u32) {
    println!(
        "| {workload} | {scenario} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
        average(totals.update, samples),
        average(totals.render.view_construction, samples),
        average(totals.render.staging_reconciliation, samples),
        average(totals.render.layout, samples),
        average(totals.render.paint, samples),
        average(totals.render.diff, samples),
        average(totals.render.commit.unwrap_or_default(), samples),
        average(totals.render.post_commit.unwrap_or_default(), samples),
        average(totals.render.total, samples),
    );
}

fn average(duration: Duration, samples: u32) -> u128 {
    duration.as_nanos() / u128::from(samples)
}

const fn mode_name(mode: CollectionMode) -> &'static str {
    match mode {
        CollectionMode::Fixed => "fixed",
        CollectionMode::Variable => "variable",
    }
}

fn viewport_height(terminal_height: u16) -> usize {
    usize::from(terminal_height.saturating_sub(4).max(1))
}
