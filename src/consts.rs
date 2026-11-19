//! Mathematical constants of the [`Fixed`] type.
//!
//! The set mirrors the constants provided by `core::f64::consts`; every value
//! is rounded to the nearest representable [`Fixed`].

use crate::fixed::Fixed;

/// The ratio of a circle's circumference to its diameter.
pub const PI: Fixed = Fixed::from_raw(13_493_037_705);

/// The ratio of a circle's circumference to its diameter, doubled.
pub const TAU: Fixed = Fixed::from_raw(26_986_075_409);

/// The ratio of a circle's circumference to its diameter, halved.
pub const FRAC_PI_2: Fixed = Fixed::from_raw(6_746_518_852);

/// The ratio of a circle's circumference to its diameter, divided by three.
pub const FRAC_PI_3: Fixed = Fixed::from_raw(4_497_679_235);

/// The ratio of a circle's circumference to its diameter, divided by four.
pub const FRAC_PI_4: Fixed = Fixed::from_raw(3_373_259_426);

/// The ratio of a circle's circumference to its diameter, divided by six.
pub const FRAC_PI_6: Fixed = Fixed::from_raw(2_248_839_617);

/// The ratio of a circle's circumference to its diameter, divided by eight.
pub const FRAC_PI_8: Fixed = Fixed::from_raw(1_686_629_713);

/// The reciprocal of [`PI`].
pub const FRAC_1_PI: Fixed = Fixed::from_raw(1_367_130_551);

/// Twice the reciprocal of [`PI`].
pub const FRAC_2_PI: Fixed = Fixed::from_raw(2_734_261_102);

/// The reciprocal of the square root of two.
pub const FRAC_1_SQRT_2: Fixed = Fixed::from_raw(3_037_000_500);

/// Twice the reciprocal of the square root of pi.
pub const FRAC_2_SQRT_PI: Fixed = Fixed::from_raw(4_846_351_620);

/// The square root of two.
pub const SQRT_2: Fixed = Fixed::from_raw(6_074_001_000);

/// The square root of pi.
pub const SQRT_PI: Fixed = Fixed::from_raw(7_612_631_323);

/// Euler's number.
pub const E: Fixed = Fixed::from_raw(11_674_931_555);

/// The base-2 logarithm of [`E`].
pub const LOG2_E: Fixed = Fixed::from_raw(6_196_328_019);

/// The base-10 logarithm of [`E`].
pub const LOG10_E: Fixed = Fixed::from_raw(1_865_280_597);

/// The natural logarithm of two.
pub const LN_2: Fixed = Fixed::from_raw(2_977_044_472);

/// The natural logarithm of ten.
pub const LN_10: Fixed = Fixed::from_raw(9_889_527_671);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::Fixed;

    #[test]
    fn constants_are_close_to_the_reference_values() {
        assert!((PI.to_f64() - std::f64::consts::PI).abs() < 1e-9);
        assert!((TAU.to_f64() - std::f64::consts::TAU).abs() < 1e-9);
        assert!((FRAC_PI_2.to_f64() - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
        assert!((FRAC_PI_3.to_f64() - std::f64::consts::FRAC_PI_3).abs() < 1e-9);
        assert!((FRAC_PI_4.to_f64() - std::f64::consts::FRAC_PI_4).abs() < 1e-9);
        assert!((FRAC_PI_6.to_f64() - std::f64::consts::FRAC_PI_6).abs() < 1e-9);
        assert!((FRAC_PI_8.to_f64() - std::f64::consts::FRAC_PI_8).abs() < 1e-9);
        assert!((FRAC_1_PI.to_f64() - std::f64::consts::FRAC_1_PI).abs() < 1e-9);
        assert!((FRAC_2_PI.to_f64() - std::f64::consts::FRAC_2_PI).abs() < 1e-9);
        assert!((FRAC_1_SQRT_2.to_f64() - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-9);
        assert!((FRAC_2_SQRT_PI.to_f64() - std::f64::consts::FRAC_2_SQRT_PI).abs() < 1e-9);
        assert!((SQRT_2.to_f64() - std::f64::consts::SQRT_2).abs() < 1e-9);
        assert!((SQRT_PI.to_f64() - 1.772_453_850_905_516).abs() < 1e-9);
        assert!((E.to_f64() - std::f64::consts::E).abs() < 1e-9);
        assert!((LOG2_E.to_f64() - std::f64::consts::LOG2_E).abs() < 1e-9);
        assert!((LOG10_E.to_f64() - std::f64::consts::LOG10_E).abs() < 1e-9);
        assert!((LN_2.to_f64() - std::f64::consts::LN_2).abs() < 1e-9);
        assert!((LN_10.to_f64() - std::f64::consts::LN_10).abs() < 1e-9);
    }

    #[test]
    fn identities_hold() {
        assert!((TAU.to_f64() - (Fixed::from_int(2) * PI).to_f64()).abs() < 1e-9);
        assert!((FRAC_PI_2.to_f64() * 2.0 - PI.to_f64()).abs() < 1e-9);
        assert!((FRAC_PI_4.to_f64() * 4.0 - PI.to_f64()).abs() < 1e-9);
        assert!((SQRT_2.to_f64() * SQRT_2.to_f64() - 2.0).abs() < 1e-9);
        assert!(
            (LN_2.to_f64() + LN_2.to_f64() - (Fixed::from_int(2) * LN_2).to_f64()).abs() < 1e-9
        );
    }
}
