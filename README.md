# fixed96q32

Fixed-point arithmetic with a 128-bit layout: **1 sign bit, 95 integral bits
and 32 fractional bits**. The crate is `no_std` friendly, never converts the
values into floating point numbers and implements the advanced mathematics
with the CORDIC algorithm.

## Features

- Exact fixed-point arithmetic over the whole `[-2**95, 2**95 - 2**-32]` range;
- `no_std` compatible, without `alloc` and without the standard library;
- advanced mathematics (square root, logarithm, exponent, trigonometric and
  hyperbolic functions, powers) implemented with the CORDIC algorithm;
- explicit overflow handling: the operators panic, the `checked_*`,
  `saturating_*`, `wrapping_*` and `overflowing_*` methods cover the rest;
- interoperability with `num-traits` and `serde` behind the feature flags;
- a shortest round-tripping decimal representation, so `0.004` prints as
  `0.004`.

## Layout

The value is stored as a 128-bit two's complement integer, and the number
equals `raw / 2**32`:

```text
 127                        32 31                        0
┌───┬────────────────────────┬───┬────────────────────────┐
│ S │      integer part      │ 0 │      fraction part      │
└───┴────────────────────────┴───┴────────────────────────┘
   1          95 bits                    32 bits
```

The smallest positive value is `2**-32` (`Fixed::EPSILON`), the largest one is
`2**95 - 2**-32` (`Fixed::MAX`) and the smallest one is `-2**95`
(`Fixed::MIN`).

## Quick start

```rust
use fixed96q32::Fixed;

let value = Fixed::from_parts(3, 17_179_869);
assert_eq!(value.to_string(), "3.004");

let sum = Fixed::from_int(3) + Fixed::from_parts(0, 17_179_869);
assert_eq!(sum, Fixed::from_parts(3, 17_179_869));

let parsed: Fixed = "0.004".parse().unwrap();
assert_eq!(parsed, Fixed::from_parts(0, 17_179_869));
```

The builder methods of the type:

| Method | Meaning |
|---|---|
| `Fixed::from_int(i64)` | an integral value, for example `3` |
| `Fixed::from_parts(i64, u32)` | an integral and a fractional part, the fraction is expressed in units of `2**-32` |
| `Fixed::from_raw(i128)` | the raw representation |
| `Fixed::from_bits(u128)` | the bit pattern |
| `Fixed::to_raw()` / `Fixed::to_bits()` | the raw representation and the bit pattern |
| `Fixed::to_f64()` / `Fixed::to_i64()` | the lossy conversions |

## Arithmetic and overflow

The operators behave like the operators of the primitive integer types: they
panic when the result leaves the representable range.

```rust
use fixed96q32::Fixed;

assert_eq!(Fixed::from_int(1) + Fixed::from_int(2), Fixed::from_int(3));
```

When the overflow has to be handled, four families of methods are available:

| Family | Behaviour |
|---|---|
| `checked_*` | returns `Option<Self>`, `None` on overflow |
| `saturating_*` | clamps the result to `Fixed::MIN` / `Fixed::MAX` |
| `wrapping_*` | takes the result modulo `2**128` |
| `overflowing_*` | returns the wrapped result together with the overflow flag |

```rust
use fixed96q32::Fixed;

assert_eq!(Fixed::MAX.checked_add(Fixed::ONE), None);
assert_eq!(Fixed::MAX.saturating_add(Fixed::ONE), Fixed::MAX);
assert_eq!(Fixed::ONE.overflowing_add(Fixed::ONE), (Fixed::from_int(2), false));
```

The multiplication and the division truncate the discarded bits towards zero,
so `Fixed::ONE / Fixed::from_int(3)` equals `0.3333333333`. The division by
zero panics, and the remainder is truncated towards zero, exactly like the
remainder of the floating point types.

## Conversions

The integral types are converted with `TryFrom`, the fractional part is
truncated towards zero, and the value that does not fit into the range of the
target type is reported with an error:

```rust
use fixed96q32::Fixed;
use fixed96q32::froms::ConversionError;

assert_eq!(i32::try_from(Fixed::from_parts(2, 7)), Ok(2));
assert_eq!(u8::try_from(Fixed::from_int(256)), Err(ConversionError::OutOfRange));
```

The pointer sized `isize` is converted with the infallible `From`, its integral
part is clamped to the range of the type:

```rust
use fixed96q32::Fixed;

assert_eq!(isize::from(Fixed::from_parts(-2, 7)), -2);
```

## Advanced mathematics

The module [`math`](src/math.rs) implements the functions of `f64` without a
single conversion to a floating point number. The trigonometric functions use
the rotation mode of the CORDIC algorithm, the inverse trigonometric ones use
the vectoring mode, the logarithm and the exponential function use the
logarithmic and the exponential modes, and the square root is computed by the
exact digit-by-digit algorithm.

```rust
use fixed96q32::Fixed;

assert_eq!(Fixed::from_int(4).sqrt().unwrap(), Fixed::from_int(2));
assert_eq!(Fixed::ONE.ln().unwrap(), Fixed::ZERO);
assert_eq!(Fixed::from_int(2).powi(10).unwrap(), Fixed::from_int(1024));

let sine = Fixed::ONE.sin().unwrap().to_f64();
assert!((sine - 0.841_470_984_807_896_5).abs() < 1e-9);
```

The functions return `None` when the argument is outside of the domain of the
function or when the result does not fit into the representable range. The
internal computations use 64 fractional bits, so the accuracy of the results is
close to the limit of the type, about one unit in the last place.

## Feature flags

| Feature | Contents |
|---|---|
| `num-traits` | the traits of the `num-traits` crate: `Zero`, `One`, `Num`, `Signed`, `ToPrimitive`, `FromPrimitive`, `NumCast`, the `Checked*`, `Saturating*`, `Overflowing*` and `Wrapping*` families, `Pow`, `Inv`, `MulAdd`, `Euclid` |
| `serde` | the `Serialize` and `Deserialize` implementations |

```toml
[dependencies]
fixed96q32 = { version = "0.1", features = ["num-traits", "serde"] }
```

The human readable formats, such as JSON, receive the decimal representation of
the value, for example `"0.004"`, which keeps the exact value of the number.
The binary formats receive the raw representation split into the `hi` and the
`lo` halves.

## `no_std`

The crate does not use the standard library:

```toml
[dependencies]
fixed96q32 = { version = "0.1", default-features = false }
```

## Modules

| Module | Contents |
|---|---|
| [`fixed`](src/fixed.rs) | the `Fixed` type and its builder methods |
| [`ops`](src/ops.rs) | the implementation of the `core::ops` traits |
| [`froms`](src/froms.rs) | the `From` and `TryFrom` conversions |
| [`fmt`](src/fmt.rs) | the `Display` and the `FromStr` implementations |
| [`math`](src/math.rs) | the advanced mathematics, the CORDIC engine lives in [`math/cordic`](src/math/cordic.rs) |
| [`consts`](src/consts.rs) | the mathematical constants, the analogues of `core::f64::consts` |
| [`num_traits`](src/num_traits.rs) | the interoperability with `num-traits` |
| [`serde`](src/serde.rs) | the interoperability with `serde` |

## Benchmarks

The benchmarks are written with the [criterion](https://github.com/bheisler/criterion)
framework and cover the arithmetic, the conversions, the formatting and the
advanced mathematics:

```console
cargo bench
```

The processor cycles of every operation are measured separately with the
`rdtsc` instruction, so the numbers do not depend on the frequency scaling:

```console
cargo bench --bench cycles
```

The table below holds the median values of a run on a x86-64 processor, the
exact numbers depend on the machine:

| Operation | Time | Operation | Time |
|---|---|---|---|
| `add`, `sub` | 1.4 ns | `sqrt` | 46 ns |
| `mul` | 4.1 ns | `cbrt` | 276 ns |
| `div` | 37 ns | `ln` | 8 ns |
| `rem` | 4.7 ns | `log2`, `log10` | 20 ns |
| `checked_add` | 1.9 ns | `exp` | 132 ns |
| `checked_mul` | 4.3 ns | `sin`, `cos` | 115 ns |
| `saturating_add` | 1.4 ns | `tan` | 126 ns |
| `wrapping_add` | 1.4 ns | `asin`, `acos` | 111 ns |
| `overflowing_add` | 6.5 ns | `atan`, `atan2` | 95 ns |
| `from_int` | 1.2 ns | `sinh`, `cosh` | 255 ns |
| `from_f64` | 9.8 ns | `tanh` | 161 ns |
| `to_f64` | 4.9 ns | `powi` | 16 ns |
| `display` | 164 ns | `powf` | 135 ns |
| `parse` | 13 ns | `hypot` | 55 ns |

The division is the slowest arithmetic operation, because the quotient is
extracted bit by bit to keep the full precision of the type. The CORDIC
iterations dominate the mathematical functions, and the display is the slowest
operation of the formatting, because the shortest round-tripping decimal
representation is searched.

## Verification

```console
cargo test --all-features
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt --check
```

## License

The crate is distributed under the terms of the MIT license, see
[LICENSE](LICENSE).
