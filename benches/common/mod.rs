//! The shared configuration of the benchmarks.

use std::time::Duration;

use criterion::Criterion;

/// Returns the criterion configuration of the crate.
///
/// The default settings of the framework are tuned for the long benchmarks,
/// the settings below keep the whole suite short enough for the daily use.
pub fn config() -> Criterion {
    Criterion::default()
        .sample_size(20)
        .warm_up_time(Duration::from_millis(300))
        .measurement_time(Duration::from_secs(1))
        .noise_threshold(0.05)
        .significance_level(0.01)
}
