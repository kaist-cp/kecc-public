use kecc::run_ir::*;
use kecc::*;
use std::path::Path;

// TODO: cover all examples in the future
#[test]
fn ir_interpreter_test() {
    // Test toy example
    assert_eq!(run_example("examples/foo.c"), Ok(Value::Int(-1)));

    // Test toy example with negate unary operator
    assert_eq!(run_example("examples/negate.c"), Ok(Value::Int(1)));

    // Test fibonacci function with for-loop
    assert_eq!(run_example("examples/fib3.c"), Ok(Value::Int(34)));

    // Test fibonacci function with while-loop
    assert_eq!(run_example("examples/fib4.c"), Ok(Value::Int(34)));

    // Test fibonacci function with do-while-loop
    assert_eq!(run_example("examples/fib5.c"), Ok(Value::Int(34)));

    // Test fibonacci function with recursive function call
    assert_eq!(run_example("examples/fibonacci.c"), Ok(Value::Int(34)));

    // Test example with global variable
    assert_eq!(run_example("examples/foo3.c"), Ok(Value::Int(30)));

    // Test example with comma expressions
    assert_eq!(run_example("examples/comma.c"), Ok(Value::Int(7)));

    // Test example with complex function call
    assert_eq!(run_example("examples/foo4.c"), Ok(Value::Int(6)));

    // Test example with pointer
    assert_eq!(run_example("examples/pointer.c"), Ok(Value::Int(3)));

    // Test example with sizeof
    assert_eq!(run_example("examples/sizeof.c"), Ok(Value::Int(4)));

    // Test example with alignof
    assert_eq!(run_example("examples/alignof.c"), Ok(Value::Int(4)));

    // Test example with simple for statement
    assert_eq!(run_example("examples/simple_for.c"), Ok(Value::Int(55)));

    // Test example with conditional expression
    assert_eq!(run_example("examples/cond.c"), Ok(Value::Int(5)));

    // Test example with switch statement
    assert_eq!(run_example("examples/switch.c"), Ok(Value::Int(2)));
}

fn run_example(example_path: &str) -> Result<Value, InterpreterError> {
    let example_path = Path::new(example_path);
    let unit = Parse::default()
        .translate(&example_path)
        .expect("parse failed");
    let ir = Irgen::default()
        .translate(&unit)
        .expect("failed to generate ir");

    // TODO: consider command line arguments in the future
    // TODO: randomly generate argument values
    let args = Vec::new();

    run_ir(&ir, args)
}
