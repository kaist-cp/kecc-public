#!/usr/bin/env bash

rm -f out*.txt

#ulimit -t 3000                                                                                                                                                                                             
#ulimit -v 2000000                                                                                                                                                                                          

if
  (! gcc -Wall -Wextra test_reduced.c > out_gcc.txt 2>&1 ||\
  ! $KECC_BIN --parse test_reduced.c >/dev/null 2>&1)
then
  exit 1
fi

if
  [ $FUZZ_ARG = '-i' ] &&\
  (! clang -pedantic -Wall -Werror=strict-prototypes -c test_reduced.c > out_clang.txt 2>&1 ||\
  grep 'main-return-type' out_clang.txt ||\
  grep 'conversions than data arguments' out_clang.txt ||\
  grep 'int-conversion' out_clang.txt ||\
  grep 'ordered comparison between pointer and zero' out_clang.txt ||\
  grep 'ordered comparison between pointer and integer' out_clang.txt ||\
  grep 'eliding middle term' out_clang.txt ||\
  grep 'end of non-void function' out_clang.txt ||\
  grep 'invalid in C99' out_clang.txt ||\
  grep 'specifies type' out_clang.txt ||\
  grep 'should return a value' out_clang.txt ||\
  grep 'uninitialized' out_clang.txt ||\
  grep 'incompatible pointer to' out_clang.txt ||\
  grep 'incompatible integer to' out_clang.txt ||\
  grep 'type specifier missing' out_clang.txt ||\
  grep 'implicit-function-declaration' out_clang.txt ||\
  grep 'infinite-recursion' out_clang.txt ||\
  grep 'pointer-bool-conversion' out_clang.txt ||\
  grep 'non-void function does not return a value' out_clang.txt ||\
  grep 'too many arguments in call' out_clang.txt ||\
  grep 'declaration does not declare anything' out_clang.txt ||\
  grep 'not equal to a null pointer is always true' out_clang.txt ||\
  grep 'empty struct is a GNU extension' out_clang.txt ||\
  grep 'uninitialized' out_gcc.txt ||\
  grep 'without a cast' out_gcc.txt ||\
  grep 'control reaches end' out_gcc.txt ||\
  grep 'return type defaults' out_gcc.txt ||\
  grep 'cast from pointer to integer' out_gcc.txt ||\
  grep 'useless type name in empty declaration' out_gcc.txt ||\
  grep 'no semicolon at end' out_gcc.txt ||\
  grep 'type defaults to' out_gcc.txt ||\
  grep 'too few arguments for format' out_gcc.txt ||\
  grep 'incompatible pointer' out_gcc.txt ||\
  grep 'ordered comparison of pointer with integer' out_gcc.txt ||\
  grep 'declaration does not declare anything' out_gcc.txt ||\
  grep 'expects type' out_gcc.txt ||\
  grep 'pointer from integer' out_gcc.txt ||\
  grep 'incompatible implicit' out_gcc.txt ||\
  grep 'excess elements in struct initializer' out_gcc.txt ||\
  grep 'comparison between pointer and integer' out_gcc.txt ||\
  grep 'division by zero' out_gcc.txt)
then
  exit 1
fi

if
  [ $FUZZ_ARG = '-i' ] &&\
  $CLANG_ANALYZE &&\
  (! clang --analyze -c test_reduced.c > out_analyzer.txt 2>&1 ||\
  grep 'garbage value' out_analyzer.txt)
then
  exit 1
fi

$FUZZ_BIN $FUZZ_ARG test_reduced.c > out_fuzz.txt 2>&1

grep $FUZZ_ERRMSG out_fuzz.txt
