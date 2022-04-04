# KECC Fuzzer User's Manual

## Introduction

You can find a buggy input program for your homework implementation by using fuzzer.

Usual debugging process consists of two stages. These stages are:

### Fuzzing

Fuzzer randomly generates input C program and feeds it to your implementation for homework. If the output value is not the expected value, then fuzzer stops generating and terminates. Now you can debug your solution by inspecting generated `test_polished.c`. Generated buggy input program is guaranteed undefined-behavior free. (We use clang's [UndefinedBehaviorSanitizer](https://clang.llvm.org/docs/UndefinedBehaviorSanitizer.html) to detect undefined behavior in the generated input program and skip it if any undefined behavior detected.)

### Reducing

After the fuzzing stage, you may found a buggy input program in `test_polished.c`. However, it is highly likely that the input program is too big (about 3~5K lines of code) to manually inspect. In this stage, we use `creduce` to reduce the buggy input program as much as possible. Reduced buggy input program is saved to `test_reduced.c`.

**[NOTICE]** The buggy input program after reducing stage may contain undefined behaviors. To workaround this, we recommend you to (1) use `--clang-analyze` option to use clang static analyzer for reducing, or (2) do manual binary search to reduce the program by following this:

- If the code is 4000 lines, delete the latter 2000, and try again
- If there is no error, undo and delete only the latter 1000. If there is, delete lines 1000 to 2000, etc.
- It is important that you do not delete upper lines you know are safe (e.g, have lines 1000 to 2000 alive when checking 2000to 3000) as they might have some include for macros that can lead to compile errors if left out.

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

## How to use fuzzer on your local machine for HW2

_This section is copied from [#318](https://github.com/kaist-cp/cs420/issues/318)._

This is a more detailed explain how to use `fuzzer` and `creduce` on your local machine for HW2.

### Bugs

As mentioned in [#314](https://github.com/kaist-cp/cs420/issues/314), the main problems of `fuzzer` and `creduce` are the following points.

- The current version is not working.

  If you run `python3 tests/fuzz.py -i`, you might get an error.

- `creduce` gives out program with undefined behavior or misleading output.

### How to make it work

Here are some workaround if any of the above problems occurred.

1. Make sure you have the correct development environment.

   - OS: Ubuntu 20.04
   - Clang: 10.0.0
   - Python: 3.8.10

2. If the current version of `fuzzer` and `creduce` doesn't work, try to modify `tests/reduce-criteria-template.sh`.

    Change this line:

    ```sh
    cargo run --manifest-path $PROJECT_DIR/Cargo.toml --features=build-bin --release --bin fuzz -- $FUZZ_ARG test_reduced.c 2>&1 | grep -q 'assertion failed'
    ```

    into this:

    ```sh
    cargo run --manifest-path $PROJECT_DIR/Cargo.toml --release --bin fuzz -- $FUZZ_ARG test_reduced.c
    if [ "$?" = 101 ]
    then
      exit 0
    else
      exit 1
    fi
    ```

3. If it still gives you the error after the above two steps, then it might be some of the implementations are not correct. Please refer to [#314](https://github.com/kaist-cp/cs420/issues/314), make sure the implementation of dealing with different kinds of types are correct.

4. If your `fuzzer` starts working properly, you will start passing some random generated C program. If you get an undefined behavior output after `creduce`, just skip this and rerun your `fuzzer`. However, if the `creduce` gives you the below program

    ```c
    # 1 "" 3
    void main() {}
    ```

    Then it might be the inappropriate implementation of different types. For example, make sure you handle `Int` with different `width` properly. Again refer to [#314](https://github.com/kaist-cp/cs420/issues/314).

5. Don't be afraid of the giant output of fuzzer, sometimes directly debugging on the raw output of fuzzer might not be as bad as you think.

6. Live monitoring the result of `creduce` might help you to figure out the issue too. Please refer to [#314](https://github.com/kaist-cp/cs420/issues/314) and [#218](https://github.com/kaist-cp/cs420/issues/218).

### Summary

- Don't try to depend on the fuzzer at the beginning.
  
  It takes a lot of time to reduce a buggy program and you don't know if `creduce` will give you the desired output. So use the fuzzer as a criteria to judge your implementation is correct or not instead of depending on it for TDD at the beginning. Once your implementation reach a certain point that can pass decent amount of test cases, use fuzzer as a tool for TDD.

- Undefined behavior output can be ignored, rerun your fuzzer.
