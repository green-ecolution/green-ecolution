mod fixtures;

use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use domain::{
    authorization::{AccessContext, EffectivePermissions},
    settings::resolve,
};

const SIZES: [usize; 5] = [10, 100, 1_000, 10_000, 100_000];

/// The chain is bounded by the org tree's depth, so it never reaches the sizes
/// the other groups use. Measuring it anyway keeps an eye on the fold's cost
/// per level.
const CHAIN_DEPTHS: [usize; 4] = [1, 4, 16, 64];

fn authorization(c: &mut Criterion) {
    let mut group = c.benchmark_group("authorization");
    for size in SIZES {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(
            BenchmarkId::new("from_pairs_flat", size),
            &size,
            |b, &size| b.iter(|| black_box(fixtures::flat_hierarchy(size))),
        );

        group.bench_with_input(
            BenchmarkId::new("is_descendant_or_self_deep", size),
            &size,
            |b, &size| {
                let (hierarchy, root, leaf) = fixtures::deep_hierarchy(size);
                b.iter(|| hierarchy.is_descendant_or_self(black_box(leaf), black_box(root)))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("descendants_or_self_flat", size),
            &size,
            |b, &size| {
                let (hierarchy, root, _) = fixtures::flat_hierarchy(size);
                b.iter(|| black_box(hierarchy.descendants_or_self(black_box(root))))
            },
        );
    }
    group.finish();
}

/// Grant count is its own axis, not the org tree's. A user holds a handful of
/// role grants however large the installation gets, and `visible_orgs` walks
/// the whole tree once per grant — folding both into one loop would square the
/// work and measure a shape nobody runs.
fn authorization_grants(c: &mut Criterion) {
    const ORG_NODES: usize = 1_000;
    const GRANT_COUNTS: [usize; 4] = [1, 4, 16, 64];

    let mut group = c.benchmark_group("authorization_grants");
    let permission = fixtures::single_permission();

    for count in GRANT_COUNTS {
        group.throughput(Throughput::Elements(count as u64));

        group.bench_with_input(
            BenchmarkId::new("from_grants", count),
            &count,
            |b, &count| {
                b.iter_batched(
                    || fixtures::grants(count),
                    |grants| black_box(EffectivePermissions::from_grants(grants)),
                    BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(BenchmarkId::new("allows_in", count), &count, |b, &count| {
            let (hierarchy, grants, probe) = fixtures::hierarchy_with_grants(ORG_NODES, count);
            let context = AccessContext {
                permissions: EffectivePermissions::from_grants(grants),
                hierarchy,
            };
            b.iter(|| context.allows_in(black_box(permission), black_box(probe)))
        });

        group.bench_with_input(
            BenchmarkId::new("visible_orgs", count),
            &count,
            |b, &count| {
                let (hierarchy, grants, _) = fixtures::hierarchy_with_grants(ORG_NODES, count);
                let context = AccessContext {
                    permissions: EffectivePermissions::from_grants(grants),
                    hierarchy,
                };
                b.iter(|| black_box(context.visible_orgs(black_box(permission))))
            },
        );
    }
    group.finish();
}

fn settings_resolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("settings");
    let defaults = fixtures::instance_defaults();
    for depth in CHAIN_DEPTHS {
        group.throughput(Throughput::Elements(depth as u64));
        group.bench_with_input(BenchmarkId::new("resolve", depth), &depth, |b, &depth| {
            let (chain, target) = fixtures::settings_chain(depth);
            b.iter(|| black_box(resolve(&defaults, black_box(&chain), black_box(target))))
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    authorization,
    authorization_grants,
    settings_resolution
);
criterion_main!(benches);
