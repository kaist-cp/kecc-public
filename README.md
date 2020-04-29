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
RUST_MIN_STACK=8388608 cargo test             # debug build test
RUST_MIN_STACK=8388608 cargo test --release   # release build test

RUST_MIN_STACK=8388608 cargo test <test-name> # run a particular test
```

`RUST_MIN_STACK=8388608` is necessary for deep call stack for irgen tests. `<test-name>` can be
`test_examples_write_c`, `test_examples_irgen`, ...


## Fuzzing

We encourage you to do homework using the test-driven development approach (TDD). You randomly
generate test input, and if it fails, then reduce it as much as possible and manually inspect the
reduced test input. For example, for homework 1, do:

```sh
# randomly generates test inputs and tests them
python3 tests/fuzz.py --print

# reduces the failing test input as much as possible
python3 tests/fuzz.py --print --reduce

# fix your code for the reduced test input
cat tests/test_reduced.c
```

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
program is too big to manually inspect. We use `creduce` that reduces the buggy input program as
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

**[NOTICE]** The fuzzer supports Ubuntu 18.04 or 20.04 only. It may work for other platforms, but if it
doesn't, please run the fuzzer in Ubuntu 18.04 or 20.04.


## Running RISC-V Binaries

### Install

```sh
# Ubuntu 20.04 or higher
apt install gcc-10-riscv64-linux-gnu qemu-user-static
```

### Cross-Compilation and Architecture-Emulation

```sh
# Compile C source code into RISC-V assembly
riscv64-linux-gnu-gcc-10 hello.c -S -o hello.S

# Link to an RISC-V executable
riscv64-linux-gnu-gcc-10 -static hello.S -o hello

# Emulate the executable
qemu-riscv64-static ./hello

# Check the return value
echo $?
```
