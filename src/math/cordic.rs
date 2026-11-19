//! The CORDIC engine of the [`Fixed`] type.
//!
//! The module holds the tables of the algorithm and the arithmetic of the
//! internal Q64 format, which is used by the public methods of the
//! [`Fixed`](crate::Fixed) type.

use crate::fixed::{Fixed, mul_wide};

/// The number of the CORDIC iterations.
pub(crate) const ITERATIONS: usize = 48;

/// One in the internal Q64 format.
pub(crate) const ONE_Q64: i128 = 1i128 << 64;

/// The table of the angles `atan(2**-i)` in the Q64 format.
pub(crate) const ATAN_TABLE: [i128; ITERATIONS] = [
    14488038916154245685, // atan(2^-0)
    8552788783625223587,  // atan(2^-1)
    4519058702220769989,  // atan(2^-2)
    2293944758691655110,  // atan(2^-3)
    1151423813380928739,  // atan(2^-4)
    576273212193474858,   // atan(2^-5)
    288206923339027287,   // atan(2^-6)
    144112256152217971,   // atan(2^-7)
    72057227537407417,    // atan(2^-8)
    36028751206084335,    // atan(2^-9)
    18014392782862199,    // atan(2^-10)
    9007198538913212,     // atan(2^-11)
    4503599537892014,     // atan(2^-12)
    2251799802500437,     // atan(2^-13)
    1125899905444523,     // atan(2^-14)
    562949953246549,      // atan(2^-15)
    281474976688811,      // atan(2^-16)
    140737488352597,      // atan(2^-17)
    70368744177323,       // atan(2^-18)
    35184372088789,       // atan(2^-19)
    17592186044411,       // atan(2^-20)
    8796093022207,        // atan(2^-21)
    4398046511104,        // atan(2^-22)
    2199023255552,        // atan(2^-23)
    1099511627776,        // atan(2^-24)
    549755813888,         // atan(2^-25)
    274877906944,         // atan(2^-26)
    137438953472,         // atan(2^-27)
    68719476736,          // atan(2^-28)
    34359738368,          // atan(2^-29)
    17179869184,          // atan(2^-30)
    8589934592,           // atan(2^-31)
    4294967296,           // atan(2^-32)
    2147483648,           // atan(2^-33)
    1073741824,           // atan(2^-34)
    536870912,            // atan(2^-35)
    268435456,            // atan(2^-36)
    134217728,            // atan(2^-37)
    67108864,             // atan(2^-38)
    33554432,             // atan(2^-39)
    16777216,             // atan(2^-40)
    8388608,              // atan(2^-41)
    4194304,              // atan(2^-42)
    2097152,              // atan(2^-43)
    1048576,              // atan(2^-44)
    524288,               // atan(2^-45)
    262144,               // atan(2^-46)
    131072,               // atan(2^-47)
];

/// The table of the logarithms `ln(1 + 2**-i)` in the Q64 format.
pub(crate) const LOG_TABLE: [i128; ITERATIONS] = [
    7479511080090283979, // ln(1 + 2^-1)
    4116271982791902040, // ln(1 + 2^-2)
    2172713514977912298, // ln(1 + 2^-3)
    1118326883213202225, // ln(1 + 2^-4)
    567636912149151763,  // ln(1 + 2^-5)
    286001761100004474,  // ln(1 + 2^-6)
    143555153080252560,  // ln(1 + 2^-7)
    71917221983051266,   // ln(1 + 2^-8)
    35993658392855450,   // ln(1 + 2^-9)
    18005608138891807,   // ln(1 + 2^-10)
    9005000947051281,    // ln(1 + 2^-11)
    4503049961018713,    // ln(1 + 2^-12)
    2251662385915563,    // ln(1 + 2^-13)
    1125865548502293,    // ln(1 + 2^-14)
    562941363661479,     // ln(1 + 2^-15)
    281472829248853,     // ln(1 + 2^-16)
    140736951487147,     // ln(1 + 2^-17)
    70368609960277,      // ln(1 + 2^-18)
    35184338534443,      // ln(1 + 2^-19)
    17592177655813,      // ln(1 + 2^-20)
    8796090925057,       // ln(1 + 2^-21)
    4398045986816,       // ln(1 + 2^-22)
    2199023124480,       // ln(1 + 2^-23)
    1099511595008,       // ln(1 + 2^-24)
    549755805696,        // ln(1 + 2^-25)
    274877904896,        // ln(1 + 2^-26)
    137438952960,        // ln(1 + 2^-27)
    68719476608,         // ln(1 + 2^-28)
    34359738336,         // ln(1 + 2^-29)
    17179869176,         // ln(1 + 2^-30)
    8589934590,          // ln(1 + 2^-31)
    4294967296,          // ln(1 + 2^-32)
    2147483648,          // ln(1 + 2^-33)
    1073741824,          // ln(1 + 2^-34)
    536870912,           // ln(1 + 2^-35)
    268435456,           // ln(1 + 2^-36)
    134217728,           // ln(1 + 2^-37)
    67108864,            // ln(1 + 2^-38)
    33554432,            // ln(1 + 2^-39)
    16777216,            // ln(1 + 2^-40)
    8388608,             // ln(1 + 2^-41)
    4194304,             // ln(1 + 2^-42)
    2097152,             // ln(1 + 2^-43)
    1048576,             // ln(1 + 2^-44)
    524288,              // ln(1 + 2^-45)
    262144,              // ln(1 + 2^-46)
    131072,              // ln(1 + 2^-47)
    65536,               // ln(1 + 2^-48)
];

/// The table of the reciprocals `1 / (1 + 2**-i)` in the Q64 format.
const RECIPROCAL_TABLE: [i128; ITERATIONS] = [
    12297829382473034411, // 1 / (1 + 2^-1)
    14757395258967641293, // 1 / (1 + 2^-2)
    16397105843297379214, // 1 / (1 + 2^-3)
    17361641481138401521, // 1 / (1 + 2^-4)
    17887751829051686416, // 1 / (1 + 2^-5)
    18162948011037096976, // 1 / (1 + 2^-6)
    18303746057634283774, // 1 / (1 + 2^-7)
    18374966859414961921, // 1 / (1 + 2^-8)
    18410785508263724030, // 1 / (1 + 2^-9)
    18428747250223005712, // 1 / (1 + 2^-10)
    18437741270354886144, // 1 / (1 + 2^-11)
    18442241573325438960, // 1 / (1 + 2^-12)
    18444492548740222976, // 1 / (1 + 2^-13)
    18445618242517991680, // 1 / (1 + 2^-14)
    18446181140935475216, // 1 / (1 + 2^-15)
    18446462603027742721, // 1 / (1 + 2^-16)
    18446603337294929920, // 1 / (1 + 2^-17)
    18446673705233808384, // 1 / (1 + 2^-18)
    18446708889404571520, // 1 / (1 + 2^-19)
    18446726481540284400, // 1 / (1 + 2^-20)
    18446735277620723710, // 1 / (1 + 2^-21)
    18446739675664089088, // 1 / (1 + 2^-22)
    18446741874686558208, // 1 / (1 + 2^-23)
    18446742974197989376, // 1 / (1 + 2^-24)
    18446743523953754112, // 1 / (1 + 2^-25)
    18446743798831648768, // 1 / (1 + 2^-26)
    18446743936270599168, // 1 / (1 + 2^-27)
    18446744004990075136, // 1 / (1 + 2^-28)
    18446744039349813312, // 1 / (1 + 2^-29)
    18446744056529682448, // 1 / (1 + 2^-30)
    18446744065119617028, // 1 / (1 + 2^-31)
    18446744069414584321, // 1 / (1 + 2^-32)
    18446744071562067968, // 1 / (1 + 2^-33)
    18446744072635809792, // 1 / (1 + 2^-34)
    18446744073172680704, // 1 / (1 + 2^-35)
    18446744073441116160, // 1 / (1 + 2^-36)
    18446744073575333888, // 1 / (1 + 2^-37)
    18446744073642442752, // 1 / (1 + 2^-38)
    18446744073675997184, // 1 / (1 + 2^-39)
    18446744073692774400, // 1 / (1 + 2^-40)
    18446744073701163008, // 1 / (1 + 2^-41)
    18446744073705357312, // 1 / (1 + 2^-42)
    18446744073707454464, // 1 / (1 + 2^-43)
    18446744073708503040, // 1 / (1 + 2^-44)
    18446744073709027328, // 1 / (1 + 2^-45)
    18446744073709289472, // 1 / (1 + 2^-46)
    18446744073709420544, // 1 / (1 + 2^-47)
    18446744073709486080, // 1 / (1 + 2^-48)
];

pub(crate) const PI_Q64: i128 = 57952155664616982739;
pub(crate) const FRAC_PI_2_Q64: i128 = 28976077832308491370;
pub(crate) const LN_2_Q64: i128 = 12786308645202655660;
pub(crate) const LN_10_Q64: i128 = 42475197918399869020;
pub(crate) const K_INV_Q64: i128 = 11201839480117811816;
pub(crate) const DEGREES_PER_RADIAN_Q64: i128 = 1056920581181520441539;
pub(crate) const RADIANS_PER_DEGREE_Q64: i128 = 321956420358983237;

/// Converts a value into the internal Q64 format.
#[inline]
pub(crate) fn to_internal(value: Fixed) -> Option<i128> {
    let raw = value.to_raw();
    if raw.unsigned_abs() >= (1u128 << 95) {
        return None;
    }
    Some(raw << 32)
}

/// Converts an internal Q64 value into a [`Fixed`], rounding to the nearest.
#[inline]
pub(crate) fn from_internal(value: i128) -> Option<Fixed> {
    let magnitude = value.unsigned_abs();
    let rounded = (magnitude + (1u128 << 31)) >> 32;

    if value < 0 {
        if rounded > (1u128 << 127) {
            return None;
        }
        Some(Fixed::from_raw((rounded as i128).wrapping_neg()))
    } else if rounded > i128::MAX as u128 {
        None
    } else {
        Some(Fixed::from_raw(rounded as i128))
    }
}

/// Multiplies two internal values, truncating the discarded bits.
pub(crate) fn multiply(left: i128, right: i128) -> i128 {
    let limbs = mul_wide(left.unsigned_abs(), right.unsigned_abs());
    let low = ((limbs[1] as u128) << 64) | limbs[0] as u128;
    let high = ((limbs[3] as u128) << 64) | limbs[2] as u128;
    let magnitude = ((low >> 64) | (high << 64)) as i128;

    if (left < 0) != (right < 0) {
        magnitude.wrapping_neg()
    } else {
        magnitude
    }
}

/// Divides two internal values, truncating the discarded bits.
pub(crate) fn divide(numerator: i128, denominator: i128) -> Option<i128> {
    if denominator == 0 {
        return None;
    }

    let negative = (numerator < 0) != (denominator < 0);
    let dividend = numerator.unsigned_abs();
    let divisor = denominator.unsigned_abs();

    let units = dividend / divisor;
    if units >= (1u128 << 63) {
        return None;
    }

    // The fractional part is extracted in two steps of 32 bits, the hardware
    // division is much faster than the bit by bit one.
    let mut remainder = dividend % divisor;
    let mut fraction: u64 = 0;

    for _ in 0..2 {
        remainder <<= 32;
        let part = remainder / divisor;
        remainder %= divisor;
        fraction = (fraction << 32) | part as u64;
    }

    let magnitude = ((units << 64) | fraction as u128) as i128;
    Some(if negative {
        magnitude.wrapping_neg()
    } else {
        magnitude
    })
}

/// Computes the sine and the cosine of an angle with the rotation mode.
pub(crate) fn sine_and_cosine(angle: i128) -> Option<(i128, i128)> {
    let quadrant = divide_round(angle, FRAC_PI_2_Q64);
    let remainder = angle - quadrant * FRAC_PI_2_Q64;

    let mut x = K_INV_Q64;
    let mut y = 0;
    let mut z = remainder;

    for (index, table) in ATAN_TABLE.iter().enumerate() {
        let dx = x >> index;
        let dy = y >> index;

        if z >= 0 {
            x -= dy;
            y += dx;
            z -= table;
        } else {
            x += dy;
            y -= dx;
            z += table;
        }
    }

    // The gain of the algorithm is compensated by the initial value, which is
    // the reciprocal of the gain.
    let sine = y;
    let cosine = x;

    Some(match quadrant.rem_euclid(4) {
        0 => (sine, cosine),
        1 => (cosine, -sine),
        2 => (-sine, -cosine),
        _ => (-cosine, sine),
    })
}

/// Computes the arc tangent of the ratio with the vectoring mode.
pub(crate) fn arc_tangent(y: i128, x: i128) -> Option<i128> {
    let mut x = x;
    let mut y = y;
    let mut z = 0;

    for (index, table) in ATAN_TABLE.iter().enumerate() {
        let dx = x >> index;
        let dy = y >> index;

        if y > 0 {
            let new_x = x + dy;
            let new_y = y - dx;
            x = new_x;
            y = new_y;
            z += table;
        } else if y < 0 {
            let new_x = x - dy;
            let new_y = y + dx;
            x = new_x;
            y = new_y;
            z -= table;
        }
    }

    Some(z)
}

/// Computes the natural logarithm with the logarithmic mode.
pub(crate) fn logarithm(value: i128) -> Option<i128> {
    if value <= 0 {
        return None;
    }

    // The value is split into the mantissa in `[1, 2)` and the exponent.
    let magnitude = value as u128;
    let bits = 128 - magnitude.leading_zeros();
    let exponent = bits as i32 - 65;
    let mantissa = if exponent >= 0 {
        (magnitude >> exponent) as i128
    } else {
        (magnitude << (-exponent)) as i128
    };

    // The mantissa is expanded into the product of the factors `1 + 2**-i`.
    let mut product = ONE_Q64;
    let mut logarithm = 0;

    if mantissa != ONE_Q64 {
        for index in 1..=ITERATIONS {
            if product <= mantissa {
                product += product >> index;
                logarithm += LOG_TABLE[index - 1];
            } else {
                product = multiply(product, RECIPROCAL_TABLE[index - 1]);
                logarithm -= LOG_TABLE[index - 1];
            }
        }
    }

    Some(logarithm + (exponent as i128) * LN_2_Q64)
}

/// Computes the exponential function with the exponential mode.
pub(crate) fn exponential(internal: i128) -> Option<i128> {
    // The value is split into the integer multiple of `ln(2)` and the
    // remainder, which stays within the range of the expansion.
    let exponent = divide_round(internal, LN_2_Q64);
    let remainder = internal - exponent * LN_2_Q64;

    let mut product = ONE_Q64;
    let mut logarithm = 0;

    if remainder != 0 {
        for index in 1..=ITERATIONS {
            if logarithm <= remainder {
                product += product >> index;
                logarithm += LOG_TABLE[index - 1];
            } else {
                product = multiply(product, RECIPROCAL_TABLE[index - 1]);
                logarithm -= LOG_TABLE[index - 1];
            }
        }
    }

    // The result is the product scaled by `2**exponent`.
    if !(-126..=62).contains(&exponent) {
        return None;
    }

    if exponent >= 0 {
        Some(product << exponent)
    } else {
        Some(product >> (-exponent))
    }
}

/// Divides two values, rounding the result to the nearest integer.
pub(crate) fn divide_round(numerator: i128, denominator: i128) -> i128 {
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;

    if remainder * 2 >= denominator {
        quotient + 1
    } else if remainder * 2 < -denominator {
        quotient - 1
    } else {
        quotient
    }
}

/// Computes the square root of an internal value.
pub(crate) fn sqrt_internal(value: i128) -> Option<i128> {
    if value < 0 {
        return None;
    }

    if value == 0 {
        return Some(0);
    }

    let magnitude = value as u128;
    let root = integer_sqrt(magnitude);
    let remainder = magnitude - root * root;
    let correction = (remainder << 32) / (root << 1);

    Some(((root << 32) + correction) as i128)
}

/// Computes the integer square root of a value.
pub(crate) fn integer_sqrt(value: u128) -> u128 {
    let mut remainder = 0u128;
    let mut root = 0u128;
    let bits = 128 - value.leading_zeros();
    let mut shift = bits.div_ceil(2) * 2;

    while shift > 0 {
        shift -= 2;
        remainder = (remainder << 2) | ((value >> shift) & 3);
        root <<= 1;
        let trial = (root << 1) | 1;
        if remainder >= trial {
            remainder -= trial;
            root += 1;
        }
    }

    root
}
