# KECC: KAIST Educational C Compiler

## Install

Install [rustup](https://rustup.rs/).


## Build

```sh
cargo build            # debug build
cargo build --release  # release build
```

## Documentation

```sh
cargo doc --document-private-items          # built in target/doc
cargo doc --open --document-private-items   # opens in default broswer. Firefox or Chrome recommended.
```

## Run

```sh
cargo run --features=build-bin -- -h                                     # print options
cargo run --features=build-bin -- -p            examples/c/fibonacci.c   # parse
cargo run --features=build-bin -- -i            examples/c/fibonacci.c   # irgen
cargo run --features=build-bin -- -O --iroutput examples/c/fibonacci.c   # optimize
cargo run --features=build-bin --               examples/c/fibonacci.c   # compile

cargo run --features=build-bin -- --irrun examples/c/fibonacci.c    # interprets the IR
cargo run --features=build-bin -- --irviz fibonacci.png examples/c/fibonacci.c    # visualizes the IR

cargo run --features=build-bin --release -- examples/c/fibonacci.c  # compile with release build
```

For more information, please refer to the [KECC User's Manual](bin/README.md).


## Test

```sh
cargo install cargo-nextest
RUST_MIN_STACK=33554432 cargo nextest run             # debug build test
RUST_MIN_STACK=33554432 cargo nextest run --release   # release build test

RUST_MIN_STACK=33554432 cargo nextest run test_examples_write_c       # run write_c test

RUST_MIN_STACK=33554432 cargo nextest run test_examples_irgen_small   # run irgen test using a small subset of examples
RUST_MIN_STACK=33554432 cargo nextest run test_examples_irgen         # run irgen test

RUST_MIN_STACK=33554432 cargo nextest run test_examples_simplify_cfg  # run simplify_cfg test
RUST_MIN_STACK=33554432 cargo nextest run test_examples_mem2reg       # run mem2reg test
RUST_MIN_STACK=33554432 cargo nextest run test_examples_deadcode      # run deadcode test
RUST_MIN_STACK=33554432 cargo nextest run test_examples_gvn           # run gvn test

RUST_MIN_STACK=33554432 cargo nextest run test_examples_asmgen_small  # run asmgen test using a small subset of examples
RUST_MIN_STACK=33554432 cargo nextest run test_examples_asmgen        # run asmgen test

RUST_MIN_STACK=33554432 cargo nextest run test_examples_end_to_end    # run irgen, optimize and asmgen pipeline test
```

`RUST_MIN_STACK=33554432` is necessary for deep call stack for irgen tests.


## Fuzzing

We encourage you to do homework using the test-driven development (TDD)approach. You will
randomly generate a test input, and if it fails,
reduce it as much as possible and
manually inspect the reduced test input.
For example:

```sh
# Randomly generates test inputs and tests them
python3 tests/fuzz.py <fuzz-option>

# Reduces the failing test input as much as possible
python3 tests/fuzz.py <fuzz-option> --reduce

# Fix your code for the reduced test input
cat tests/test_reduced.c
```

`<fuzz-option>` can be `--print` or `--irgen`. It shall be the one used in [Run](#run).
For more information, please refer to the [Fuzzer User's Manual](tests/README.md).

### Install

```sh
# Ubuntu 20.04
sudo apt install -y build-essential clang make cmake python3 csmith libcsmith-dev creduce
pip3 install tqdm
```

### Run

The following script generates 10 random test cases and tests your C AST printer:

```sh
python3 tests/fuzz.py --help        # print options
python3 tests/fuzz.py --print -n10  # test C AST printer for 10 times
```

We use `csmith` to randomly generate C source codes.
`csmith` will be automatically downloaded and built by the test script.
For more information, we refer to the [Csmith](https://embed.cs.utah.edu/csmith/) homepage.

### Reduce

When the fuzzer finds a buggy input program for your compiler,
the input program is likely too big to manually inspect.
We use `creduce` that reduces the buggy input program as much as possible.

Suppose `tests/test_polished.c` is the buggy input program.
Then the following script reduces the program to `tests/test_reduced.c`:

```sh
python3 tests/fuzz.py <fuzz-option> --reduce
```

`<fuzz-option>` can be `--print` or `--irgen`. It shall be the one used in [Run](#run).

### How does it reduces the test case?

The script performs unguided test-case reduction using `creduce`: given a buggy program, it
randomly reduces the program;
check if the reduced program still fails on the test, and
if so, replaces the given program with the reduced one;
repeat until you get a small enough buggy program.
For more information, we refer to the [Creduce](https://embed.cs.utah.edu/creduce/) homepage.

**[NOTICE]** The fuzzer only supports Ubuntu 20.04.
It may work for other platforms, but if it doesn't, please run the fuzzer in Ubuntu 20.04.

## Running RISC-V Binaries

### RISC-V Documentation

- ISA: <https://riscv.org/technical/specifications/>
- ELF calling convention: <https://github.com/riscv-non-isa/riscv-elf-psabi-doc>

### Install

```sh
# Ubuntu 20.04
sudo apt install gcc-riscv64-linux-gnu g++-riscv64-linux-gnu qemu-user-static gdb-multiarch
```

### Cross-Compilation and Architecture-Emulation

```sh
# Compile C source code into RISC-V assembly
riscv64-linux-gnu-gcc hello.c -S -fsigned-char -o hello.S

# Link to an RISC-V executable
riscv64-linux-gnu-gcc -static hello.S -o hello

# Emulate the executable
qemu-riscv64-static ./hello

# Check the return value
echo $?
```

### Debugging Assembly

You can use QEMU's debugging facilities to investigate whether the generated assembly works correctly.

Open two terminal windows.
In one, compile the assembly with `-ggdb` option and start up a gdb server with 8888 port.
(If 8888 is already in use, then try with a different port like 8889, 8890, ...)

```sh
# Link to an RISC-V executable with `-ggdb` option
riscv64-linux-gnu-gcc -ggdb -static hello.S -o hello

# Emulate the executable and wait for a debugging connection from GDB
qemu-riscv64-static -g 8888 hello
```

In the second terminal, run `gdb-multiarch` and set some configurations.
You should see something like this,

```
$ gdb-multiarch
GNU gdb (Ubuntu 9.2-0ubuntu1~20.04.1) 9.2
Copyright (C) 2020 Free Software Foundation, Inc.
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.
Type "show copying" and "show warranty" for details.
This GDB was configured as "x86_64-linux-gnu".
Type "show configuration" for configuration details.
For bug reporting instructions, please see:
<http://www.gnu.org/software/gdb/bugs/>.
Find the GDB manual and other documentation resources online at:
    <http://www.gnu.org/software/gdb/documentation/>.

For help, type "help".
Type "apropos word" to search for commands related to "word".
(gdb) set arc riscv:rv64
The target architecture is assumed to be riscv:rv64
(gdb) target remote localhost:8888
Remote debugging using localhost:8888
warning: No executable has been specified and target does not support
determining executable automatically.  Try using the "file" command.
0x0000000000010348 in ?? ()
(gdb) file hello
A program is being debugged already.
Are you sure you want to change the file? (y or n) y
Reading symbols from hello...
(gdb) disas main
Dump of assembler code for function main:
   0x0000000000010446 <+0>:     addi    sp,sp,-104
   0x000000000001044a <+4>:     sd      ra,88(sp)
   0x000000000001044c <+6>:     sd      s0,96(sp)
   0x000000000001044e <+8>:     addi    s0,sp,104
End of assembler dump.
(gdb)
```

Now you can debug the assembly using the GDB commands.
For more information on GDB commands, see:

- Full guide: http://sourceware.org/gdb/current/onlinedocs/gdb/
- Cheatsheet: https://cs.brown.edu/courses/cs033/docs/guides/gdb.pdf

## Run Benchmark for Performance Competition

```sh
cd bench
make run
```


## Submission

- Submit the corresponding files to [gg.kaist.ac.kr](https://gg.kaist.ac.kr).
- Run `./scripts/make-submissions.sh` to generate `irgen.zip` to `final.zip`,
  which you should submit for homework 2 to the final project.

## Running on a local machine

- https://github.com/kaist-cp/cs420/issues/314
- https://github.com/kaist-cp/cs420/issues/460
