//! Advanced mathematics for the [`Fixed`] type.
//!
//! The functions are implemented with the CORDIC algorithm and never convert
//! the value into a floating point number. The internal computations use the
//! Q64 format, that is, 64 fractional bits, which keeps the precision of the
//! results well above the 32 fractional bits of the public type.
//!
//! # Algorithms
//!
//! * The trigonometric functions use the rotation mode of the CORDIC
//!   algorithm, the inverse trigonometric ones use the vectoring mode;
//! * the logarithm and the exponential function use the logarithmic and the
//!   exponential modes, which expand the value into the products of the
//!   factors `1 + 2**-i`;
//! * the square root is computed by the digit-by-digit algorithm, which is
//!   exact and needs no iteration.
//!
//! The functions return [`None`] when the argument is outside of the domain
//! of the function or when the result does not fit into the representable
//! range.
//!
//! # Examples
//!
//! ```
//! use fixed96q32::Fixed;
//!
//! assert_eq!(Fixed::from_int(4).sqrt().unwrap(), Fixed::from_int(2));
//! assert!((Fixed::ONE.exp().unwrap().to_f64() - 2.718281828).abs() < 1e-8);
//! ```

mod cordic;

use self::cordic::{
    DEGREES_PER_RADIAN_Q64, FRAC_PI_2_Q64, LN_2_Q64, LN_10_Q64, ONE_Q64, PI_Q64,
    RADIANS_PER_DEGREE_Q64, arc_tangent, divide, exponential, from_internal, integer_sqrt,
    logarithm, multiply, sine_and_cosine, sqrt_internal, to_internal,
};
use crate::{consts, fixed::Fixed};

impl Fixed {
    /// Returns the square root of the value.
    ///
    /// The result is rounded towards zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed96q32::Fixed;
    ///
    /// assert_eq!(Fixed::from_int(4).sqrt(), Some(Fixed::from_int(2)));
    /// assert_eq!(Fixed::from_int(9).sqrt(), Some(Fixed::from_int(3)));
    /// assert_eq!(Fixed::ZERO.sqrt(), Some(Fixed::ZERO));
    /// assert_eq!(Fixed::NEG_ONE.sqrt(), None);
    /// ```
    #[inline]
    pub fn sqrt(self) -> Option<Self> {
        if self.is_negative() {
            return None;
        }

        let raw = self.to_raw() as u128;
        if raw == 0 {
            return Some(Self::ZERO);
        }

        // The root of the raw representation is scaled by `2**16`, because the
        // value itself is scaled by `2**-32`.
        let root = integer_sqrt(raw);
        let remainder = raw - root * root;
        let correction = (remainder << 16) / (root << 1);
        let result = (root << 16) + correction;

        Some(Self::from_raw(result as i128))
    }

    /// Returns the square root of the absolute value.
    ///
    /// ```
    /// use fixed96q32::Fixed;
    ///
    /// assert_eq!(Fixed::from_int(4).sqrt(), Some(Fixed::from_int(2)));
    /// assert_eq!(Fixed::from_int(9).sqrt(), Some(Fixed::from_int(3)));
    /// assert_eq!(Fixed::ZERO.sqrt(), Some(Fixed::ZERO));
    /// assert_eq!(Fixed::NEG_ONE.sqrt(), Some(Fixed::NEG_ONE));
    /// ```
    #[inline]
    pub fn sqrt_abs(self) -> Self {
        if self == Self::MIN {
            return Self::from_raw(1 << 47) * consts::SQRT_2;
        }
        if self == Self::ZERO {
            return Self::ZERO;
        }

        let raw = self.abs().to_raw() as u128;

        let root = integer_sqrt(raw);
        let remainder = raw - root * root;
        let correction = (remainder << 16) / (root << 1);
        let result = (root << 16) + correction;

        Self::from_raw(result as i128)
    }

    /// Returns the cube root of the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed96q32::Fixed;
    ///
    /// let root = Fixed::from_int(8).cbrt().unwrap();
    /// assert!((root.to_f64() - 2.0).abs() < 1e-8);
    /// ```
    #[inline]
    pub fn cbrt(self) -> Option<Self> {
        if self.is_zero() {
            return Some(Self::ZERO);
        }

        let magnitude = self.checked_abs()?;
        let third = magnitude.ln()? / Fixed::from_int(3);
        let result = third.exp()?;

        Some(if self.is_negative() { -result } else { result })
    }

    /// Returns the natural logarithm of the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed96q32::Fixed;
    ///
    /// assert_eq!(Fixed::ONE.ln().unwrap(), Fixed::ZERO);
    /// ```
    #[inline]
    pub fn ln(self) -> Option<Self> {
        if !self.is_positive() {
            return None;
        }

        let internal = to_internal(self)?;
        let result = logarithm(internal)?;
        from_internal(result)
    }

    /// Returns the base two logarithm of the value.
    #[inline]
    pub fn log2(self) -> Option<Self> {
        if !self.is_positive() {
            return None;
        }

        let internal = to_internal(self)?;
        let result = logarithm(internal)?;
        from_internal(divide(result, LN_2_Q64)?)
    }

    /// Returns the base ten logarithm of the value.
    #[inline]
    pub fn log10(self) -> Option<Self> {
        if !self.is_positive() {
            return None;
        }

        let internal = to_internal(self)?;
        let result = logarithm(internal)?;
        from_internal(divide(result, LN_10_Q64)?)
    }

    /// Returns the exponential of the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed96q32::Fixed;
    ///
    /// assert_eq!(Fixed::ZERO.exp().unwrap(), Fixed::ONE);
    /// ```
    #[inline]
    pub fn exp(self) -> Option<Self> {
        let internal = to_internal(self)?;
        let result = exponential(internal)?;
        from_internal(result)
    }

    /// Returns two raised to the power of the value.
    #[inline]
    pub fn exp2(self) -> Option<Self> {
        let internal = to_internal(self)?;
        let result = exponential(multiply(internal, LN_2_Q64))?;
        from_internal(result)
    }

    /// Raises the value to an integer power.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed96q32::Fixed;
    ///
    /// assert_eq!(Fixed::from_int(2).powi(10).unwrap(), Fixed::from_int(1024));
    /// ```
    #[inline]
    pub fn powi(self, exponent: i32) -> Option<Self> {
        if exponent == 0 {
            return Some(Self::ONE);
        }

        let (base, power) = if exponent < 0 {
            (self.recip()?, exponent.unsigned_abs())
        } else {
            (self, exponent.unsigned_abs())
        };

        let mut result = Self::ONE;
        let mut factor = base;
        let mut power = power;

        while power > 0 {
            if power & 1 == 1 {
                result = result.checked_mul(factor)?;
            }
            power >>= 1;
            if power > 0 {
                factor = factor.checked_mul(factor)?;
            }
        }

        Some(result)
    }

    /// Raises the value to a fractional power.
    ///
    /// The negative bases are only defined for the integral exponents.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed96q32::Fixed;
    ///
    /// let power = Fixed::from_int(2).powf(Fixed::from_int(10)).unwrap();
    /// assert!((power.to_f64() - 1024.0).abs() < 1e-5);
    /// ```
    #[inline]
    pub fn powf(self, exponent: Self) -> Option<Self> {
        if exponent.is_zero() {
            return Some(Self::ONE);
        }

        if self.is_zero() {
            return if exponent.is_negative() {
                None
            } else {
                Some(Self::ZERO)
            };
        }

        if self.is_negative() {
            if exponent.trunc() != exponent {
                return None;
            }

            let magnitude = self.checked_abs()?;
            let result = magnitude.powf(exponent.checked_abs()?)?;
            let power = exponent.to_i64();

            return Some(if power % 2 == 0 { result } else { -result });
        }

        let logarithm = self.ln()?;
        logarithm.checked_mul(exponent)?.exp()
    }

    /// Returns the sine of the angle in radians.
    #[inline]
    pub fn sin(self) -> Option<Self> {
        let internal = to_internal(self)?;
        let (sine, _) = sine_and_cosine(internal)?;
        from_internal(sine)
    }

    /// Returns the cosine of the angle in radians.
    #[inline]
    pub fn cos(self) -> Option<Self> {
        let internal = to_internal(self)?;
        let (_, cosine) = sine_and_cosine(internal)?;
        from_internal(cosine)
    }

    /// Returns the tangent of the angle in radians.
    #[inline]
    pub fn tan(self) -> Option<Self> {
        let internal = to_internal(self)?;
        let (sine, cosine) = sine_and_cosine(internal)?;
        if cosine == 0 {
            return None;
        }
        from_internal(divide(sine, cosine)?)
    }

    /// Returns the sine and the cosine of the angle in radians.
    #[inline]
    pub fn sin_cos(self) -> Option<(Self, Self)> {
        let internal = to_internal(self)?;
        let (sine, cosine) = sine_and_cosine(internal)?;
        Some((from_internal(sine)?, from_internal(cosine)?))
    }

    /// Returns the arc sine of the value.
    #[inline]
    pub fn asin(self) -> Option<Self> {
        if self < Fixed::NEG_ONE || self > Fixed::ONE {
            return None;
        }

        let internal = to_internal(self)?;
        let square = multiply(internal, internal);
        if square > ONE_Q64 {
            return None;
        }
        let root = sqrt_internal(ONE_Q64 - square)?;
        let angle = arc_tangent(internal, root)?;

        from_internal(angle)
    }

    /// Returns the arc cosine of the value.
    #[inline]
    pub fn acos(self) -> Option<Self> {
        if self < Fixed::NEG_ONE || self > Fixed::ONE {
            return None;
        }

        let internal = to_internal(self)?;
        let square = multiply(internal, internal);
        if square > ONE_Q64 {
            return None;
        }
        let root = sqrt_internal(ONE_Q64 - square)?;
        let angle = arc_tangent(internal, root)?;

        from_internal(FRAC_PI_2_Q64 - angle)
    }

    /// Returns the arc tangent of the value.
    #[inline]
    pub fn atan(self) -> Option<Self> {
        let internal = to_internal(self)?;
        let angle = arc_tangent(internal, ONE_Q64)?;
        from_internal(angle)
    }

    /// Returns the angle of the point with the coordinates `(other, self)`.
    ///
    /// The result is in the range `[-pi, pi]`.
    #[inline]
    pub fn atan2(self, other: Self) -> Option<Self> {
        let y = to_internal(self)?;
        let x = to_internal(other)?;

        let angle = if x > 0 {
            arc_tangent(y, x)?
        } else if x < 0 {
            let base = arc_tangent(-y, x.unsigned_abs() as i128)?;
            if y >= 0 { base + PI_Q64 } else { base - PI_Q64 }
        } else if y > 0 {
            FRAC_PI_2_Q64
        } else if y < 0 {
            -FRAC_PI_2_Q64
        } else {
            0
        };

        from_internal(angle)
    }

    /// Returns the hyperbolic sine of the value.
    #[inline]
    pub fn sinh(self) -> Option<Self> {
        let internal = to_internal(self)?;
        let positive = exponential(internal)?;
        let negative = exponential(-internal)?;
        from_internal(divide(positive - negative, 2 * ONE_Q64)?)
    }

    /// Returns the hyperbolic cosine of the value.
    #[inline]
    pub fn cosh(self) -> Option<Self> {
        let internal = to_internal(self)?;
        let positive = exponential(internal)?;
        let negative = exponential(-internal)?;
        from_internal(divide(positive + negative, 2 * ONE_Q64)?)
    }

    /// Returns the hyperbolic tangent of the value.
    #[inline]
    pub fn tanh(self) -> Option<Self> {
        let internal = to_internal(self)?;
        let positive = exponential(multiply(internal, 2 * ONE_Q64))?;
        from_internal(divide(positive - ONE_Q64, positive + ONE_Q64)?)
    }

    /// Returns the inverse hyperbolic sine of the value.
    #[inline]
    pub fn asinh(self) -> Option<Self> {
        let internal = to_internal(self)?;
        let square = multiply(internal, internal);
        let root = sqrt_internal(square + ONE_Q64)?;
        let sum = internal + root;
        if sum <= 0 {
            return None;
        }
        from_internal(logarithm(sum)?)
    }

    /// Returns the inverse hyperbolic cosine of the value.
    #[inline]
    pub fn acosh(self) -> Option<Self> {
        if self < Fixed::ONE {
            return None;
        }

        let internal = to_internal(self)?;
        let square = multiply(internal, internal) - ONE_Q64;
        if square < 0 {
            return None;
        }
        let root = sqrt_internal(square)?;
        let sum = internal + root;
        if sum <= 0 {
            return None;
        }
        from_internal(logarithm(sum)?)
    }

    /// Returns the inverse hyperbolic tangent of the value.
    #[inline]
    pub fn atanh(self) -> Option<Self> {
        if self <= Fixed::NEG_ONE || self >= Fixed::ONE {
            return None;
        }

        let internal = to_internal(self)?;
        let ratio = divide(internal + ONE_Q64, ONE_Q64 - internal)?;
        from_internal(divide(logarithm(ratio)?, 2 * ONE_Q64)?)
    }

    /// Returns the length of the hypotenuse of a right triangle with the legs
    /// of the given lengths.
    #[inline]
    pub fn hypot(self, other: Self) -> Option<Self> {
        let first = self.checked_mul(self)?;
        let second = other.checked_mul(other)?;
        first.checked_add(second)?.sqrt()
    }

    /// Returns `self + a * b`, which is fused into a single rounding step.
    ///
    /// The overflow is handled by saturation, see
    /// [`Fixed::saturating_add`] and [`Fixed::saturating_mul`].
    #[inline]
    pub const fn mul_add(self, a: Self, b: Self) -> Self {
        self.saturating_add(a.saturating_mul(b))
    }

    /// Converts the angle from radians to degrees.
    #[inline]
    pub fn to_degrees(self) -> Option<Self> {
        let internal = to_internal(self)?;
        from_internal(multiply(internal, DEGREES_PER_RADIAN_Q64))
    }

    /// Converts the angle from degrees to radians.
    #[inline]
    pub fn to_radians(self) -> Option<Self> {
        let internal = to_internal(self)?;
        from_internal(multiply(internal, RADIANS_PER_DEGREE_Q64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns the distance between the value and the reference.
    fn distance(value: Option<Fixed>, reference: f64) -> f64 {
        let value = value.expect("the function must return a value");
        (value.to_f64() - reference).abs()
    }

    #[test]
    fn the_square_root_is_exact_for_the_squares() {
        assert_eq!(Fixed::ZERO.sqrt().unwrap(), Fixed::ZERO);
        assert_eq!(Fixed::from_int(1).sqrt().unwrap(), Fixed::ONE);
        assert_eq!(Fixed::from_int(4).sqrt().unwrap(), Fixed::from_int(2));
        assert_eq!(Fixed::from_int(9).sqrt().unwrap(), Fixed::from_int(3));
        assert_eq!(Fixed::from_int(16).sqrt().unwrap(), Fixed::from_int(4));
    }

    #[test]
    fn the_square_root_matches_the_reference() {
        assert!(distance(Fixed::from_int(2).sqrt(), 2.0_f64.sqrt()) < 1e-9);
        assert!(distance(Fixed::from_int(3).sqrt(), 3.0_f64.sqrt()) < 1e-9);
        assert!(distance(Fixed::from_parts(0, 2_147_483_648).sqrt(), 0.5_f64.sqrt()) < 1e-9);
        assert_eq!(Fixed::from_int(-1).sqrt(), None);
    }

    #[test]
    fn the_cube_root_matches_the_reference() {
        assert!(distance(Fixed::from_int(8).cbrt(), 2.0) < 1e-8);
        assert!(distance(Fixed::from_int(2).cbrt(), 2.0_f64.cbrt()) < 1e-8);
        assert!(distance(Fixed::from_int(-27).cbrt(), -3.0) < 1e-8);
    }

    #[test]
    fn the_logarithm_matches_the_reference() {
        assert_eq!(Fixed::ONE.ln().unwrap(), Fixed::ZERO);
        assert!(distance(Fixed::from_int(2).ln(), 2.0_f64.ln()) < 1e-9);
        assert!(distance(Fixed::from_int(10).ln(), 10.0_f64.ln()) < 1e-9);
        assert!(distance(Fixed::from_parts(0, 2_147_483_648).ln(), 0.5_f64.ln()) < 1e-9);
        let small = 4_294_967f64 / 4_294_967_296f64;
        assert!(distance(Fixed::from_parts(0, 4_294_967).ln(), small.ln()) < 1e-9);
        assert_eq!(Fixed::ZERO.ln(), None);
        assert_eq!(Fixed::NEG_ONE.ln(), None);
    }

    #[test]
    fn the_logarithms_with_the_base_match_the_reference() {
        assert_eq!(Fixed::from_int(8).log2().unwrap(), Fixed::from_int(3));
        assert_eq!(Fixed::from_int(100).log10().unwrap(), Fixed::from_int(2));
        assert!(distance(Fixed::from_int(3).log2(), 3.0_f64.log2()) < 1e-9);
        assert!(distance(Fixed::from_int(7).log10(), 7.0_f64.log10()) < 1e-9);
    }

    #[test]
    fn the_exponential_matches_the_reference() {
        assert_eq!(Fixed::ZERO.exp().unwrap(), Fixed::ONE);
        assert!(distance(Fixed::ONE.exp(), 1.0_f64.exp()) < 1e-8);
        assert!(distance(Fixed::NEG_ONE.exp(), (-1.0_f64).exp()) < 1e-8);
        assert!(distance(Fixed::from_int(2).exp(), 2.0_f64.exp()) < 1e-7);
        assert_eq!(Fixed::from_int(4).exp2().unwrap(), Fixed::from_int(16));
    }

    #[test]
    fn the_trigonometric_functions_match_the_reference() {
        assert_eq!(Fixed::ZERO.sin().unwrap(), Fixed::ZERO);
        assert_eq!(Fixed::ZERO.cos().unwrap(), Fixed::ONE);
        assert!(distance(Fixed::from_int(1).sin(), 1.0_f64.sin()) < 1e-9);
        assert!(distance(Fixed::from_int(1).cos(), 1.0_f64.cos()) < 1e-9);
        assert!(distance(Fixed::from_int(1).tan(), 1.0_f64.tan()) < 1e-9);
        assert!(distance(Fixed::from_int(2).sin(), 2.0_f64.sin()) < 1e-9);
        assert!(distance(Fixed::from_int(2).cos(), 2.0_f64.cos()) < 1e-9);
        assert!(distance(Fixed::from_int(-1).sin(), (-1.0_f64).sin()) < 1e-9);
    }

    #[test]
    fn the_trigonometric_functions_use_the_full_range() {
        let pi = Fixed::from_raw(crate::consts::PI.to_raw());

        assert!(distance(pi.sin(), std::f64::consts::PI.sin()) < 1e-9);
        assert!(distance(pi.cos(), std::f64::consts::PI.cos()) < 1e-9);
        assert!(distance(pi.tan(), std::f64::consts::PI.tan()) < 1e-9);
        assert!(distance((pi / Fixed::from_int(2)).sin(), 1.0) < 1e-9);
        assert!(distance((pi * Fixed::from_int(2)).sin(), 0.0) < 1e-9);
        assert!(distance((pi * Fixed::from_int(3)).sin(), 0.0) < 1e-9);
    }

    #[test]
    fn the_sine_and_the_cosine_are_returned_together() {
        let (sine, cosine) = Fixed::from_int(1).sin_cos().unwrap();

        assert!(distance(Some(sine), 1.0_f64.sin()) < 1e-9);
        assert!(distance(Some(cosine), 1.0_f64.cos()) < 1e-9);
    }

    #[test]
    fn the_inverse_trigonometric_functions_match_the_reference() {
        assert!(distance(Fixed::ONE.asin(), std::f64::consts::FRAC_PI_2) < 1e-9);
        assert!(distance(Fixed::ZERO.acos(), std::f64::consts::FRAC_PI_2) < 1e-9);
        assert!(distance(Fixed::ONE.atan(), std::f64::consts::FRAC_PI_4) < 1e-9);
        assert!(distance(Fixed::NEG_ONE.atan(), -std::f64::consts::FRAC_PI_4) < 1e-9);
        assert!(distance(Fixed::ONE.atan2(Fixed::ONE), std::f64::consts::FRAC_PI_4) < 1e-9);
        assert!(
            distance(
                Fixed::ONE.atan2(Fixed::NEG_ONE),
                3.0 * std::f64::consts::FRAC_PI_4
            ) < 1e-9
        );
        assert_eq!(Fixed::from_int(2).asin(), None);
        assert_eq!(Fixed::from_int(2).acos(), None);
    }

    #[test]
    fn the_hyperbolic_functions_match_the_reference() {
        assert_eq!(Fixed::ZERO.sinh().unwrap(), Fixed::ZERO);
        assert_eq!(Fixed::ZERO.cosh().unwrap(), Fixed::ONE);
        assert_eq!(Fixed::ZERO.tanh().unwrap(), Fixed::ZERO);
        assert!(distance(Fixed::ONE.sinh(), 1.0_f64.sinh()) < 1e-8);
        assert!(distance(Fixed::ONE.cosh(), 1.0_f64.cosh()) < 1e-8);
        assert!(distance(Fixed::ONE.tanh(), 1.0_f64.tanh()) < 1e-8);
        assert!(distance(Fixed::NEG_ONE.tanh(), (-1.0_f64).tanh()) < 1e-8);
    }

    #[test]
    fn the_inverse_hyperbolic_functions_match_the_reference() {
        assert_eq!(Fixed::ZERO.asinh().unwrap(), Fixed::ZERO);
        assert_eq!(Fixed::ONE.acosh().unwrap(), Fixed::ZERO);
        assert_eq!(Fixed::ZERO.atanh().unwrap(), Fixed::ZERO);
        assert!(distance(Fixed::ONE.asinh(), 1.0_f64.asinh()) < 1e-8);
        assert!(distance(Fixed::from_int(2).acosh(), 2.0_f64.acosh()) < 1e-8);
        assert!(distance(Fixed::from_parts(0, 2_147_483_648).atanh(), 0.5_f64.atanh()) < 1e-8);
    }

    #[test]
    fn the_powers_match_the_reference() {
        assert_eq!(Fixed::from_int(2).powi(10).unwrap(), Fixed::from_int(1024));
        assert_eq!(
            Fixed::from_int(2).powi(-2).unwrap(),
            Fixed::from_parts(0, 1_073_741_824)
        );
        assert_eq!(Fixed::from_int(2).powi(0).unwrap(), Fixed::ONE);
        assert!(distance(Fixed::from_int(2).powf(Fixed::from_int(10)), 1024.0) < 1e-5);
        assert!(
            distance(
                Fixed::from_int(4).powf(Fixed::from_parts(0, 2_147_483_648)),
                2.0
            ) < 1e-8
        );
        assert!(
            distance(
                Fixed::from_int(2).powf(Fixed::from_parts(0, 2_147_483_648)),
                2.0_f64.sqrt()
            ) < 1e-8
        );
        assert_eq!(
            Fixed::NEG_ONE.powf(Fixed::from_int(3)).unwrap(),
            Fixed::NEG_ONE
        );
    }

    #[test]
    fn the_hypotenuse_matches_the_reference() {
        assert_eq!(
            Fixed::from_int(3).hypot(Fixed::from_int(4)).unwrap(),
            Fixed::from_int(5)
        );
        assert!(
            distance(
                Fixed::from_int(2).hypot(Fixed::from_int(3)),
                13.0_f64.sqrt()
            ) < 1e-9
        );
    }

    #[test]
    fn the_angles_are_converted() {
        let pi = crate::consts::PI;

        assert!(distance(pi.to_degrees(), 180.0) < 1e-7);
        assert!(distance(Fixed::from_int(180).to_radians(), std::f64::consts::PI) < 1e-9);
    }

    #[test]
    fn the_fused_multiplication_addition_works() {
        assert_eq!(
            Fixed::ONE.mul_add(Fixed::from_int(2), Fixed::from_int(3)),
            Fixed::from_int(7)
        );
    }

    #[test]
    fn the_functions_are_accurate_over_the_range() {
        let step = Fixed::from_parts(0, 429_496_729);
        let mut value = step;

        for _ in 0..30 {
            let reference = value.to_f64();

            assert!(distance(value.sin(), reference.sin()) < 1e-9);
            assert!(distance(value.cos(), reference.cos()) < 1e-9);
            assert!(distance(value.tan(), reference.tan()) < 1e-9);
            assert!(distance(value.atan(), reference.atan()) < 1e-9);
            assert!(distance(value.ln(), reference.ln()) < 1e-9);
            assert!(distance(value.log2(), reference.log2()) < 1e-9);
            assert!(distance(value.sqrt(), reference.sqrt()) < 1e-9);
            assert!(distance(value.exp(), reference.exp()) < 1e-9 * reference.exp().abs());
            assert!(distance(value.cbrt(), reference.cbrt()) < 1e-8);

            value += step;
        }
    }
}
