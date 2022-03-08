# KECC Fuzzer User's Manual

## Introduction

You can find a buggy input program for your homework implementation by using fuzzer.

Usual debugging process consists of two stages. These stages are:

### Fuzzing

Fuzzer randomly generates input C program and feeds it to your implementation for homework. If the output value is not the expected value, then fuzzer stops generating and terminates. Now you can debug your solution by inspecting generated `test_polished.c`. Generated buggy input program is guaranteed undefined-behavior free. (We use clang's [UndefinedBehaviorSanitizer](https://clang.llvm.org/docs/UndefinedBehaviorSanitizer.html) to detect undefined behavior in the generated input program and skip it if any undefined behavior detected.)

### Reducing

After the fuzzing stage, you may found a buggy input program in `test_polished.c`. However, it is highly likely that the input program is too big (about 3~5K lines of code) to manually inspect. In this stage, we use `creduce` to reduce the buggy input program as much as possible. Reduced buggy input program is saved to `test_reduced.c`.

**[NOTICE]** The buggy input program after reducing stage may contain undefined behaviors. To workaround this, please refer to this issue: [kaist-cp/cs420#218](https://github.com/kaist-cp/cs420/issues/218)

For now, fuzzer is supported only for Homework 1 (C AST Printer) and Homework 2 (IRgen).

## Installation

```bash
# Ubuntu 20.04
sudo apt install -y make cmake python3 csmith libcsmith-dev creduce
```

**[NOTICE]** The fuzzer supports Ubuntu 20.04 only. It may work for other platforms, but if it doesn't, please run the fuzzer in Ubuntu 20.04.

## Usage

```bash
python3 tests/fuzz.py [OPTIONS]
```

Use `-h` option to display all available options.

## Examples

### Homework 1

- Fuzzing

  ```bash
  # Try infinitely many test cases until find a buggy one.
  python3 tests/fuzz.py -p
  
  # Try 300 test cases until find a buggy one.
  python3 tests/fuzz.py -p -n 300
  ```

- Reducing

  ```bash
  # Reduce `test_polished.c` to `test_reduced.c`.
  python3 tests/fuzz.py -p -r
  ```

### Homework 2

- Fuzzing

  ```bash
  # Try infinitely many test cases until find a buggy one.
  python3 tests/fuzz.py -i
  
  # Try 300 test cases until find a buggy one.
  python3 tests/fuzz.py -i -n 300
  ```

- Reducing

  ```bash
  # Reduce `test_polished.c` to `test_reduced.c`.
  python3 tests/fuzz.py -i -r
  ```

## Suggested Readings

- Suggestions about how to use fuzzer and creduce: [kaist-cp/cs420#318](https://github.com/kaist-cp/cs420/issues/318)
- Suggestion for who is running KECC on their local machine: [kaist-cp/cs420#314](https://github.com/kaist-cp/cs420/issues/314)
