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
import random
from pathlib import Path

REPLACE_DICT = {
    "volatile ": "",
    "static ": "",
    "extern ": "",
    "__restrict": "",
    "long __undefined;": "",
    "return 0;": "return (unsigned char)(crc32_context);",
    r"__attribute__\s*\(\(.*\)\)": "",
    "_Float128": "double",
    "long double": "double",
    "(\+[0-9^FL]*)L": r"\1",
    "union": "struct",
    r"enum[\w\s]*\{[^\}]*\};": "",
    r"typedef enum[\w\s]*\{[^;]*;[\s_A-Z]*;": "",
    "const char \*const sys_errlist\[\];": "",      # ArraySize::Unknown 삭제
    r"[^\n]*printf[^;]*;": "",
    r"[^\n]*scanf[^;]*;": "",
    " restrict": "",
    "inline ": "",
    "_Nullable": "",
    "\"g_\w*\", ": "",              # transparent_crc에서 프린트 목적으로 받은 StringLiteral 삭제
    "char\* vname, ": "",           # transparent_crc에서 사용하지 않는 파라미터 삭제
    r"[^\n]*_IO_2_1_[^;]*;": "",    # extern을 지우면서 생긴 size를 알 수 없는 struct 삭제
    r"__asm\s*\([^\)]*\)": "",      # asm extension in mac
    r"__asm__\s*\([^\)]*\)": "",    # asm extension in linux
    "typedef __builtin_va_list __gnuc_va_list;": "",
    "typedef __gnuc_va_list va_list;": "",
    r"fabsf\(": "(",
    
    # todo: need to consider the case below in the future:
    # avoid compile-time constant expressed as complex expression such as `1 + 1`
    "char _unused2[^;]*;": "char _unused2[10];", 
}
CSMITH_DIR = "csmith-2.3.0"
SKIP_TEST = 102

def install_csmith(tests_dir):
    global CSMITH_DIR

    usr_bin_path = "/usr/bin/csmith"
    usr_inc_path = "/usr/include/csmith"
    if os.path.exists(usr_bin_path):
        assert os.path.exists(usr_inc_path)
        return usr_bin_path, usr_inc_path

    bin_path = os.path.abspath(os.path.join(tests_dir, CSMITH_DIR, "src/csmith"))
    inc_path = os.path.abspath(os.path.join(tests_dir, CSMITH_DIR, "runtime"))
    if not os.path.exists(bin_path):
        csmith_filename = "{}.tar.gz".format(CSMITH_DIR)
        try:
            args = ["curl", "https://embed.cs.utah.edu/csmith/{}".format(csmith_filename), "-o", csmith_filename]
            proc = subprocess.Popen(args, cwd=tests_dir)
            proc.communicate()
            if proc.returncode != 0:
                raise Exception("Failed to download Csmith (exit code: {}): `{}`".format(proc.returncode, " ".join(args)))
        except subprocess.TimeoutExpired as e:
            proc.kill()
            raise e

        try:
            args = ["tar", "xzvf", csmith_filename]
            proc = subprocess.Popen(args, cwd=tests_dir)
            proc.communicate()
            if proc.returncode != 0:
                raise Exception("Failed to extract Csmith (exit code: {}): `{}`".format(proc.returncode, " ".join(args)))
        except subprocess.TimeoutExpired as e:
            proc.kill()
            raise e

        csmith_root_dir = os.path.join(tests_dir, CSMITH_DIR)
        try:
            proc = subprocess.Popen("cmake . && make -j", shell=True, cwd=csmith_root_dir)
            proc.communicate()
            if proc.returncode != 0:
                raise Exception("Failed to build Csmith (exit code: {})".format(proc.returncode))
        except subprocess.TimeoutExpired as e:
            proc.kill()
            raise e

    return bin_path, inc_path

def generate(tests_dir, bin_path, seed=None, easy=False):
    """Feeding options to built Csmith to randomly generate test case.

    For generality, I disabled most of the features that are enabled by default.
    FYI, please take a look at `-h` flag. By adding or deleting one of `--blah-blah...`
    in `options` list below, csmith will be able to generate corresponding test case.
    A developer may customize the options to meet one's needs for testing.
    """
    global CSMITH_DIR
    options = [
        "--no-argc", "--no-arrays",
        "--no-jumps", "--no-pointers",
        "--no-structs", "--no-unions",
        "--float", "--strict-float",
    ]
    if seed is not None:
        options += ["--seed", str(seed)]
    if easy:
        options += [
            "--max-block-depth", "2",
            "--max-block-size", "2",
            "--max-struct-fields", "3",
        ]
    args = [bin_path] + options
    
    try:
        proc = subprocess.Popen(args, cwd=tests_dir, stdout=subprocess.PIPE)
        (src, err) = proc.communicate()
        return src.decode()
    except subprocess.TimeoutExpired as e:
        proc.kill()
        raise e

def polish(src, inc_path):
    """Polishing test case to fit in kecc parser specification.
    """
    global REPLACE_DICT, CSMITH_DIR

    try:
        args = ["gcc",
                "-I", inc_path,
                "-E",
                "-",
        ]
        proc = subprocess.Popen(args, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        (src_preprocessed, err) = proc.communicate(src.encode())
        src_preprocessed = src_preprocessed.decode()
    except subprocess.TimeoutExpired as e:
        proc.kill()
        raise e

    src_replaced = src_preprocessed
    for _from, _to in REPLACE_DICT.items():
        src_replaced = re.sub(_from, _to, src_replaced)

    return src_replaced

def make_reduce_criteria(tests_dir, fuzz_arg):
    """Make executable reduce_criteria.sh
    """
    # Make shell script i.e. dependent to KECC path
    arg_dict = {
        "$PROJECT_DIR": str(Path(tests_dir).parent),
        "$FUZZ_ARG": fuzz_arg,
    }
    with open(os.path.join(tests_dir, "reduce-criteria-template.sh"), "r") as t:
        temp = t.read()
        for _from, _to in arg_dict.items():
            temp = temp.replace(_from, _to)
        with open(os.path.join(tests_dir, "reduce-criteria.sh"), "w") as f:
            f.write(temp)

    # chmod the script executable
    try:
        args = ["chmod", "u+x", "reduce-criteria.sh"]
        proc = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, cwd=tests_dir)
        proc.communicate()
        if proc.returncode != 0:
            raise Exception("`{}` failed with exit code {}.".format(" ".join(args), proc.returncode))
    except subprocess.TimeoutExpired as e:
        proc.kill()
        raise e

def creduce(tests_dir, fuzz_arg):
    """Reduce `tests/test_polished.c` to `tests/test_reduced.c`

    First, we copy test_polished.c to test_reduced.c.
    Then, when Creduce reduces test_reduced.c, it overwrites partially reduced program to itself.
    Original file is moved to test_reduced.c.orig which is then identical to test_polished.c.
    """
    make_reduce_criteria(tests_dir, fuzz_arg)

    try:
        args = ["cp", "test_polished.c", "test_reduced.c"]
        proc = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, cwd=tests_dir)
        proc.communicate()
        if proc.returncode != 0:
            raise Exception("`{}` failed with exit code {}.".format(" ".join(args), proc.returncode))
    except subprocess.TimeoutExpired as e:
        proc.kill()
        raise e

    try:
        # --tidy: Do not make a backup copy of each file to reduce as file.orig
        args = ["creduce", "--tidy", "./reduce-criteria.sh", "test_reduced.c"]
        proc = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, cwd=tests_dir)
        (out, err) = proc.communicate()
        if proc.returncode != 0:
            print(out.decode())
            raise Exception("Reducing test_reduced.c by `{}` failed with exit code {}.".format(" ".join(args), proc.returncode))
    except subprocess.TimeoutExpired as e:
        proc.kill()
        raise e

def fuzz(tests_dir, fuzz_arg, num_iter, easy=False):
    global SKIP_TEST

    csmith_bin, csmith_inc = install_csmith(tests_dir)
    try:
        if num_iter is None:
            print("Fuzzing with infinitely many test cases.  Please press [ctrl+C] to break.")
        else:
            assert num_iter > 0
            print("Fuzzing with {} test cases.".format(num_iter))

        i = 0
        skip = 0
        while True:
            print("Test case #{} (skipped: {})".format(i, skip))
            src = generate(
                tests_dir, csmith_bin, 
                seed = random.randint(1, 987654321), easy=easy
            )
            with open(os.path.join(tests_dir, "test.c"), 'w') as dst:
                dst.write(src)

            src_polished = polish(src, csmith_inc)
            with open(os.path.join(tests_dir, "test_polished.c"), 'w') as dst:
                dst.write(src_polished)

            try:
                args = ["cargo", "run", "--features=build-bin", "--release", "--bin", "fuzz", "--", fuzz_arg, os.path.join(tests_dir, "test_polished.c")]
                proc = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, cwd=tests_dir)
                proc.communicate(timeout=60)

                # KECC sets an exit code of 102 when the test skipped.
                if proc.returncode == SKIP_TEST:
                    skip += 1
                    continue
                elif proc.returncode != 0:
                    raise Exception("Test `{}` failed with exit code {}.".format(" ".join(args), proc.returncode))

                i += 1
                if num_iter is not None:
                    if i > num_iter: break
            except subprocess.TimeoutExpired as e:
                proc.kill()
                skip += 1
    except KeyboardInterrupt:
        proc.terminate()
        print("\n[Ctrl+C] interrupted")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Fuzzing KECC.')
    parser.add_argument('-n', '--num', type=int, help='The number of tests', default=None)
    parser.add_argument('-p', '--print', action='store_true', help='Fuzzing C AST printer')
    parser.add_argument('-i', '--irgen', action='store_true', help='Fuzzing irgen')
    parser.add_argument('-r', '--reduce', action='store_true', help="Reducing input file")
    parser.add_argument('--skip-build', action='store_true', help="Skipping cargo build")
    parser.add_argument('--easy', action='store_true', help="Generate more easy code by csmith option")
    parser.add_argument('--seed', type=int, help="Provide seed of fuzz generation", default=-1)
    args = parser.parse_args()

    if args.print and args.irgen:
        raise Exception("Choose an option used for fuzzing: '--print' or '--irgen', NOT both")
    
    if args.print:
        fuzz_arg = "-p"
    elif args.irgen:
        fuzz_arg = "-i"
    else:
        raise Exception("Specify fuzzing argument")

    if args.seed != -1:
        print('Set seed as', args.seed)
        random.seed(args.seed)
    else:
        print('Use default random seed')

    tests_dir = os.path.abspath(os.path.dirname(__file__))

    if not args.skip_build:
        print("Building KECC..")
        try:
            proc = subprocess.Popen(["cargo", "build", "--release"], cwd=tests_dir)
            proc.communicate()
        except subprocess.TimeoutExpired as e:
            proc.kill()
            raise e
    else: 
        print("Skip building. You should manually build the binary. Please execute `cargo build --release` to build.")

    if args.reduce:
        creduce(tests_dir, fuzz_arg)
    else:
        fuzz(tests_dir, fuzz_arg, args.num, args.easy)
