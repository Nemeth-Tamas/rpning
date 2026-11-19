use rpning::Calculator;
use rpning::language::compile;

fn stack(calc: &Calculator) -> Vec<String> {
    calc.stack().iter().map(ToString::to_string).collect()
}

#[test]
fn compiled_nested_macros_namespaces_descriptions_and_aliases() {
    let mut calc = Calculator::new();
    calc.evaluate(": math.square \"Square X\" dup * ; : math.fourth math.square math.square ; : sq math.square ; 3 math.fourth 4 sq +").unwrap();
    assert_eq!(stack(&calc), ["97"]);
    assert_eq!(calc.macros()["math.square"].description, "Square X");
    let program = compile("2 math.square 1+").unwrap();
    calc.execute_program(&program, false).unwrap();
    assert_eq!(stack(&calc), ["97", "5"]);
    assert!(calc.undo());
    assert_eq!(stack(&calc), ["97"]);
}

#[test]
fn conditionals_nested_counted_loops_and_while_compile_correct_jumps() {
    let mut calc = Calculator::new();
    calc.evaluate("0 if 999 else 0 3 times 2 times 1+ end end then")
        .unwrap();
    assert_eq!(stack(&calc), ["6"]);
    calc.evaluate("clear 1 if 0 if 9 else 8 then else 7 then 0 while dup 5 < do 1+ end")
        .unwrap();
    assert_eq!(stack(&calc), ["8", "5"]);
    calc.evaluate("clear 5 0 times 999 end 0 if 999 then")
        .unwrap();
    assert_eq!(stack(&calc), ["5"]);
}

#[test]
fn local_variables_are_isolated_per_call_and_recursion_is_supported() {
    let mut calc = Calculator::new();
    calc.evaluate(": factorial dup 1 <= if drop 1 else dup 1- factorial * then ; 8 factorial")
        .unwrap();
    assert_eq!(stack(&calc), ["40320"]);
    calc.evaluate(": inner 100 let:x get:x ; : outer let:x inner get:x + ; 7 outer")
        .unwrap();
    assert_eq!(stack(&calc), ["40320", "107"]);
    let previous = stack(&calc);
    assert!(calc.evaluate("get:x").is_err());
    assert_eq!(stack(&calc), previous);
}

#[test]
fn failures_rollback_stack_registers_definitions_and_preserve_redo() {
    let mut calc = Calculator::new();
    calc.evaluate("7 sto:saved").unwrap();
    calc.evaluate("2+").unwrap();
    calc.undo();
    assert!(calc.evaluate(": newword 1 ; 88 sto:saved 0 /").is_err());
    assert_eq!(stack(&calc), ["7"]);
    assert!(!calc.macros().contains_key("newword"));
    assert_eq!(calc.register("saved").unwrap().to_string(), "7");
    assert!(calc.redo());
    assert_eq!(stack(&calc), ["9"]);
}

#[test]
fn macro_definitions_participate_in_undo_redo_and_roundtrip_import() {
    let mut calc = Calculator::new();
    calc.evaluate(": vat \"ÁFA \\\"tax\\\"\" 1.27 * ;").unwrap();
    assert!(calc.undo());
    assert!(calc.macros().is_empty());
    assert!(calc.redo());
    let source = calc.export_macros();
    let mut other = Calculator::new();
    other.import_macros(&source).unwrap();
    other.evaluate("100 vat").unwrap();
    assert_eq!(stack(&other), ["127"]);
    assert!(other.import_macros(": valid 1 ; clear").is_err());
    assert!(!other.macros().contains_key("valid"));
}

#[test]
fn tracing_returns_compiled_steps_and_parser_rejects_invalid_programs() {
    let mut calc = Calculator::new();
    let trace = calc.evaluate_traced(": square dup * ; 5 square").unwrap();
    assert!(trace.iter().any(|line| line.contains("depth=1")));
    assert_eq!(stack(&calc), ["25"]);
    for bad in [
        ": dup 1 ;",
        ": unfinished 2",
        "if 1",
        "1 times 2",
        "while 1 do",
        "else",
        "1 let:",
        ": nested : other 1 ; ;",
        "\"unterminated",
    ] {
        assert!(compile(bad).is_err(), "accepted {bad}");
    }
}

#[test]
fn execution_limits_stop_recursion_and_infinite_loops_atomically() {
    let mut calc = Calculator::new();
    calc.evaluate("42").unwrap();
    for source in [
        ": recurse recurse ; recurse",
        "while 1 do end",
        "-1 times 2 end",
        "1/2 times 2 end",
    ] {
        assert!(calc.evaluate(source).is_err(), "accepted {source}");
        assert_eq!(stack(&calc), ["42"]);
        assert!(calc.macros().is_empty());
    }
}

#[test]
fn failed_large_compiled_transaction_preserves_registers_stack_and_redo() {
    let mut calc = Calculator::new();
    calc.evaluate("1000 times 1 end 7 sto:saved").unwrap();
    calc.clear_history();
    let original = stack(&calc);
    calc.evaluate("1+").unwrap();
    calc.undo();
    assert!(
        calc.evaluate(": mutate 99 sto:saved 10 times dup + end ; mutate 0 /")
            .is_err()
    );
    assert_eq!(stack(&calc), original);
    assert_eq!(calc.register("saved").unwrap().to_string(), "7");
    assert!(calc.macros().is_empty());
    assert!(calc.redo());
    assert_eq!(calc.top().unwrap().to_string(), "8");
}
