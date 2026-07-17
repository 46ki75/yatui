#![allow(missing_docs)]
//! Stable public-path benchmarks for text, frame, and UI layout work.

use std::{hint::black_box, time::Duration};

use arborui::{
    CursorState, Dimension, Element, Invalidation, LayoutStyle, Point, Size, Style, UiTree,
    WidthPolicy, layout::FlexDirection, measure, render::Renderer,
};
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

fn text_measurement(criterion: &mut Criterion) {
    let unicode =
        "ASCII a\u{301} \u{754c} \u{1f469}\u{200d}\u{1f4bb} \u{1f1e6}\u{1f1e7} \u{2764}\u{fe0f} "
            .repeat(256);
    let mut group = criterion.benchmark_group("text/measure");
    group.throughput(Throughput::Bytes(unicode.len() as u64));
    for policy in [WidthPolicy::Unicode, WidthPolicy::Cjk, WidthPolicy::WcWidth] {
        group.bench_with_input(
            BenchmarkId::new("unicode_10k", format!("{policy:?}")),
            &policy,
            |bencher, policy| {
                bencher.iter(|| measure(black_box(&unicode), black_box(*policy)));
            },
        );
    }
    group.finish();
}

fn render_preparation(criterion: &mut Criterion) {
    let size = Size::new(80, 24);
    let mut group = criterion.benchmark_group("render/prepare");
    group.throughput(Throughput::Elements(u64::from(size.area())));

    group.bench_function("80x24/one_cell_change", |bencher| {
        bencher.iter_batched(
            || initialized_renderer(size),
            |mut renderer| {
                let frame = renderer
                    .prepare(size, CursorState::HIDDEN, |canvas| {
                        canvas.draw_text(
                            Point::new(40, 12),
                            black_box("x"),
                            Style::default(),
                            None,
                        )?;
                        Ok(())
                    })
                    .expect("benchmark frame must render");
                black_box(frame.patch().runs.len());
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("80x24/full_repaint", |bencher| {
        bencher.iter(|| {
            let mut renderer = Renderer::new(size, WidthPolicy::Unicode);
            let frame = renderer
                .prepare(size, CursorState::HIDDEN, |canvas| {
                    for y in 0..i32::from(size.height) {
                        canvas.draw_text(
                            Point::new(0, y),
                            black_box("Unicode \u{754c} \u{1f469}\u{200d}\u{1f4bb} status line"),
                            Style::default(),
                            None,
                        )?;
                    }
                    Ok(())
                })
                .expect("benchmark frame must render");
            black_box(frame.patch().runs.len());
        });
    });
    group.finish();
}

fn initialized_renderer(size: Size) -> Renderer {
    let mut renderer = Renderer::new(size, WidthPolicy::Unicode);
    let frame = renderer
        .prepare(size, CursorState::HIDDEN, |_| Ok(()))
        .expect("initial benchmark frame must render");
    renderer
        .commit(frame)
        .expect("initial benchmark frame must commit");
    renderer
}

fn repeated_layout(criterion: &mut Criterion) {
    const VIEWPORT: Size = Size::new(120, 40);
    let mut phase_group = criterion.benchmark_group("ui/repeated_layout_phase");
    for (depth, node_count) in [(5, 63_u64), (8, 511), (11, 4_095)] {
        phase_group.throughput(Throughput::Elements(node_count));
        phase_group.bench_with_input(
            BenchmarkId::new("balanced", node_count),
            &depth,
            |bencher, depth| {
                let view = balanced_layout_tree(*depth);
                let (mut tree, mut renderer, root) = initialized_ui(&view, VIEWPORT);
                bencher.iter_custom(|iterations| {
                    let mut layout = Duration::ZERO;
                    for _ in 0..iterations {
                        assert!(tree.invalidate(root, Invalidation::Layout));
                        let (prepared, timings) = tree
                            .prepare_timed(&view, VIEWPORT, &mut renderer)
                            .expect("benchmark frame must prepare");
                        layout = layout.saturating_add(black_box(timings.layout));
                        tree.commit(prepared, &mut renderer)
                            .expect("benchmark frame must commit");
                    }
                    layout
                });
            },
        );
    }
    phase_group.finish();

    let mut turn_group = criterion.benchmark_group("ui/repeated_layout_turn");
    for (depth, node_count) in [(5, 63_u64), (8, 511), (11, 4_095)] {
        turn_group.throughput(Throughput::Elements(node_count));
        turn_group.bench_with_input(
            BenchmarkId::new("balanced", node_count),
            &depth,
            |bencher, depth| {
                let view = balanced_layout_tree(*depth);
                let (mut tree, mut renderer, root) = initialized_ui(&view, VIEWPORT);
                bencher.iter(|| {
                    assert!(tree.invalidate(root, Invalidation::Layout));
                    let prepared = tree
                        .prepare(&view, VIEWPORT, &mut renderer)
                        .expect("benchmark frame must prepare");
                    tree.commit(prepared, &mut renderer)
                        .expect("benchmark frame must commit");
                    black_box(renderer.current().size());
                });
            },
        );
    }
    turn_group.finish();

    let mut leaf_group = criterion.benchmark_group("ui/repeated_layout_one_leaf_change");
    for (depth, node_count) in [(5, 63_u64), (8, 511), (11, 4_095)] {
        leaf_group.throughput(Throughput::Elements(node_count));
        leaf_group.bench_with_input(
            BenchmarkId::new("balanced", node_count),
            &depth,
            |bencher, depth| {
                let views = [
                    balanced_layout_tree_variant(*depth, false),
                    balanced_layout_tree_variant(*depth, true),
                ];
                let (mut tree, mut renderer, _) = initialized_ui(&views[0], VIEWPORT);
                let mut variant = 0;
                bencher.iter(|| {
                    variant ^= 1;
                    let prepared = tree
                        .prepare(&views[variant], VIEWPORT, &mut renderer)
                        .expect("benchmark frame must prepare");
                    tree.commit(prepared, &mut renderer)
                        .expect("benchmark frame must commit");
                    black_box(renderer.current().size());
                });
            },
        );
    }
    leaf_group.finish();

    let mut structure_group = criterion.benchmark_group("ui/repeated_layout_structure_change");
    for (depth, node_count) in [(5, 64_u64), (8, 512), (11, 4_096)] {
        structure_group.bench_with_input(
            BenchmarkId::new("balanced", node_count),
            &depth,
            |bencher, depth| {
                let views = [
                    balanced_layout_tree_with_extra(*depth, false),
                    balanced_layout_tree_with_extra(*depth, true),
                ];
                let (mut tree, mut renderer, _) = initialized_ui(&views[0], VIEWPORT);
                let mut variant = 0;
                bencher.iter(|| {
                    variant ^= 1;
                    let prepared = tree
                        .prepare(&views[variant], VIEWPORT, &mut renderer)
                        .expect("benchmark frame must prepare");
                    tree.commit(prepared, &mut renderer)
                        .expect("benchmark frame must commit");
                    black_box(renderer.current().size());
                });
            },
        );
    }
    structure_group.finish();
}

fn balanced_layout_tree(depth: u32) -> Element<'static, ()> {
    balanced_layout_tree_variant(depth, false)
}

fn balanced_layout_tree_variant(depth: u32, change_first_leaf: bool) -> Element<'static, ()> {
    if depth == 0 {
        return Element::container([]).layout(LayoutStyle::new().size(
            Dimension::cells(if change_first_leaf { 2 } else { 1 }),
            Dimension::cells(1),
        ));
    }
    Element::container([
        balanced_layout_tree_variant(depth - 1, change_first_leaf),
        balanced_layout_tree_variant(depth - 1, false),
    ])
    .layout(LayoutStyle::new().direction(if depth & 1 == 0 {
        FlexDirection::Row
    } else {
        FlexDirection::Column
    }))
}

fn balanced_layout_tree_with_extra(depth: u32, extra: bool) -> Element<'static, ()> {
    let mut children = vec![balanced_layout_tree(depth)];
    if extra {
        children.push(
            Element::container([])
                .layout(LayoutStyle::new().size(Dimension::cells(1), Dimension::cells(1))),
        );
    }
    Element::container(children)
}

fn initialized_ui(view: &Element<'_, ()>, size: Size) -> (UiTree, Renderer, arborui::NodeId) {
    let mut tree = UiTree::new();
    let mut renderer = Renderer::new(size, WidthPolicy::Unicode);
    let prepared = tree
        .prepare(view, size, &mut renderer)
        .expect("initial benchmark frame must prepare");
    tree.commit(prepared, &mut renderer)
        .expect("initial benchmark frame must commit");
    let root = tree.root().expect("benchmark tree must have a root");
    (tree, renderer, root)
}

criterion_group!(
    benches,
    text_measurement,
    render_preparation,
    repeated_layout
);
criterion_main!(benches);
