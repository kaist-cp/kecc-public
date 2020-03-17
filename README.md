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

`<test-name>` can be `test_ast_print`, `ir_smoke`, ...


## Fuzzing

### Install

```sh
# Ubuntu 18.04 or higher
apt install -y make cmake python3

# MacOS
xcode-select install
brew install cmake python3
```

### Run

The following script generates 10 random test cases and tests your C AST printer:

```sh
python3 tests/fuzz.py --help        # print options
python3 tests/fuzz.py --print -n10  # test C AST printer for 10 times
```

We use [Csmith](https://embed.cs.utah.edu/csmith/) to randomly generate C source codes.  Csmith will
be automatically downloaded and built by the test script.
