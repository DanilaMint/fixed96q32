//! Fixed-point arithmetic with a 128-bit layout.
//!
//! [`Fixed`] is a signed fixed-point number that keeps 1 bit for the sign,
//! 95 bits for the integral part and 32 bits for the fractional part.
//! The value of a `Fixed` is `raw / 2**32`, therefore the smallest positive
//! representable value is `2**-32` and the representable range is
//! `[-2**95, 2**95 - 2**-32]`.
//!
//! The crate is `no_std` friendly and provides:
//!
//! * [`Fixed`] — the scalar itself, together with its checked, saturating
//!   and wrapping arithmetic primitives;
//! * [`ops`] — the implementations of the [`core::ops`] traits;
//! * [`froms`] — the [`core::convert`] conversions;
//! * [`fmt`] — the [`core::fmt::Display`] and [`core::str::FromStr`]
//!   implementations;
//! * [`math`] — the advanced mathematics (square root, logarithm, exponent,
//!   trigonometric functions, …) implemented with the CORDIC algorithm;
//! * [`consts`] — mathematical constants;
//! * [`num_traits`] — the `num-traits` interoperability (feature
//!   `num-traits`);
//! * [`serde`] — the `serde` interoperability (feature `serde`).
//!
//! # Examples
//!
//! ```
//! use fixed96q32::Fixed;
//!
//! let a = Fixed::from_int(3);
//! let b = Fixed::from_parts(0, 17_179_869);
//! assert_eq!(a + b, Fixed::from_parts(3, 17_179_869));
//! assert_eq!(Fixed::ONE.to_string(), "1");
//! ```
//!
//! # Arithmetic
//!
//! The [`core::ops`] implementations panic on overflow, exactly like the
//! operators of the primitive integer types do. The `checked_*` methods of
//! [`Fixed`] report the overflow with [`None`], the `saturating_*` methods
//! clamp the result to [`Fixed::MIN`] / [`Fixed::MAX`] and the `wrapping_*`
//! methods take the result modulo `2**128`.
//!
//! Multiplication and division truncate the discarded bits towards zero.
//!
//! [`num_traits`]: crate::num_traits
//! [`serde`]: crate::serde

#![no_std]

#[cfg(test)]
extern crate std;

/// The examples of the readme file are checked as the documentation tests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeExamples;

pub mod consts;
pub mod fixed;
pub mod fmt;
pub mod froms;
pub mod math;
pub mod ops;

#[cfg(feature = "num-traits")]
pub mod num_traits;

#[cfg(feature = "serde")]
pub mod serde;

pub use fixed::Fixed;
