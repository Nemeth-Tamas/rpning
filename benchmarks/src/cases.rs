//! Benchmark cases for integer, rational and binary-float arithmetic.
use malachite::{Float as MFloat, Integer as MInteger, Rational as MRational};
#[cfg(feature = "compare-rug")]
use rug::{Float as RFloat, Integer as RInteger, Rational as RRational};

use crate::data::{integer_input_strings, rational_input_strings};
use crate::timing::{Case, Config, Row, bench};

pub(crate) fn integer_cases(rows: &mut Vec<Row>, config: Config, digits: usize) {
    let source = integer_input_strings(digits);
    let mal: Vec<(MInteger, MInteger)> = source
        .iter()
        .map(|(a, b)| (a.parse().unwrap(), b.parse().unwrap()))
        .collect();

    #[cfg(feature = "compare-rug")]
    let rug: Vec<(RInteger, RInteger)> = source
        .iter()
        .map(|(a, b)| {
            (
                RInteger::from_str_radix(a, 10).unwrap(),
                RInteger::from_str_radix(b, 10).unwrap(),
            )
        })
        .collect();

    #[cfg(feature = "compare-rug")]
    for ((ma, mb), (ra, rb)) in mal.iter().zip(&rug) {
        assert_eq!(
            (ma.clone() + mb.clone()).to_string(),
            (ra.clone() + rb.clone()).to_string(),
            "integer add"
        );
        assert_eq!(
            (ma.clone() * mb.clone()).to_string(),
            (ra.clone() * rb.clone()).to_string(),
            "integer mul"
        );
        assert_eq!(
            (ma.clone() / mb.clone()).to_string(),
            (ra.clone() / rb.clone()).to_string(),
            "integer div"
        );
        assert_eq!(
            (ma.clone() % mb.clone()).to_string(),
            (ra.clone() % rb.clone()).to_string(),
            "integer rem"
        );
    }

    bench(
        rows,
        Case {
            family: "integer",
            operation: "add",
            size: digits,
            unit: "decimal_digits",
            backend: "malachite",
        },
        &mal,
        config,
        |a, b| a + b,
    );
    #[cfg(feature = "compare-rug")]
    bench(
        rows,
        Case {
            family: "integer",
            operation: "add",
            size: digits,
            unit: "decimal_digits",
            backend: "rug",
        },
        &rug,
        config,
        |a, b| a + b,
    );

    bench(
        rows,
        Case {
            family: "integer",
            operation: "mul",
            size: digits,
            unit: "decimal_digits",
            backend: "malachite",
        },
        &mal,
        config,
        |a, b| a * b,
    );
    #[cfg(feature = "compare-rug")]
    bench(
        rows,
        Case {
            family: "integer",
            operation: "mul",
            size: digits,
            unit: "decimal_digits",
            backend: "rug",
        },
        &rug,
        config,
        |a, b| a * b,
    );

    bench(
        rows,
        Case {
            family: "integer",
            operation: "div",
            size: digits,
            unit: "decimal_digits",
            backend: "malachite",
        },
        &mal,
        config,
        |a, b| a / b,
    );
    #[cfg(feature = "compare-rug")]
    bench(
        rows,
        Case {
            family: "integer",
            operation: "div",
            size: digits,
            unit: "decimal_digits",
            backend: "rug",
        },
        &rug,
        config,
        |a, b| a / b,
    );

    bench(
        rows,
        Case {
            family: "integer",
            operation: "rem",
            size: digits,
            unit: "decimal_digits",
            backend: "malachite",
        },
        &mal,
        config,
        |a, b| a % b,
    );
    #[cfg(feature = "compare-rug")]
    bench(
        rows,
        Case {
            family: "integer",
            operation: "rem",
            size: digits,
            unit: "decimal_digits",
            backend: "rug",
        },
        &rug,
        config,
        |a, b| a % b,
    );
}

pub(crate) fn rational_cases(rows: &mut Vec<Row>, config: Config, digits: usize) {
    let source = rational_input_strings(digits);
    let mal: Vec<(MRational, MRational)> = source
        .iter()
        .map(|(a, b)| (a.parse().unwrap(), b.parse().unwrap()))
        .collect();

    #[cfg(feature = "compare-rug")]
    let rug: Vec<(RRational, RRational)> = source
        .iter()
        .map(|(a, b)| {
            (
                RRational::from_str_radix(a, 10).unwrap(),
                RRational::from_str_radix(b, 10).unwrap(),
            )
        })
        .collect();

    #[cfg(feature = "compare-rug")]
    for ((ma, mb), (ra, rb)) in mal.iter().zip(&rug) {
        assert_eq!(
            (ma.clone() + mb.clone()).to_string(),
            (ra.clone() + rb.clone()).to_string(),
            "rational add"
        );
        assert_eq!(
            (ma.clone() * mb.clone()).to_string(),
            (ra.clone() * rb.clone()).to_string(),
            "rational mul"
        );
        assert_eq!(
            (ma.clone() / mb.clone()).to_string(),
            (ra.clone() / rb.clone()).to_string(),
            "rational div"
        );
    }

    bench(
        rows,
        Case {
            family: "rational",
            operation: "add",
            size: digits,
            unit: "decimal_digits_per_part",
            backend: "malachite",
        },
        &mal,
        config,
        |a, b| a + b,
    );
    #[cfg(feature = "compare-rug")]
    bench(
        rows,
        Case {
            family: "rational",
            operation: "add",
            size: digits,
            unit: "decimal_digits_per_part",
            backend: "rug",
        },
        &rug,
        config,
        |a, b| a + b,
    );

    bench(
        rows,
        Case {
            family: "rational",
            operation: "mul",
            size: digits,
            unit: "decimal_digits_per_part",
            backend: "malachite",
        },
        &mal,
        config,
        |a, b| a * b,
    );
    #[cfg(feature = "compare-rug")]
    bench(
        rows,
        Case {
            family: "rational",
            operation: "mul",
            size: digits,
            unit: "decimal_digits_per_part",
            backend: "rug",
        },
        &rug,
        config,
        |a, b| a * b,
    );

    bench(
        rows,
        Case {
            family: "rational",
            operation: "div",
            size: digits,
            unit: "decimal_digits_per_part",
            backend: "malachite",
        },
        &mal,
        config,
        |a, b| a / b,
    );
    #[cfg(feature = "compare-rug")]
    bench(
        rows,
        Case {
            family: "rational",
            operation: "div",
            size: digits,
            unit: "decimal_digits_per_part",
            backend: "rug",
        },
        &rug,
        config,
        |a, b| a / b,
    );
}

pub(crate) fn float_cases(rows: &mut Vec<Row>, config: Config, bits: u32) {
    let parts = [
        (1u32, 3u32, 2u32, 7u32),
        (7, 11, 5, 13),
        (13, 17, 11, 19),
        (23, 29, 31, 37),
    ];
    let mal: Vec<(MFloat, MFloat)> = parts
        .iter()
        .map(|&(a, b, c, d)| {
            (
                MFloat::from_rational_prec(MRational::from_unsigneds(a, b), u64::from(bits)).0,
                MFloat::from_rational_prec(MRational::from_unsigneds(c, d), u64::from(bits)).0,
            )
        })
        .collect();

    #[cfg(feature = "compare-rug")]
    let rug: Vec<(RFloat, RFloat)> = parts
        .iter()
        .map(|&(a, b, c, d)| {
            (
                RFloat::with_val(bits, RRational::from((a, b))),
                RFloat::with_val(bits, RRational::from((c, d))),
            )
        })
        .collect();

    // A 1-ULP cross-library difference is possible for intermediate floating
    // values, so we do not compare their decimal renderings for equality.
    bench(
        rows,
        Case {
            family: "float",
            operation: "add",
            size: bits as usize,
            unit: "precision_bits",
            backend: "malachite",
        },
        &mal,
        config,
        |a, b| a + b,
    );
    #[cfg(feature = "compare-rug")]
    bench(
        rows,
        Case {
            family: "float",
            operation: "add",
            size: bits as usize,
            unit: "precision_bits",
            backend: "rug",
        },
        &rug,
        config,
        |a, b| a + b,
    );

    bench(
        rows,
        Case {
            family: "float",
            operation: "mul",
            size: bits as usize,
            unit: "precision_bits",
            backend: "malachite",
        },
        &mal,
        config,
        |a, b| a * b,
    );
    #[cfg(feature = "compare-rug")]
    bench(
        rows,
        Case {
            family: "float",
            operation: "mul",
            size: bits as usize,
            unit: "precision_bits",
            backend: "rug",
        },
        &rug,
        config,
        |a, b| a * b,
    );
}
