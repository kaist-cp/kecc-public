# KECC: KAIST Educational C Compiler

## Install

Install [rustup](https://rustup.rs/).


## Build

```sh
cargo build            # debug build
cargo build --release  # release build
```


## Run

```sh
cargo run -- -h                       # print options
cargo run -- -p examples/fibonacci.c  # parse
cargo run -- -i examples/fibonacci.c  # irgen
cargo run --    examples/fibonacci.c  # compile

cargo run --release -- examples/fibonacci.c  # compile with release build
```


## Test

```
cargo test             # debug build test
cargo test --release   # release build test

cargo test <test-name> # run a particular test
```

`<test-name>` can be `test_examples_write_c`, `test_examples_irgen`, ...


## Fuzzing

### Install

```sh
# Ubuntu 18.04 or higher
apt install -y make cmake python3
apt install -y csmith libcsmith-dev creduce
```

### Run

The following script generates 10 random test cases and tests your C AST printer:

```sh
python3 tests/fuzz.py --help        # print options
python3 tests/fuzz.py --print -n10  # test C AST printer for 10 times
```

We use `csmith` to randomly generate C source codes. `csmith` will be automatically downloaded and
built by the test script. For more information, we refer to the
[Csmith](https://embed.cs.utah.edu/csmith/) homepage.

### Reduce

When the fuzzer finds a buggy input program for your compiler, it is highly likely that the input
program is too big to manually inspect.  We use `creduce` that reduces the buggy input program as
much as possible.

Suppose `tests/test_polished.c` is the buggy input program. Then the following script reduces the
program to `tests/test_reduced.c`:

```sh
python3 tests/fuzz.py --reduce <fuzz-option>
```

`<fuzz-option>` can be `--print` or `--irgen`. It shall be the one used in [Run](#run).

### How it reduces test case?

The script performs unguided test-case reduction using `creduce`: given a buggy program, it randomly
reduces the program; check if the reduced program still fails on the test, and if so, replaces the
given program with the reduced one; repeat until you get a small enough buggy program. For more
information, we refer to the [Creduce](https://embed.cs.utah.edu/creduce/) homepage.
