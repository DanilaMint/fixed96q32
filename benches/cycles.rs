//! The measurement of the processor cycles per operation.
//!
//! The counters of the processor are read with the `rdtsc` instruction, so
//! the results do not depend on the wall clock and on the frequency scaling.

#![cfg(target_arch = "x86_64")]

use std::arch::x86_64::{__rdtscp, _rdtsc};
use std::hint::black_box;
use std::time::{Duration, Instant};

use fixed96q32::Fixed;

/// The number of the iterations of a single measurement.
const COUNT: u64 = 4_000_000;

/// Measures the number of the cycles of a single operation.
fn cycles<R>(mut operation: impl FnMut() -> R) -> f64 {
    for _ in 0..10_000 {
        black_box(operation());
    }

    let start = unsafe { _rdtsc() };
    for _ in 0..COUNT {
        black_box(operation());
    }
    let end = unsafe { _rdtsc() };

    (end - start) as f64 / COUNT as f64
}

/// Measures the frequency of the time stamp counter.
fn counter_frequency() -> f64 {
    let start = Instant::now();
    let counter = unsafe { _rdtsc() };
    std::thread::sleep(Duration::from_millis(500));
    let end = unsafe { __rdtscp(&mut 0) };
    let elapsed = start.elapsed().as_secs_f64();

    (end - counter) as f64 / elapsed
}

fn main() {
    let frequency = counter_frequency();

    // The overhead of the measurement loop is measured separately and
    // subtracted from every measurement.
    let overhead = cycles(|| black_box(0u64));

    println!(
        "\nthe time stamp counter runs at {:.3} GHz",
        frequency / 1e9
    );
    println!("the overhead of the measurement loop is {overhead:.2} cycles\n");

    let left = Fixed::from_int(3);
    let right = Fixed::from_parts(1, 500_000_000);
    let value = Fixed::from_parts(3, 500_000_000);
    let unit = Fixed::from_parts(0, 2_147_483_648);

    let mut rows: Vec<(&str, f64)> = Vec::new();

    macro_rules! measure {
        ($($name:expr => $operation:expr),* $(,)?) => {
            $(
                rows.push(($name, cycles(|| $operation) - overhead));
            )*
        };
    }

    measure! {
        "add" => black_box(left) + black_box(right),
        "sub" => black_box(left) - black_box(right),
        "mul" => black_box(left) * black_box(right),
        "div" => black_box(left) / black_box(right),
        "rem" => black_box(left) % black_box(right),
        "neg" => -black_box(left),
        "abs" => black_box(left).abs(),
        "checked_add" => black_box(left).checked_add(black_box(right)),
        "checked_mul" => black_box(left).checked_mul(black_box(right)),
        "checked_div" => black_box(left).checked_div(black_box(right)),
        "saturating_add" => black_box(left).saturating_add(black_box(right)),
        "wrapping_add" => black_box(left).wrapping_add(black_box(right)),
        "overflowing_add" => black_box(left).overflowing_add(black_box(right)),
        "floor" => black_box(value).floor(),
        "ceil" => black_box(value).ceil(),
        "round" => black_box(value).round(),
        "trunc" => black_box(value).trunc(),
        "fract" => black_box(value).fract(),
        "from_int" => Fixed::from_int(black_box(3)),
        "from_parts" => Fixed::from_parts(black_box(3), black_box(500_000_000)),
        "from_raw" => Fixed::from_raw(black_box(1_500_000_000)),
        "from_f64" => Fixed::from_f64(black_box(1.5)),
        "to_raw" => black_box(value).to_raw(),
        "to_f64" => black_box(value).to_f64(),
        "to_i64" => black_box(value).to_i64(),
        "display" => black_box(value).to_string(),
        "parse" => black_box("3.5").parse::<Fixed>(),
        "sqrt" => black_box(value).sqrt(),
        "cbrt" => black_box(value).cbrt(),
        "ln" => black_box(value).ln(),
        "log2" => black_box(value).log2(),
        "log10" => black_box(value).log10(),
        "exp" => black_box(value).exp(),
        "exp2" => black_box(value).exp2(),
        "sin" => black_box(value).sin(),
        "cos" => black_box(value).cos(),
        "tan" => black_box(value).tan(),
        "asin" => black_box(unit).asin(),
        "acos" => black_box(unit).acos(),
        "atan" => black_box(value).atan(),
        "atan2" => black_box(value).atan2(black_box(Fixed::ONE)),
        "sinh" => black_box(value).sinh(),
        "cosh" => black_box(value).cosh(),
        "tanh" => black_box(value).tanh(),
        "asinh" => black_box(value).asinh(),
        "acosh" => black_box(value).acosh(),
        "atanh" => black_box(unit).atanh(),
        "powi" => black_box(value).powi(black_box(10)),
        "powf" => black_box(value).powf(black_box(Fixed::from_int(10))),
        "hypot" => black_box(value).hypot(black_box(Fixed::from_int(3))),
    }

    println!(
        "  {:<20} {:>10} {:>12} {:>12}",
        "operation", "cycles", "ns", "cycles at 4.4GHz"
    );

    for (name, count) in rows {
        println!(
            "  {:<20} {:>10.1} {:>12.2} {:>12.1}",
            name,
            count,
            count / (frequency / 1e9),
            count * 4.4 / (frequency / 1e9)
        );
    }
}
