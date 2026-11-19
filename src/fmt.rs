//! Formatting and parsing of the [`Fixed`] type.

use core::fmt::{self, Debug, Display};
use core::str::FromStr;

use crate::fixed::Fixed;

/// Mask of the fractional part of the raw magnitude.
const FRACTION_MASK: u128 = (1u128 << 32) - 1;

/// The error returned when a string cannot be parsed into a [`Fixed`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParseError {
    /// The input does not contain any digit.
    Empty,
    /// The input contains a character that is not allowed at its position.
    InvalidCharacter,
    /// The fractional part does not fit into 32 bits.
    TooManyFractionalDigits,
    /// The value does not fit into the representable range.
    OutOfRange,
}

impl Display for ParseError {
    /// Writes a human readable description of the error.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            ParseError::Empty => "the input does not contain any digit",
            ParseError::InvalidCharacter => "the input contains an invalid character",
            ParseError::TooManyFractionalDigits => "the fractional part does not fit into 32 bits",
            ParseError::OutOfRange => "the value does not fit into the representable range",
        };
        formatter.write_str(message)
    }
}

impl core::error::Error for ParseError {}

impl Display for Fixed {
    /// Formats the value as a decimal number, for example `0.004`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed96q32::Fixed;
    ///
    /// assert_eq!(Fixed::from_parts(0, 17_179_869).to_string(), "0.004");
    /// assert_eq!(Fixed::from_int(-7).to_string(), "-7");
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let negative = self.is_negative();
        let magnitude = self.to_raw().unsigned_abs();

        let integer = magnitude >> 32;
        let fraction = magnitude & FRACTION_MASK;

        if negative {
            formatter.write_str("-")?;
        }
        write_u128(formatter, integer)?;

        if fraction != 0 {
            let (digits, length) = shortest_fraction(fraction);
            formatter.write_str(".")?;
            let text = core::str::from_utf8(&digits[..length]).map_err(|_| fmt::Error)?;
            formatter.write_str(text)?;
        }

        Ok(())
    }
}

impl Debug for Fixed {
    /// Formats the value in the same way as [`Display`].
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self, formatter)
    }
}

impl FromStr for Fixed {
    /// Parses a decimal number, for example `0.004` or `-1.5e3`.
    ///
    /// The fractional part must not exceed 32 digits and the value must fit
    /// into the representable range.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::str::FromStr;
    /// use fixed96q32::Fixed;
    ///
    /// assert_eq!("0.004".parse::<Fixed>().unwrap(), Fixed::from_parts(0, 17_179_869));
    /// ```
    type Err = ParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let bytes = text.as_bytes();
        let mut index = 0;

        let mut negative = false;
        if index < bytes.len() && (bytes[index] == b'+' || bytes[index] == b'-') {
            negative = bytes[index] == b'-';
            index += 1;
        }

        let mut integer: u128 = 0;
        let mut digits = 0;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            integer = integer
                .checked_mul(10)
                .and_then(|value| value.checked_add((bytes[index] - b'0') as u128))
                .ok_or(ParseError::OutOfRange)?;
            digits += 1;
            index += 1;
        }

        let mut fraction: u128 = 0;
        let mut denominator: u128 = 1;
        let mut fraction_digits = 0;
        if index < bytes.len() && bytes[index] == b'.' {
            index += 1;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                if fraction_digits == 32 {
                    return Err(ParseError::TooManyFractionalDigits);
                }
                fraction = fraction * 10 + (bytes[index] - b'0') as u128;
                denominator *= 10;
                fraction_digits += 1;
                digits += 1;
                index += 1;
            }
        }
        if digits == 0 {
            return Err(ParseError::Empty);
        }

        let mut exponent: i32 = 0;
        if index < bytes.len() && (bytes[index] == b'e' || bytes[index] == b'E') {
            index += 1;
            let mut exponent_negative = false;
            if index < bytes.len() && (bytes[index] == b'+' || bytes[index] == b'-') {
                exponent_negative = bytes[index] == b'-';
                index += 1;
            }
            let mut exponent_digits = 0;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                exponent = exponent
                    .checked_mul(10)
                    .and_then(|value| value.checked_add((bytes[index] - b'0') as i32))
                    .ok_or(ParseError::OutOfRange)?;
                exponent_digits += 1;
                index += 1;
            }
            if exponent_digits == 0 {
                return Err(ParseError::InvalidCharacter);
            }
            if exponent_negative {
                exponent = -exponent;
            }
        }

        if index != bytes.len() {
            return Err(ParseError::InvalidCharacter);
        }

        let fraction = scale_fraction(fraction, denominator);
        let integer = integer
            .checked_mul(1u128 << 32)
            .and_then(|value| value.checked_add(fraction))
            .ok_or(ParseError::OutOfRange)?;

        let raw = apply_exponent(integer, exponent)?;

        if negative {
            if raw > (1u128 << 127) {
                return Err(ParseError::OutOfRange);
            }
            Ok(Fixed::from_raw((raw as i128).wrapping_neg()))
        } else if raw > i128::MAX as u128 {
            Err(ParseError::OutOfRange)
        } else {
            Ok(Fixed::from_raw(raw as i128))
        }
    }
}

/// Converts a fractional part into the raw representation, truncating the
/// discarded bits.
fn scale_fraction(fraction: u128, denominator: u128) -> u128 {
    if fraction == 0 {
        return 0;
    }

    // The fraction is split in two halves to keep every intermediate value
    // within the range of `u128`.
    let half = (fraction << 16) / denominator;
    let remainder = (fraction << 16) % denominator;
    (half << 16) + ((remainder << 16) / denominator)
}

/// Applies a decimal exponent to a raw value.
fn apply_exponent(raw: u128, exponent: i32) -> Result<u128, ParseError> {
    if exponent == 0 {
        return Ok(raw);
    }

    let (factor, multiply) = if exponent > 0 {
        (pow10(exponent as u32)?, true)
    } else {
        (pow10((-exponent) as u32)?, false)
    };

    if multiply {
        raw.checked_mul(factor).ok_or(ParseError::OutOfRange)
    } else {
        Ok(raw / factor)
    }
}

/// Computes `10**power`.
fn pow10(power: u32) -> Result<u128, ParseError> {
    let mut result: u128 = 1;
    for _ in 0..power {
        result = result.checked_mul(10).ok_or(ParseError::OutOfRange)?;
    }
    Ok(result)
}

/// Writes an unsigned integer in its decimal representation.
fn write_u128(formatter: &mut fmt::Formatter<'_>, mut value: u128) -> fmt::Result {
    let mut buffer = [0u8; 39];
    let mut index = buffer.len();

    loop {
        index -= 1;
        buffer[index] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }

    let digits = core::str::from_utf8(&buffer[index..]).map_err(|_| fmt::Error)?;
    formatter.write_str(digits)
}

/// Computes the exact decimal digits of a fractional part.
///
/// The fraction `1 / 2**32` has 32 decimal digits, because the denominator is
/// a power of two.
fn exact_fraction_digits(fraction: u128) -> [u8; 32] {
    let mut digits = [0u8; 32];
    let mut remainder = fraction;

    for digit in digits.iter_mut() {
        remainder *= 10;
        *digit = b'0' + (remainder >> 32) as u8;
        remainder &= FRACTION_MASK;
    }

    digits
}

/// Computes the shortest decimal fraction that is parsed back into the given
/// fractional part.
///
/// The digits are returned together with their length; the shortest
/// representation is the one that survives a round trip through
/// [`FromStr`].
fn shortest_fraction(fraction: u128) -> ([u8; 32], usize) {
    let exact = exact_fraction_digits(fraction);
    let mut digits = [0u8; 32];

    for length in 1..=32 {
        digits[..length].copy_from_slice(&exact[..length]);
        if should_round(&exact, length) {
            increment(&mut digits[..length]);
        }

        let value = digits_to_u128(&digits[..length]);
        if scale_fraction(value, power_of_ten(length)) == fraction {
            return (digits, length);
        }
    }

    (digits, 32)
}

/// Returns `true` if the digits after the given length have to round the
/// fraction up.
fn should_round(exact: &[u8; 32], length: usize) -> bool {
    match exact.get(length) {
        Some(digit) if *digit > b'5' => true,
        Some(digit) if *digit == b'5' => exact[length + 1..].iter().any(|digit| *digit != b'0'),
        _ => false,
    }
}

/// Increments a sequence of decimal digits, propagating the carry.
fn increment(digits: &mut [u8]) {
    for digit in digits.iter_mut().rev() {
        if *digit == b'9' {
            *digit = b'0';
        } else {
            *digit += 1;
            return;
        }
    }
}

/// Computes `10**power` for a power of at most 32.
fn power_of_ten(power: usize) -> u128 {
    let mut value: u128 = 1;
    for _ in 0..power {
        value *= 10;
    }
    value
}

/// Converts a sequence of decimal digits into an integer.
fn digits_to_u128(digits: &[u8]) -> u128 {
    let mut value: u128 = 0;
    for digit in digits {
        value = value * 10 + (digit - b'0') as u128;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::format;
    use std::string::ToString;

    #[test]
    fn display_prints_plain_decimals() {
        assert_eq!(Fixed::ZERO.to_string(), "0");
        assert_eq!(Fixed::ONE.to_string(), "1");
        assert_eq!(Fixed::from_int(-3).to_string(), "-3");
        assert_eq!(Fixed::from_parts(0, 17_179_869).to_string(), "0.004");
        assert_eq!(Fixed::HALF.to_string(), "0.5");
        assert_eq!(Fixed::from_parts(-1, 2_147_483_648).to_string(), "-1.5");
        assert_eq!(Fixed::MIN.to_string(), "-39614081257132168796771975168");
    }

    #[test]
    fn debug_matches_display() {
        assert_eq!(format!("{:?}", Fixed::HALF), format!("{}", Fixed::HALF));
    }

    #[test]
    fn parsing_reads_decimal_numbers() {
        assert_eq!("0".parse::<Fixed>().unwrap(), Fixed::ZERO);
        assert_eq!("1".parse::<Fixed>().unwrap(), Fixed::ONE);
        assert_eq!("-1".parse::<Fixed>().unwrap(), Fixed::NEG_ONE);
        assert_eq!("+2".parse::<Fixed>().unwrap(), Fixed::from_int(2));
        assert_eq!("0.5".parse::<Fixed>().unwrap(), Fixed::HALF);
        assert_eq!(
            "0.004".parse::<Fixed>().unwrap(),
            Fixed::from_parts(0, 17_179_869)
        );
        assert_eq!(
            "-1.5".parse::<Fixed>().unwrap(),
            Fixed::from_parts(-1, 2_147_483_648)
        );
    }

    #[test]
    fn parsing_round_trips() {
        for value in [
            Fixed::ZERO,
            Fixed::ONE,
            Fixed::NEG_ONE,
            Fixed::HALF,
            Fixed::from_parts(0, 17_179_869),
            Fixed::from_parts(-123, 456_789),
        ] {
            assert_eq!(value.to_string().parse::<Fixed>().unwrap(), value);
        }
    }

    #[test]
    fn parsing_supports_exponents() {
        assert_eq!("1e3".parse::<Fixed>().unwrap(), Fixed::from_int(1000));
        assert_eq!(
            "1e-3".parse::<Fixed>().unwrap(),
            Fixed::from_parts(0, 4_294_967)
        );
        assert_eq!("1.5e2".parse::<Fixed>().unwrap(), Fixed::from_int(150));
    }

    #[test]
    fn parsing_rejects_invalid_input() {
        assert_eq!("".parse::<Fixed>(), Err(ParseError::Empty));
        assert_eq!("-".parse::<Fixed>(), Err(ParseError::Empty));
        assert_eq!("abc".parse::<Fixed>(), Err(ParseError::Empty));
        assert_eq!("1.2.3".parse::<Fixed>(), Err(ParseError::InvalidCharacter));
        assert_eq!("1e".parse::<Fixed>(), Err(ParseError::InvalidCharacter));
        assert_eq!(
            "0.000000000000000000000000000000001".parse::<Fixed>(),
            Err(ParseError::TooManyFractionalDigits)
        );
    }

    #[test]
    fn parsing_rejects_out_of_range_values() {
        assert_eq!(
            "99999999999999999999999999999999999999".parse::<Fixed>(),
            Err(ParseError::OutOfRange)
        );
    }

    #[test]
    fn parse_errors_are_displayed() {
        assert_eq!(
            ParseError::Empty.to_string(),
            "the input does not contain any digit"
        );
        assert!(!ParseError::InvalidCharacter.to_string().is_empty());
    }
}
