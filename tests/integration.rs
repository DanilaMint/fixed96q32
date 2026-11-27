//! Integration tests of the public API of the crate.

use std::string::ToString;

use fixed96q32::consts::{PI, SQRT_2};
use fixed96q32::fixed::Fixed;

#[test]
fn values_survive_a_formatting_round_trip() {
    let values = [
        Fixed::ZERO,
        Fixed::ONE,
        Fixed::NEG_ONE,
        Fixed::HALF,
        Fixed::MIN,
        Fixed::MAX,
        Fixed::from_parts(0, 17_179_869),
        Fixed::from_parts(-1, 2_147_483_648),
        Fixed::from_parts(123_456, 789_012),
        Fixed::from_parts(-123_456, 789_012),
    ];

    for value in values {
        let text = value.to_string();
        assert_eq!(
            text.parse::<Fixed>().unwrap(),
            value,
            "round trip of {text}"
        );
    }
}

#[test]
fn values_are_formatted_like_plain_decimal_numbers() {
    assert_eq!(Fixed::ZERO.to_string(), "0");
    assert_eq!(Fixed::ONE.to_string(), "1");
    assert_eq!(Fixed::from_int(-42).to_string(), "-42");
    assert_eq!(Fixed::from_parts(0, 17_179_869).to_string(), "0.004");
    assert_eq!(Fixed::HALF.to_string(), "0.5");
    assert_eq!(Fixed::from_parts(-1, 2_147_483_648).to_string(), "-1.5");
    assert_eq!(Fixed::MIN.to_string(), "-39614081257132168796771975168");
}

#[test]
#[should_panic(expected = "attempt to add two Fixed values with overflow")]
fn the_addition_panics_on_overflow() {
    let _ = Fixed::MAX + Fixed::ONE;
}

#[test]
#[should_panic(expected = "attempt to multiply two Fixed values with overflow")]
fn the_multiplication_panics_on_overflow() {
    let _ = Fixed::MAX * Fixed::from_int(2);
}

#[test]
fn the_overflow_can_be_handled_explicitly() {
    assert_eq!(Fixed::MAX.checked_add(Fixed::ONE), None);
    assert_eq!(Fixed::MAX.saturating_add(Fixed::ONE), Fixed::MAX);
    assert_eq!(Fixed::MIN.saturating_sub(Fixed::ONE), Fixed::MIN);
    assert_eq!(Fixed::MAX.saturating_mul(Fixed::from_int(2)), Fixed::MAX);

    assert_eq!(
        Fixed::MAX.overflowing_add(Fixed::ONE),
        (Fixed::MAX.wrapping_add(Fixed::ONE), true)
    );
    assert_eq!(
        Fixed::ONE.overflowing_add(Fixed::ONE),
        (Fixed::from_int(2), false)
    );

    assert_eq!(
        Fixed::MAX.wrapping_add(Fixed::ONE),
        Fixed::MAX.wrapping_add(Fixed::ONE)
    );
}

#[test]
#[should_panic(expected = "attempt to divide a Fixed value by zero")]
fn the_division_by_zero_panics() {
    let _ = Fixed::ONE / Fixed::ZERO;
}

#[test]
fn the_checked_operations_report_the_overflow() {
    assert_eq!(Fixed::MAX.checked_add(Fixed::ONE), None);
    assert_eq!(Fixed::MAX.checked_sub(Fixed::NEG_ONE), None);
    assert_eq!(Fixed::MAX.checked_mul(Fixed::from_int(2)), None);
    assert_eq!(Fixed::ONE.checked_div(Fixed::ZERO), None);
    assert_eq!(
        Fixed::from_int(6).checked_div(Fixed::from_int(3)),
        Some(Fixed::from_int(2))
    );
}

#[test]
fn the_conversions_preserve_the_value() {
    assert_eq!(Fixed::from(42i32), Fixed::from_int(42));
    assert_eq!(Fixed::from(42u64), Fixed::from_int(42));
    assert_eq!(
        Fixed::try_from(1.5f64).unwrap(),
        Fixed::from_parts(1, 2_147_483_648)
    );
    assert_eq!(Fixed::try_from(1.5f64).unwrap().to_f64(), 1.5);
    assert!(Fixed::try_from(f64::NAN).is_err());
    assert!(Fixed::try_from(f64::INFINITY).is_err());
}

#[test]
fn the_fixed_numbers_are_converted_into_integers() {
    assert_eq!(i32::try_from(Fixed::from_parts(2, 7)), Ok(2));
    assert_eq!(i32::try_from(Fixed::from_parts(-2, 7)), Ok(-2));
    assert_eq!(u64::try_from(Fixed::from_int(42)), Ok(42));
    assert_eq!(usize::try_from(Fixed::from_int(42)), Ok(42));
    assert_eq!(i128::try_from(Fixed::MAX), Ok((1i128 << 95) - 1));

    assert_eq!(
        i8::try_from(Fixed::from_int(128)),
        Err(fixed96q32::froms::ConversionError::OutOfRange)
    );
    assert_eq!(
        u8::try_from(Fixed::NEG_ONE),
        Err(fixed96q32::froms::ConversionError::OutOfRange)
    );

    assert_eq!(isize::from(Fixed::from_parts(2, 7)), 2);
    assert_eq!(isize::from(Fixed::MAX), isize::MAX);
}

#[test]
fn the_constants_match_the_reference_values() {
    assert!((PI.to_f64() - std::f64::consts::PI).abs() < 1e-9);
    assert!((SQRT_2.to_f64() - std::f64::consts::SQRT_2).abs() < 1e-9);
    assert!(((SQRT_2 * SQRT_2).to_f64() - 2.0).abs() < 1e-9);
}

#[cfg(feature = "num-traits")]
#[test]
fn the_num_traits_interoperability_works() {
    use ::num_traits::{FromPrimitive, One, Zero};

    assert_eq!(Fixed::zero(), Fixed::ZERO);
    assert_eq!(Fixed::one(), Fixed::ONE);
    assert_eq!(Fixed::from_f64(1.5), Fixed::from_parts(1, 2_147_483_648));
    assert!(Fixed::from_i64(-3).unwrap().is_negative());
    assert!(Fixed::from_i64(3).unwrap().is_positive());
    assert!((Fixed::one() + Fixed::one()).is_positive());
    assert_eq!(Fixed::from_i64(3).unwrap().abs(), Fixed::from_int(3));
}

#[cfg(feature = "serde")]
#[test]
fn the_serde_interoperability_works() {
    let value = Fixed::from_parts(-12, 345_678_901);

    let text = serde_json::to_string(&value).unwrap();
    assert!(text.starts_with('"') && text.ends_with('"'));
    assert_eq!(serde_json::from_str::<Fixed>(&text).unwrap(), value);

    let from_number = serde_json::from_str::<Fixed>("1.5").unwrap();
    assert_eq!(from_number, Fixed::from_parts(1, 2_147_483_648));

    let from_integer = serde_json::from_str::<Fixed>("42").unwrap();
    assert_eq!(from_integer, Fixed::from_int(42));
}
