//! The benchmarks of the arithmetic operations.

mod common;

use criterion::{Criterion, black_box};
use fixed96q32::Fixed;

fn arithmetic(criterion: &mut Criterion) {
    let left = Fixed::from_int(3);
    let right = Fixed::from_parts(1, 500_000_000);

    let mut group = criterion.benchmark_group("arithmetic");

    group.bench_function("add", |bencher| {
        bencher.iter(|| black_box(left) + black_box(right))
    });
    group.bench_function("sub", |bencher| {
        bencher.iter(|| black_box(left) - black_box(right))
    });
    group.bench_function("mul", |bencher| {
        bencher.iter(|| black_box(left) * black_box(right))
    });
    group.bench_function("div", |bencher| {
        bencher.iter(|| black_box(left) / black_box(right))
    });
    group.bench_function("rem", |bencher| {
        bencher.iter(|| black_box(left) % black_box(right))
    });
    group.bench_function("neg", |bencher| bencher.iter(|| -black_box(left)));
    group.bench_function("abs", |bencher| bencher.iter(|| black_box(left).abs()));

    group.finish();
}

fn overflow(criterion: &mut Criterion) {
    let left = Fixed::from_int(3);
    let right = Fixed::from_parts(1, 500_000_000);

    let mut group = criterion.benchmark_group("overflow");

    group.bench_function("checked_add", |bencher| {
        bencher.iter(|| black_box(left).checked_add(black_box(right)))
    });
    group.bench_function("checked_mul", |bencher| {
        bencher.iter(|| black_box(left).checked_mul(black_box(right)))
    });
    group.bench_function("checked_div", |bencher| {
        bencher.iter(|| black_box(left).checked_div(black_box(right)))
    });
    group.bench_function("saturating_add", |bencher| {
        bencher.iter(|| black_box(left).saturating_add(black_box(right)))
    });
    group.bench_function("wrapping_add", |bencher| {
        bencher.iter(|| black_box(left).wrapping_add(black_box(right)))
    });
    group.bench_function("overflowing_add", |bencher| {
        bencher.iter(|| black_box(left).overflowing_add(black_box(right)))
    });

    group.finish();
}

fn rounding(criterion: &mut Criterion) {
    let value = Fixed::from_parts(3, 500_000_000);

    let mut group = criterion.benchmark_group("rounding");

    group.bench_function("floor", |bencher| bencher.iter(|| black_box(value).floor()));
    group.bench_function("ceil", |bencher| bencher.iter(|| black_box(value).ceil()));
    group.bench_function("round", |bencher| bencher.iter(|| black_box(value).round()));
    group.bench_function("trunc", |bencher| bencher.iter(|| black_box(value).trunc()));
    group.bench_function("fract", |bencher| bencher.iter(|| black_box(value).fract()));

    group.finish();
}

fn conversions(criterion: &mut Criterion) {
    let value = Fixed::from_parts(3, 500_000_000);

    let mut group = criterion.benchmark_group("conversions");

    group.bench_function("from_int", |bencher| {
        bencher.iter(|| black_box(Fixed::from_int(black_box(3))))
    });
    group.bench_function("from_parts", |bencher| {
        bencher.iter(|| black_box(Fixed::from_parts(black_box(3), black_box(500_000_000))))
    });
    group.bench_function("from_raw", |bencher| {
        bencher.iter(|| black_box(Fixed::from_raw(black_box(1_500_000_000))))
    });
    group.bench_function("from_f64", |bencher| {
        bencher.iter(|| black_box(Fixed::from_f64(black_box(1.5))))
    });
    group.bench_function("to_raw", |bencher| {
        bencher.iter(|| black_box(value).to_raw())
    });
    group.bench_function("to_f64", |bencher| {
        bencher.iter(|| black_box(value).to_f64())
    });
    group.bench_function("to_i64", |bencher| {
        bencher.iter(|| black_box(value).to_i64())
    });

    group.finish();
}

fn formatting(criterion: &mut Criterion) {
    let value = Fixed::from_parts(3, 500_000_000);

    let mut group = criterion.benchmark_group("formatting");

    group.bench_function("display", |bencher| {
        bencher.iter(|| black_box(value).to_string())
    });
    group.bench_function("parse", |bencher| {
        bencher.iter(|| black_box("3.5").parse::<Fixed>())
    });

    group.finish();
}

fn main() {
    let mut criterion = common::config();

    arithmetic(&mut criterion);
    overflow(&mut criterion);
    rounding(&mut criterion);
    conversions(&mut criterion);
    formatting(&mut criterion);

    criterion.final_summary();
}
