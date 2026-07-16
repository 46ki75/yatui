#![allow(missing_docs)]

use std::{hint::black_box, time::Duration};

use arborui_comparison_collection_lab_ratatui::{
    ComparisonAction, RatatuiCollectionLab, draw_test_terminal,
};
use arborui_example_collection_lab::{CollectionLab, CollectionMode, Message};
use arborui_test::{Size, TestApp};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ratatui::{Terminal, backend::TestBackend};

const ITEM_COUNT: usize = 100_000;
const BASE_WIDTH: u16 = 48;
const BASE_HEIGHT: u16 = 12;
const RESIZED_HEIGHT: u16 = 16;

#[derive(Clone, Copy)]
enum Scenario {
    PageDown,
    End,
    Resize,
    Selection,
    Reverse,
    UnchangedRedraw,
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

fn application_turns(criterion: &mut Criterion) {
    line_navigation(criterion);
    cold_initial_render(criterion);
    scenario_turns(criterion);
}

fn line_navigation(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("comparison/collection-turn/line-navigation");
    group.throughput(Throughput::Elements(1));
    for item_count in [1_000usize, 100_000, 1_000_000] {
        for mode in [CollectionMode::Fixed, CollectionMode::Variable] {
            let mode_name = mode_name(mode);
            group.bench_with_input(
                BenchmarkId::new(format!("arborui/{mode_name}"), item_count),
                &item_count,
                |bencher, count| {
                    let mut application = TestApp::new(
                        CollectionLab::new(mode, *count, viewport_height(BASE_HEIGHT)),
                        base_size(),
                    );
                    let mut down = true;
                    bencher.iter_custom(|iterations| {
                        let started = std::time::Instant::now();
                        for _ in 0..iterations {
                            let message = if down { Message::Down } else { Message::Up };
                            down = !down;
                            black_box(application.send(message));
                        }
                        started.elapsed()
                    });
                },
            );
            group.bench_with_input(
                BenchmarkId::new(format!("ratatui/{mode_name}"), item_count),
                &item_count,
                |bencher, count| {
                    let (mut application, mut terminal) = ratatui_fixture(mode, *count);
                    let mut down = true;
                    bencher.iter_custom(|iterations| {
                        let mut elapsed = Duration::ZERO;
                        for _ in 0..iterations {
                            let action = if down {
                                ComparisonAction::Down
                            } else {
                                ComparisonAction::Up
                            };
                            down = !down;
                            let started = std::time::Instant::now();
                            ratatui_turn(&mut application, &mut terminal, action);
                            elapsed = elapsed.saturating_add(started.elapsed());
                        }
                        black_box(application.semantic_state());
                        elapsed
                    });
                },
            );
        }
    }
    group.finish();
}

fn cold_initial_render(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("comparison/collection-turn/cold-initial-render");
    group.throughput(Throughput::Elements(1));
    for mode in [CollectionMode::Fixed, CollectionMode::Variable] {
        let mode_name = mode_name(mode);
        group.bench_function(BenchmarkId::new("arborui", mode_name), |bencher| {
            bencher.iter(|| {
                black_box(TestApp::new(
                    CollectionLab::new(mode, ITEM_COUNT, viewport_height(BASE_HEIGHT)),
                    base_size(),
                ));
            });
        });
        group.bench_function(BenchmarkId::new("ratatui", mode_name), |bencher| {
            bencher.iter(|| {
                let mut application =
                    RatatuiCollectionLab::new(mode, ITEM_COUNT, BASE_WIDTH, BASE_HEIGHT);
                let mut terminal = Terminal::new(TestBackend::new(BASE_WIDTH, BASE_HEIGHT))
                    .expect("test terminal must open");
                draw_test_terminal(&mut terminal, &mut application)
                    .expect("initial frame must draw");
                black_box((application, terminal));
            });
        });
    }
    group.finish();
}

fn scenario_turns(criterion: &mut Criterion) {
    for scenario in Scenario::ALL {
        let mut group =
            criterion.benchmark_group(format!("comparison/collection-turn/{}", scenario.name()));
        group.throughput(Throughput::Elements(1));
        for mode in [CollectionMode::Fixed, CollectionMode::Variable] {
            let mode_name = mode_name(mode);
            group.bench_function(BenchmarkId::new("arborui", mode_name), |bencher| {
                let mut application = TestApp::new(
                    CollectionLab::new(mode, ITEM_COUNT, viewport_height(BASE_HEIGHT)),
                    base_size(),
                );
                prepare_arborui_selection(&mut application, scenario);
                bencher.iter_custom(|iterations| {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        let started = std::time::Instant::now();
                        arborui_turn(&mut application, scenario);
                        elapsed = elapsed.saturating_add(started.elapsed());
                        reset_arborui(&mut application, scenario);
                    }
                    black_box(arborui_semantic_marker(&application));
                    elapsed
                });
            });
            group.bench_function(BenchmarkId::new("ratatui", mode_name), |bencher| {
                let (mut application, mut terminal) = ratatui_fixture(mode, ITEM_COUNT);
                prepare_ratatui_selection(&mut application, &mut terminal, scenario);
                bencher.iter_custom(|iterations| {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        let started = std::time::Instant::now();
                        ratatui_scenario_turn(&mut application, &mut terminal, scenario);
                        elapsed = elapsed.saturating_add(started.elapsed());
                        reset_ratatui(&mut application, &mut terminal, scenario);
                    }
                    black_box(application.semantic_state());
                    elapsed
                });
            });
        }
        group.finish();
    }
}

fn arborui_turn(application: &mut TestApp<CollectionLab>, scenario: Scenario) {
    match scenario {
        Scenario::PageDown => {
            black_box(application.send(Message::PageDown));
        }
        Scenario::End => {
            black_box(application.send(Message::End));
        }
        Scenario::Resize => {
            black_box(application.resize(Size::new(BASE_WIDTH, RESIZED_HEIGHT)));
        }
        Scenario::Selection => {
            black_box(application.send(Message::SelectActive));
        }
        Scenario::Reverse => {
            black_box(application.send(Message::Reverse));
        }
        Scenario::UnchangedRedraw => {
            black_box(application.send(Message::Home));
        }
    }
}

fn reset_arborui(application: &mut TestApp<CollectionLab>, scenario: Scenario) {
    match scenario {
        Scenario::PageDown | Scenario::End => {
            application.send(Message::Home);
        }
        Scenario::Resize => {
            application.resize(base_size());
        }
        Scenario::Selection => {
            application.send(Message::Down);
            application.send(Message::SelectActive);
            application.send(Message::Home);
        }
        Scenario::Reverse => {
            application.send(Message::Reverse);
        }
        Scenario::UnchangedRedraw => {}
    }
}

fn prepare_arborui_selection(application: &mut TestApp<CollectionLab>, scenario: Scenario) {
    if matches!(scenario, Scenario::Selection) {
        application.send(Message::Down);
        application.send(Message::SelectActive);
        application.send(Message::Home);
    }
}

fn ratatui_scenario_turn(
    application: &mut RatatuiCollectionLab,
    terminal: &mut Terminal<TestBackend>,
    scenario: Scenario,
) {
    match scenario {
        Scenario::PageDown => {
            ratatui_turn(application, terminal, ComparisonAction::PageDown);
        }
        Scenario::End => ratatui_turn(application, terminal, ComparisonAction::End),
        Scenario::Resize => ratatui_resize(application, terminal, RESIZED_HEIGHT),
        Scenario::Selection => {
            ratatui_turn(application, terminal, ComparisonAction::SelectActive);
        }
        Scenario::Reverse => ratatui_turn(application, terminal, ComparisonAction::Reverse),
        Scenario::UnchangedRedraw => {
            ratatui_turn(application, terminal, ComparisonAction::Home);
        }
    }
}

fn reset_ratatui(
    application: &mut RatatuiCollectionLab,
    terminal: &mut Terminal<TestBackend>,
    scenario: Scenario,
) {
    match scenario {
        Scenario::PageDown | Scenario::End => {
            ratatui_turn(application, terminal, ComparisonAction::Home);
        }
        Scenario::Resize => ratatui_resize(application, terminal, BASE_HEIGHT),
        Scenario::Selection => {
            ratatui_turn(application, terminal, ComparisonAction::Down);
            ratatui_turn(application, terminal, ComparisonAction::SelectActive);
            ratatui_turn(application, terminal, ComparisonAction::Home);
        }
        Scenario::Reverse => {
            ratatui_turn(application, terminal, ComparisonAction::Reverse);
        }
        Scenario::UnchangedRedraw => {}
    }
}

fn prepare_ratatui_selection(
    application: &mut RatatuiCollectionLab,
    terminal: &mut Terminal<TestBackend>,
    scenario: Scenario,
) {
    if matches!(scenario, Scenario::Selection) {
        ratatui_turn(application, terminal, ComparisonAction::Down);
        ratatui_turn(application, terminal, ComparisonAction::SelectActive);
        ratatui_turn(application, terminal, ComparisonAction::Home);
    }
}

fn ratatui_fixture(
    mode: CollectionMode,
    item_count: usize,
) -> (RatatuiCollectionLab, Terminal<TestBackend>) {
    let mut application = RatatuiCollectionLab::new(mode, item_count, BASE_WIDTH, BASE_HEIGHT);
    let mut terminal =
        Terminal::new(TestBackend::new(BASE_WIDTH, BASE_HEIGHT)).expect("test terminal must open");
    draw_test_terminal(&mut terminal, &mut application).expect("initial frame must draw");
    (application, terminal)
}

fn ratatui_turn(
    application: &mut RatatuiCollectionLab,
    terminal: &mut Terminal<TestBackend>,
    action: ComparisonAction,
) {
    application.apply(action);
    draw_test_terminal(terminal, application).expect("frame must draw");
}

fn ratatui_resize(
    application: &mut RatatuiCollectionLab,
    terminal: &mut Terminal<TestBackend>,
    height: u16,
) {
    terminal.backend_mut().resize(BASE_WIDTH, height);
    ratatui_turn(
        application,
        terminal,
        ComparisonAction::Resize {
            width: BASE_WIDTH,
            height,
        },
    );
}

fn arborui_semantic_marker(application: &TestApp<CollectionLab>) -> (Option<u64>, Option<u64>) {
    (
        application.application().active_key(),
        application.application().selected_key(),
    )
}

fn viewport_height(terminal_height: u16) -> usize {
    terminal_height.saturating_sub(4).max(1) as usize
}

const fn base_size() -> Size {
    Size::new(BASE_WIDTH, BASE_HEIGHT)
}

const fn mode_name(mode: CollectionMode) -> &'static str {
    match mode {
        CollectionMode::Fixed => "fixed",
        CollectionMode::Variable => "variable",
    }
}

criterion_group!(benches, application_turns);
criterion_main!(benches);
