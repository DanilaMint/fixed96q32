"""Generates the high precision constants for the `src/consts.rs` module.

The values are printed as the raw representations of the Q32 format.

Usage: python3 tools/gen_consts.py
"""

from decimal import Decimal, getcontext, ROUND_HALF_EVEN

getcontext().prec = 80

PI = Decimal(
    "3.14159265358979323846264338327950288419716939937510582097494459230781640628620899"
)
E = Decimal(
    "2.71828182845904523536028747135266249775724709369995957496696762772407663035354759"
)


def q(value: Decimal, bits: int) -> int:
    """Rounds the value to the nearest integer scaled by `2**bits`."""
    scaled = value * (Decimal(2) ** bits)
    return int(scaled.quantize(Decimal(1), rounding=ROUND_HALF_EVEN))


def main() -> None:
    ln2 = Decimal(2).ln()
    ln10 = Decimal(10).ln()
    sqrt2 = Decimal(2).sqrt()
    sqrt_pi = PI.sqrt()

    consts = {
        "PI": PI,
        "TAU": 2 * PI,
        "FRAC_PI_2": PI / 2,
        "FRAC_PI_3": PI / 3,
        "FRAC_PI_4": PI / 4,
        "FRAC_PI_6": PI / 6,
        "FRAC_PI_8": PI / 8,
        "FRAC_1_PI": 1 / PI,
        "FRAC_2_PI": 2 / PI,
        "FRAC_1_SQRT_2": 1 / sqrt2,
        "FRAC_2_SQRT_PI": 2 / sqrt_pi,
        "SQRT_2": sqrt2,
        "SQRT_PI": sqrt_pi,
        "E": E,
        "LOG2_E": 1 / ln2,
        "LOG10_E": 1 / ln10,
        "LN_2": ln2,
        "LN_10": ln10,
    }

    for name, value in consts.items():
        print(f"{name}: {q(value, 32)}  # {value:.12f}")


if __name__ == "__main__":
    main()
