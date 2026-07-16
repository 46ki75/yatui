#![allow(missing_docs)]
//! Visible-range lookup benchmarks for the application-local providers.

use std::{hint::black_box, num::NonZeroUsize};

use arborui_example_collection_lab::{FixedHeightProvider, VariableHeightProvider};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

fn visible_ranges(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("collection/visible-range");
    for item_count in [1_000usize, 100_000, 1_000_000] {
        group.throughput(Throughput::Elements(1));
        let fixed = FixedHeightProvider::new(item_count, NonZeroUsize::MIN, 2);
        group.bench_with_input(
            BenchmarkId::new("fixed", item_count),
            &item_count,
            |bencher, count| {
                let scroll = count.saturating_sub(40) / 2;
                bencher.iter(|| fixed.visible_range(black_box(scroll), black_box(40)));
            },
        );

        let variable = VariableHeightProvider::new(
            (0..item_count).filter_map(|index| {
                Some((
                    u64::try_from(index).ok()?,
                    NonZeroUsize::new(index % 3 + 1)?,
                ))
            }),
            8,
        );
        group.bench_with_input(
            BenchmarkId::new("variable", item_count),
            &item_count,
            |bencher, count| {
                let scroll = count.saturating_sub(40);
                bencher.iter(|| variable.visible_range(black_box(scroll), black_box(40)));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, visible_ranges);
criterion_main!(benches);
