//! Conversions between the [`Fixed`] type and the primitive numeric types.

use core::fmt::{self, Display};

use crate::fixed::Fixed;
use crate::fmt::ParseError;

/// The error returned when a conversion cannot be performed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConversionError {
    /// The value does not fit into the representable range.
    OutOfRange,
    /// The value is not a number.
    NotANumber,
    /// The textual representation of the value is invalid.
    InvalidFormat,
}

impl Display for ConversionError {
    /// Writes a human readable description of the error.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            ConversionError::OutOfRange => "the value does not fit into the representable range",
            ConversionError::NotANumber => "the value is not a number",
            ConversionError::InvalidFormat => "the textual representation of the value is invalid",
        };
        formatter.write_str(message)
    }
}

impl core::error::Error for ConversionError {}

impl From<i8> for Fixed {
    /// Converts a byte-sized integer, which is always exact.
    #[inline]
    fn from(value: i8) -> Self {
        Fixed::from_int(value as i64)
    }
}

impl From<i16> for Fixed {
    /// Converts a short integer, which is always exact.
    #[inline]
    fn from(value: i16) -> Self {
        Fixed::from_int(value as i64)
    }
}

impl From<i32> for Fixed {
    /// Converts an integer, which is always exact.
    #[inline]
    fn from(value: i32) -> Self {
        Fixed::from_int(value as i64)
    }
}

impl From<i64> for Fixed {
    /// Converts a long integer, which is always exact.
    #[inline]
    fn from(value: i64) -> Self {
        Fixed::from_int(value)
    }
}

impl From<u8> for Fixed {
    /// Converts an unsigned byte, which is always exact.
    #[inline]
    fn from(value: u8) -> Self {
        Fixed::from_raw((value as i128) << 32)
    }
}

impl From<u16> for Fixed {
    /// Converts an unsigned short, which is always exact.
    #[inline]
    fn from(value: u16) -> Self {
        Fixed::from_raw((value as i128) << 32)
    }
}

impl From<u32> for Fixed {
    /// Converts an unsigned integer, which is always exact.
    #[inline]
    fn from(value: u32) -> Self {
        Fixed::from_raw((value as i128) << 32)
    }
}

impl From<u64> for Fixed {
    /// Converts an unsigned long, which is always exact.
    #[inline]
    fn from(value: u64) -> Self {
        Fixed::from_raw((value as i128) << 32)
    }
}

impl TryFrom<i128> for Fixed {
    /// Converts a 128-bit integer, which may not fit into the range of the
    /// type.
    type Error = ConversionError;

    #[inline]
    fn try_from(value: i128) -> Result<Self, Self::Error> {
        let limit = 1i128 << 95;
        if value < -limit || value >= limit {
            Err(ConversionError::OutOfRange)
        } else {
            Ok(Fixed::from_raw(value << 32))
        }
    }
}

impl TryFrom<u128> for Fixed {
    /// Converts an unsigned 128-bit integer, which may not fit into the range
    /// of the type.
    type Error = ConversionError;

    #[inline]
    fn try_from(value: u128) -> Result<Self, Self::Error> {
        let limit = 1u128 << 95;
        if value >= limit {
            Err(ConversionError::OutOfRange)
        } else {
            Ok(Fixed::from_raw((value as i128) << 32))
        }
    }
}

impl TryFrom<f32> for Fixed {
    /// Converts a single precision float, rounding to the nearest
    /// representable value.
    ///
    /// # Errors
    ///
    /// Returns [`ConversionError::NotANumber`] for `NaN` and
    /// [`ConversionError::OutOfRange`] for the infinities and for the values
    /// outside of the representable range.
    type Error = ConversionError;

    #[inline]
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        from_floating_point(value as f64)
    }
}

impl TryFrom<f64> for Fixed {
    /// Converts a double precision float, rounding to the nearest
    /// representable value.
    ///
    /// # Errors
    ///
    /// Returns [`ConversionError::NotANumber`] for `NaN` and
    /// [`ConversionError::OutOfRange`] for the infinities and for the values
    /// outside of the representable range.
    type Error = ConversionError;

    #[inline]
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        from_floating_point(value)
    }
}

impl TryFrom<&str> for Fixed {
    /// Parses a decimal number, for example `0.004`.
    ///
    /// # Errors
    ///
    /// Returns [`ConversionError::InvalidFormat`] if the text is not a valid
    /// decimal number and [`ConversionError::OutOfRange`] if the value does
    /// not fit into the representable range.
    type Error = ConversionError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse::<Fixed>().map_err(|error| match error {
            ParseError::OutOfRange => ConversionError::OutOfRange,
            _ => ConversionError::InvalidFormat,
        })
    }
}

impl From<Fixed> for f64 {
    /// Converts a fixed-point number into a double precision float, losing
    /// precision for the large values.
    #[inline]
    fn from(value: Fixed) -> Self {
        value.to_f64()
    }
}

impl From<Fixed> for f32 {
    /// Converts a fixed-point number into a single precision float, losing
    /// precision
    #[inline]
    fn from(value: Fixed) -> Self {
        value.to_f32()
    }
}

/// Converts a floating point value into a fixed-point number.
fn from_floating_point(value: f64) -> Result<Fixed, ConversionError> {
    if value.is_nan() {
        return Err(ConversionError::NotANumber);
    }
    if value.is_infinite() {
        return Err(ConversionError::OutOfRange);
    }

    let scaled = value * 4_294_967_296.0;
    let limit = 170_141_183_460_469_231_731_687_303_715_884_105_728.0;
    if scaled >= limit || scaled < -limit {
        return Err(ConversionError::OutOfRange);
    }

    let truncated = scaled as i128;
    let fraction = scaled - truncated as f64;
    let rounded = if fraction >= 0.5 {
        truncated + 1
    } else if fraction <= -0.5 {
        truncated - 1
    } else {
        truncated
    };

    Ok(Fixed::from_raw(rounded))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::ToString;

    #[test]
    fn signed_integers_are_converted_exactly() {
        assert_eq!(Fixed::from(0i8), Fixed::ZERO);
        assert_eq!(Fixed::from(-1i16), Fixed::NEG_ONE);
        assert_eq!(Fixed::from(123i32), Fixed::from_int(123));
        assert_eq!(Fixed::from(-123i64), Fixed::from_int(-123));
    }

    #[test]
    fn unsigned_integers_are_converted_exactly() {
        assert_eq!(Fixed::from(0u8), Fixed::ZERO);
        assert_eq!(Fixed::from(255u16), Fixed::from_int(255));
        assert_eq!(
            Fixed::from(4_294_967_295u32),
            Fixed::from_int(4_294_967_295)
        );
        assert_eq!(
            Fixed::from(u64::MAX),
            Fixed::from_raw((u64::MAX as i128) << 32)
        );
    }

    #[test]
    fn wide_integers_report_overflow() {
        assert_eq!(
            Fixed::try_from(1i128 << 95),
            Err(ConversionError::OutOfRange)
        );
        assert_eq!(
            Fixed::try_from(-(1i128 << 95) - 1),
            Err(ConversionError::OutOfRange)
        );
        assert_eq!(
            Fixed::try_from((1i128 << 95) - 1),
            Ok(Fixed::from_raw(((1i128 << 95) - 1) << 32))
        );
        assert_eq!(
            Fixed::try_from(1u128 << 95),
            Err(ConversionError::OutOfRange)
        );
    }

    #[test]
    fn floats_are_rounded_to_the_nearest_value() {
        assert_eq!(Fixed::try_from(0.5f64), Ok(Fixed::HALF));
        assert_eq!(
            Fixed::try_from(-0.5f64),
            Ok(Fixed::from_raw(-(1i128 << 31)))
        );
        assert_eq!(Fixed::try_from(1.0f64), Ok(Fixed::ONE));
        assert_eq!(
            Fixed::try_from(0.25f32),
            Ok(Fixed::from_parts(0, 1_073_741_824))
        );
    }

    #[test]
    fn floats_report_invalid_values() {
        assert_eq!(Fixed::try_from(f64::NAN), Err(ConversionError::NotANumber));
        assert_eq!(
            Fixed::try_from(f64::INFINITY),
            Err(ConversionError::OutOfRange)
        );
        assert_eq!(
            Fixed::try_from(f64::NEG_INFINITY),
            Err(ConversionError::OutOfRange)
        );
        assert_eq!(Fixed::try_from(1e300f64), Err(ConversionError::OutOfRange));
    }

    #[test]
    fn strings_are_parsed() {
        assert_eq!(
            Fixed::try_from("1.5"),
            Ok(Fixed::from_parts(1, 2_147_483_648))
        );
        assert_eq!(Fixed::try_from("abc"), Err(ConversionError::InvalidFormat));
        assert_eq!(
            Fixed::try_from("99999999999999999999999999999999999999999"),
            Err(ConversionError::OutOfRange)
        );
    }

    #[test]
    fn fixed_numbers_are_converted_into_floats() {
        assert_eq!(f64::from(Fixed::HALF), 0.5);
        assert_eq!(f32::from(Fixed::ONE), 1.0);
    }

    #[test]
    fn conversion_errors_are_displayed() {
        assert_eq!(
            ConversionError::OutOfRange.to_string(),
            "the value does not fit into the representable range"
        );
        assert_eq!(
            ConversionError::NotANumber.to_string(),
            "the value is not a number"
        );
        assert_eq!(
            ConversionError::InvalidFormat.to_string(),
            "the textual representation of the value is invalid"
        );
    }
}
