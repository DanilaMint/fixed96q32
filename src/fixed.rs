//! The [`Fixed`] scalar: a signed fixed-point number with 32 fractional bits.

/// Mask of the fractional part of the raw magnitude.
const FRACTION_MASK_U128: u128 = (1u128 << 32) - 1;

/// Mask of the low 64 bits.
const MASK64: u128 = u64::MAX as u128;

const EPSILON_F64: f64 = 1. / (1u64 << Fixed::FRACTIONAL_BITS) as f64;
const EPSILON_F32: f32 = 1. / (1u64 << Fixed::FRACTIONAL_BITS) as f32;

/// A signed fixed-point number.
///
/// The value is stored as a 128-bit two's complement integer `raw` and the
/// represented number equals `raw / 2**32`. The layout consists of 1 sign bit,
/// 95 integral bits and 32 fractional bits:
///
/// ```text
///  127                    32 31                    0
/// ┌───┬────────────────────┬───┬────────────────────┐
/// │ S │     integer part   │ 0 │   fractional part  │
/// └───┴────────────────────┴───┴────────────────────┘
/// ```
///
/// # Examples
///
/// ```
/// use fixed96q32::Fixed;
///
/// let value = Fixed::from_parts(1, 2_147_483_648);
/// assert_eq!(value.to_string(), "1.5");
/// assert_eq!((Fixed::from_int(-2) + value).to_string(), "-0.5");
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Fixed {
    raw: i128,
}

impl Fixed {
    /// Number of bits reserved for the fractional part.
    pub const FRACTIONAL_BITS: u32 = 32;

    /// Number of bits reserved for the integral part, the sign bit excluded.
    pub const INTEGER_BITS: u32 = 95;

    /// Total width of the representation in bits.
    pub const TOTAL_BITS: u32 = 128;

    /// The value `0`.
    pub const ZERO: Self = Self { raw: 0 };

    /// The value `1`.
    pub const ONE: Self = Self { raw: 1i128 << 32 };

    /// The value `-1`.
    pub const NEG_ONE: Self = Self {
        raw: -(1i128 << 32),
    };

    /// The value `0.5`.
    pub const HALF: Self = Self { raw: 1i128 << 31 };

    /// The smallest positive representable value, `2**-32`.
    pub const EPSILON: Self = Self { raw: 1 };

    /// The smallest representable value, `-2**95`.
    pub const MIN: Self = Self { raw: i128::MIN };

    /// The largest representable value, `2**95 - 2**-32`.
    pub const MAX: Self = Self { raw: i128::MAX };

    /// Creates a value from its raw representation.
    ///
    /// The raw representation is the integer `raw` such that the value equals
    /// `raw / 2**32`.
    #[inline]
    pub const fn from_raw(raw: i128) -> Self {
        Self { raw }
    }

    /// Returns the raw representation of the value.
    #[inline]
    pub const fn to_raw(self) -> i128 {
        self.raw
    }

    /// Creates a value from its bit pattern.
    #[inline]
    pub const fn from_bits(bits: u128) -> Self {
        Self { raw: bits as i128 }
    }

    /// Returns the bit pattern of the value.
    #[inline]
    pub const fn to_bits(self) -> u128 {
        self.raw as u128
    }

    /// Creates a value from an integral number.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed96q32::Fixed;
    ///
    /// assert_eq!(Fixed::from_int(7).to_raw(), 7 * (1i128 << 32));
    /// ```
    #[inline]
    pub const fn from_int(value: i64) -> Self {
        Self {
            raw: (value as i128) << 32,
        }
    }

    /// Creates a value from `f64`
    ///
    /// NaN converts to 0,
    /// Too big absolute values saturates to `Fixed::MAX` or `Fixed::MIN`
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed96q32::Fixed;
    ///
    /// assert_eq!(Fixed::from_f64(7.5), Fixed::from_int(7) + Fixed::HALF);
    /// assert_eq!(Fixed::from_f64(f64::NAN), Fixed::ZERO);
    /// assert_eq!(Fixed::from_f64(100.0.exp()), Fixed::MAX);
    /// assert_eq!(Fixed::from_f64(f64::NEG_INFINITY), Fixed::MIN);
    /// ```
    #[inline]
    pub const fn from_f64(value: f64) -> Self {
        let inner = value * (1u64 << Self::FRACTIONAL_BITS) as f64;

        Self::from_raw(inner as i128)
    }

    /// Converts the value to an `f64`, losing precision for large values.
    #[inline]
    pub const fn to_f64(self) -> f64 {
        self.raw as f64 * EPSILON_F64
    }

    /// Converts the value to an `f32`, losing precision
    #[inline]
    pub const fn to_f32(self) -> f32 {
        self.raw as f32 * EPSILON_F32
    }

    /// Creates a value from an integral and a fractional part.
    ///
    /// The fractional part is expressed in units of `2**-32`, therefore it
    /// always fits into the 32 fractional bits of the type. The sign of the
    /// integral part applies to both halves, so the value `-2` with the
    /// fractional part `7` equals `-2.0000000016`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed96q32::Fixed;
    ///
    /// assert_eq!(Fixed::from_parts(0, 17_179_869).to_string(), "0.004");
    /// assert_eq!(Fixed::from_parts(-1, 2_147_483_648).to_string(), "-1.5");
    /// ```
    #[inline]
    pub const fn from_parts(integer: i64, fraction: u32) -> Self {
        let magnitude = ((integer.unsigned_abs() as u128) << 32) | fraction as u128;
        let raw = magnitude as i128;
        if integer < 0 {
            Self {
                raw: raw.wrapping_neg(),
            }
        } else {
            Self { raw }
        }
    }

    /// Returns the integral part of the value, rounded towards `-inf`.
    #[inline]
    pub const fn integer_part(self) -> i128 {
        self.raw >> 32
    }

    /// Returns the fractional part of the value in units of `2**-32`.
    ///
    /// The returned bits are always non-negative; use [`Fixed::fract`] to
    /// obtain the fractional part with the sign of the value.
    #[inline]
    pub const fn fract_bits(self) -> u32 {
        (self.raw.unsigned_abs() & FRACTION_MASK_U128) as u32
    }

    /// Returns `true` if the value is zero.
    #[inline]
    pub const fn is_zero(self) -> bool {
        self.raw == 0
    }

    /// Returns `true` if the value is strictly negative.
    #[inline]
    pub const fn is_negative(self) -> bool {
        self.raw < 0
    }

    /// Returns `true` if the value is strictly positive.
    #[inline]
    pub const fn is_positive(self) -> bool {
        self.raw > 0
    }

    /// Returns the absolute value of the number.
    ///
    /// [`Fixed::MIN`] has no positive counterpart, therefore the result is
    /// clamped to [`Fixed::MAX`].
    #[inline]
    pub const fn abs(self) -> Self {
        match self.checked_abs() {
            Some(value) => value,
            None => Self::MAX,
        }
    }

    /// Returns the absolute value of the number, or [`None`] for
    /// [`Fixed::MIN`].
    #[inline]
    pub const fn checked_abs(self) -> Option<Self> {
        if self.raw == i128::MIN {
            None
        } else if self.raw < 0 {
            Some(Self { raw: -self.raw })
        } else {
            Some(self)
        }
    }

    /// Returns the negated value, or [`None`] for [`Fixed::MIN`].
    #[inline]
    pub const fn checked_neg(self) -> Option<Self> {
        if self.raw == i128::MIN {
            None
        } else {
            Some(Self { raw: -self.raw })
        }
    }

    /// Returns `1` for positive values, `-1` for negative values and `0` for
    /// zero.
    #[inline]
    pub const fn signum(self) -> Self {
        if self.raw > 0 {
            Self::ONE
        } else if self.raw < 0 {
            Self::NEG_ONE
        } else {
            Self::ZERO
        }
    }

    /// Returns the truncated value, rounded towards zero.
    #[inline]
    pub const fn trunc(self) -> Self {
        let (negative, magnitude) = self.split();
        Self::assemble(negative, (magnitude >> 32) << 32)
    }

    /// Returns the value rounded towards `-inf`.
    #[inline]
    pub const fn floor(self) -> Self {
        let (negative, magnitude) = self.split();
        let mut units = magnitude >> 32;
        if negative && magnitude & FRACTION_MASK_U128 != 0 {
            units += 1;
        }
        Self::assemble(negative, units << 32)
    }

    /// Returns the value rounded towards `+inf`.
    #[inline]
    pub const fn ceil(self) -> Self {
        let (negative, magnitude) = self.split();
        let mut units = magnitude >> 32;
        if !negative && magnitude & FRACTION_MASK_U128 != 0 {
            units += 1;
        }
        Self::assemble(negative, units << 32)
    }

    /// Returns the value rounded to the nearest integer, halfway cases away
    /// from zero.
    #[inline]
    pub const fn round(self) -> Self {
        let (negative, magnitude) = self.split();
        let mut units = magnitude >> 32;
        if magnitude & FRACTION_MASK_U128 >= (1u128 << 31) {
            units += 1;
        }
        Self::assemble(negative, units << 32)
    }

    /// Returns the fractional part of the value.
    ///
    /// The result carries the sign of the value, exactly like the `fract`
    /// method of the primitive floating point types.
    #[inline]
    pub const fn fract(self) -> Self {
        let magnitude = self.raw.unsigned_abs() & FRACTION_MASK_U128;
        Self::assemble(self.raw < 0, magnitude)
    }

    /// Adds two values, returning [`None`] on overflow.
    #[inline]
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        match self.raw.checked_add(rhs.raw) {
            Some(raw) => Some(Self { raw }),
            None => None,
        }
    }

    /// Subtracts two values, returning [`None`] on overflow.
    #[inline]
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        match self.raw.checked_sub(rhs.raw) {
            Some(raw) => Some(Self { raw }),
            None => None,
        }
    }

    /// Multiplies two values, returning [`None`] on overflow.
    ///
    /// The discarded bits of the exact product are truncated towards zero.
    #[inline]
    pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
        let limbs = mul_wide(self.raw.unsigned_abs(), rhs.raw.unsigned_abs());

        // The exact product is scaled by `2**-64`; shifting it right by 32 bits
        // restores the `2**-32` scale of the type.
        let low = ((limbs[1] as u128) << 64) | limbs[0] as u128;
        let high = ((limbs[3] as u128) << 64) | limbs[2] as u128;
        if high >> 32 != 0 {
            return None;
        }
        let magnitude = (low >> 32) | (high << 96);
        let negative = (self.raw < 0) != (rhs.raw < 0);

        if negative {
            if magnitude > (1u128 << 127) {
                None
            } else {
                Some(Self {
                    raw: (magnitude as i128).wrapping_neg(),
                })
            }
        } else if magnitude > i128::MAX as u128 {
            None
        } else {
            Some(Self {
                raw: magnitude as i128,
            })
        }
    }

    /// Divides two values, returning [`None`] on overflow or when the divisor
    /// is zero.
    ///
    /// The discarded bits of the exact quotient are truncated towards zero.
    #[inline]
    pub const fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.raw == 0 {
            return None;
        }

        let numerator = self.raw.unsigned_abs();
        let denominator = rhs.raw.unsigned_abs();

        let units = numerator / denominator;
        let mut remainder = numerator % denominator;

        // Extract the 32 fractional bits with a shift-and-subtract loop. The
        // remainder is always smaller than the divisor, hence the shift by one
        // bit cannot overflow.
        let mut fraction: u64 = 0;
        let mut bit = 0;
        while bit < Self::FRACTIONAL_BITS {
            remainder <<= 1;
            fraction <<= 1;
            if remainder >= denominator {
                remainder -= denominator;
                fraction |= 1;
            }
            bit += 1;
        }

        if units > (u128::MAX >> Self::FRACTIONAL_BITS) {
            return None;
        }
        let magnitude = (units << 32) | fraction as u128;
        let negative = (self.raw < 0) != (rhs.raw < 0);

        if negative {
            if magnitude > (1u128 << 127) {
                None
            } else {
                Some(Self {
                    raw: (magnitude as i128).wrapping_neg(),
                })
            }
        } else if magnitude > i128::MAX as u128 {
            None
        } else {
            Some(Self {
                raw: magnitude as i128,
            })
        }
    }

    /// Adds two values, wrapping the result on overflow.
    ///
    /// The result is the sum of the raw representations taken modulo
    /// `2**128`, exactly like the `wrapping_add` method of the primitive
    /// integer types.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed96q32::Fixed;
    ///
    /// assert_eq!(Fixed::from_int(1).wrapping_add(Fixed::from_int(2)), Fixed::from_int(3));
    /// ```
    #[inline]
    pub const fn wrapping_add(self, rhs: Self) -> Self {
        Self {
            raw: self.raw.wrapping_add(rhs.raw),
        }
    }

    /// Subtracts two values, wrapping the result on overflow.
    #[inline]
    pub const fn wrapping_sub(self, rhs: Self) -> Self {
        Self {
            raw: self.raw.wrapping_sub(rhs.raw),
        }
    }

    /// Multiplies two values, wrapping the result on overflow.
    ///
    /// The discarded bits of the exact product are truncated towards zero.
    #[inline]
    pub const fn wrapping_mul(self, rhs: Self) -> Self {
        let limbs = mul_wide(self.raw.unsigned_abs(), rhs.raw.unsigned_abs());
        let low = ((limbs[1] as u128) << 64) | limbs[0] as u128;
        let negative = (self.raw < 0) != (rhs.raw < 0);

        let raw = if negative {
            0u128.wrapping_sub(low) as i128
        } else {
            low as i128
        };

        Self { raw }
    }

    /// Negates the value, wrapping the result on overflow.
    ///
    /// [`Fixed::MIN`] has no positive counterpart, therefore the result is
    /// [`Fixed::MIN`].
    #[inline]
    pub const fn wrapping_neg(self) -> Self {
        Self {
            raw: self.raw.wrapping_neg(),
        }
    }

    /// Adds two values, returning the wrapped result and the overflow flag.
    ///
    /// The flag is set when the exact result leaves the representable range.
    #[inline]
    pub const fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        let raw = self.raw.wrapping_add(rhs.raw);
        let overflow = self.checked_add(rhs).is_none();
        (Self { raw }, overflow)
    }

    /// Subtracts two values, returning the wrapped result and the overflow
    /// flag.
    #[inline]
    pub const fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
        let raw = self.raw.wrapping_sub(rhs.raw);
        let overflow = self.checked_sub(rhs).is_none();
        (Self { raw }, overflow)
    }

    /// Multiplies two values, returning the wrapped result and the overflow
    /// flag.
    #[inline]
    pub const fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
        let wrapped = self.wrapping_mul(rhs);
        let overflow = self.checked_mul(rhs).is_none();
        (wrapped, overflow)
    }

    /// Adds two values, clamping the result to [`Fixed::MIN`] or
    /// [`Fixed::MAX`] on overflow.
    #[inline]
    pub const fn saturating_add(self, rhs: Self) -> Self {
        match self.checked_add(rhs) {
            Some(value) => value,
            None => {
                if rhs.raw < 0 {
                    Self::MIN
                } else {
                    Self::MAX
                }
            }
        }
    }

    /// Subtracts two values, clamping the result to [`Fixed::MIN`] or
    /// [`Fixed::MAX`] on overflow.
    #[inline]
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        match self.checked_sub(rhs) {
            Some(value) => value,
            None => {
                if rhs.raw > 0 {
                    Self::MIN
                } else {
                    Self::MAX
                }
            }
        }
    }

    /// Multiplies two values, clamping the result to [`Fixed::MIN`] or
    /// [`Fixed::MAX`] on overflow.
    #[inline]
    pub const fn saturating_mul(self, rhs: Self) -> Self {
        match self.checked_mul(rhs) {
            Some(value) => value,
            None => {
                if (self.raw < 0) != (rhs.raw < 0) {
                    Self::MIN
                } else {
                    Self::MAX
                }
            }
        }
    }

    /// Divides two values, clamping the result to [`Fixed::MIN`] or
    /// [`Fixed::MAX`] on overflow.
    ///
    /// Returns [`None`] when the divisor is zero.
    #[inline]
    pub const fn saturating_div(self, rhs: Self) -> Option<Self> {
        match self.checked_div(rhs) {
            Some(value) => Some(value),
            None => {
                if rhs.raw == 0 {
                    None
                } else if (self.raw < 0) != (rhs.raw < 0) {
                    Some(Self::MIN)
                } else {
                    Some(Self::MAX)
                }
            }
        }
    }

    /// Returns the reciprocal of the value, or [`None`] when the value is
    /// zero or the result overflows.
    #[inline]
    pub const fn recip(self) -> Option<Self> {
        Self::ONE.checked_div(self)
    }

    /// Converts the value to an `i64`, truncating towards zero and clamping
    /// the result to the range of `i64`.
    #[inline]
    pub fn to_i64(self) -> i64 {
        let truncated = self.trunc().integer_part();
        if truncated > i64::MAX as i128 {
            i64::MAX
        } else if truncated < i64::MIN as i128 {
            i64::MIN
        } else {
            truncated as i64
        }
    }

    /// Converts the value to an `i128`, truncating towards zero.
    #[inline]
    pub fn to_i128(self) -> i128 {
        self.trunc().integer_part()
    }

    /// Converts the value to a `u64`, truncating towards zero and clamping
    /// negative values to zero.
    #[inline]
    pub fn to_u64(self) -> u64 {
        let truncated = self.trunc().integer_part();
        if truncated < 0 {
            0
        } else if truncated > u64::MAX as i128 {
            u64::MAX
        } else {
            truncated as u64
        }
    }

    /// Converts the value to a `u128`, truncating towards zero and clamping
    /// negative values to zero.
    #[inline]
    pub fn to_u128(self) -> u128 {
        let truncated = self.trunc().integer_part();
        if truncated < 0 { 0 } else { truncated as u128 }
    }

    /// Splits the value into its sign and its magnitude.
    #[inline]
    const fn split(self) -> (bool, u128) {
        (self.raw < 0, self.raw.unsigned_abs())
    }

    /// Builds a value out of its sign and its magnitude, clamping the result
    /// to the representable range.
    #[inline]
    const fn assemble(negative: bool, magnitude: u128) -> Self {
        if negative {
            if magnitude > (1u128 << 127) {
                Self::MIN
            } else {
                Self {
                    raw: (magnitude as i128).wrapping_neg(),
                }
            }
        } else if magnitude > i128::MAX as u128 {
            Self::MAX
        } else {
            Self {
                raw: magnitude as i128,
            }
        }
    }
}

/// Computes the 256-bit product of two 128-bit unsigned integers.
///
/// The limbs are ordered from the least significant one to the most
/// significant one.
pub(crate) const fn mul_wide(a: u128, b: u128) -> [u64; 4] {
    let a_low = a as u64;
    let a_high = (a >> 64) as u64;
    let b_low = b as u64;
    let b_high = (b >> 64) as u64;

    let p00 = a_low as u128 * b_low as u128;
    let p01 = a_low as u128 * b_high as u128;
    let p10 = a_high as u128 * b_low as u128;
    let p11 = a_high as u128 * b_high as u128;

    let middle = (p00 >> 64) + (p01 & MASK64) + (p10 & MASK64);
    let high = (middle >> 64) + (p01 >> 64) + (p10 >> 64) + (p11 & MASK64);

    [
        p00 as u64,
        middle as u64,
        high as u64,
        ((high >> 64) + (p11 >> 64)) as u64,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_have_the_expected_raw_values() {
        assert_eq!(Fixed::ZERO.to_raw(), 0);
        assert_eq!(Fixed::ONE.to_raw(), 1i128 << 32);
        assert_eq!(Fixed::NEG_ONE.to_raw(), -(1i128 << 32));
        assert_eq!(Fixed::HALF.to_raw(), 1i128 << 31);
        assert_eq!(Fixed::EPSILON.to_raw(), 1);
        assert_eq!(Fixed::MIN.to_raw(), i128::MIN);
        assert_eq!(Fixed::MAX.to_raw(), i128::MAX);
    }

    #[test]
    fn from_int_scales_the_value() {
        assert_eq!(Fixed::from_int(0).to_raw(), 0);
        assert_eq!(Fixed::from_int(1).to_raw(), 1i128 << 32);
        assert_eq!(Fixed::from_int(-3).to_raw(), -(3i128 << 32));
        assert_eq!(Fixed::from_int(i64::MAX).to_raw(), (i64::MAX as i128) << 32);
    }

    #[test]
    fn from_parts_combines_both_halves() {
        assert_eq!(Fixed::from_parts(3, 0).to_raw(), 3i128 << 32);
        assert_eq!(Fixed::from_parts(0, 4_000_000_000).to_raw(), 4_000_000_000);
        assert_eq!(Fixed::from_parts(-1, 1).to_raw(), (-1i128 << 32) - 1);
    }

    #[test]
    fn bits_round_trip() {
        let value = Fixed::from_parts(-12, 345);
        assert_eq!(Fixed::from_bits(value.to_bits()), value);
    }

    #[test]
    fn integer_part_is_rounded_down() {
        assert_eq!(Fixed::from_parts(2, 7).integer_part(), 2);
        assert_eq!(Fixed::from_parts(-2, 7).integer_part(), -3);
        assert_eq!(Fixed::from_parts(0, 0).integer_part(), 0);
    }

    #[test]
    fn fract_bits_returns_the_low_bits() {
        assert_eq!(Fixed::from_parts(0, 123).fract_bits(), 123);
        assert_eq!(Fixed::from_parts(0, 0).fract_bits(), 0);
        assert_eq!(Fixed::from_raw(-1).fract_bits(), 1);
        assert_eq!(Fixed::from_parts(-1, 500_000_000).fract_bits(), 500_000_000);
    }

    #[test]
    fn predicates_follow_the_sign() {
        assert!(Fixed::ZERO.is_zero());
        assert!(!Fixed::ZERO.is_negative());
        assert!(!Fixed::ZERO.is_positive());
        assert!(Fixed::NEG_ONE.is_negative());
        assert!(Fixed::ONE.is_positive());
    }

    #[test]
    fn abs_removes_the_sign() {
        assert_eq!(Fixed::from_parts(-3, 5).abs(), Fixed::from_parts(3, 5));
        assert_eq!(Fixed::ZERO.abs(), Fixed::ZERO);
        assert_eq!(Fixed::MIN.abs(), Fixed::MAX);
        assert_eq!(Fixed::MIN.checked_abs(), None);
    }

    #[test]
    fn signum_reports_the_direction() {
        assert_eq!(Fixed::from_int(5).signum(), Fixed::ONE);
        assert_eq!(Fixed::from_int(-5).signum(), Fixed::NEG_ONE);
        assert_eq!(Fixed::ZERO.signum(), Fixed::ZERO);
    }

    #[test]
    fn trunc_rounds_towards_zero() {
        assert_eq!(Fixed::from_parts(2, 7).trunc(), Fixed::from_int(2));
        assert_eq!(Fixed::from_parts(-2, 7).trunc(), Fixed::from_int(-2));
        assert_eq!(Fixed::MIN.trunc(), Fixed::MIN);
    }

    #[test]
    fn floor_rounds_towards_negative_infinity() {
        assert_eq!(Fixed::from_parts(2, 7).floor(), Fixed::from_int(2));
        assert_eq!(Fixed::from_parts(-2, 7).floor(), Fixed::from_int(-3));
        assert_eq!(Fixed::from_parts(-2, 0).floor(), Fixed::from_int(-2));
    }

    #[test]
    fn ceil_rounds_towards_positive_infinity() {
        assert_eq!(Fixed::from_parts(2, 7).ceil(), Fixed::from_int(3));
        assert_eq!(Fixed::from_parts(-2, 7).ceil(), Fixed::from_int(-2));
        assert_eq!(Fixed::from_parts(-2, 0).ceil(), Fixed::from_int(-2));
    }

    #[test]
    fn round_uses_half_away_from_zero() {
        assert_eq!(Fixed::from_parts(2, 1 << 31).round(), Fixed::from_int(3));
        assert_eq!(Fixed::from_parts(-2, 1 << 31).round(), Fixed::from_int(-3));
        assert_eq!(
            Fixed::from_parts(2, (1 << 31) - 1).round(),
            Fixed::from_int(2)
        );
    }

    #[test]
    fn fract_keeps_the_sign_of_the_value() {
        assert_eq!(Fixed::from_parts(2, 7).fract(), Fixed::from_parts(0, 7));
        assert_eq!(Fixed::from_parts(-2, 7).fract(), Fixed::from_raw(-7));
    }

    #[test]
    fn checked_add_reports_overflow() {
        assert_eq!(Fixed::ONE.checked_add(Fixed::ONE), Some(Fixed::from_int(2)));
        assert_eq!(Fixed::MAX.checked_add(Fixed::ONE), None);
        assert_eq!(Fixed::MIN.checked_add(Fixed::NEG_ONE), None);
    }

    #[test]
    fn checked_sub_reports_overflow() {
        assert_eq!(Fixed::ONE.checked_sub(Fixed::ONE), Some(Fixed::ZERO));
        assert_eq!(Fixed::MIN.checked_sub(Fixed::ONE), None);
    }

    #[test]
    fn checked_mul_truncates_towards_zero() {
        assert_eq!(
            Fixed::from_parts(0, 1 << 30).checked_mul(Fixed::from_parts(0, 1 << 30)),
            Some(Fixed::from_parts(0, 1 << 28))
        );
        assert_eq!(
            Fixed::from_int(-3).checked_mul(Fixed::from_int(2)),
            Some(Fixed::from_int(-6))
        );
        assert_eq!(Fixed::MAX.checked_mul(Fixed::from_int(2)), None);
    }

    #[test]
    fn checked_mul_keeps_the_full_precision() {
        let lhs = Fixed::from_raw(1i128 << 64);
        let rhs = Fixed::from_raw(1i128 << 64);
        assert_eq!(lhs.checked_mul(rhs), Some(Fixed::from_raw(1i128 << 96)));
    }

    #[test]
    fn checked_div_truncates_towards_zero() {
        assert_eq!(
            Fixed::from_int(1).checked_div(Fixed::from_int(2)),
            Some(Fixed::HALF)
        );
        assert_eq!(
            Fixed::from_int(-1).checked_div(Fixed::from_int(2)),
            Some(Fixed::from_raw(-(1i128 << 31)))
        );
        assert_eq!(Fixed::ONE.checked_div(Fixed::ZERO), None);
    }

    #[test]
    fn checked_div_keeps_the_fractional_part() {
        let third = Fixed::ONE.checked_div(Fixed::from_int(3)).unwrap();
        assert_eq!(third.to_raw(), 1431655765);
    }

    #[test]
    fn saturating_operations_clamp_the_result() {
        assert_eq!(Fixed::MAX.saturating_add(Fixed::ONE), Fixed::MAX);
        assert_eq!(Fixed::MIN.saturating_sub(Fixed::ONE), Fixed::MIN);
        assert_eq!(Fixed::MAX.saturating_mul(Fixed::from_int(2)), Fixed::MAX);
        assert_eq!(Fixed::MIN.saturating_mul(Fixed::from_int(2)), Fixed::MIN);
        assert_eq!(Fixed::ONE.saturating_div(Fixed::ZERO), None);
    }

    #[test]
    fn recip_inverts_the_value() {
        assert_eq!(
            Fixed::from_int(4).recip(),
            Some(Fixed::from_parts(0, 1 << 30))
        );
        assert_eq!(Fixed::ZERO.recip(), None);
    }

    #[test]
    fn conversions_truncate_and_clamp() {
        assert_eq!(Fixed::from_parts(2, 7).to_i64(), 2);
        assert_eq!(Fixed::from_parts(-2, 7).to_i64(), -2);
        assert_eq!(Fixed::MAX.to_i64(), i64::MAX);
        assert_eq!(Fixed::MIN.to_i64(), i64::MIN);
        assert_eq!(Fixed::from_parts(-2, 7).to_u64(), 0);
        assert_eq!(Fixed::from_parts(2, 7).to_u128(), 2);
        assert_eq!(Fixed::from_parts(0, 1 << 31).to_f64(), 0.5);
    }
}
