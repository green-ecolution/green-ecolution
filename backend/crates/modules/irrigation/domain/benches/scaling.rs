mod fixtures;

use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

const SIZES: [usize; 5] = [10, 100, 1_000, 10_000, 100_000];

fn cluster_recalculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cluster");
    for size in SIZES {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(
            BenchmarkId::new("recalculate_centroid", size),
            &size,
            |b, &size| {
                let coords = fixtures::coordinates(size);
                b.iter_batched(
                    || fixtures::cluster_with_trees(0),
                    |mut cluster| cluster.recalculate_centroid(black_box(&coords)),
                    BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("recalculate_watering_status", size),
            &size,
            |b, &size| {
                let statuses = fixtures::statuses(size);
                b.iter_batched(
                    || fixtures::cluster_with_trees(0),
                    |mut cluster| cluster.recalculate_watering_status(black_box(&statuses)),
                    BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("replace_trees", size),
            &size,
            |b, &size| {
                b.iter_batched(
                    || (fixtures::cluster_with_trees(0), fixtures::tree_ids(size)),
                    |(mut cluster, ids)| cluster.replace_trees(black_box(ids)),
                    BatchSize::SmallInput,
                )
            },
        );
    }
    group.finish();
}

/// Constant work: three watermarks, one calibration lookup. It is the baseline
/// the growing groups are read against. If this one moves between runs, the
/// machine moved, not the code.
fn watering_status_reference(c: &mut Criterion) {
    let tree = fixtures::tree_planted_in(2015);
    let watermarks = fixtures::watermarks();
    let today = fixtures::fixed_today();

    c.bench_function("tree/calculate_watering_status_from_watermarks", |b| {
        b.iter(|| {
            black_box(tree.calculate_watering_status_from_watermarks(
                black_box(&watermarks),
                black_box(today),
            ))
        })
    });
}

criterion_group!(benches, cluster_recalculation, watering_status_reference);
criterion_main!(benches);
