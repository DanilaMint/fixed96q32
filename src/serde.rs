//! Serialization and deserialization with the `serde` crate.
//!
//! The module is enabled by the `serde` feature.
//!
//! # Representation
//!
//! The human readable formats, such as JSON, receive the decimal
//! representation of the value, for example `"0.004"`. The text form keeps
//! the exact value of a fixed-point number, while the JSON numbers are
//! limited to the precision of a double.
//!
//! The binary formats receive the raw representation of the value split into
//! the `hi` and the `lo` halves, which is exact and compact.
//!
//! The deserialization accepts the decimal strings, the integers and the
//! floating point numbers, therefore the values written by the other
//! producers are read as well.
//!
//! # Examples
//!
//! ```
//! use fixed96q32::Fixed;
//!
//! let value = Fixed::from_parts(0, 17_179_869);
//! let text = serde_json::to_string(&value).unwrap();
//! assert_eq!(text, "\"0.004\"");
//!
//! let parsed: Fixed = serde_json::from_str(&text).unwrap();
//! assert_eq!(parsed, value);
//! ```

use core::fmt;

use ::serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use ::serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::fixed::Fixed;

/// The name of the serialized structure.
const STRUCT_NAME: &str = "Fixed";

/// The names of the fields of the serialized structure.
const STRUCT_FIELDS: &[&str] = &["hi", "lo"];

/// The raw representation of the binary form.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct RawParts {
    /// The high half of the raw representation.
    hi: u64,
    /// The low half of the raw representation.
    lo: u64,
}

impl RawParts {
    /// Splits the raw representation of a value into its two halves.
    #[inline]
    fn from_fixed(value: Fixed) -> Self {
        let bits = value.to_raw() as u128;
        Self {
            hi: (bits >> 64) as u64,
            lo: bits as u64,
        }
    }

    /// Reassembles the raw representation out of its two halves.
    #[inline]
    fn to_fixed(self) -> Fixed {
        let bits = ((self.hi as u128) << 64) | (self.lo as u128);
        Fixed::from_raw(bits as i128)
    }
}

impl Serialize for Fixed {
    /// Serializes the value.
    ///
    /// The human readable formats receive the decimal representation of the
    /// value, the binary formats receive the raw representation split into
    /// the `hi` and the `lo` halves.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            return serializer.collect_str(self);
        }

        let parts = RawParts::from_fixed(*self);
        let mut state = serializer.serialize_struct(STRUCT_NAME, 2)?;
        state.serialize_field("hi", &parts.hi)?;
        state.serialize_field("lo", &parts.lo)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Fixed {
    /// Deserializes the value.
    ///
    /// The decimal strings, the integers and the floating point numbers are
    /// accepted, as well as the binary form with the `hi` and the `lo` halves.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_any(FixedVisitor)
        } else {
            deserializer.deserialize_struct(STRUCT_NAME, STRUCT_FIELDS, FixedVisitor)
        }
    }
}

/// The visitor of the [`Fixed`] values.
struct FixedVisitor;

impl<'de> Visitor<'de> for FixedVisitor {
    /// The visited value.
    type Value = Fixed;

    /// Describes the expected input.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a fixed-point number as a decimal string, an integer or a float")
    }

    /// Reads the decimal representation of a value.
    fn visit_str<E: de::Error>(self, value: &str) -> Result<Fixed, E> {
        value.parse::<Fixed>().map_err(de::Error::custom)
    }

    /// Reads an integral value.
    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Fixed, E> {
        Ok(Fixed::from_int(value))
    }

    /// Reads an unsigned integral value.
    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Fixed, E> {
        Ok(Fixed::from_raw((value as i128) << 32))
    }

    /// Reads a wide integral value.
    fn visit_i128<E: de::Error>(self, value: i128) -> Result<Fixed, E> {
        Fixed::try_from(value).map_err(de::Error::custom)
    }

    /// Reads a wide unsigned integral value.
    fn visit_u128<E: de::Error>(self, value: u128) -> Result<Fixed, E> {
        Fixed::try_from(value).map_err(de::Error::custom)
    }

    /// Reads a floating point value.
    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Fixed, E> {
        Fixed::try_from(value).map_err(de::Error::custom)
    }

    /// Reads the two halves of the raw representation.
    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Fixed, A::Error> {
        let hi: u64 = sequence
            .next_element()?
            .ok_or_else(|| de::Error::custom("the missing high half of the raw representation"))?;
        let lo: u64 = sequence
            .next_element()?
            .ok_or_else(|| de::Error::custom("the missing low half of the raw representation"))?;
        Ok(RawParts { hi, lo }.to_fixed())
    }

    /// Reads the two halves of the raw representation.
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Fixed, A::Error> {
        let mut parts = RawParts { hi: 0, lo: 0 };

        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "hi" => parts.hi = map.next_value()?,
                "lo" => parts.lo = map.next_value()?,
                _ => {
                    let _ = map.next_value::<de::IgnoredAny>()?;
                }
            }
        }

        Ok(parts.to_fixed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::format;
    use std::vec;
    use std::vec::Vec;

    #[test]
    fn the_values_are_serialized_as_strings() {
        let value = Fixed::from_parts(0, 17_179_869);
        assert_eq!(serde_json::to_string(&value).unwrap(), "\"0.004\"");
        assert_eq!(serde_json::to_string(&Fixed::ZERO).unwrap(), "\"0\"");
        assert_eq!(serde_json::to_string(&Fixed::NEG_ONE).unwrap(), "\"-1\"");
    }

    #[test]
    fn the_strings_are_deserialized() {
        let value: Fixed = serde_json::from_str("\"0.004\"").unwrap();
        assert_eq!(value, Fixed::from_parts(0, 17_179_869));
        assert_eq!(
            serde_json::from_str::<Fixed>("\"-1.5\"").unwrap(),
            Fixed::from_parts(-1, 2_147_483_648)
        );
    }

    #[test]
    fn the_integers_are_deserialized() {
        assert_eq!(
            serde_json::from_str::<Fixed>("3").unwrap(),
            Fixed::from_int(3)
        );
        assert_eq!(
            serde_json::from_str::<Fixed>("-3").unwrap(),
            Fixed::from_int(-3)
        );
        assert_eq!(
            serde_json::from_str::<Fixed>("18446744073709551615").unwrap(),
            Fixed::from_raw((u64::MAX as i128) << 32)
        );
    }

    #[test]
    fn the_floats_are_deserialized() {
        assert_eq!(
            serde_json::from_str::<Fixed>("1.5").unwrap(),
            Fixed::from_parts(1, 2_147_483_648)
        );
        assert_eq!(
            serde_json::from_str::<Fixed>("-0.5").unwrap(),
            Fixed::from_raw(-(1i128 << 31))
        );
    }

    #[test]
    fn the_values_round_trip() {
        let values = [
            Fixed::ZERO,
            Fixed::ONE,
            Fixed::NEG_ONE,
            Fixed::HALF,
            Fixed::from_parts(0, 17_179_869),
            Fixed::from_parts(-123, 456_789),
            Fixed::MIN,
            Fixed::MAX,
        ];

        for value in values {
            let text = serde_json::to_string(&value).unwrap();
            let parsed: Fixed = serde_json::from_str(&text).unwrap();
            assert_eq!(parsed, value);
        }
    }

    #[test]
    fn the_invalid_input_is_rejected() {
        assert!(serde_json::from_str::<Fixed>("\"abc\"").is_err());
        assert!(serde_json::from_str::<Fixed>("\"1.2.3\"").is_err());
        assert!(serde_json::from_str::<Fixed>("\"\"").is_err());
        assert!(serde_json::from_str::<Fixed>("null").is_err());
    }

    #[test]
    fn the_raw_halves_round_trip() {
        for value in [
            Fixed::ZERO,
            Fixed::ONE,
            Fixed::NEG_ONE,
            Fixed::MIN,
            Fixed::MAX,
        ] {
            let parts = RawParts::from_fixed(value);
            assert_eq!(parts.to_fixed(), value);
        }
    }

    #[test]
    fn the_binary_form_is_deserialized() {
        let hi = 0x0123_4567_89ab_cdefu64;
        let lo = 0xfedc_ba98_7654_3210u64;
        let expected = RawParts { hi, lo }.to_fixed();

        let values: Vec<u64> = vec![hi, lo];
        let deserializer = serde::de::value::SeqDeserializer::<_, serde::de::value::Error>::new(
            values.into_iter(),
        );
        let value = Fixed::deserialize(deserializer).unwrap();
        assert_eq!(value, expected);
    }

    #[test]
    fn the_binary_form_is_deserialized_from_an_object() {
        let hi = 0x0123_4567_89ab_cdefu64;
        let lo = 0xfedc_ba98_7654_3210u64;
        let expected = RawParts { hi, lo }.to_fixed();

        let text = format!("{{\"hi\": {hi}, \"lo\": {lo}}}");
        let value: Fixed = serde_json::from_str(&text).unwrap();
        assert_eq!(value, expected);
    }
}
