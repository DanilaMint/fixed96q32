//! Implementation of the [`core::ops`] traits for the [`Fixed`] type.
//!
//! The operators panic on overflow, exactly like the operators of the
//! primitive integer types do. The [`Fixed::checked_add`],
//! [`Fixed::saturating_add`] and [`Fixed::wrapping_add`] methods and their
//! counterparts cover the cases when the overflow has to be handled.

use core::iter::{Product, Sum};
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

use crate::fixed::Fixed;

impl Add for Fixed {
    /// Adds two values.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed96q32::Fixed;
    ///
    /// assert_eq!(Fixed::ONE + Fixed::HALF, Fixed::from_parts(1, 2_147_483_648));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the result overflows the representable range.
    type Output = Fixed;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        match self.checked_add(rhs) {
            Some(value) => value,
            None => panic!("attempt to add two Fixed values with overflow"),
        }
    }
}

impl AddAssign for Fixed {
    /// Adds a value.
    ///
    /// # Panics
    ///
    /// Panics if the result overflows the representable range.
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Fixed {
    /// Subtracts two values.
    ///
    /// # Panics
    ///
    /// Panics if the result overflows the representable range.
    type Output = Fixed;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        match self.checked_sub(rhs) {
            Some(value) => value,
            None => panic!("attempt to subtract two Fixed values with overflow"),
        }
    }
}

impl SubAssign for Fixed {
    /// Subtracts a value.
    ///
    /// # Panics
    ///
    /// Panics if the result overflows the representable range.
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for Fixed {
    /// Multiplies two values.
    ///
    /// # Panics
    ///
    /// Panics if the result overflows the representable range.
    type Output = Fixed;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        match self.checked_mul(rhs) {
            Some(value) => value,
            None => panic!("attempt to multiply two Fixed values with overflow"),
        }
    }
}

impl MulAssign for Fixed {
    /// Multiplies by a value.
    ///
    /// # Panics
    ///
    /// Panics if the result overflows the representable range.
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Div for Fixed {
    /// Divides two values.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero or if the result overflows the
    /// representable range.
    type Output = Fixed;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        match self.checked_div(rhs) {
            Some(value) => value,
            None => {
                if rhs.is_zero() {
                    panic!("attempt to divide a Fixed value by zero");
                }
                panic!("attempt to divide two Fixed values with overflow");
            }
        }
    }
}

impl DivAssign for Fixed {
    /// Divides by a value.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero or if the result overflows the
    /// representable range.
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl Rem for Fixed {
    /// Returns the remainder of the division, truncated towards zero.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    type Output = Fixed;

    #[inline]
    fn rem(self, rhs: Self) -> Self::Output {
        assert!(
            rhs.to_raw() != 0,
            "attempt to calculate the remainder of a division by zero"
        );
        Fixed::from_raw(self.to_raw() % rhs.to_raw())
    }
}

impl RemAssign for Fixed {
    /// Assigns the remainder of the division, truncated towards zero.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[inline]
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs;
    }
}

impl Neg for Fixed {
    /// Negates the value.
    ///
    /// # Panics
    ///
    /// Panics for [`Fixed::MIN`], which has no positive counterpart.
    type Output = Fixed;

    #[inline]
    fn neg(self) -> Self::Output {
        match self.checked_neg() {
            Some(value) => value,
            None => panic!("attempt to negate a Fixed value with overflow"),
        }
    }
}

impl Sum for Fixed {
    /// Adds up the values.
    ///
    /// # Panics
    ///
    /// Panics if the result overflows the representable range.
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Fixed::ZERO, |accumulator, value| accumulator + value)
    }
}

impl Product for Fixed {
    /// Multiplies the values.
    ///
    /// # Panics
    ///
    /// Panics if the result overflows the representable range.
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Fixed::ONE, |accumulator, value| accumulator * value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_operators_work_inside_the_range() {
        assert_eq!(Fixed::ONE + Fixed::ONE, Fixed::from_int(2));
        assert_eq!(Fixed::ONE - Fixed::ONE, Fixed::ZERO);
        assert_eq!(Fixed::from_int(3) * Fixed::from_int(4), Fixed::from_int(12));
        assert_eq!(
            Fixed::from_int(-3) * Fixed::from_int(4),
            Fixed::from_int(-12)
        );
    }

    #[test]
    #[should_panic(expected = "attempt to add two Fixed values with overflow")]
    fn the_addition_panics_on_overflow() {
        let _ = Fixed::MAX + Fixed::ONE;
    }

    #[test]
    #[should_panic(expected = "attempt to subtract two Fixed values with overflow")]
    fn the_subtraction_panics_on_overflow() {
        let _ = Fixed::MIN - Fixed::ONE;
    }

    #[test]
    #[should_panic(expected = "attempt to multiply two Fixed values with overflow")]
    fn the_multiplication_panics_on_overflow() {
        let _ = Fixed::MAX * Fixed::from_int(2);
    }

    #[test]
    #[should_panic(expected = "attempt to divide a Fixed value by zero")]
    fn the_division_by_zero_panics() {
        let _ = Fixed::ONE / Fixed::ZERO;
    }

    #[test]
    fn the_division_keeps_the_fractional_part() {
        assert_eq!(Fixed::from_int(1) / Fixed::from_int(2), Fixed::HALF);
        assert_eq!(
            Fixed::from_int(-1) / Fixed::from_int(2),
            Fixed::from_raw(-(1i128 << 31))
        );
        assert_eq!(Fixed::from_int(7) / Fixed::from_int(7), Fixed::ONE);
    }

    #[test]
    #[should_panic(expected = "remainder of a division by zero")]
    fn the_remainder_by_zero_panics() {
        let _ = Fixed::ONE % Fixed::ZERO;
    }

    #[test]
    fn the_remainder_is_truncated_towards_zero() {
        assert_eq!(
            Fixed::from_parts(7, 500_000_000) % Fixed::from_int(2),
            Fixed::from_parts(1, 500_000_000)
        );
        assert_eq!(
            Fixed::from_parts(-7, 500_000_000) % Fixed::from_int(2),
            Fixed::from_parts(-1, 500_000_000)
        );
        assert_eq!(Fixed::from_int(7) % Fixed::from_int(3), Fixed::ONE);
    }

    #[test]
    fn the_negation_flips_the_sign() {
        assert_eq!(-Fixed::ONE, Fixed::NEG_ONE);
        assert_eq!(-Fixed::NEG_ONE, Fixed::ONE);
        assert_eq!(-Fixed::ZERO, Fixed::ZERO);
    }

    #[test]
    #[should_panic(expected = "attempt to negate a Fixed value with overflow")]
    fn the_negation_panics_on_overflow() {
        let _ = -Fixed::MIN;
    }

    #[test]
    fn the_assignment_operators_update_the_value() {
        let mut value = Fixed::ONE;
        value += Fixed::ONE;
        assert_eq!(value, Fixed::from_int(2));

        value -= Fixed::ONE;
        assert_eq!(value, Fixed::ONE);

        value *= Fixed::from_int(4);
        assert_eq!(value, Fixed::from_int(4));

        value /= Fixed::from_int(2);
        assert_eq!(value, Fixed::from_int(2));

        value %= Fixed::from_int(3);
        assert_eq!(value, Fixed::from_int(2));
    }

    #[test]
    fn sum_and_product_fold_the_values() {
        let values = [Fixed::ONE, Fixed::from_int(2), Fixed::from_int(3)];
        assert_eq!(values.iter().copied().sum::<Fixed>(), Fixed::from_int(6));
        assert_eq!(
            values.iter().copied().product::<Fixed>(),
            Fixed::from_int(6)
        );
    }
}
