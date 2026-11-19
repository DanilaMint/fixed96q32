//! The benchmarks of the advanced mathematics.

mod common;

use criterion::{Criterion, black_box};
use fixed96q32::Fixed;

fn roots(criterion: &mut Criterion) {
    let value = Fixed::from_int(2);

    let mut group = criterion.benchmark_group("math/roots");

    group.bench_function("sqrt", |bencher| bencher.iter(|| black_box(value).sqrt()));
    group.bench_function("cbrt", |bencher| bencher.iter(|| black_box(value).cbrt()));

    group.finish();
}

fn logarithms(criterion: &mut Criterion) {
    let value = Fixed::from_int(2);

    let mut group = criterion.benchmark_group("math/logarithms");

    group.bench_function("ln", |bencher| bencher.iter(|| black_box(value).ln()));
    group.bench_function("log2", |bencher| bencher.iter(|| black_box(value).log2()));
    group.bench_function("log10", |bencher| bencher.iter(|| black_box(value).log10()));
    group.bench_function("exp", |bencher| bencher.iter(|| black_box(value).exp()));
    group.bench_function("exp2", |bencher| bencher.iter(|| black_box(value).exp2()));

    group.finish();
}

fn trigonometry(criterion: &mut Criterion) {
    let value = Fixed::from_int(1);

    let mut group = criterion.benchmark_group("math/trigonometry");

    group.bench_function("sin", |bencher| bencher.iter(|| black_box(value).sin()));
    group.bench_function("cos", |bencher| bencher.iter(|| black_box(value).cos()));
    group.bench_function("tan", |bencher| bencher.iter(|| black_box(value).tan()));
    group.bench_function("sin_cos", |bencher| {
        bencher.iter(|| black_box(value).sin_cos())
    });
    group.bench_function("asin", |bencher| bencher.iter(|| black_box(value).asin()));
    group.bench_function("acos", |bencher| bencher.iter(|| black_box(value).acos()));
    group.bench_function("atan", |bencher| bencher.iter(|| black_box(value).atan()));
    group.bench_function("atan2", |bencher| {
        bencher.iter(|| black_box(value).atan2(black_box(Fixed::ONE)))
    });

    group.finish();
}

fn hyperbolics(criterion: &mut Criterion) {
    let value = Fixed::from_int(1);

    let mut group = criterion.benchmark_group("math/hyperbolics");

    group.bench_function("sinh", |bencher| bencher.iter(|| black_box(value).sinh()));
    group.bench_function("cosh", |bencher| bencher.iter(|| black_box(value).cosh()));
    group.bench_function("tanh", |bencher| bencher.iter(|| black_box(value).tanh()));
    group.bench_function("asinh", |bencher| bencher.iter(|| black_box(value).asinh()));
    group.bench_function("acosh", |bencher| bencher.iter(|| black_box(value).acosh()));
    group.bench_function("atanh", |bencher| bencher.iter(|| black_box(value).atanh()));

    group.finish();
}

fn powers(criterion: &mut Criterion) {
    let value = Fixed::from_int(2);

    let mut group = criterion.benchmark_group("math/powers");

    group.bench_function("powi", |bencher| {
        bencher.iter(|| black_box(value).powi(black_box(10)))
    });
    group.bench_function("powf", |bencher| {
        bencher.iter(|| black_box(value).powf(black_box(Fixed::from_int(10))))
    });
    group.bench_function("hypot", |bencher| {
        bencher.iter(|| black_box(value).hypot(black_box(Fixed::from_int(3))))
    });

    group.finish();
}

fn main() {
    let mut criterion = common::config();

    roots(&mut criterion);
    logarithms(&mut criterion);
    trigonometry(&mut criterion);
    hyperbolics(&mut criterion);
    powers(&mut criterion);

    criterion.final_summary();
}
