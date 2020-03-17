#!/usr/bin/env python3

"""Fuzzes KECC.

For customization, one may restrict/loosen the replacement rule by adding/deleting the pair into
below `REPLACE_DICT`.

"""

import os
import subprocess
import itertools
import argparse
import sys
import re

REPLACE_DICT = {
    "#include \"csmith.h\"": "",
    "volatile ": "",
    "uint16_t": "unsigned int",
    "uint32_t": "unsigned int",
    "int16_t": "int",
    "int32_t": "int",
    "uint": "unsigned int",
    "static ": "",
}
CSMITH_DIR = "csmith-2.3.0"

def install_csmith(tests_dir, bin_file):
    global CSMITH_DIR
    csmith_root_dir = os.path.join(tests_dir, CSMITH_DIR)
    if not os.path.exists(bin_file):
        subprocess.Popen(["curl", "https://embed.cs.utah.edu/csmith/" + CSMITH_DIR + ".tar.gz", "-o", CSMITH_DIR + ".tar.gz"], cwd=tests_dir).communicate()
        subprocess.Popen(["tar", "xzvf", CSMITH_DIR + ".tar.gz"], cwd=tests_dir).communicate()
        subprocess.Popen("cmake .; make -j", shell=True, cwd=csmith_root_dir).communicate()
    else:
        print("Using the existing csmith...")

def generate(tests_dir, bin_file, runtime, file_name):
    """Feeding options to built Csmith to randomly generate test case.

    For generality, I disabled most of the features that are enabled by default.
    FYI, please take a look at `-h` flag. By adding or deleting one of `--blah-blah...`
    in `options` list below, csmith will be able to generate corresponding test case.
    A developer may customize the options to meet one's needs for testing.
    """
    global CSMITH_DIR
    options = [
        "--no-argc", "--no-arrays", "--no-checksum",
        "--no-jumps", "--no-longlong", "--no-int8",
        "--no-uint8", "--no-safe-math", "--no-pointers",
        "--no-structs", "--no-unions", "--no-builtins"
    ]
    args = [bin_file] + options
    dst_path = os.path.join(runtime, file_name)

    with open(dst_path, 'w') as f_dst:
        subprocess.Popen(args, cwd=tests_dir, stdout=f_dst).wait()
        f_dst.flush()

    return dst_path

def preprocess(src_path, file_name):
    """Preprocessing test case to fit in kecc parser specification.

    It resolves an issue that arbitrarily included header file to hinder parsing.
    """
    global REPLACE_DICT, CSMITH_DIR
    with open(src_path, 'r') as src:
        src = str(src.read())

        for _from, _to in REPLACE_DICT.items():
            src = src.replace(_from, _to)

        with open(os.path.join(os.path.dirname(src_path), file_name), 'w') as dst:
            dst.write(str(src))

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Fuzzing KECC.')
    parser.add_argument('-n', '--num', type=int, help='The number of tests')
    parser.add_argument('-p', '--print', action='store_true', help='Fuzzing C AST printer')
    args = parser.parse_args()

    if args.print:
        cargo_arg = "-p"
    else:
        raise "Specify fuzzing argument"

    tests_dir = os.path.abspath(os.path.dirname(__file__))
    csmith_bin = os.path.abspath(os.path.join(tests_dir, CSMITH_DIR, "src/csmith"))
    csmith_runtime = os.path.abspath(os.path.join(tests_dir, CSMITH_DIR, "runtime/"))
    install_csmith(tests_dir, csmith_bin)

    # Run cargo test infinitely
    raw_test_file = "raw_test.c"
    test_file = "test.c"
    try:
        if args.num is None:
            print("Fuzzing with infinitely many test cases.  Please press [ctrl+C] to break.")
            iterator = itertools.count(0)
        else:
            print("Fuzzing with {} test cases.".format(args.num))
            iterator = range(args.num)

        for i in iterator:
            print("Test case #{}".format(i))
            preprocess(generate(tests_dir, csmith_bin, csmith_runtime, raw_test_file), test_file)
            args = ["cargo", "run", "--release", "--bin", "fuzz", "--", cargo_arg, os.path.join(csmith_runtime, test_file)]

            try:
                proc = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, cwd=tests_dir)
                (out, err) = proc.communicate(timeout=10)
                if proc.returncode != 0:
                    raise Exception("Test `{}` failed with exit code {}.".format(" ".join(args), proc.returncode))
            except subprocess.TimeoutExpired as e:
                proc.kill()
                raise e

    except KeyboardInterrupt:
        proc.terminate()
        print("\n[Ctrl+C] interrupted")
