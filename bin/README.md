# KECC User's Manual

## Usage

```sh
cargo run --features=build-bin -- [OPTIONS] <INPUT>
```

## Command Line Options

**Stage Selection Options**

- `-p`, `--print`

  Run the C AST printer, print the generated C AST from the input C file.

- `-i`, `--irgen`

  Run the IRgen, print the generated IR from the input C file.

- **no** stage selection option

  If no stage selection option is specified, print the generated Assembly from the input C or IR file.

**IR Optimization Options**

- `-O`

  Enable all optimizations (simpilfy-cfg, mem2reg, gvn, deadcode) supported in KECC.

- `--simplify-cfg`

  Perform simplify-cfg.

- `--mem2reg`

  Perform mem2reg.

- `--gvn`

  Perform global value numbering.

- `--deadcode`

  Perform deadcode elimination.

**Driver Options**

- `-h`, `--help`

  Display available options.

- `-o`, `--output` \<FILE>

  Write output to \<FILE>.

- `--parse`

  Parse the input C file. If parse failed, it returns the error message.

- `--iroutput`

  Print the output IR.

- `--irparse`

  Parse the input IR file. If parse failed, it returns the error message.

- `--irprint`

  Print the input IR AST.

- `--irrun`

  Execute the input IR file and print the return value.

- `--irviz` \<FILE>

  Save visualized IR file to \<FILE>.  `graphviz` package need to be installed.

- `-V`, `--version`

  Print the version information.

## Examples

**Homework 1**

- Print the generated C AST from `examples/c/fibonacci.c`

  ```sh
  cargo run --features=build-bin -- -p examples/c/fibonacci.c
  ```

**Homework 2**

- Print the generated IR from `examples/c/fibonacci.c`

  ```sh
  cargo run --features=build-bin -- -i examples/c/fibonacci.c
  ```

- Save the IR visualization of generated IR from `examples/c/fibonacci.c` to `fibonacci.png`

  ```sh
  cargo run --features=build-bin -- --irviz fibonacci.png examples/c/fibonacci.c
  ```

- Interpret generated IR from `examples/c/fibonacci.c`

  ```sh
  cargo run --features=build-bin -- --irrun examples/c/fibonacci.c
  ```

**Homework 3**

- Perform simplify-cfg to `examples/simplify_cfg/const_prop.input.ir` and print the optimized IR

  ```sh
  cargo run --features=build-bin -- --iroutput --simplify-cfg examples/simplify_cfg/const_prop.input.ir
  ```

**Homework 4**

- Perform mem2reg to `examples/mem2reg/mem2reg.input.ir` and print the optimized IR

  ```sh
  cargo run --features=build-bin -- --iroutput --mem2reg examples/mem2reg/mem2reg.input.ir
  ```

**Homework 5**

- Perform global value numbering to `examples/gvn/gvn.input.ir` and print the optimized IR

  ```sh
  cargo run --features=build-bin -- --iroutput --gvn examples/gvn/gvn.input.ir
  ```

**Homework 6**

- Perform deadcode elimination to `examples/deadcode/deadcode.input.ir` and print the optimized IR

  ```sh
  cargo run --features=build-bin -- --iroutput --deadcode examples/deadcode/deadcode.input.ir
  ```

**Homework 7**

- Print the generated Assembly from `examples/c/fibonacci.c` and `examples/ir0/fibonacci.ir`

  ```sh
  cargo run --features=build-bin -- examples/c/fibonacci.c    # Generate Assembly from `examples/c/fibonacci.c`
  cargo run --features=build-bin -- examples/ir0/fibonacci.ir # Generate Assembly from `examples/ir0/fibonacci.ir`
  ```

- Print the generated Assembly from `examples/c/fibonacci.c` and `examples/ir0/fibonacci.ir` with all IR optimizations enabled

  ```sh
  cargo run --features=build-bin -- -O examples/c/fibonacci.c     # Generate Assembly from `examples/c/fibonacci.c`
  cargo run --features=build-bin -- -O examples/ir0/fibonacci.ir  # Generate Assembly from `examples/ir0/fibonacci.ir`
  ```

