//! Interoperability with the `num-traits` crate.
//!
//! The module is enabled by the `num-traits` feature and implements the
//! numeric traits of `num-traits` for the [`Fixed`] type.
//!
//! # Semantics
//!
//! * The operators of the crate panic on overflow, therefore the traits that
//!   report an overflow return the wrapped value together with the overflow
//!   flag, exactly like the primitive integer types do.
//! * The conversions of [`ToPrimitive`] follow the conventions of
//!   `num-traits`: the value is truncated towards zero and [`None`] is
//!   returned when the result does not fit into the target type. The inherent
//!   conversion methods of [`Fixed`] clamp the result instead.
//! * [`Num::from_str_radix`] supports the radix `10` only; every other radix
//!   is rejected with [`ParseError::InvalidCharacter`].
//!
//! # Examples
//!
//! ```
//! use fixed96q32::Fixed;
//! use fixed96q32::num_traits::{Num, One, Zero};
//!
//! assert_eq!(Fixed::zero(), Fixed::ZERO);
//! assert_eq!(Fixed::one(), Fixed::ONE);
//!
//! let value = <Fixed as Num>::from_str_radix("0.5", 10).unwrap();
//! assert_eq!(value, Fixed::HALF);
//! ```

pub use num_traits::{
    Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedRem, CheckedSub, Euclid,
    FromPrimitive, Inv, MulAdd, Num, NumCast, One, Pow, Saturating, SaturatingAdd, SaturatingMul,
    SaturatingSub, Signed, ToPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub, Zero,
    ops::overflowing::{OverflowingAdd, OverflowingMul, OverflowingSub},
};

use crate::fixed::Fixed;
use crate::fmt::ParseError;

/// The largest integral value that a [`Fixed`] can hold.
const MAX_INTEGER: i128 = (1i128 << 95) - 1;

/// The smallest integral value that a [`Fixed`] can hold.
const MIN_INTEGER: i128 = -(1i128 << 95);

/// The sign related helpers that complement [`Signed`].
///
/// The `num-traits` crate does not expose the predicates of the sign bit,
/// therefore they are provided by this extension trait.
pub trait SignedExt {
    /// Returns `true` if the value is a negative zero.
    fn is_negative_zero(&self) -> bool;

    /// Returns `true` if the sign of the value is negative.
    fn is_sign_negative(&self) -> bool;

    /// Returns `true` if the sign of the value is positive.
    fn is_sign_positive(&self) -> bool;
}

/// Raising a value to an integer power.
///
/// The `num-traits` crate does not expose this trait, therefore it is
/// provided by the module.
pub trait PowI {
    /// The resulting type after the exponentiation.
    type Output;

    /// Raises the value to the given power.
    fn powi(self, exp: i32) -> Self::Output;
}

impl Zero for Fixed {
    /// Returns the additive identity element, `0`.
    #[inline]
    fn zero() -> Self {
        Self::ZERO
    }

    /// Returns `true` if the value is zero.
    #[inline]
    fn is_zero(&self) -> bool {
        Fixed::is_zero(*self)
    }
}

impl One for Fixed {
    /// Returns the multiplicative identity element, `1`.
    #[inline]
    fn one() -> Self {
        Self::ONE
    }
}

impl Num for Fixed {
    /// The error returned when the string cannot be parsed.
    type FromStrRadixErr = ParseError;

    /// Parses a decimal number, for example `0.004`.
    ///
    /// Only the radix `10` is supported; every other radix is rejected with
    /// [`ParseError::InvalidCharacter`].
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed96q32::Fixed;
    /// use fixed96q32::num_traits::Num;
    ///
    /// let value = <Fixed as Num>::from_str_radix("0.004", 10).unwrap();
    /// assert_eq!(value, Fixed::from_parts(0, 17_179_869));
    /// ```
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        if radix != 10 {
            return Err(ParseError::InvalidCharacter);
        }
        str.parse::<Fixed>()
    }
}

impl Bounded for Fixed {
    /// Returns the smallest representable value, `-2**95`.
    #[inline]
    fn min_value() -> Self {
        Self::MIN
    }

    /// Returns the largest representable value, `2**95 - 2**-32`.
    #[inline]
    fn max_value() -> Self {
        Self::MAX
    }
}

impl Signed for Fixed {
    /// Returns the absolute value of the number.
    ///
    /// Unlike the signed integers, [`Fixed::MIN`] has no positive
    /// counterpart, therefore the result is clamped to [`Fixed::MAX`].
    #[inline]
    fn abs(&self) -> Self {
        Fixed::abs(*self)
    }

    /// Returns the positive difference between the value and `other`.
    ///
    /// The result is zero when the value is less than or equal to `other`.
    #[inline]
    fn abs_sub(&self, other: &Self) -> Self {
        if self.to_raw() <= other.to_raw() {
            Self::ZERO
        } else {
            Fixed::saturating_sub(*self, *other)
        }
    }

    /// Returns `1` for the positive values, `-1` for the negative values and
    /// `0` for zero.
    #[inline]
    fn signum(&self) -> Self {
        Fixed::signum(*self)
    }

    /// Returns `true` if the value is strictly positive.
    #[inline]
    fn is_positive(&self) -> bool {
        Fixed::is_positive(*self)
    }

    /// Returns `true` if the value is strictly negative.
    #[inline]
    fn is_negative(&self) -> bool {
        Fixed::is_negative(*self)
    }
}

impl SignedExt for Fixed {
    /// Returns `false`, because the fixed-point numbers have no negative zero.
    #[inline]
    fn is_negative_zero(&self) -> bool {
        false
    }

    /// Returns `true` if the sign bit of the value is set.
    #[inline]
    fn is_sign_negative(&self) -> bool {
        self.to_raw() < 0
    }

    /// Returns `true` if the sign bit of the value is clear.
    #[inline]
    fn is_sign_positive(&self) -> bool {
        self.to_raw() >= 0
    }
}

impl ToPrimitive for Fixed {
    /// Converts the value into an `isize`, truncating towards zero.
    #[inline]
    fn to_isize(&self) -> Option<isize> {
        truncated_integer(*self).try_into().ok()
    }

    /// Converts the value into an `i8`, truncating towards zero.
    #[inline]
    fn to_i8(&self) -> Option<i8> {
        truncated_integer(*self).try_into().ok()
    }

    /// Converts the value into an `i16`, truncating towards zero.
    #[inline]
    fn to_i16(&self) -> Option<i16> {
        truncated_integer(*self).try_into().ok()
    }

    /// Converts the value into an `i32`, truncating towards zero.
    #[inline]
    fn to_i32(&self) -> Option<i32> {
        truncated_integer(*self).try_into().ok()
    }

    /// Converts the value into an `i64`, truncating towards zero.
    #[inline]
    fn to_i64(&self) -> Option<i64> {
        truncated_integer(*self).try_into().ok()
    }

    /// Converts the value into an `i128`, truncating towards zero.
    #[inline]
    fn to_i128(&self) -> Option<i128> {
        Some(truncated_integer(*self))
    }

    /// Converts the value into a `usize`, truncating towards zero.
    #[inline]
    fn to_usize(&self) -> Option<usize> {
        truncated_integer(*self).try_into().ok()
    }

    /// Converts the value into a `u8`, truncating towards zero.
    #[inline]
    fn to_u8(&self) -> Option<u8> {
        truncated_integer(*self).try_into().ok()
    }

    /// Converts the value into a `u16`, truncating towards zero.
    #[inline]
    fn to_u16(&self) -> Option<u16> {
        truncated_integer(*self).try_into().ok()
    }

    /// Converts the value into a `u32`, truncating towards zero.
    #[inline]
    fn to_u32(&self) -> Option<u32> {
        truncated_integer(*self).try_into().ok()
    }

    /// Converts the value into a `u64`, truncating towards zero.
    #[inline]
    fn to_u64(&self) -> Option<u64> {
        truncated_integer(*self).try_into().ok()
    }

    /// Converts the value into a `u128`, truncating towards zero.
    #[inline]
    fn to_u128(&self) -> Option<u128> {
        truncated_integer(*self).try_into().ok()
    }

    /// Converts the value into an `f32`, losing precision.
    #[inline]
    fn to_f32(&self) -> Option<f32> {
        Some(Fixed::to_f64(*self) as f32)
    }

    /// Converts the value into an `f64`, losing precision.
    #[inline]
    fn to_f64(&self) -> Option<f64> {
        Some(Fixed::to_f64(*self))
    }
}

impl FromPrimitive for Fixed {
    /// Converts an integer, which is always exact.
    #[inline]
    fn from_i64(n: i64) -> Option<Self> {
        Some(Fixed::from_int(n))
    }

    /// Converts an unsigned integer, which is always exact.
    #[inline]
    fn from_u64(n: u64) -> Option<Self> {
        Some(Fixed::from_raw((n as i128) << 32))
    }

    /// Converts a 128-bit integer, which may not fit into the range of the
    /// type.
    #[inline]
    fn from_i128(n: i128) -> Option<Self> {
        Fixed::try_from(n).ok()
    }

    /// Converts an unsigned 128-bit integer, which may not fit into the range
    /// of the type.
    #[inline]
    fn from_u128(n: u128) -> Option<Self> {
        Fixed::try_from(n).ok()
    }

    /// Converts a double precision float, rounding to the nearest
    /// representable value.
    #[inline]
    fn from_f64(n: f64) -> Option<Self> {
        Fixed::try_from(n).ok()
    }

    /// Converts a single precision float, rounding to the nearest
    /// representable value.
    #[inline]
    fn from_f32(n: f32) -> Option<Self> {
        Fixed::try_from(n).ok()
    }
}

impl NumCast for Fixed {
    /// Converts a value of any numeric type.
    ///
    /// The `ToPrimitive` trait erases the distinction between the integer and
    /// the floating point variants, therefore the integral path is used only
    /// when it reproduces the value of the source exactly.
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        let float = n.to_f64();
        match (n.to_i128(), float) {
            (Some(integer), Some(value)) if value == integer as f64 => {
                Fixed::try_from(integer).ok()
            }
            (_, Some(value)) => Fixed::try_from(value).ok(),
            _ => None,
        }
    }
}

impl CheckedAdd for Fixed {
    /// Adds two values, returning [`None`] on overflow.
    #[inline]
    fn checked_add(&self, v: &Self) -> Option<Self> {
        Fixed::checked_add(*self, *v)
    }
}

impl CheckedSub for Fixed {
    /// Subtracts two values, returning [`None`] on overflow.
    #[inline]
    fn checked_sub(&self, v: &Self) -> Option<Self> {
        Fixed::checked_sub(*self, *v)
    }
}

impl CheckedMul for Fixed {
    /// Multiplies two values, returning [`None`] on overflow.
    #[inline]
    fn checked_mul(&self, v: &Self) -> Option<Self> {
        Fixed::checked_mul(*self, *v)
    }
}

impl CheckedDiv for Fixed {
    /// Divides two values, returning [`None`] on overflow or when the divisor
    /// is zero.
    #[inline]
    fn checked_div(&self, v: &Self) -> Option<Self> {
        Fixed::checked_div(*self, *v)
    }
}

impl CheckedRem for Fixed {
    /// Returns the remainder of the division, returning [`None`] when the
    /// divisor is zero.
    #[inline]
    fn checked_rem(&self, v: &Self) -> Option<Self> {
        if Fixed::is_zero(*v) {
            None
        } else {
            Some(*self % *v)
        }
    }
}

impl CheckedNeg for Fixed {
    /// Negates the value, returning [`None`] for [`Fixed::MIN`], which has no
    /// positive counterpart.
    #[inline]
    fn checked_neg(&self) -> Option<Self> {
        self.checked_abs().map(|value| -value)
    }
}

impl Pow<u32> for Fixed {
    /// The resulting type after the exponentiation.
    type Output = Self;

    /// Raises the value to the given power.
    ///
    /// The intermediate results follow the semantics of the operators, so the
    /// overflow panics. The value `0**0` is `1`, like in the other numeric
    /// types.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed96q32::Fixed;
    /// use fixed96q32::num_traits::Pow;
    ///
    /// assert_eq!(Pow::pow(Fixed::from_int(2), 10), Fixed::from_int(1024));
    /// ```
    fn pow(self, rhs: u32) -> Self::Output {
        if rhs == 0 {
            return Self::ONE;
        }

        let mut base = self;
        let mut exponent = rhs;
        let mut result = Self::ONE;

        while exponent > 0 {
            if exponent & 1 == 1 {
                result *= base;
            }
            exponent >>= 1;
            if exponent > 0 {
                base *= base;
            }
        }

        result
    }
}

impl PowI for Fixed {
    /// The resulting type after the exponentiation.
    type Output = Self;

    /// Raises the value to the given power, including the negative powers.
    ///
    /// The negative power is the reciprocal of the positive one; the result
    /// saturates when the reciprocal is not representable.
    fn powi(self, exp: i32) -> Self::Output {
        if exp >= 0 {
            return Pow::pow(self, exp as u32);
        }

        let magnitude = Pow::pow(self, exp.unsigned_abs());
        match magnitude.recip() {
            Some(value) => value,
            None => {
                if self.is_negative() {
                    Self::MIN
                } else {
                    Self::MAX
                }
            }
        }
    }
}

impl Inv for Fixed {
    /// The reciprocal of a value is either a value or nothing.
    type Output = Option<Self>;

    /// Returns the reciprocal of the value, or [`None`] when the value is
    /// zero.
    #[inline]
    fn inv(self) -> Self::Output {
        self.recip()
    }
}

impl MulAdd for Fixed {
    /// The resulting type after the fused multiply-add.
    type Output = Self;

    /// Computes `(self * a) + b`.
    ///
    /// The type has no fused instruction, therefore the product is rounded
    /// before the addition, exactly like the ordinary operators do. The
    /// overflow panics, like the operators do.
    fn mul_add(self, a: Self, b: Self) -> Self::Output {
        self * a + b
    }
}

impl Euclid for Fixed {
    /// Divides the value by `v` in the Euclidean style.
    ///
    /// The quotient is the unique integer `q` such that
    /// `0 <= self - q * v < v.abs()`.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    fn div_euclid(&self, v: &Self) -> Self {
        assert!(
            !Fixed::is_zero(*v),
            "attempt to divide a Fixed value by zero"
        );
        let quotient = euclid_quotient(self.to_raw(), v.to_raw());
        Fixed::from_raw(quotient << 32)
    }

    /// Returns the least non-negative remainder of the division by `v`.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    fn rem_euclid(&self, v: &Self) -> Self {
        assert!(
            !Fixed::is_zero(*v),
            "attempt to calculate the remainder of a division by zero"
        );
        let remainder = euclid_remainder(self.to_raw(), v.to_raw());
        Fixed::from_raw(remainder as i128)
    }
}

impl Saturating for Fixed {
    /// Adds two values, clamping the result to the representable range.
    #[inline]
    fn saturating_add(self, v: Self) -> Self {
        Fixed::saturating_add(self, v)
    }

    /// Subtracts two values, clamping the result to the representable range.
    #[inline]
    fn saturating_sub(self, v: Self) -> Self {
        Fixed::saturating_sub(self, v)
    }
}

impl SaturatingAdd for Fixed {
    /// Adds two values, clamping the result to the representable range.
    #[inline]
    fn saturating_add(&self, v: &Self) -> Self {
        Fixed::saturating_add(*self, *v)
    }
}

impl SaturatingSub for Fixed {
    /// Subtracts two values, clamping the result to the representable range.
    #[inline]
    fn saturating_sub(&self, v: &Self) -> Self {
        Fixed::saturating_sub(*self, *v)
    }
}

impl SaturatingMul for Fixed {
    /// Multiplies two values, clamping the result to the representable range.
    #[inline]
    fn saturating_mul(&self, v: &Self) -> Self {
        Fixed::saturating_mul(*self, *v)
    }
}

impl OverflowingAdd for Fixed {
    /// Adds two values, returning the wrapped result and the overflow flag.
    #[inline]
    fn overflowing_add(&self, v: &Self) -> (Self, bool) {
        Fixed::overflowing_add(*self, *v)
    }
}

impl OverflowingSub for Fixed {
    /// Subtracts two values, returning the wrapped result and the overflow
    /// flag.
    #[inline]
    fn overflowing_sub(&self, v: &Self) -> (Self, bool) {
        Fixed::overflowing_sub(*self, *v)
    }
}

impl OverflowingMul for Fixed {
    /// Multiplies two values, returning the wrapped result and the overflow
    /// flag.
    #[inline]
    fn overflowing_mul(&self, v: &Self) -> (Self, bool) {
        Fixed::overflowing_mul(*self, *v)
    }
}

impl WrappingAdd for Fixed {
    /// Adds two values, wrapping the result on overflow.
    #[inline]
    fn wrapping_add(&self, v: &Self) -> Self::Output {
        Fixed::wrapping_add(*self, *v)
    }
}

impl WrappingSub for Fixed {
    /// Subtracts two values, wrapping the result on overflow.
    #[inline]
    fn wrapping_sub(&self, v: &Self) -> Self::Output {
        Fixed::wrapping_sub(*self, *v)
    }
}

impl WrappingMul for Fixed {
    /// Multiplies two values, wrapping the result on overflow.
    #[inline]
    fn wrapping_mul(&self, v: &Self) -> Self::Output {
        Fixed::wrapping_mul(*self, *v)
    }
}

impl WrappingNeg for Fixed {
    /// Negates the value, wrapping the result on overflow.
    #[inline]
    fn wrapping_neg(&self) -> Self {
        Fixed::wrapping_neg(*self)
    }
}

/// Returns the integral part of the value, truncated towards zero.
fn truncated_integer(value: Fixed) -> i128 {
    value.trunc().integer_part()
}

/// Returns the Euclidean quotient of two raw values.
///
/// The quotient is the unique integer `q` such that
/// `0 <= value - q * divisor < divisor.abs()`. The result is clamped to the
/// integral range of [`Fixed`].
fn euclid_quotient(value: i128, divisor: i128) -> i128 {
    let magnitude = divisor.unsigned_abs();
    let remainder = euclid_remainder(value, divisor);

    let quotient = if value < 0 {
        (value.unsigned_abs() + remainder) / magnitude
    } else {
        (value.unsigned_abs() - remainder) / magnitude
    };

    let negative = (value < 0) != (divisor < 0);
    if negative {
        if quotient > MIN_INTEGER.unsigned_abs() {
            MIN_INTEGER
        } else {
            -(quotient as i128)
        }
    } else if quotient > MAX_INTEGER.unsigned_abs() {
        MAX_INTEGER
    } else {
        quotient as i128
    }
}

/// Returns the least non-negative remainder of the division of two raw values.
///
/// The divisor must not be zero.
fn euclid_remainder(value: i128, divisor: i128) -> u128 {
    let magnitude = divisor.unsigned_abs();
    let remainder = value.unsigned_abs() % magnitude;
    if value < 0 && remainder != 0 {
        magnitude - remainder
    } else {
        remainder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::ToString;

    #[test]
    fn the_identities_are_the_constants() {
        assert_eq!(Fixed::zero(), Fixed::ZERO);
        assert_eq!(Fixed::one(), Fixed::ONE);
        assert!(<Fixed as Zero>::is_zero(&Fixed::ZERO));
        assert!(!<Fixed as Zero>::is_zero(&Fixed::ONE));
    }

    #[test]
    fn the_decimal_radix_is_supported() {
        let value = <Fixed as Num>::from_str_radix("0.004", 10).unwrap();
        assert_eq!(value, Fixed::from_parts(0, 17_179_869));
        assert!(<Fixed as Num>::from_str_radix("abc", 10).is_err());
    }

    #[test]
    fn the_other_radices_are_rejected() {
        let error = <Fixed as Num>::from_str_radix("10", 16).unwrap_err();
        assert_eq!(error, ParseError::InvalidCharacter);
        assert_eq!(error.to_string(), "the input contains an invalid character");
    }

    #[test]
    fn the_bounds_match_the_constants() {
        assert_eq!(<Fixed as Bounded>::min_value(), Fixed::MIN);
        assert_eq!(<Fixed as Bounded>::max_value(), Fixed::MAX);
    }

    #[test]
    fn the_sign_helpers_follow_the_sign() {
        assert!(!<Fixed as Signed>::is_positive(&Fixed::ZERO));
        assert!(!<Fixed as Signed>::is_negative(&Fixed::ZERO));
        assert!(<Fixed as Signed>::is_positive(&Fixed::ONE));
        assert!(<Fixed as Signed>::is_negative(&Fixed::NEG_ONE));
        assert_eq!(
            <Fixed as Signed>::signum(&Fixed::from_int(-5)),
            Fixed::NEG_ONE
        );
    }

    #[test]
    fn the_absolute_value_is_clamped() {
        assert_eq!(
            <Fixed as Signed>::abs(&Fixed::from_parts(-3, 5)),
            Fixed::from_parts(3, 5)
        );
        assert_eq!(<Fixed as Signed>::abs(&Fixed::ZERO), Fixed::ZERO);
        assert_eq!(<Fixed as Signed>::abs(&Fixed::MIN), Fixed::MAX);
    }

    #[test]
    fn the_positive_difference_is_computed() {
        let value = Fixed::from_int(7);
        let other = Fixed::from_int(3);
        assert_eq!(
            <Fixed as Signed>::abs_sub(&value, &other),
            Fixed::from_int(4)
        );
        assert_eq!(<Fixed as Signed>::abs_sub(&other, &value), Fixed::ZERO);
        assert_eq!(
            <Fixed as Signed>::abs_sub(&Fixed::NEG_ONE, &other),
            Fixed::ZERO
        );
    }

    #[test]
    fn the_sign_bit_is_inspected() {
        assert!(!<Fixed as SignedExt>::is_negative_zero(&Fixed::ZERO));
        assert!(!<Fixed as SignedExt>::is_sign_negative(&Fixed::ZERO));
        assert!(<Fixed as SignedExt>::is_sign_positive(&Fixed::ZERO));
        assert!(<Fixed as SignedExt>::is_sign_negative(&Fixed::NEG_ONE));
        assert!(<Fixed as SignedExt>::is_sign_negative(&Fixed::MIN));
        assert!(<Fixed as SignedExt>::is_sign_positive(&Fixed::MAX));
    }

    #[test]
    fn the_conversions_truncate_towards_zero() {
        let value = Fixed::from_parts(2, 7);
        assert_eq!(<Fixed as ToPrimitive>::to_i64(&value), Some(2));
        assert_eq!(<Fixed as ToPrimitive>::to_i128(&value), Some(2));
        assert_eq!(<Fixed as ToPrimitive>::to_i32(&value), Some(2));
        assert_eq!(
            <Fixed as ToPrimitive>::to_f64(&value),
            Some(2.0000000016298145)
        );
        assert_eq!(
            <Fixed as ToPrimitive>::to_i64(&Fixed::from_parts(-2, 7)),
            Some(-2)
        );
        assert_eq!(
            <Fixed as ToPrimitive>::to_f32(&Fixed::from_parts(0, 1 << 31)),
            Some(0.5)
        );
    }

    #[test]
    fn the_conversions_reject_the_out_of_range_values() {
        assert_eq!(<Fixed as ToPrimitive>::to_i64(&Fixed::MAX), None);
        assert_eq!(<Fixed as ToPrimitive>::to_i64(&Fixed::MIN), None);
        assert_eq!(<Fixed as ToPrimitive>::to_u64(&Fixed::NEG_ONE), None);
        assert_eq!(<Fixed as ToPrimitive>::to_u8(&Fixed::from_int(256)), None);
        assert_eq!(<Fixed as ToPrimitive>::to_usize(&Fixed::from_int(-1)), None);
        assert_eq!(
            <Fixed as ToPrimitive>::to_f64(&Fixed::MAX),
            Some(Fixed::MAX.to_f64())
        );
    }

    #[test]
    fn the_primitive_conversions_are_exact() {
        assert_eq!(
            <Fixed as FromPrimitive>::from_i64(-7),
            Some(Fixed::from_int(-7))
        );
        assert_eq!(
            <Fixed as FromPrimitive>::from_u64(7),
            Some(Fixed::from_int(7))
        );
        assert_eq!(<Fixed as FromPrimitive>::from_f64(0.5), Some(Fixed::HALF));
        assert_eq!(
            <Fixed as FromPrimitive>::from_f32(0.25),
            Some(Fixed::from_parts(0, 1_073_741_824))
        );
        assert_eq!(<Fixed as FromPrimitive>::from_i128(1i128 << 95), None);
        assert_eq!(<Fixed as FromPrimitive>::from_u128(1u128 << 95), None);
    }

    #[test]
    fn the_numeric_cast_keeps_the_fraction() {
        assert_eq!(
            <Fixed as NumCast>::from(1.5f64),
            Some(Fixed::from_parts(1, 2_147_483_648))
        );
        assert_eq!(<Fixed as NumCast>::from(-3i64), Some(Fixed::from_int(-3)));
        assert_eq!(<Fixed as NumCast>::from(0.5f32), Some(Fixed::HALF));
    }

    #[test]
    fn the_checked_operations_report_the_overflow() {
        assert_eq!(
            <Fixed as CheckedAdd>::checked_add(&Fixed::ONE, &Fixed::ONE),
            Some(Fixed::from_int(2))
        );
        assert_eq!(
            <Fixed as CheckedSub>::checked_sub(&Fixed::MIN, &Fixed::ONE),
            None
        );
        assert_eq!(
            <Fixed as CheckedMul>::checked_mul(&Fixed::MAX, &Fixed::from_int(2)),
            None
        );
        assert_eq!(
            <Fixed as CheckedDiv>::checked_div(&Fixed::ONE, &Fixed::ZERO),
            None
        );
        assert_eq!(
            <Fixed as CheckedRem>::checked_rem(&Fixed::ONE, &Fixed::ZERO),
            None
        );
        assert_eq!(
            <Fixed as CheckedRem>::checked_rem(&Fixed::from_int(7), &Fixed::from_int(3)),
            Some(Fixed::ONE)
        );
        assert_eq!(
            <Fixed as CheckedNeg>::checked_neg(&Fixed::ONE),
            Some(Fixed::NEG_ONE)
        );
        assert_eq!(<Fixed as CheckedNeg>::checked_neg(&Fixed::MIN), None);
    }

    #[test]
    fn the_power_is_computed() {
        assert_eq!(Pow::pow(Fixed::from_int(2), 10), Fixed::from_int(1024));
        assert_eq!(Pow::pow(Fixed::from_int(2), 0), Fixed::ONE);
        assert_eq!(Pow::pow(Fixed::ZERO, 0), Fixed::ONE);
        assert_eq!(Pow::pow(Fixed::from_int(-2), 3), Fixed::from_int(-8));
    }

    #[test]
    #[should_panic(expected = "attempt to multiply two Fixed values with overflow")]
    fn the_power_panics_on_overflow() {
        let _ = Pow::pow(Fixed::MAX, 2);
    }

    #[test]
    fn the_signed_power_is_computed() {
        assert_eq!(
            <Fixed as PowI>::powi(Fixed::from_int(2), 3),
            Fixed::from_int(8)
        );
        assert_eq!(<Fixed as PowI>::powi(Fixed::from_int(2), -1), Fixed::HALF);
        assert_eq!(<Fixed as PowI>::powi(Fixed::NEG_ONE, -1), Fixed::NEG_ONE);
        assert_eq!(<Fixed as PowI>::powi(Fixed::ZERO, -1), Fixed::MAX);
    }

    #[test]
    fn the_reciprocal_is_computed() {
        assert_eq!(
            Inv::inv(Fixed::from_int(4)),
            Some(Fixed::from_parts(0, 1 << 30))
        );
        assert_eq!(Inv::inv(Fixed::ZERO), None);
    }

    #[test]
    fn the_fused_multiply_add_is_computed() {
        assert_eq!(
            MulAdd::mul_add(Fixed::from_int(1), Fixed::from_int(2), Fixed::from_int(3)),
            Fixed::from_int(5)
        );
    }

    #[test]
    #[should_panic(expected = "attempt to multiply two Fixed values with overflow")]
    fn the_fused_multiply_add_panics_on_overflow() {
        let _ = MulAdd::mul_add(Fixed::MAX, Fixed::from_int(2), Fixed::ZERO);
    }

    #[test]
    fn the_euclidean_division_is_computed() {
        let seven = Fixed::from_int(7);
        let four = Fixed::from_int(4);
        assert_eq!(Euclid::div_euclid(&seven, &four), Fixed::from_int(1));
        assert_eq!(
            Euclid::div_euclid(&Fixed::NEG_ONE, &four),
            Fixed::from_int(-1)
        );
        assert_eq!(
            Euclid::div_euclid(&seven, &Fixed::NEG_ONE),
            Fixed::from_int(-7)
        );
        assert_eq!(
            Euclid::div_euclid(&Fixed::NEG_ONE, &Fixed::NEG_ONE),
            Fixed::ONE
        );
        assert_eq!(Euclid::rem_euclid(&seven, &four), Fixed::from_int(3));
        assert_eq!(
            Euclid::rem_euclid(&Fixed::NEG_ONE, &four),
            Fixed::from_int(3)
        );
        assert_eq!(Euclid::rem_euclid(&seven, &Fixed::NEG_ONE), Fixed::ZERO);
        assert_eq!(
            Euclid::rem_euclid(&Fixed::NEG_ONE, &Fixed::NEG_ONE),
            Fixed::ZERO
        );
    }

    #[test]
    fn the_euclidean_division_handles_the_fractions() {
        let value = Fixed::from_parts(7, 500_000_000);
        let divisor = Fixed::from_int(2);
        assert_eq!(Euclid::div_euclid(&value, &divisor), Fixed::from_int(3));
        assert_eq!(
            Euclid::rem_euclid(&value, &divisor),
            Fixed::from_parts(1, 500_000_000)
        );
        assert_eq!(
            Euclid::rem_euclid(&Fixed::from_parts(-7, 500_000_000), &divisor),
            Fixed::from_parts(0, 3_794_967_296)
        );
    }

    #[test]
    fn the_euclidean_division_survives_the_extremes() {
        assert_eq!(Euclid::rem_euclid(&Fixed::MIN, &Fixed::ONE), Fixed::ZERO);
        assert_eq!(Euclid::div_euclid(&Fixed::MIN, &Fixed::ONE), Fixed::MIN);
        assert_eq!(
            Euclid::div_euclid(&Fixed::MAX, &Fixed::ONE),
            Fixed::MAX.trunc()
        );
    }

    #[test]
    fn the_euclidean_division_keeps_the_invariants() {
        let values = [
            Fixed::from_parts(7, 500_000_000),
            Fixed::from_parts(-7, 500_000_000),
            Fixed::from_int(7),
            Fixed::from_int(-7),
            Fixed::HALF,
            Fixed::from_raw(-1),
            Fixed::ZERO,
        ];
        let divisors = [
            Fixed::from_int(2),
            Fixed::from_int(-2),
            Fixed::from_int(3),
            Fixed::from_int(-1),
            Fixed::HALF,
        ];

        for value in values {
            for divisor in divisors {
                let quotient = Euclid::div_euclid(&value, &divisor);
                let remainder = Euclid::rem_euclid(&value, &divisor);

                assert!(!remainder.is_negative());
                assert!(remainder < divisor.abs());
                assert_eq!(value, quotient * divisor + remainder);
            }
        }
    }

    #[test]
    fn the_saturating_operations_clamp_the_result() {
        assert_eq!(
            <Fixed as Saturating>::saturating_add(Fixed::MAX, Fixed::ONE),
            Fixed::MAX
        );
        assert_eq!(
            <Fixed as Saturating>::saturating_sub(Fixed::MIN, Fixed::ONE),
            Fixed::MIN
        );
        assert_eq!(
            <Fixed as SaturatingAdd>::saturating_add(&Fixed::MAX, &Fixed::ONE),
            Fixed::MAX
        );
        assert_eq!(
            <Fixed as SaturatingSub>::saturating_sub(&Fixed::MIN, &Fixed::ONE),
            Fixed::MIN
        );
        assert_eq!(
            <Fixed as SaturatingMul>::saturating_mul(&Fixed::MAX, &Fixed::from_int(2)),
            Fixed::MAX
        );
    }

    #[test]
    fn the_overflowing_operations_report_the_overflow() {
        assert_eq!(
            <Fixed as OverflowingAdd>::overflowing_add(&Fixed::ONE, &Fixed::ONE),
            (Fixed::from_int(2), false)
        );
        assert_eq!(
            <Fixed as OverflowingAdd>::overflowing_add(&Fixed::MAX, &Fixed::ONE),
            (Fixed::MAX.wrapping_add(Fixed::ONE), true)
        );
        assert_eq!(
            <Fixed as OverflowingSub>::overflowing_sub(&Fixed::MIN, &Fixed::ONE),
            (Fixed::MIN.wrapping_sub(Fixed::ONE), true)
        );
        assert_eq!(
            <Fixed as OverflowingMul>::overflowing_mul(&Fixed::MAX, &Fixed::from_int(2)),
            (Fixed::MAX.wrapping_mul(Fixed::from_int(2)), true)
        );
    }

    #[test]
    fn the_wrapping_operations_wrap_the_result() {
        assert_eq!(
            <Fixed as WrappingAdd>::wrapping_add(&Fixed::ONE, &Fixed::ONE),
            Fixed::from_int(2)
        );
        assert_eq!(
            <Fixed as WrappingAdd>::wrapping_add(&Fixed::MAX, &Fixed::ONE),
            Fixed::MAX.wrapping_add(Fixed::ONE)
        );
        assert_eq!(
            <Fixed as WrappingSub>::wrapping_sub(&Fixed::MIN, &Fixed::ONE),
            Fixed::MIN.wrapping_sub(Fixed::ONE)
        );
        assert_eq!(
            <Fixed as WrappingMul>::wrapping_mul(&Fixed::MAX, &Fixed::from_int(2)),
            Fixed::MAX.wrapping_mul(Fixed::from_int(2))
        );
        assert_eq!(
            <Fixed as WrappingNeg>::wrapping_neg(&Fixed::ONE),
            Fixed::NEG_ONE
        );
        assert_eq!(
            <Fixed as WrappingNeg>::wrapping_neg(&Fixed::MIN),
            Fixed::MIN
        );
    }
}
