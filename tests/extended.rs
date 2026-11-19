use rpning::Calculator;

#[test]
fn bases_bits_fixed_width_and_probability_are_exact() {
    for (source, expected) in [
        ("0xff 0b1010 band", "10"),
        ("0o17 0x10 bor", "31"),
        ("0xff 0x0f bxor", "240"),
        ("0 bnot", "-1"),
        ("-8 2 shr", "-2"),
        ("1 100 shl", "1267650600228229401496703205376"),
        ("255 i8", "-1"),
        ("-1 u8", "255"),
        ("65537 u16", "1"),
        ("257 8 asuint", "1"),
        ("128 8 asint", "-128"),
        ("10 3 choose", "120"),
        ("10 3 perm", "720"),
        ("2 1 1/2 binomp", "1/2"),
        ("10 0 0 binomp", "1"),
        ("10 10 1 binomp", "1"),
        ("0 0 1/3 binomp", "1"),
        ("-0xff 0b1+", "-254"),
    ] {
        let mut calc = Calculator::new();
        calc.evaluate(source).unwrap();
        assert_eq!(calc.top().unwrap().to_string(), expected, "{source}");
    }
}

#[test]
fn counted_statistics_use_exact_rationals_and_preserve_items_below_dataset() {
    for (source, expected) in [
        ("1 2 3 3 sum", "6"),
        ("1 2 3 3 mean", "2"),
        ("4 1 3 2 4 median", "5/2"),
        ("1 2 3 3 var", "2/3"),
        ("1 2 3 3 svar", "1"),
        ("1 2 3 3 sstdev", "1"),
        ("1/3 2/3 2 mean", "1/2"),
        ("5 -2 8 3 min", "-2"),
        ("5 -2 8 3 max", "8"),
        ("0 sum", "0"),
    ] {
        let mut calc = Calculator::new();
        calc.evaluate(&format!("99 {source}")).unwrap();
        assert_eq!(calc.stack()[0].to_string(), "99");
        assert_eq!(calc.top().unwrap().to_string(), expected, "{source}");
        assert_eq!(calc.depth(), 2);
    }
}

#[test]
fn invalid_extended_operations_are_atomic_and_preserve_redo() {
    for source in [
        "1/2 2 band",
        "1 -1 shl",
        "1 100001 shl",
        "7 0 asint",
        "2 mean",
        "0 mean",
        "3 1 svar",
        "10 11 choose",
        "2 1 2 binomp",
        "0xZZ",
        "1 2 3 3 stdev",
    ] {
        let mut calc = Calculator::new();
        calc.evaluate("7 sto:saved").unwrap();
        calc.evaluate("1+").unwrap();
        calc.undo();
        assert!(calc.evaluate(source).is_err(), "{source}");
        assert_eq!(calc.top().unwrap().to_string(), "7");
        assert_eq!(calc.register("saved").unwrap().to_string(), "7");
        assert!(calc.redo());
    }
}

#[test]
fn alternate_display_falls_back_losslessly_for_nonintegers() {
    use rpning::{display::DisplayMode, number::parse_literal, precision::Rounding};
    assert_eq!(
        DisplayMode::Hex.format(&parse_literal("-255").unwrap(), Rounding::NearestEven),
        "-0xff"
    );
    assert_eq!(
        DisplayMode::Binary.format(&parse_literal("5").unwrap(), Rounding::NearestEven),
        "0b101"
    );
    assert_eq!(
        DisplayMode::Octal.format(&parse_literal("1/3").unwrap(), Rounding::NearestEven),
        "1/3"
    );
    assert!(rpning::extended::format_base(&parse_literal("1").unwrap(), 0).is_none());
}
