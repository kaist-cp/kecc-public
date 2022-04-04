#!/usr/bin/env bash

rm -f out*.txt

#ulimit -t 3000                                                                                                                                                                                             
#ulimit -v 2000000                                                                                                                                                                                          

if
  (! gcc test_reduced.c > cc_out.txt 2>&1 ||\
  ! $KECC_BIN --parse test_reduced.c >/dev/null 2>&1)
then
  exit 1
fi

if
  [ $FUZZ_ARG = '-i' ] &&\
  (! clang -pedantic -Wall -Werror=strict-prototypes -c test_reduced.c > out.txt 2>&1 ||\
  grep 'main-return-type' out.txt ||\
  grep 'conversions than data arguments' out.txt ||\
  grep 'int-conversion' out.txt ||\
  grep 'ordered comparison between pointer and zero' out.txt ||\
  grep 'ordered comparison between pointer and integer' out.txt ||\
  grep 'eliding middle term' out.txt ||\
  grep 'end of non-void function' out.txt ||\
  grep 'invalid in C99' out.txt ||\
  grep 'specifies type' out.txt ||\
  grep 'should return a value' out.txt ||\
  grep 'uninitialized' out.txt ||\
  grep 'incompatible pointer to' out.txt ||\
  grep 'incompatible integer to' out.txt ||\
  grep 'type specifier missing' out.txt ||\
  grep 'implicit-function-declaration' out.txt ||\
  grep 'infinite-recursion' out.txt ||\
  grep 'pointer-bool-conversion' out.txt ||\
  grep 'non-void function does not return a value' out.txt ||\
  grep 'too many arguments in call' out.txt ||\
  grep 'declaration does not declare anything' out.txt ||\
  grep 'not equal to a null pointer is always true' out.txt ||\
  grep 'empty struct is a GNU extension' out.txt ||\
  ! gcc -Wall -Wextra test_reduced.c > outa.txt 2>&1 ||\
  grep 'uninitialized' outa.txt ||\
  grep 'without a cast' outa.txt ||\
  grep 'control reaches end' outa.txt ||\
  grep 'return type defaults' outa.txt ||\
  grep 'cast from pointer to integer' outa.txt ||\
  grep 'useless type name in empty declaration' outa.txt ||\
  grep 'no semicolon at end' outa.txt ||\
  grep 'type defaults to' outa.txt ||\
  grep 'too few arguments for format' outa.txt ||\
  grep 'incompatible pointer' out_gcc.txt ||\
  grep 'ordered comparison of pointer with integer' outa.txt ||\
  grep 'declaration does not declare anything' outa.txt ||\
  grep 'expects type' outa.txt ||\
  grep 'pointer from integer' outa.txt ||\
  grep 'incompatible implicit' outa.txt ||\
  grep 'excess elements in struct initializer' outa.txt ||\
  grep 'comparison between pointer and integer' outa.txt ||\
  grep 'division by zero' outa.txt ||\
  ! clang -Wall -Wextra --analyze -c test_reduced.c > outb.txt 2>&1 ||\
  grep 'garbage value' outb.txt)
then
  exit 1
fi

$FUZZ_BIN $FUZZ_ARG test_reduced.c
if [ "$?" = 101 ]
then
  exit 0
else
  exit 1
fi
