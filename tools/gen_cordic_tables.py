"""Generates the CORDIC tables and the high precision constants for `src/math/cordic.rs`.

The tables are printed in the Q64 format, that is, the raw values of the
numbers with 64 fractional bits.

Usage: python3 tools/gen_cordic_tables.py
"""

from decimal import Decimal, getcontext, ROUND_HALF_EVEN

getcontext().prec = 80

ITERATIONS = 48

PI = Decimal(
    "3.14159265358979323846264338327950288419716939937510582097494459230781640628620899"
)
K_INV = Decimal("0.6072529350088812561694467525")


def atan(x: Decimal) -> Decimal:
    """Arctangent via the argument reduction and the Taylor series."""
    factor = 1
    for _ in range(4):
        x = x / (1 + (1 + x * x).sqrt())
        factor *= 2

    square = x * x
    term = x
    total = x
    n = 1
    while True:
        term = -term * square
        add = term / Decimal(2 * n + 1)
        if abs(add) < Decimal("1e-70"):
            break
        total += add
        n += 1

    return total * factor


def atanh(x: Decimal) -> Decimal:
    """Hyperbolic arctangent: `atanh(x) = ln((1 + x) / (1 - x)) / 2`."""
    return ((1 + x) / (1 - x)).ln() / 2


def q64(value: Decimal) -> int:
    """Rounds the value to the nearest integer scaled by `2**64`."""
    scaled = value * (Decimal(2) ** 64)
    return int(scaled.quantize(Decimal(1), rounding=ROUND_HALF_EVEN))


def main() -> None:
    print("/// The table of the angles `atan(2**-i)` in the Q64 format.")
    print("const ATAN_TABLE: [i128; ITERATIONS] = [")
    for i in range(ITERATIONS):
        print(f"    {q64(atan(Decimal(2) ** -i))}, // atan(2^-{i})")
    print("];")
    print()
    print("/// The table of the logarithms `ln(1 + 2**-i)` in the Q64 format.")
    print("const LOG_TABLE: [i128; ITERATIONS] = [")
    for i in range(1, ITERATIONS + 1):
        value = (1 + Decimal(2) ** -i).ln()
        print(f"    {q64(value)}, // ln(1 + 2^-{i})")
    print("];")
    print()
    print("/// The table of the reciprocals `1 / (1 + 2**-i)` in the Q64 format.")
    print("const RECIPROCAL_TABLE: [i128; ITERATIONS] = [")
    for i in range(1, ITERATIONS + 1):
        value = 1 / (1 + Decimal(2) ** -i)
        print(f"    {q64(value)}, // 1 / (1 + 2^-{i})")
    print("];")
    print()
    print(f"const PI_Q64: i128 = {q64(PI)};")
    print(f"const FRAC_PI_2_Q64: i128 = {q64(PI / 2)};")
    print(f"const LN_2_Q64: i128 = {q64(Decimal(2).ln())};")
    print(f"const LN_10_Q64: i128 = {q64(Decimal(10).ln())};")
    print(f"const K_INV_Q64: i128 = {q64(K_INV)};")
    print(f"const DEGREES_PER_RADIAN_Q64: i128 = {q64(Decimal(180) / PI)};")
    print(f"const RADIANS_PER_DEGREE_Q64: i128 = {q64(PI / Decimal(180))};")


if __name__ == "__main__":
    main()
